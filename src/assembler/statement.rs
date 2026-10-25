use std::collections::HashMap;

use super::{
    AssemblyLine, AssemblyResult, Operand, Symbol, SymbolType, Token, parse_instruction,
    parse_operands, qualify_label, qualify_local_tokens, resolve_immediate, tokenize,
};

/// One source line, classified during the analysis pass.
pub(crate) enum Statement {
    /// `NAME: ...` (the rest of the line is the payload)
    Label {
        name: String,
        payload: Vec<Token>,
    },
    /// `NAME EQU value`
    Equ {
        name: String,
        operands: Vec<Operand>,
    },
    /// Any other non-empty line (instruction or directive)
    Instruction(Vec<Token>),
    Empty,
}

pub(crate) struct SourceLine {
    pub line_no: usize, // 1-based
    pub address: u16,
    pub statement: Statement,
}

pub(super) fn assemble_source(code: &str) -> Result<AssemblyResult, String> {
    let analysis = analyze(code)?;
    emit_assembly(analysis)
}

/// Symbols collected by the analysis pass, with an address cache so instruction
/// sizing never has to rebuild a `HashMap` inside the per-line loop.
#[derive(Default)]
struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    addresses: HashMap<String, u16>,
}

impl SymbolTable {
    fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    fn insert(&mut self, name: String, symbol: Symbol) {
        self.addresses.insert(name.clone(), symbol.address);
        self.symbols.insert(name, symbol);
    }

    fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    fn addresses(&self) -> &HashMap<String, u16> {
        &self.addresses
    }

    fn into_symbols(self) -> HashMap<String, Symbol> {
        self.symbols
    }
}

struct Analysis {
    lines: Vec<SourceLine>,
    symbols: HashMap<String, Symbol>,
    line_addresses: HashMap<usize, u16>,
}

