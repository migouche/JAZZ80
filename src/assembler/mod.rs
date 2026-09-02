use std::collections::HashMap;
use std::str::FromStr;

use crate::cpu::alu::rot::RotOperation;
use crate::cpu::{ALUOperation, Condition, RegOps, Rp2Ops, RpOps};

pub mod keywords;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Number(u16),
    String(String),
    Comma,
    OpenParen,
    CloseParen,
    Plus,
    Minus,
    Colon,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(String),          // A, B, HL, IX...
    Immediate(u16),            // 1234
    IndirectImmediate(u16),    // (1234)
    IndirectRegister(String),  // (HL), (BC), (IX)
    IndirectIndex(String, i8), // (IX+d), (IY+d)
    Condition(String),         // NZ, Z, NC...
    Label(String),
    IndirectLabel(String),
    StringLiteral(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Label,         // Code label
    Constant,      // EQU constant
    Byte,          // DB/DEFB (single byte)
    Word,          // DW/DEFW (single word)
    String(usize), // Length
    Array(usize),  // Length (for DS or multi-byte DB)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symbol {
    pub address: u16,
    pub kind: SymbolType,
    pub source_order: usize,
}

#[derive(Debug, Clone)]
pub struct AssemblyLine {
    pub address: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AssemblyResult {
    pub(crate) bytes: Vec<u8>,
    pub(crate) symbols: HashMap<String, Symbol>,
    pub(crate) address_to_line: HashMap<u16, usize>,
    pub(crate) line_to_address: HashMap<usize, u16>,
    pub(crate) image: Vec<(u16, u8)>,
    pub lines: HashMap<usize, AssemblyLine>,
}

pub fn assemble_binary(code: &str) -> Result<Vec<u8>, String> {
    let image = assemble_source(code)?.image;
    let Some(first_address) = image.iter().map(|(address, _)| *address).min() else {
        return Ok(Vec::new());
    };
    let Some(highest_address) = image.iter().map(|(address, _)| *address).max() else {
        return Ok(Vec::new());
    };

    let mut bytes = vec![0; (highest_address - first_address) as usize + 1];
    for (address, byte) in image {
        bytes[(address - first_address) as usize] = byte;
    }
    Ok(bytes)
}

pub fn assemble_with_metadata(code: &str) -> Result<AssemblyResult, String> {
    assemble_source(code)
}

fn assemble_source(code: &str) -> Result<AssemblyResult, String> {
    let mut labels: HashMap<String, Symbol> = HashMap::new();
    let mut current_pc = 0u16;
    let mut instructions = Vec::new();
    let mut line_addresses = HashMap::new();
    let mut latest_global_label: Option<String> = None;
    let mut active_data_label: Option<String> = None;

    for (line_idx, line) in code.lines().enumerate() {
        line_addresses.insert(line_idx + 1, current_pc);
        let clean_line = line.split(';').next().unwrap_or("").trim();
        if clean_line.is_empty() {
            continue;
        }

        let mut tokens =
            tokenize(clean_line).map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
        if tokens.is_empty() {
            continue;
        }

        let mut current_label: Option<String> = None;

        if tokens.len() == 3
            && let (Token::Identifier(name), Token::Identifier(mnemonic)) = (&tokens[0], &tokens[1])
            && mnemonic == "EQU"
        {
            let constant_name = qualify_label(name, latest_global_label.as_deref())
                .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
            if labels.contains_key(&constant_name) {
                return Err(format!(
                    "Line {}: Duplicate symbol '{}'",
                    line_idx + 1,
                    constant_name
                ));
            }
            let equ_operands = parse_operands(&tokens[2..])
                .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
            if equ_operands.len() != 1 {
                return Err(format!("Line {}: EQU expects one value", line_idx + 1));
            }
            let known_symbols: HashMap<String, u16> =
                labels.iter().map(|(k, v)| (k.clone(), v.address)).collect();
            let value = resolve_immediate(&equ_operands[0], &known_symbols, false)
                .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
            labels.insert(
                constant_name,
                Symbol {
                    address: value,
                    kind: SymbolType::Constant,
                    source_order: line_idx,
                },
            );
            continue;
        }

        if tokens.len() >= 2
            && let (Token::Identifier(name), Token::Colon) = (&tokens[0], &tokens[1])
        {
            let label_name = qualify_label(name, latest_global_label.as_deref())
                .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;
            if labels.contains_key(&label_name) {
                return Err(format!(
                    "Line {}: Duplicate label '{}'",
                    line_idx + 1,
                    label_name
                ));
            }
            if !name.starts_with('.') {
                latest_global_label = Some(label_name.clone());
            }
            current_label = Some(label_name.clone());

            let mut sym_kind = SymbolType::Label;
            if tokens.len() > 2
                && let Token::Identifier(next_mnemonic) = &tokens[2]
            {
                match next_mnemonic.as_str() {
                    "DB" | "DEFB" => sym_kind = SymbolType::Byte,
                    "DW" | "DEFW" => sym_kind = SymbolType::Word,
                    "DS" | "DEFS" => sym_kind = SymbolType::Array(0),
                    _ => {}
                }
            }

            labels.insert(
                label_name,
                Symbol {
                    address: current_pc,
                    kind: sym_kind,
                    source_order: line_idx,
                },
            );
            tokens.drain(0..2);
        }

        tokens = qualify_local_tokens(&tokens, latest_global_label.as_deref())
            .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;

        if tokens.is_empty() {
            active_data_label = current_label;
            continue;
        }

        let is_db = matches!(
            tokens.first(),
            Some(Token::Identifier(mnemonic)) if mnemonic == "DB" || mnemonic == "DEFB"
        );
        let data_label = current_label.clone().or_else(|| active_data_label.clone());
        let continuation = current_label.is_none() && is_db && active_data_label.is_some();
        if is_db {
            if current_label.is_some() {
                active_data_label = current_label.clone();
            }
        } else {
            active_data_label = None;
        }

        let known_symbols: HashMap<String, u16> =
            labels.iter().map(|(k, v)| (k.clone(), v.address)).collect();
        let bytes = parse_instruction(&tokens, current_pc, &known_symbols, true)
            .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;

        if let Some(label) = data_label
            && let Some(sym) = labels.get_mut(&label)
            && let Token::Identifier(mnemonic) = &tokens[0]
        {
            match mnemonic.as_str() {
                "DB" | "DEFB" => {
                    let contains_string = parse_operands(&tokens[1..])
                        .map(|ops| ops.iter().any(|op| matches!(op, Operand::StringLiteral(_))))
                        .unwrap_or(false);
                    let previous_len = if current_label.is_some()
                        || (continuation && matches!(sym.kind, SymbolType::Label))
                    {
                        0
                    } else {
                        match sym.kind {
                            SymbolType::String(len) | SymbolType::Array(len) => len,
                            SymbolType::Byte => 1,
                            _ => 0,
                        }
                    };
                    let row_kind = if contains_string {
                        SymbolType::String(bytes.len())
                    } else if bytes.len() == 1 {
                        SymbolType::Byte
                    } else {
                        SymbolType::Array(bytes.len())
                    };

                    if continuation && !matches!(sym.kind, SymbolType::Label) {
                        let row_name = format!("{}[{}]", label, line_idx + 1);
                        labels.insert(
                            row_name,
                            Symbol {
                                address: current_pc,
                                kind: row_kind,
                                source_order: line_idx,
                            },
                        );
                    } else {
                        let total_len = previous_len + bytes.len();
                        sym.kind = if contains_string {
                            SymbolType::String(total_len)
                        } else if total_len == 1 {
                            SymbolType::Byte
                        } else {
                            SymbolType::Array(total_len)
                        };
                    }
                }
                "DS" | "DEFS" => {
                    sym.kind = SymbolType::Array(bytes.len());
                }
                _ => {}
            }
        }

        instructions.push((line_idx, current_pc, tokens));
        let available = u16::MAX as usize - current_pc as usize;
        if bytes.len() > available {
            return Err(format!("Line {}: address space exceeds 64K", line_idx + 1));
        }
        current_pc = current_pc
            .checked_add(bytes.len() as u16)
            .ok_or_else(|| format!("Line {}: address space exceeds 64K", line_idx + 1))?;
    }

    let mut output = Vec::new();
    let mut address_to_line = HashMap::new();
    let mut image = Vec::new();
    let mut lines = HashMap::new();
    let label_addresses: HashMap<String, u16> =
        labels.iter().map(|(k, v)| (k.clone(), v.address)).collect();

    for (line_idx, pc, tokens) in instructions {
        let bytes = parse_instruction(&tokens, pc, &label_addresses, false)
            .map_err(|e| format!("Line {}: {}", line_idx + 1, e))?;

        lines.insert(
            line_idx + 1,
            AssemblyLine {
                address: pc,
                bytes: bytes.clone(),
            },
        );

        output.extend(&bytes);

        let is_org = matches!(tokens.first(), Some(Token::Identifier(m)) if m == "ORG");
        if !is_org {
            for (offset, byte) in bytes.iter().enumerate() {
                let address = pc
                    .checked_add(offset as u16)
                    .ok_or_else(|| format!("Line {}: address space exceeds 64K", line_idx + 1))?;
                image.push((address, *byte));
            }
        }

        address_to_line.insert(pc, line_idx + 1);
    }
    Ok(AssemblyResult {
        bytes: output,
        symbols: labels,
        address_to_line,
        line_to_address: line_addresses,
        image,
        lines,
    })
}

fn qualify_label(name: &str, latest_global: Option<&str>) -> Result<String, String> {
    if let Some(local_name) = name.strip_prefix('.') {
        if local_name.is_empty() {
            return Err("Local label cannot be empty".to_string());
        }
        if let Some(global_name) = latest_global {
            return Ok(format!("{}.{}", global_name, local_name));
        }
        return Err(format!("Local label '{}' has no global label", name));
    }
    Ok(name.to_string())
}

fn qualify_local_tokens(
    tokens: &[Token],
    latest_global: Option<&str>,
) -> Result<Vec<Token>, String> {
    tokens
        .iter()
        .map(|token| match token {
            Token::Identifier(name) if name.starts_with('.') => {
                Ok(Token::Identifier(qualify_label(name, latest_global)?))
            }
            token => Ok(token.clone()),
        })
        .collect()
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '"' => {
                chars.next(); // consume "
                let mut s = String::new();
                let mut terminated = false;
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        terminated = true;
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                if !terminated {
                    return Err("Unterminated string literal".to_string());
                }
                tokens.push(Token::String(s));
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_alphabetic() || c == '_' || c == '.' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '\'' || c == '.' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let upper = ident.to_ascii_uppercase();

                // Check for Hex number without leading zero (e.g. FFFFH)
                // This is technically compliant with some assemblers but violates strict Z80 (requires leading 0-9)
                // We add this to satisfy tests expecting FFFFH as a number.
                let is_hex_candidate = upper.ends_with('H')
                    && upper.len() > 1
                    && upper[..upper.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_hexdigit());

                if is_hex_candidate {
                    if let Ok(val) = u16::from_str_radix(&upper[..upper.len() - 1], 16) {
                        tokens.push(Token::Number(val));
                    } else {
                        // Overflow or error, fallback to identifier
                        tokens.push(Token::Identifier(upper));
                    }
                } else {
                    tokens.push(Token::Identifier(upper));
                }
            }
            c if c.is_ascii_digit() => {
                let mut num_str = String::new();
                let mut is_hex = false;

                // Check for 0x prefix
                if c == '0' {
                    chars.next(); // consume 0
                    if let Some(&nc) = chars.peek() {
                        if nc == 'x' || nc == 'X' {
                            chars.next(); // consume x
                            is_hex = true;
                        } else {
                            num_str.push('0'); // restore 0 if not hex prefix
                        }
                    } else {
                        num_str.push('0');
                    }
                }

                // Allow standard hex digits if not prefix (handling implicit hex if needed, but sticking to 0x for clarity unless we see chars)
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Check for 'h' suffix
                if !is_hex
                    && !num_str.is_empty()
                    && let Some(&nc) = chars.peek()
                    && (nc == 'H' || nc == 'h')
                {
                    is_hex = true;
                    chars.next();
                }

                let val = if is_hex {
                    u16::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?
                } else {
                    num_str.parse::<u16>().map_err(|_| "Invalid number")?
                };
                tokens.push(Token::Number(val));
            }
            // Handle simple hex like $FF
            '$' => {
                chars.next();
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val = u16::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?;
                tokens.push(Token::Number(val));
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}

fn parse_operands(tokens: &[Token]) -> Result<Vec<Operand>, String> {
    let mut operands = Vec::new();
    if tokens.is_empty() {
        return Ok(operands);
    }

    let mut i = 0;
    while i < tokens.len() {
        if i > 0 {
            if let Token::Comma = tokens[i] {
                i += 1;
            } else {
                return Err("Expected comma".to_string());
            }
        }
        if i >= tokens.len() {
            break;
        }

        match &tokens[i] {
            Token::Minus => {
                i += 1;
                if i >= tokens.len() {
                    return Err("Expected number after minus".to_string());
                }
                if let Token::Number(n) = tokens[i] {
                    let val = -(n as i16) as u16;
                    operands.push(Operand::Immediate(val));
                    i += 1;
                } else {
                    return Err("Expected number after minus".to_string());
                }
            }
            Token::Identifier(r) => {
                if is_condition(r) {
                    operands.push(Operand::Condition(r.clone()));
                } else if is_register(r) {
                    operands.push(Operand::Register(r.clone()));
                } else {
                    operands.push(Operand::Label(r.clone()));
                }
                i += 1;
            }
            Token::String(s) => {
                operands.push(Operand::StringLiteral(s.clone()));
                i += 1;
            }
            Token::Number(n) => {
                operands.push(Operand::Immediate(*n));
                i += 1;
            }
            Token::OpenParen => {
                i += 1;
                if i >= tokens.len() {
                    return Err("Unexpected end in parens".to_string());
                }

                match &tokens[i] {
                    Token::Number(n) => {
                        operands.push(Operand::IndirectImmediate(*n));
                        i += 1;
                    }
                    Token::Identifier(r) => {
                        if is_register(r) {
                            let reg = r.clone();
                            i += 1;
                            if i < tokens.len() && matches!(tokens[i], Token::Plus | Token::Minus) {
                                // (IX+d)
                                let sign = match tokens[i] {
                                    Token::Plus => 1,
                                    Token::Minus => -1,
                                    _ => 1,
                                };
                                i += 1;
                                if i >= tokens.len() {
                                    return Err("Expected offset".to_string());
                                }
                                if let Token::Number(offset) = tokens[i] {
                                    let final_offset = (offset as i16 * sign as i16) as i8;
                                    operands.push(Operand::IndirectIndex(reg, final_offset));
                                    i += 1;
                                } else {
                                    return Err("Expected number offset".to_string());
                                }
                            } else {
                                operands.push(Operand::IndirectRegister(reg));
                            }
                        } else {
                            operands.push(Operand::IndirectLabel(r.clone()));
                            i += 1;
                        }
                    }
                    _ => return Err("Invalid start of indirect operand".to_string()),
                }

                if i >= tokens.len() || tokens[i] != Token::CloseParen {
                    return Err("Expected )".to_string());
                }
                i += 1;
            }
            _ => return Err("Invalid operand".to_string()),
        }
    }
    Ok(operands)
}

fn is_condition(s: &str) -> bool {
    matches!(s, "NZ" | "Z" | "NC" | "PO" | "PE" | "P" | "M")
}

fn is_register(s: &str) -> bool {
    keywords::REGISTERS.contains(&s)
}

fn resolve_immediate(
    op: &Operand,
    labels: &HashMap<String, u16>,
    is_dry_run: bool,
) -> Result<u16, String> {
    match op {
        Operand::Immediate(n) => Ok(*n),
        Operand::Label(s) => resolve_label(s, labels, is_dry_run),
        _ => Err("Not an immediate".to_string()),
    }
}

fn resolve_label(
    label: &str,
    labels: &HashMap<String, u16>,
    is_dry_run: bool,
) -> Result<u16, String> {
    labels
        .get(label)
        .copied()
        .or_else(|| is_dry_run.then_some(0))
        .ok_or_else(|| format!("Label not found: {}", label))
}

fn resolve_indirect(
    op: &Operand,
    labels: &HashMap<String, u16>,
    is_dry_run: bool,
) -> Result<u16, String> {
    match op {
        Operand::IndirectImmediate(n) => Ok(*n),
        Operand::IndirectLabel(s) => resolve_label(s, labels, is_dry_run),
        _ => Err("Not an indirect address".to_string()),
    }
}

fn parse_instruction(
    tokens: &[Token],
    pc: u16,
    labels: &HashMap<String, u16>,
    is_dry_run: bool,
) -> Result<Vec<u8>, String> {
    let mnemonic = match &tokens[0] {
        Token::Identifier(m) => m,
        _ => return Err("Expected mnemonic".to_string()),
    };
    let operands = parse_operands(&tokens[1..])?;

    match mnemonic.as_str() {
        "LD" => encode_ld(&operands, labels, is_dry_run),
        "INC" => encode_alu_uni(0x04, &operands),
        "DEC" => encode_alu_uni(0x05, &operands),
        "ADD" => encode_add(&operands, labels, is_dry_run),
        "ADC" => encode_alu_op(ALUOperation::ADC, &operands, labels, is_dry_run),
        "SUB" => encode_alu_op(ALUOperation::SUB, &operands, labels, is_dry_run),
        "SBC" => encode_sbc(&operands, labels, is_dry_run),
        "AND" => encode_alu_op(ALUOperation::AND, &operands, labels, is_dry_run),
        "XOR" => encode_alu_op(ALUOperation::XOR, &operands, labels, is_dry_run),
        "OR" => encode_alu_op(ALUOperation::OR, &operands, labels, is_dry_run),
        "CP" => encode_alu_op(ALUOperation::CP, &operands, labels, is_dry_run),

        "HALT" => Ok(vec![0x76]),
        "NOP" => Ok(vec![0x00]),
        "DI" => Ok(vec![0xF3]),
        "EI" => Ok(vec![0xFB]),
        "RETI" => Ok(vec![0xED, 0x4D]),
        "RETN" => Ok(vec![0xED, 0x45]),
        "IM" => encode_im(&operands),
        "EX" => encode_ex(&operands),
        "EXX" => Ok(vec![0xD9]),
        "DAA" => Ok(vec![0x27]),
        "CPL" => Ok(vec![0x2F]),
        "CCF" => Ok(vec![0x3F]),
        "SCF" => Ok(vec![0x37]),
        "RLA" => Ok(vec![0x17]),
        "RRA" => Ok(vec![0x1F]),
        "RLCA" => Ok(vec![0x07]),
        "RRCA" => Ok(vec![0x0F]),

        "JP" => encode_jp(&operands, labels, is_dry_run),
        "JR" => encode_jr(&operands, pc, labels, is_dry_run),
        "DJNZ" => encode_djnz(&operands, pc, labels, is_dry_run),
        "CALL" => encode_call(&operands, labels, is_dry_run),
        "RET" => encode_ret(&operands),
        "RST" => encode_rst(&operands),

        "PUSH" => encode_push(&operands),
        "POP" => encode_pop(&operands),

        "IN" => encode_in(&operands, labels, is_dry_run),
        "OUT" => encode_out(&operands, labels, is_dry_run),

        "ORG" => {
            if operands.len() != 1 {
                return Err("ORG expects 1 operand".to_string());
            }
            let target = resolve_immediate(&operands[0], labels, is_dry_run)?;
            if target < pc {
                // In dry run (Pass 1), we might have temporary 0 labels producing bad targets,
                // but ORG usually uses constants.
                // If using labels in ORG (advanced), Pass 1 might fail.
                // We assume ORG uses constants or pre-defined symbols.
                return Err("ORG cannot go backwards".to_string());
            }
            Ok(vec![0; (target - pc) as usize])
        }
        "DB" | "DEFB" => {
            let mut bytes = Vec::new();
            for op in operands {
                match op {
                    Operand::Immediate(n) => {
                        if n > 255 {
                            return Err(format!("Value {} too large for DB", n));
                        }
                        bytes.push(n as u8);
                    }
                    Operand::StringLiteral(s) => {
                        bytes.extend_from_slice(s.as_bytes());
                    }
                    Operand::Label(l) => {
                        let val = resolve_label(&l, labels, is_dry_run)?;
                        if !is_dry_run && val > 255 {
                            return Err(format!(
                                "Label '{}' value {} is too large for DB (byte)",
                                l, val
                            ));
                        }
                        bytes.push((val & 0xFF) as u8);
                    }
                    _ => return Err("Invalid DB operand".to_string()),
                }
            }
            Ok(bytes)
        }
        "DW" | "DEFW" => {
            let mut bytes = Vec::new();
            for op in operands {
                match op {
                    Operand::Immediate(n) => {
                        bytes.push((n & 0xFF) as u8);
                        bytes.push((n >> 8) as u8);
                    }
                    Operand::Label(l) => {
                        let val = resolve_label(&l, labels, is_dry_run)?;
                        bytes.push((val & 0xFF) as u8);
                        bytes.push((val >> 8) as u8);
                    }
                    _ => return Err("Invalid DW operand".to_string()),
                }
            }
            Ok(bytes)
        }

        "DS" | "DEFS" => {
            if operands.is_empty() {
                return Err("DS requires at least one operand (count)".to_string());
            }

            let count = resolve_immediate(&operands[0], labels, is_dry_run)?;

            let fill_value = if operands.len() >= 2 {
                (resolve_immediate(&operands[1], labels, is_dry_run)? & 0xFF) as u8
            } else {
                0
            };

            Ok(vec![fill_value; count as usize])
        }

        "LDI" => Ok(vec![0xED, 0xA0]),
        "LDIR" => Ok(vec![0xED, 0xB0]),
        "LDD" => Ok(vec![0xED, 0xA8]),
        "LDDR" => Ok(vec![0xED, 0xB8]),
        "CPI" => Ok(vec![0xED, 0xA1]),
        "CPIR" => Ok(vec![0xED, 0xB1]),
        "CPD" => Ok(vec![0xED, 0xA9]),
        "CPDR" => Ok(vec![0xED, 0xB9]),
        "INI" => Ok(vec![0xED, 0xA2]),
        "INIR" => Ok(vec![0xED, 0xB2]),
        "IND" => Ok(vec![0xED, 0xAA]),
        "INDR" => Ok(vec![0xED, 0xBA]),
        "OUTI" => Ok(vec![0xED, 0xA3]),
        "OTIR" => Ok(vec![0xED, 0xB3]),
        "OUTD" => Ok(vec![0xED, 0xAB]),
        "OTDR" => Ok(vec![0xED, 0xBB]),

        "RLC" => encode_rot_op(RotOperation::RLC, &operands),
        "RRC" => encode_rot_op(RotOperation::RRC, &operands),
        "RL" => encode_rot_op(RotOperation::RL, &operands),
        "RR" => encode_rot_op(RotOperation::RR, &operands),
        "SLA" => encode_rot_op(RotOperation::SLA, &operands),
        "SRA" => encode_rot_op(RotOperation::SRA, &operands),
        "SLL" => encode_rot_op(RotOperation::SLL, &operands),
        "SRL" => encode_rot_op(RotOperation::SRL, &operands),
        "BIT" => encode_bit_op(0x40, &operands),
        "RES" => encode_bit_op(0x80, &operands),
        "SET" => encode_bit_op(0xC0, &operands),

        _ => Err(format!("Unknown mnemonic: {}", mnemonic)),
    }
}

// --- Encoders ---

fn get_r_code(reg: &str) -> Option<u8> {
    RegOps::from_str(reg).ok().map(|r| r as u8)
}

fn get_rp_code(reg: &str) -> Option<u8> {
    RpOps::from_str(reg).ok().map(|rp| rp as u8)
}

fn get_rp2_code(reg: &str) -> Option<u8> {
    Rp2Ops::from_str(reg).ok().map(|rp| rp as u8)
}

fn encode_ld(ops: &[Operand], labels: &HashMap<String, u16>, dry: bool) -> Result<Vec<u8>, String> {
    if ops.len() != 2 {
        return Err("LD requires 2 ops".to_string());
    }
    match (&ops[0], &ops[1]) {
        (Operand::Register(d), Operand::Register(s)) => {
            if let (Some(dc), Some(sc)) = (get_r_code(d), get_r_code(s)) {
                return Ok(vec![0x40 | (dc << 3) | sc]);
            }
            if d == "SP" {
                if s == "HL" {
                    return Ok(vec![0xF9]);
                }
                if s == "IX" {
                    return Ok(vec![0xDD, 0xF9]);
                }
                if s == "IY" {
                    return Ok(vec![0xFD, 0xF9]);
                }
            }
            if d == "I" && s == "A" {
                return Ok(vec![0xED, 0x47]);
            }
            if d == "R" && s == "A" {
                return Ok(vec![0xED, 0x4F]);
            }
            if d == "A" && s == "I" {
                return Ok(vec![0xED, 0x57]);
            }
            if d == "A" && s == "R" {
                return Ok(vec![0xED, 0x5F]);
            }
            Err("Invalid LD Register combination".to_string())
        }
        (Operand::Register(r), op2) if matches!(op2, Operand::Immediate(_) | Operand::Label(_)) => {
            let n = resolve_immediate(op2, labels, dry)?;
            if let Some(rc) = get_r_code(r) {
                return Ok(vec![0x06 | (rc << 3), n as u8]);
            }
            if let Some(rpc) = get_rp_code(r) {
                return Ok(vec![0x01 | (rpc << 4), (n & 0xFF) as u8, (n >> 8) as u8]);
            }
            if r == "IX" {
                return Ok(vec![0xDD, 0x21, (n & 0xFF) as u8, (n >> 8) as u8]);
            }
            if r == "IY" {
                return Ok(vec![0xFD, 0x21, (n & 0xFF) as u8, (n >> 8) as u8]);
            }
            Err("Invalid LD Register Immediate".to_string())
        }
        (Operand::Register(r), Operand::IndirectRegister(ir)) => {
            if let Some(rc) = get_r_code(r) {
                if ir == "HL" {
                    return Ok(vec![0x46 | (rc << 3)]);
                }
                if r == "A" && ir == "BC" {
                    return Ok(vec![0x0A]);
                }
                if r == "A" && ir == "DE" {
                    return Ok(vec![0x1A]);
                }
            }
            Err("Invalid LD r, (reg)".to_string())
        }
        (Operand::IndirectRegister(ir), Operand::Register(r)) => {
            if let Some(rc) = get_r_code(r) {
                if ir == "HL" {
                    return Ok(vec![0x70 | rc]);
                }
                if r == "A" && ir == "BC" {
                    return Ok(vec![0x02]);
                }
                if r == "A" && ir == "DE" {
                    return Ok(vec![0x12]);
                }
            }
            Err("Invalid LD (reg), r".to_string())
        }
        (Operand::IndirectRegister(ir), op2)
            if ir == "HL" && matches!(op2, Operand::Immediate(_) | Operand::Label(_)) =>
        {
            let n = resolve_immediate(op2, labels, dry)?;
            Ok(vec![0x36, n as u8])
        }
        (Operand::Register(r), Operand::IndirectIndex(idx, d)) => {
            if let Some(rc) = get_r_code(r) {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                return Ok(vec![prefix, 0x46 | (rc << 3), *d as u8]);
            }
            Err("Invalid LD r, (idx+d)".to_string())
        }
        (Operand::IndirectIndex(idx, d), Operand::Register(r)) => {
            if let Some(rc) = get_r_code(r) {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                return Ok(vec![prefix, 0x70 | rc, *d as u8]);
            }
            Err("Invalid LD (idx+d), r".to_string())
        }
        (Operand::IndirectIndex(idx, d), op2)
            if matches!(op2, Operand::Immediate(_) | Operand::Label(_)) =>
        {
            let n = resolve_immediate(op2, labels, dry)?;
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            Ok(vec![prefix, 0x36, *d as u8, n as u8])
        }
        (Operand::Register(r), op2)
            if matches!(
                op2,
                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
            ) =>
        {
            let nn = resolve_indirect(op2, labels, dry)?;
            let low = (nn & 0xFF) as u8;
            let high = (nn >> 8) as u8;
            if r == "A" {
                return Ok(vec![0x3A, low, high]);
            }
            if r == "HL" {
                return Ok(vec![0x2A, low, high]);
            }
            if r == "BC" || r == "DE" || r == "SP" {
                let rpc = get_rp_code(r).unwrap();
                return Ok(vec![0xED, 0x4B | (rpc << 4), low, high]);
            }
            if r == "IX" {
                return Ok(vec![0xDD, 0x2A, low, high]);
            }
            if r == "IY" {
                return Ok(vec![0xFD, 0x2A, low, high]);
            }
            Err("Invalid LD r, (nn)".to_string())
        }
        (op1, Operand::Register(r))
            if matches!(
                op1,
                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
            ) =>
        {
            let nn = resolve_indirect(op1, labels, dry)?;
            let low = (nn & 0xFF) as u8;
            let high = (nn >> 8) as u8;
            if r == "A" {
                return Ok(vec![0x32, low, high]);
            }
            if r == "HL" {
                return Ok(vec![0x22, low, high]);
            }
            if r == "BC" || r == "DE" || r == "SP" {
                let rpc = get_rp_code(r).unwrap();
                return Ok(vec![0xED, 0x43 | (rpc << 4), low, high]);
            }
            if r == "IX" {
                return Ok(vec![0xDD, 0x22, low, high]);
            }
            if r == "IY" {
                return Ok(vec![0xFD, 0x22, low, high]);
            }
            Err("Invalid LD (nn), r".to_string())
        }
        _ => Err("Unsupported instruction".to_string()),
    }
}

fn encode_alu_uni(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("Needs 1 Op".to_string());
    }
    match &ops[0] {
        Operand::Register(r) => {
            if let Some(rc) = get_r_code(r) {
                return Ok(vec![base | (rc << 3)]);
            }
            if let Some(rpc) = get_rp_code(r) {
                let val = if base == 0x04 { 0x03 } else { 0x0B };
                return Ok(vec![val | (rpc << 4)]);
            }
            if r == "IX" {
                let val = if base == 0x04 { 0x23 } else { 0x2B };
                return Ok(vec![0xDD, val]);
            }
            if r == "IY" {
                let val = if base == 0x04 { 0x23 } else { 0x2B };
                return Ok(vec![0xFD, val]);
            }
            Err("Invalid register for INC/DEC".to_string())
        }
        Operand::IndirectRegister(r) if r == "HL" => Ok(vec![base | (6 << 3)]),
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            Ok(vec![prefix, base | (6 << 3), *d as u8])
        }
        _ => Err("Invalid INC/DEC".to_string()),
    }
}

