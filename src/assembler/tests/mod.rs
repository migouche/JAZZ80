use crate::assembler::{Operand, Token, assemble, parse_operands, tokenize};

mod ddcb_fdcb;
mod ds_test;
mod scripts;
mod variables;
// =================================================================================
//                               TOKENIZER TESTS
// =================================================================================

#[test]
fn test_tokenize_all_tokens() {
    let code = "LD A, (IX+10)";
    let tokens = tokenize(code).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("LD".to_string()),
            Token::Identifier("A".to_string()),
            Token::Comma,
            Token::OpenParen,
            Token::Identifier("IX".to_string()),
            Token::Plus,
            Token::Number(10),
            Token::CloseParen,
        ]
    );
}

#[test]
fn test_tokenize_numbers() {
    // Decimal
    assert_eq!(tokenize("10").unwrap()[0], Token::Number(10));
    // Hex 0x
    assert_eq!(tokenize("0x10").unwrap()[0], Token::Number(16));
    assert_eq!(tokenize("0XFF").unwrap()[0], Token::Number(255));
    // Hex $
    assert_eq!(tokenize("$10").unwrap()[0], Token::Number(16));
    // Hex h suffix
    assert_eq!(tokenize("10h").unwrap()[0], Token::Number(16));
    assert_eq!(tokenize("0FFh").unwrap()[0], Token::Number(255));
    assert_eq!(tokenize("0A0H").unwrap()[0], Token::Number(160));

    // Edge case: 0 without x
    assert_eq!(tokenize("0").unwrap()[0], Token::Number(0));
    assert_eq!(tokenize("00").unwrap()[0], Token::Number(0));
    assert_eq!(tokenize("010").unwrap()[0], Token::Number(10));

    // words

    assert_eq!(tokenize("0x1234").unwrap()[0], Token::Number(0x1234));
    assert_eq!(tokenize("1234h").unwrap()[0], Token::Number(0x1234));
    assert_eq!(tokenize("FFFFh").unwrap()[0], Token::Number(0xFFFF));
}

#[test]
fn test_tokenize_symbols() {
    let tokens = tokenize(":,()+-").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Colon,
            Token::Comma,
            Token::OpenParen,
            Token::CloseParen,
            Token::Plus,
            Token::Minus,
        ]
    );
}

#[test]
fn test_tokenize_identifiers() {
    let tokens = tokenize("AX_1 LOOP' Z").unwrap();
    assert_eq!(tokens[0], Token::Identifier("AX_1".to_string()));
    assert_eq!(tokens[1], Token::Identifier("LOOP'".to_string()));
    assert_eq!(tokens[2], Token::Identifier("Z".to_string()));
}

#[test]
fn test_tokenize_errors() {
    assert!(tokenize("LD A, @").is_err());
    assert!(tokenize("0xG").is_err());
    assert!(tokenize("$G").is_err());
    // "10z" -> 10, z (valid)
    let t = tokenize("10z").unwrap();
    assert_eq!(t[0], Token::Number(10));
    assert_eq!(t[1], Token::Identifier("Z".to_string()));
}

// =================================================================================
//                            OPERAND PARSING TESTS
// =================================================================================

#[test]
fn test_parse_operands_basic() {
    let tokens = tokenize("A, 10, (HL), (100)").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    match &ops[0] {
        Operand::Register(s) => assert_eq!(s, "A"),
        _ => panic!(),
    }
    match &ops[1] {
        Operand::Immediate(n) => assert_eq!(*n, 10),
        _ => panic!(),
    }
    match &ops[2] {
        Operand::IndirectRegister(s) => assert_eq!(s, "HL"),
        _ => panic!(),
    }
    match &ops[3] {
        Operand::IndirectImmediate(n) => assert_eq!(*n, 100),
        _ => panic!(),
    }
}

#[test]
fn test_parse_operands_index() {
    // (IX+10)
    let tokens = tokenize("(IX+10)").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    match &ops[0] {
        Operand::IndirectIndex(r, d) => {
            assert_eq!(r, "IX");
            assert_eq!(*d, 10);
        }
        _ => panic!(),
    }

    // (IY-5)
    let tokens = tokenize("(IY-5)").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    match &ops[0] {
        Operand::IndirectIndex(r, d) => {
            assert_eq!(r, "IY");
            assert_eq!(*d, -5);
        }
        _ => panic!(),
    }

    // (IX) -> IndirectRegister
    let tokens = tokenize("(IX)").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    match &ops[0] {
        Operand::IndirectRegister(s) => assert_eq!(s, "IX"),
        _ => panic!(),
    }
}