/// First pass: classify every source line, resolve every symbol address, and
/// record the final `SymbolType` for data definitions.
fn analyze(code: &str) -> Result<Analysis, String> {
    let mut symbols = SymbolTable::default();
    let mut lines = Vec::new();
    let mut line_addresses = HashMap::new();
    let mut current_pc = 0u16;
    let mut latest_global_label: Option<String> = None;
    let mut active_data_label: Option<String> = None;

    for (line_idx, raw_line) in code.lines().enumerate() {
        let line_no = line_idx + 1;
        line_addresses.insert(line_no, current_pc);

        let clean_line = raw_line.split(';').next().unwrap_or("").trim();
        if clean_line.is_empty() {
            lines.push(SourceLine {
                line_no,
                address: current_pc,
                statement: Statement::Empty,
            });
            continue;
        }

        let mut tokens = tokenize(clean_line).map_err(|e| format!("Line {}: {}", line_no, e))?;
        if tokens.is_empty() {
            lines.push(SourceLine {
                line_no,
                address: current_pc,
                statement: Statement::Empty,
            });
            continue;
        }

        if let Some((name, equ_tokens)) = equ_parts(&tokens) {
            let constant_name = qualify_label(name, latest_global_label.as_deref())
                .map_err(|e| format!("Line {}: {}", line_no, e))?;
            if symbols.contains(&constant_name) {
                return Err(format!(
                    "Line {}: Duplicate symbol '{}'",
                    line_no, constant_name
                ));
            }
            let equ_operands =
                parse_operands(equ_tokens).map_err(|e| format!("Line {}: {}", line_no, e))?;
            if equ_operands.len() != 1 {
                return Err(format!("Line {}: EQU expects one value", line_no));
            }
            let value = resolve_immediate(&equ_operands[0], symbols.addresses(), false)
                .map_err(|e| format!("Line {}: {}", line_no, e))?;
            symbols.insert(
                constant_name.clone(),
                Symbol {
                    address: value,
                    kind: SymbolType::Constant,
                    source_order: line_idx,
                },
            );
            lines.push(SourceLine {
                line_no,
                address: current_pc,
                statement: Statement::Equ {
                    name: constant_name,
                    operands: equ_operands,
                },
            });
            continue;
        }

        let mut current_label: Option<String> = None;

        if let Some(name) = label_parts(&tokens) {
            let label_name = qualify_label(name, latest_global_label.as_deref())
                .map_err(|e| format!("Line {}: {}", line_no, e))?;
            if symbols.contains(&label_name) {
                return Err(format!(
                    "Line {}: Duplicate label '{}'",
                    line_no, label_name
                ));
            }
            if !name.starts_with('.') {
                latest_global_label = Some(label_name.clone());
            }
            current_label = Some(label_name.clone());

            let mut sym_kind = SymbolType::Label;
            if let Some(Token::Identifier(next_mnemonic)) = tokens.get(2) {
                match next_mnemonic.as_str() {
                    "DB" | "DEFB" => sym_kind = SymbolType::Byte,
                    "DW" | "DEFW" => sym_kind = SymbolType::Word,
                    "DS" | "DEFS" => sym_kind = SymbolType::Array(0),
                    _ => {}
                }
            }

            symbols.insert(
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
            .map_err(|e| format!("Line {}: {}", line_no, e))?;

        if tokens.is_empty() {
            active_data_label = current_label.clone();
            let statement = match current_label {
                Some(name) => Statement::Label {
                    name,
                    payload: tokens,
                },
                None => Statement::Empty,
            };
            lines.push(SourceLine {
                line_no,
                address: current_pc,
                statement,
            });
            continue;
        }

        let is_db = is_db_line(&tokens);
        let data_label = current_label.clone().or_else(|| active_data_label.clone());
        let continuation = current_label.is_none() && is_db && active_data_label.is_some();
        if is_db && current_label.is_some() {
            active_data_label = current_label.clone();
        } else if !is_db {
            active_data_label = None;
        }

        let bytes = parse_instruction(&tokens, current_pc, symbols.addresses(), true)
            .map_err(|e| format!("Line {}: {}", line_no, e))?;

        if let Some(label) = data_label.as_deref() {
            update_data_symbol(
                &mut symbols,
                label,
                current_pc,
                &tokens,
                &bytes,
                continuation,
                line_idx,
            );
        }

        let statement = match current_label {
            Some(name) => Statement::Label {
                name,
                payload: tokens,
            },
            None => Statement::Instruction(tokens),
        };
        lines.push(SourceLine {
            line_no,
            address: current_pc,
            statement,
        });

        let available = u16::MAX as usize - current_pc as usize;
        if bytes.len() > available {
            return Err(format!("Line {}: address space exceeds 64K", line_no));
        }
        current_pc = current_pc
            .checked_add(bytes.len() as u16)
            .ok_or_else(|| format!("Line {}: address space exceeds 64K", line_no))?;
    }

    Ok(Analysis {
        lines,
        symbols: symbols.into_symbols(),
        line_addresses,
    })
}

fn equ_parts(tokens: &[Token]) -> Option<(&str, &[Token])> {
    if tokens.len() != 3 {
        return None;
    }
    match (&tokens[0], &tokens[1]) {
        (Token::Identifier(name), Token::Identifier(mnemonic)) if mnemonic == "EQU" => {
            Some((name, &tokens[2..]))
        }
        _ => None,
    }
}

fn label_parts(tokens: &[Token]) -> Option<&str> {
    match tokens {
        [Token::Identifier(name), Token::Colon, ..] => Some(name),
        _ => None,
    }
}

fn is_db_line(tokens: &[Token]) -> bool {
    matches!(
        tokens.first(),
        Some(Token::Identifier(mnemonic)) if mnemonic == "DB" || mnemonic == "DEFB"
    )
}

/// Fold one `DB`/`DEFB`/`DS`/`DEFS` row into its label's symbol, creating a
/// `LABEL[line]` row symbol for a continuation row exactly as before.
fn update_data_symbol(
    symbols: &mut SymbolTable,
    label: &str,
    row_address: u16,
    tokens: &[Token],
    bytes: &[u8],
    continuation: bool,
    line_idx: usize,
) {
    let Some(symbol) = symbols.get(label) else {
        return;
    };
    let kind = symbol.kind;
    let Some(Token::Identifier(mnemonic)) = tokens.first() else {
        return;
    };

    match mnemonic.as_str() {
        "DB" | "DEFB" => {
            // The operands were already parsed successfully by
            // `parse_instruction`; a string literal operand can only come from
            // a string token, so this does not need a second parse.
            let contains_string = tokens
                .iter()
                .skip(1)
                .any(|token| matches!(token, Token::String(_)));
            let previous_len = if !continuation || matches!(kind, SymbolType::Label) {
                0
            } else {
                match kind {
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

            if continuation && !matches!(kind, SymbolType::Label) {
                symbols.insert(
                    format!("{}[{}]", label, line_idx + 1),
                    Symbol {
                        address: row_address,
                        kind: row_kind,
                        source_order: line_idx,
                    },
                );
            } else {
                let total_len = previous_len + bytes.len();
                let new_kind = if contains_string {
                    SymbolType::String(total_len)
                } else if total_len == 1 {
                    SymbolType::Byte
                } else {
                    SymbolType::Array(total_len)
                };
                if let Some(symbol) = symbols.get_mut(label) {
                    symbol.kind = new_kind;
                }
            }
        }
        "DS" | "DEFS" => {
            if let Some(symbol) = symbols.get_mut(label) {
                symbol.kind = SymbolType::Array(bytes.len());
            }
        }
        _ => {}
    }
}

/// Second pass: encode every recorded statement with the complete symbol table.
fn emit_assembly(analysis: Analysis) -> Result<AssemblyResult, String> {
    let Analysis {
        lines,
        symbols,
        line_addresses,
    } = analysis;
    let label_addresses: HashMap<String, u16> = symbols
        .iter()
        .map(|(name, symbol)| (name.clone(), symbol.address))
        .collect();

    let mut output = Vec::new();
    let mut address_to_line = HashMap::new();
    let mut image = Vec::new();
    let mut assembly_lines = HashMap::new();

    for source_line in lines {
        let tokens = match &source_line.statement {
            Statement::Label { name, payload } => {
                debug_assert!(symbols.contains_key(name));
                if payload.is_empty() {
                    continue;
                }
                payload.as_slice()
            }
            Statement::Instruction(tokens) => tokens.as_slice(),
            Statement::Equ { name, operands } => {
                debug_assert!(symbols.contains_key(name));
                debug_assert_eq!(operands.len(), 1);
                continue;
            }
            Statement::Empty => continue,
        };

        let bytes = parse_instruction(tokens, source_line.address, &label_addresses, false)
            .map_err(|e| format!("Line {}: {}", source_line.line_no, e))?;

        assembly_lines.insert(
            source_line.line_no,
            AssemblyLine {
                address: source_line.address,
                bytes: bytes.clone(),
            },
        );

        output.extend(&bytes);

        let is_org = matches!(tokens.first(), Some(Token::Identifier(m)) if m == "ORG");
        if !is_org {
            for (offset, byte) in bytes.iter().enumerate() {
                let address = source_line
                    .address
                    .checked_add(offset as u16)
                    .ok_or_else(|| {
                        format!("Line {}: address space exceeds 64K", source_line.line_no)
                    })?;
                image.push((address, *byte));
            }
        }

        address_to_line.insert(source_line.address, source_line.line_no);
    }

    Ok(AssemblyResult {
        bytes: output,
        symbols,
        address_to_line,
        line_to_address: line_addresses,
        image,
        lines: assembly_lines,
    })
}