fn encode_add(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() == 2
        && let Operand::Register(dest) = &ops[0]
    {
        if dest == "HL"
            && let Operand::Register(src) = &ops[1]
            && let Some(rpc) = get_rp_code(src)
        {
            return Ok(vec![0x09 | (rpc << 4)]);
        }
        if dest == "IX" || dest == "IY" {
            let prefix = if dest == "IX" { 0xDD } else { 0xFD };
            if let Operand::Register(src) = &ops[1] {
                let pp = match src.as_str() {
                    "BC" => 0,
                    "DE" => 1,
                    "IX" | "IY" => 2,
                    "SP" => 3,
                    _ => return Err("Invalid operand for ADD index".to_string()),
                };
                return Ok(vec![prefix, 0x09 | (pp << 4)]);
            }
        }
    }
    encode_alu_op(ALUOperation::ADD, ops, labels, dry)
}

fn encode_sbc(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() == 2
        && let Operand::Register(dest) = &ops[0]
        && dest == "HL"
        && let Operand::Register(src) = &ops[1]
        && let Some(rpc) = get_rp_code(src)
    {
        return Ok(vec![0xED, 0x42 | (rpc << 4)]);
    }
    encode_alu_op(ALUOperation::SBC, ops, labels, dry)
}

fn encode_alu_op(
    op: ALUOperation,
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    let op_code = op as u8;
    let base_r = 0x80 | (op_code << 3);
    let base_n = 0xC6 | (op_code << 3);
    encode_alu_bin(base_r, base_n, ops, labels, dry)
}