#[test]
fn test_parse_operands_minus_number() {
    let tokens = tokenize("-10").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    // -10 as u16 is 0xFFF6
    match &ops[0] {
        Operand::Immediate(n) => assert_eq!(*n, 0xFFF6),
        _ => panic!(),
    }
}

#[test]
fn test_parse_operands_labels_conditions() {
    let tokens = tokenize("NZ, LOOP, (LOOP)").unwrap();
    let ops = parse_operands(&tokens).unwrap();
    match &ops[0] {
        Operand::Condition(s) => assert_eq!(s, "NZ"),
        _ => panic!(),
    }
    match &ops[1] {
        Operand::Label(s) => assert_eq!(s, "LOOP"),
        _ => panic!(),
    }
    match &ops[2] {
        Operand::IndirectLabel(s) => assert_eq!(s, "LOOP"),
        _ => panic!(),
    }
}

#[test]
fn test_local_labels_use_latest_global_scope() {
    let (bytes, symbols, _, _) =
        assemble("FOO: JP .DONE\n.BODY: NOP\n.DONE: JP .BODY\nBAR: JP .DONE\n.DONE: NOP").unwrap();

    assert_eq!(
        bytes,
        vec![
            0xC3, 0x04, 0x00, 0x00, 0xC3, 0x03, 0x00, 0xC3, 0x0A, 0x00, 0x00
        ]
    );
    assert_eq!(symbols["FOO"].address, 0);
    assert_eq!(symbols["FOO.DONE"].address, 4);
    assert_eq!(symbols["FOO.BODY"].address, 3);
    assert_eq!(symbols["BAR"].address, 7);
    assert_eq!(symbols["BAR.DONE"].address, 10);
}

#[test]
fn test_parse_operands_errors() {
    assert!(parse_operands(&tokenize("A B").unwrap()).is_err()); // missing comma
    assert!(parse_operands(&tokenize("-").unwrap()).is_err()); // minus no number
    assert!(parse_operands(&tokenize("- A").unwrap()).is_err()); // minus non number
    assert!(parse_operands(&tokenize("(10").unwrap()).is_err()); // missing close paren
    assert!(parse_operands(&tokenize("(IX+").unwrap()).is_err()); // missing offset
    assert!(parse_operands(&tokenize("(IX+A)").unwrap()).is_err()); // invalid offset
    assert!(parse_operands(&tokenize("()").unwrap()).is_err()); // empty parens
}

// =================================================================================
//                           INSTRUCTION ENCODING TESTS
// =================================================================================

// Helper to assemble single line
fn asm(code: &str) -> Vec<u8> {
    assemble(code)
        .map(|(bytes, _, _, _)| bytes)
        .unwrap_or_else(|e| panic!("Failed to assemble '{}': {}", code, e))
}

fn asm_err(code: &str) {
    assert!(assemble(code).is_err(), "Expected error for '{}'", code);
}