fn encode_rot_op(op: RotOperation, ops: &[Operand]) -> Result<Vec<u8>, String> {
    let base = (op as u8) << 3;
    encode_rot_shift(base, ops)
}

fn encode_alu_bin(
    base_r: u8,
    base_n: u8,
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        if ops.len() == 2
            && let Operand::Register(r) = &ops[0]
            && r == "A"
        {
            return encode_alu_bin(base_r, base_n, &ops[1..], labels, dry);
        }
        return Err("ALU ops count error".to_string());
    }
    match &ops[0] {
        Operand::Register(r) => {
            if let Some(rc) = get_r_code(r) {
                Ok(vec![base_r | rc])
            } else if r == "IXH" {
                Ok(vec![0xDD, base_r | 4])
            } else if r == "IXL" {
                Ok(vec![0xDD, base_r | 5])
            } else {
                Err("Invalid ALU register".to_string())
            }
        }
        op if matches!(op, Operand::Immediate(_) | Operand::Label(_)) => {
            let n = resolve_immediate(op, labels, dry)?;
            Ok(vec![base_n, n as u8])
        }
        Operand::IndirectRegister(r) if r == "HL" => Ok(vec![base_r | 6]),
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            Ok(vec![prefix, base_r | 6, *d as u8])
        }
        _ => Err("Invalid ALU operand".to_string()),
    }
}