#[test]
fn test_ld() {
    // LD r, r
    assert_eq!(asm("LD A, B"), vec![0x78]);
    assert_eq!(asm("LD B, A"), vec![0x47]);

    // LD r, n
    assert_eq!(asm("LD B, 10"), vec![0x06, 10]);
    assert_eq!(asm("LD BC, 0x1234"), vec![0x01, 0x34, 0x12]);
    assert_eq!(asm("LD IX, 0x1234"), vec![0xDD, 0x21, 0x34, 0x12]);
    assert_eq!(asm("LD IY, 0x1234"), vec![0xFD, 0x21, 0x34, 0x12]);

    // LD r, (HL)
    assert_eq!(asm("LD A, (HL)"), vec![0x7E]);
    assert_eq!(asm("LD B, (HL)"), vec![0x46]);
    // LD (HL), r
    assert_eq!(asm("LD (HL), A"), vec![0x77]);
    assert_eq!(asm("LD (HL), B"), vec![0x70]);
    // LD (HL), n
    assert_eq!(asm("LD (HL), 10"), vec![0x36, 10]);

    // LD A, (BC)/(DE)
    assert_eq!(asm("LD A, (BC)"), vec![0x0A]);
    assert_eq!(asm("LD A, (DE)"), vec![0x1A]);
    // LD (BC)/(DE), A
    assert_eq!(asm("LD (BC), A"), vec![0x02]);
    assert_eq!(asm("LD (DE), A"), vec![0x12]);

    // LD r, (IX+d)
    assert_eq!(asm("LD A, (IX+5)"), vec![0xDD, 0x7E, 5]);
    assert_eq!(asm("LD B, (IY-3)"), vec![0xFD, 0x46, 0xFD]); // -3 = 0xFD
    // LD (IX+d), r
    assert_eq!(asm("LD (IX+5), A"), vec![0xDD, 0x77, 5]);
    assert_eq!(asm("LD (IY-3), B"), vec![0xFD, 0x70, 0xFD]);
    // LD (IX+d), n
    assert_eq!(asm("LD (IX+5), 10"), vec![0xDD, 0x36, 5, 10]);

    // LD A, (nn) / LD (nn), A
    assert_eq!(asm("LD A, (0x1234)"), vec![0x3A, 0x34, 0x12]);
    assert_eq!(asm("LD (0x1234), A"), vec![0x32, 0x34, 0x12]);
    // LD HL, (nn) / LD (nn), HL
    assert_eq!(asm("LD HL, (0x1234)"), vec![0x2A, 0x34, 0x12]);
    assert_eq!(asm("LD (0x1234), HL"), vec![0x22, 0x34, 0x12]);
    // LD dd, (nn) / LD (nn), dd
    assert_eq!(asm("LD BC, (0x1234)"), vec![0xED, 0x4B, 0x34, 0x12]);
    assert_eq!(asm("LD (0x1234), BC"), vec![0xED, 0x43, 0x34, 0x12]);
    assert_eq!(asm("LD SP, (0x1234)"), vec![0xED, 0x7B, 0x34, 0x12]);
    assert_eq!(asm("LD IX, (0x1234)"), vec![0xDD, 0x2A, 0x34, 0x12]);
    assert_eq!(asm("LD (0x1234), IX"), vec![0xDD, 0x22, 0x34, 0x12]);

    // SP moves
    assert_eq!(asm("LD SP, HL"), vec![0xF9]);
    assert_eq!(asm("LD SP, IX"), vec![0xDD, 0xF9]);
    assert_eq!(asm("LD SP, IY"), vec![0xFD, 0xF9]);

    // Special registers
    assert_eq!(asm("LD I, A"), vec![0xED, 0x47]);
    assert_eq!(asm("LD R, A"), vec![0xED, 0x4F]);
    assert_eq!(asm("LD A, I"), vec![0xED, 0x57]);
    assert_eq!(asm("LD A, R"), vec![0xED, 0x5F]);

    // LD Errors
    asm_err("LD A"); // Not enough ops
    asm_err("LD A, A, A"); // Too many ops
    asm_err("LD BC, DE"); // Invalid r, r 
    asm_err("LD (BC), (DE)"); // Mem to Mem
    asm_err("LD (IX), A"); // Use (IX+0)
}

#[test]
fn test_alu_uni() {
    // INC/DEC r
    assert_eq!(asm("INC A"), vec![0x3C]);
    assert_eq!(asm("DEC B"), vec![0x05]);
    // INC/DEC rr
    assert_eq!(asm("INC BC"), vec![0x03]);
    assert_eq!(asm("DEC SP"), vec![0x3B]);
    assert_eq!(asm("INC IX"), vec![0xDD, 0x23]);
    assert_eq!(asm("DEC IY"), vec![0xFD, 0x2B]);

    // INC/DEC (HL)
    assert_eq!(asm("INC (HL)"), vec![0x34]);
    assert_eq!(asm("DEC (HL)"), vec![0x35]);

    // INC/DEC (IX+d)
    assert_eq!(asm("INC (IX+5)"), vec![0xDD, 0x34, 5]);
    assert_eq!(asm("DEC (IY-5)"), vec![0xFD, 0x35, 0xFB]);

    asm_err("INC 10");
}

#[test]
fn test_add() {
    // ADD A, ...
    assert_eq!(asm("ADD A, B"), vec![0x80]);
    assert_eq!(asm("ADD A, 10"), vec![0xC6, 10]);
    assert_eq!(asm("ADD A, (HL)"), vec![0x86]);
    assert_eq!(asm("ADD A, (IX+1)"), vec![0xDD, 0x86, 1]);

    // ADD HL, ...
    assert_eq!(asm("ADD HL, BC"), vec![0x09]);
    assert_eq!(asm("ADD HL, DE"), vec![0x19]);
    assert_eq!(asm("ADD HL, HL"), vec![0x29]);
    assert_eq!(asm("ADD HL, SP"), vec![0x39]);

    // ADD IX/IY, ...
    assert_eq!(asm("ADD IX, BC"), vec![0xDD, 0x09]);
    assert_eq!(asm("ADD IX, IX"), vec![0xDD, 0x29]);
    assert_eq!(asm("ADD IY, SP"), vec![0xFD, 0x39]);

    asm_err("ADD HL, A");
}

#[test]
fn test_sbc() {
    // SBC A, ...
    assert_eq!(asm("SBC A, B"), vec![0x98]);
    assert_eq!(asm("SBC A, 10"), vec![0xDE, 10]);
    assert_eq!(asm("SBC A, (HL)"), vec![0x9E]);

    // SBC HL, ...
    assert_eq!(asm("SBC HL, BC"), vec![0xED, 0x42]);
    assert_eq!(asm("SBC HL, SP"), vec![0xED, 0x72]);
}

#[test]
fn test_other_alu_bin() {
    // SUB
    assert_eq!(asm("SUB B"), vec![0x90]);
    assert_eq!(asm("SUB 10"), vec![0xD6, 10]);
    assert_eq!(asm("SUB (IX+1)"), vec![0xDD, 0x96, 1]);
    // AND
    assert_eq!(asm("AND C"), vec![0xA1]);
    assert_eq!(asm("AND 0xFF"), vec![0xE6, 0xFF]);
    // XOR
    assert_eq!(asm("XOR A"), vec![0xAF]);
    // OR
    assert_eq!(asm("OR (HL)"), vec![0xB6]);
    // CP
    assert_eq!(asm("CP 5"), vec![0xFE, 5]);
    // IXH/IXL
    assert_eq!(asm("CP IXH"), vec![0xDD, 0xBC]);
    assert_eq!(asm("CP IXL"), vec![0xDD, 0xBD]);
}

#[test]
fn test_control_flow() {
    // JP
    assert_eq!(asm("JP 0x1234"), vec![0xC3, 0x34, 0x12]);
    assert_eq!(asm("JP NZ, 0x1234"), vec![0xC2, 0x34, 0x12]);
    assert_eq!(asm("JP (HL)"), vec![0xE9]);
    assert_eq!(asm("JP (IX)"), vec![0xDD, 0xE9]);

    // JR
    assert_eq!(asm("JR 2"), vec![0x18, 0x00]); // Jump to address 2. PC=0. 2-(0+2)=0.

    // JR NZ, d
    let code = "
        JR NZ, next
        NOP
        next:
        HALT
    ";
    // 20 01 (jump over 1 byte NOP)
    assert_eq!(asm(code), vec![0x20, 0x01, 0x00, 0x76]);

    // DJNZ
    let code = "
        loop:
        DJNZ loop
    ";
    assert_eq!(asm(code), vec![0x10, 0xFE]);

    // CALL
    assert_eq!(asm("CALL 0x1234"), vec![0xCD, 0x34, 0x12]);
    assert_eq!(asm("CALL Z, 0x1234"), vec![0xCC, 0x34, 0x12]);

    // RET
    assert_eq!(asm("RET"), vec![0xC9]);
    assert_eq!(asm("RET NZ"), vec![0xC0]);

    // RST
    assert_eq!(asm("RST 0"), vec![0xC7]);
    assert_eq!(asm("RST 0x38"), vec![0xFF]);
    asm_err("RST 1");
}

#[test]
fn test_stack_misc() {
    // PUSH/POP
    assert_eq!(asm("PUSH BC"), vec![0xC5]);
    assert_eq!(asm("PUSH AF"), vec![0xF5]);
    assert_eq!(asm("POP IX"), vec![0xDD, 0xE1]);

    // EX
    assert_eq!(asm("EX DE, HL"), vec![0xEB]);
    assert_eq!(asm("EX AF, AF'"), vec![0x08]);
    assert_eq!(asm("EX (SP), HL"), vec![0xE3]);
    assert_eq!(asm("EX (SP), IX"), vec![0xDD, 0xE3]);

    // Misc
    assert_eq!(asm("HALT"), vec![0x76]);
    assert_eq!(asm("NOP"), vec![0x00]);
    assert_eq!(asm("DI"), vec![0xF3]);
    assert_eq!(asm("EI"), vec![0xFB]);
    assert_eq!(asm("EXX"), vec![0xD9]);
    assert_eq!(asm("DAA"), vec![0x27]);
    assert_eq!(asm("CPL"), vec![0x2F]);
    assert_eq!(asm("CCF"), vec![0x3F]);
    assert_eq!(asm("SCF"), vec![0x37]);

    // Rotates
    assert_eq!(asm("RLA"), vec![0x17]);
    assert_eq!(asm("RRA"), vec![0x1F]);
    assert_eq!(asm("RLCA"), vec![0x07]);
    assert_eq!(asm("RRCA"), vec![0x0F]);

    // Block
    assert_eq!(asm("LDI"), vec![0xED, 0xA0]);
    assert_eq!(asm("LDIR"), vec![0xED, 0xB0]);
    assert_eq!(asm("CPI"), vec![0xED, 0xA1]);
    assert_eq!(asm("CPIR"), vec![0xED, 0xB1]);
}