fn get_condition_code(s: &str) -> Option<u8> {
    Condition::from_str(s).ok().map(|c| c as u8)
}

fn encode_jp(ops: &[Operand], labels: &HashMap<String, u16>, dry: bool) -> Result<Vec<u8>, String> {
    match ops.len() {
        1 => match &ops[0] {
            op if matches!(
                op,
                Operand::Immediate(_)
                    | Operand::Label(_)
                    | Operand::IndirectImmediate(_)
                    | Operand::IndirectLabel(_)
            ) =>
            {
                let nn = if matches!(
                    op,
                    Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                ) {
                    resolve_indirect(op, labels, dry)?
                } else {
                    resolve_immediate(op, labels, dry)?
                };
                Ok(vec![0xC3, (nn & 0xFF) as u8, (nn >> 8) as u8])
            }
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "HL" => Ok(vec![0xE9]),
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "IX" => {
                Ok(vec![0xDD, 0xE9])
            }
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "IY" => {
                Ok(vec![0xFD, 0xE9])
            }
            _ => Err("Invalid JP target".to_string()),
        },
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond
                && let Some(cc) = get_condition_code(c)
            {
                let nn = if matches!(
                    &ops[1],
                    Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                ) {
                    resolve_indirect(&ops[1], labels, dry)?
                } else {
                    resolve_immediate(&ops[1], labels, dry)?
                };
                return Ok(vec![0xC2 | (cc << 3), (nn & 0xFF) as u8, (nn >> 8) as u8]);
            }
            Err("Invalid JP condition/target".to_string())
        }
        _ => Err("Invalid JP args".to_string()),
    }
}

fn encode_jr(
    ops: &[Operand],
    pc: u16,
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    // JR d / JR C, d
    // Opcode size is 2 bytes. Offset is relative to PC+2.
    // Target = (PC + 2) + offset (signed i8)
    // Offset = Target - (PC + 2)
    match ops.len() {
        1 => {
            // JR d
            let target = resolve_immediate(&ops[0], labels, dry)?;
            let offset_val = (target as i32) - ((pc as i32) + 2);
            if !dry && !(-128..=127).contains(&offset_val) {
                return Err("JR offset out of range".to_string());
            }
            Ok(vec![0x18, offset_val as i8 as u8])
        }
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond
                && let Some(cc) = get_condition_code(c)
            {
                if cc > 3 {
                    return Err("Invalid JR condition".to_string());
                }
                let target = resolve_immediate(&ops[1], labels, dry)?;
                let offset_val = (target as i32) - ((pc as i32) + 2);
                if !dry && !(-128..=127).contains(&offset_val) {
                    return Err("JR offset out of range".to_string());
                }
                return Ok(vec![0x20 | (cc << 3), offset_val as i8 as u8]);
            }
            Err("Invalid JR args".to_string())
        }
        _ => Err("Invalid JR args".to_string()),
    }
}

fn encode_djnz(
    ops: &[Operand],
    pc: u16,
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("DJNZ 1 op".to_string());
    }
    let target = resolve_immediate(&ops[0], labels, dry)?;
    let offset_val = (target as i32) - ((pc as i32) + 2);
    if !dry && !(-128..=127).contains(&offset_val) {
        return Err("DJNZ offset out of range".to_string());
    }
    Ok(vec![0x10, offset_val as i8 as u8])
}