#[test]
fn test_io() {
    assert_eq!(asm("IN A, (0x10)"), vec![0xDB, 0x10]);
    assert_eq!(asm("IN B, (C)"), vec![0xED, 0x40]);
    assert_eq!(asm("OUT (0x10), A"), vec![0xD3, 0x10]);
    assert_eq!(asm("OUT (C), B"), vec![0xED, 0x41]);
}

#[test]
fn test_assembler_errors_logic() {
    // Duplicate label
    let code = "
        lbl:
        lbl:
    ";
    asm_err(code);

    // Unknown mnemonic
    asm_err("XYZ");

    // Label as operand
    let code = "
        LD A, LBL
    ";
    asm_err(code);

    // JR out of range (Pass 2)
    let mut code = String::from("start: JR end\n");
    for _ in 0..130 {
        code.push_str("NOP\n");
    }
    code.push_str("end: HALT");
    asm_err(&code);
}

#[test]
fn test_jump_limits() {
    // DJNZ out of range
    let mut code = String::from("start: DJNZ end\n");
    for _ in 0..130 {
        code.push_str("NOP\n");
    }
    code.push_str("end: HALT");
    asm_err(&code);
}

#[test]
fn test_extensive_error_coverage() {
    // -- LD Errors --
    // Invalid LD register combos
    asm_err("LD BC, DE"); // ld r16, r16 invalid usually (except SP/HL logic, but BC/DE nope)
    asm_err("LD (BC), (DE)"); // Mem to Mem
    asm_err("LD (IX), A"); // Must be (IX+d) in this assembler syntax
    asm_err("LD A, (IX)"); // Must be (IX+d)

    // Invalid LD special
    asm_err("LD I, B");
    asm_err("LD R, B");

    // LD parsing specific logic
    asm_err("LD B, (0x1234)"); // Only A, HL, RP, IX, IY allowed for (nn)
    asm_err("LD (0x1234), B"); // Only A, HL, RP, IX, IY allowed for (nn)
    asm_err("LD (BC), HL"); // Invalid LD (reg), r

    // -- ALU Errors --
    // INC/DEC
    asm_err("INC 10"); // Immediate
    asm_err("INC AF"); // Invalid reg

    // ADD
    asm_err("ADD HL, A"); // Invalid
    asm_err("ADD IX, HL"); // Invalid (IX adds with BC, DE, IX, SP)
    asm_err("ADD A, B, C"); // Too many ops

    // Binary ALU
    // SUB A, B is valid Z80 assembly (alias for SUB B)
    assert_eq!(asm("SUB A, B"), vec![0x90]);
    asm_err("SUB SP"); // Invalid ALU reg

    // -- Jump/Call/Ret Errors --
    // JP
    asm_err("JP AF"); // Invalid JP target
    // Z80 support PO. My assembler supports PO.
    assert_eq!(asm("JP PO, 0x1234"), vec![0xE2, 0x34, 0x12]);

    asm_err("JP NX, 0x1234"); // Invalid condition (NX is made up)

    // JR
    asm_err("JR PO, 0"); // Invalid condition for JR (only NZ, Z, NC, C)
    asm_err("JR A, 0"); // Invalid condition arg

    // DJNZ
    asm_err("DJNZ A, 0"); // DJNZ takes 1 op (the displacement/target)

    // CALL
    asm_err("CALL BC"); // Invalid CALL target (must be imm/label/ind-imm)
    asm_err("CALL POO, 0"); // Bad condition

    // RET
    asm_err("RET 10"); // Invalid RET arg

    // RST
    asm_err("RST 0x01"); // Invalid RST address
    asm_err("RST A"); // Invalid RST operand

    // -- Stack/Ex/IO Errors --
    // PUSH/POP
    asm_err("PUSH A"); // Invalid PUSH reg (must be AF)
    asm_err("POP B"); // Invalid POP reg

    // EX
    asm_err("EX A, B"); // Invalid EX
    asm_err("EX (SP), BC"); // Invalid EX (SP)

    // IN/OUT
    asm_err("IN (HL), A"); // IN can be IN A, (n) or IN r, (C). Not (HL)
    asm_err("OUT (BC), A"); // OUT can be OUT (n), A or OUT (C), r. Not (BC)
}