fn encode_call(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    match ops.len() {
        1 => {
            let nn = match &ops[0] {
                op if matches!(
                    op,
                    Operand::Immediate(_)
                        | Operand::IndirectImmediate(_)
                        | Operand::Label(_)
                        | Operand::IndirectLabel(_)
                ) =>
                {
                    if matches!(
                        op,
                        Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                    ) {
                        resolve_indirect(op, labels, dry)?
                    } else {
                        resolve_immediate(op, labels, dry)?
                    }
                }
                _ => return Err("CALL needs address".to_string()),
            };
            Ok(vec![0xCD, (nn & 0xFF) as u8, (nn >> 8) as u8])
        }
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond {
                if let Some(cc) = get_condition_code(c) {
                    let nn = match &ops[1] {
                        op if matches!(
                            op,
                            Operand::Immediate(_)
                                | Operand::IndirectImmediate(_)
                                | Operand::Label(_)
                                | Operand::IndirectLabel(_)
                        ) =>
                        {
                            if matches!(
                                op,
                                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                            ) {
                                resolve_indirect(op, labels, dry)?
                            } else {
                                resolve_immediate(op, labels, dry)?
                            }
                        }
                        _ => return Err("CALL needs address".to_string()),
                    };
                    Ok(vec![0xC4 | (cc << 3), (nn & 0xFF) as u8, (nn >> 8) as u8])
                } else {
                    Err("Bad cond".to_string())
                }
            } else {
                Err("Call format error".to_string())
            }
        }
        _ => Err("CALL args".to_string()),
    }
}

fn encode_ret(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.is_empty() {
        return Ok(vec![0xC9]);
    }
    if ops.len() == 1 {
        let cond = match &ops[0] {
            Operand::Condition(c) => Some(c.as_str()),
            Operand::Register(r) if r == "C" => Some("C"),
            _ => None,
        };
        if let Some(c) = cond
            && let Some(cc) = get_condition_code(c)
        {
            return Ok(vec![0xC0 | (cc << 3)]);
        }
    }
    Err("RET args".to_string())
}

fn encode_rst(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("RST 1 op".to_string());
    }
    if let Operand::Immediate(n) = &ops[0] {
        if *n & 0xC7 != 0 {
            return Err("Invalid RST address".to_string());
        }
        return Ok(vec![0xC7 | (*n as u8)]);
    }
    Err("RST invalid".to_string())
}

fn encode_push(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("PUSH 1 op".to_string());
    }
    if let Operand::Register(r) = &ops[0] {
        if let Some(c) = get_rp2_code(r) {
            return Ok(vec![0xC5 | (c << 4)]);
        }
        if r == "IX" {
            return Ok(vec![0xDD, 0xE5]);
        }
        if r == "IY" {
            return Ok(vec![0xFD, 0xE5]);
        }
    }
    Err("PUSH invalid".to_string())
}
fn encode_pop(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("POP 1 op".to_string());
    }
    if let Operand::Register(r) = &ops[0] {
        if let Some(c) = get_rp2_code(r) {
            return Ok(vec![0xC1 | (c << 4)]);
        }
        if r == "IX" {
            return Ok(vec![0xDD, 0xE1]);
        }
        if r == "IY" {
            return Ok(vec![0xFD, 0xE1]);
        }
    }
    Err("POP invalid".to_string())
}

fn encode_ex(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 2 {
        return Err("EX 2 ops".to_string());
    }
    match (&ops[0], &ops[1]) {
        (Operand::Register(r1), Operand::Register(r2)) => {
            if r1 == "DE" && r2 == "HL" {
                return Ok(vec![0xEB]);
            }
            if r1 == "AF" && (r2 == "AF'" || r2 == "AF") {
                return Ok(vec![0x08]);
            }
            Err("Invalid EX".to_string())
        }
        (Operand::IndirectRegister(r1), Operand::Register(r2)) if r1 == "SP" => {
            if r2 == "HL" {
                return Ok(vec![0xE3]);
            }
            if r2 == "IX" {
                return Ok(vec![0xDD, 0xE3]);
            }
            if r2 == "IY" {
                return Ok(vec![0xFD, 0xE3]);
            }
            Err("Invalid EX (SP)".to_string())
        }
        _ => Err("Invalid EX combo".to_string()),
    }
}

fn encode_in(ops: &[Operand], labels: &HashMap<String, u16>, dry: bool) -> Result<Vec<u8>, String> {
    if ops.len() != 2 {
        return Err("IN 2 ops".to_string());
    }
    match (&ops[0], &ops[1]) {
        (Operand::Register(r), op) if r == "A" => {
            if matches!(
                op,
                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
            ) {
                let port = resolve_indirect(op, labels, dry)?;
                Ok(vec![0xDB, port as u8])
            } else {
                Err("Invalid IN form".to_string())
            }
        }
        (Operand::Register(r), Operand::IndirectRegister(ir)) if ir == "C" => {
            if let Some(rc) = get_r_code(r) {
                Ok(vec![0xED, 0x40 | (rc << 3)])
            } else {
                Err("Invalid IN reg".to_string())
            }
        }
        _ => Err("Invalid IN form".to_string()),
    }
}
fn encode_out(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() != 2 {
        return Err("OUT 2 ops".to_string());
    }
    match (&ops[0], &ops[1]) {
        (op, Operand::Register(r)) if r == "A" => {
            if matches!(
                op,
                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
            ) {
                let port = resolve_indirect(op, labels, dry)?;
                Ok(vec![0xD3, port as u8])
            } else {
                Err("Invalid OUT form".to_string())
            }
        }
        (Operand::IndirectRegister(ir), Operand::Register(r)) if ir == "C" => {
            if let Some(rc) = get_r_code(r) {
                Ok(vec![0xED, 0x41 | (rc << 3)])
            } else {
                Err("Invalid OUT reg".to_string())
            }
        }
        _ => Err("Invalid OUT form".to_string()),
    }
}

fn encode_im(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("IM expects 1 operand".to_string());
    }
    match &ops[0] {
        Operand::Immediate(0) => Ok(vec![0xED, 0x46]),
        Operand::Immediate(1) => Ok(vec![0xED, 0x56]),
        Operand::Immediate(2) => Ok(vec![0xED, 0x5E]),
        _ => Err("Invalid IM mode (0, 1, 2)".to_string()),
    }
}

#[cfg(test)]
mod tests;

fn encode_rot_shift(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() == 1 {
        match &ops[0] {
            Operand::Register(r) => {
                if let Some(rc) = get_r_code(r) {
                    Ok(vec![0xCB, base | rc])
                } else {
                    Err("Invalid register for Rotate/Shift".to_string())
                }
            }
            Operand::IndirectRegister(r) if r == "HL" => Ok(vec![0xCB, base | 6]),
            Operand::IndirectIndex(idx, d) => {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                Ok(vec![prefix, 0xCB, *d as u8, base | 6])
            }
            _ => Err("Invalid operand for Rotate/Shift".to_string()),
        }
    } else if ops.len() == 2 {
        // Handle undocumented (IX+d), r
        if let (Operand::IndirectIndex(idx, d), Operand::Register(r)) = (&ops[0], &ops[1]) {
            if let Some(rc) = get_r_code(r) {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                // Order: Prefix, CB, d, Opcode|r
                Ok(vec![prefix, 0xCB, *d as u8, base | rc])
            } else {
                Err("Invalid register2 for Rotate/Shift".to_string())
            }
        } else {
            Err("Invalid operands for Rotate/Shift (2 ops)".to_string())
        }
    } else {
        Err("Rotate/Shift expects 1 or 2 operands".to_string())
    }
}

fn encode_bit_op(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() < 2 || ops.len() > 3 {
        return Err("Bit op expects 2 or 3 operands".to_string());
    }

    let b = match &ops[0] {
        Operand::Immediate(n) => *n,
        _ => return Err("Bit index must be a number".to_string()),
    };

    if b > 7 {
        return Err("Bit index must be 0-7".to_string());
    }

    let opcode_base = base + ((b as u8) << 3);

    match &ops[1] {
        Operand::Register(r) => {
            if ops.len() != 2 {
                return Err("Too many operands for Register target".to_string());
            }
            if let Some(rc) = get_r_code(r) {
                Ok(vec![0xCB, opcode_base | rc])
            } else {
                Err("Invalid register".to_string())
            }
        }
        Operand::IndirectRegister(reg) if reg == "HL" => {
            if ops.len() != 2 {
                return Err("Too many operands for (HL)".to_string());
            }
            Ok(vec![0xCB, opcode_base | 6])
        }
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            if ops.len() == 2 {
                Ok(vec![prefix, 0xCB, *d as u8, opcode_base | 6])
            } else {
                // 3 operands: SET b, (IX+d), r
                // Only for SET/RES. BIT does not have this.
                // BIT base is 0x40.
                if base == 0x40 {
                    return Err("BIT does not support 3 operands".to_string());
                }
                if let Operand::Register(r) = &ops[2] {
                    if let Some(rc) = get_r_code(r) {
                        Ok(vec![prefix, 0xCB, *d as u8, opcode_base | rc])
                    } else {
                        Err("Invalid register 2".to_string())
                    }
                } else {
                    Err("Third operand must be register".to_string())
                }
            }
        }
        _ => Err("Invalid target operand".to_string()),
    }
}
