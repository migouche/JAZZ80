use crate::assembler::{SymbolType, assemble_with_metadata};

// Helper to assemble single line
fn asm(code: &str) -> Vec<u8> {
    assemble_with_metadata(code)
        .map(|result| result.bytes)
        .unwrap_or_else(|e| panic!("Failed to assemble '{}': {}", code, e))
}

#[test]
fn test_org_simple() {
    let code = "
        ORG 0x0005
        DB 0xAA
    ";
    let res = asm(code);
    assert_eq!(res.len(), 6);
    assert_eq!(res[0..5], [0, 0, 0, 0, 0]);
    assert_eq!(res[5], 0xAA);
}

#[test]
fn test_variables_scenario() {
    let code = "
        ORG 0x0000
        JP START
    N:  DB 10
    RESULT: DW 0
    START:
        LD A, (N)
        HALT
    ";
    let res = asm(code);
    /*
       0000: JP 0006   (C3 06 00)
       0003: N: 10     (0A)
       0004: RESULT: 0 (00 00)
       0006: START:
       0006: LD A, (0003) (3A 03 00)
       0009: HALT      (76)
    */
    assert_eq!(
        res,
        vec![0xC3, 0x06, 0x00, 0x0A, 0x00, 0x00, 0x3A, 0x03, 0x00, 0x76]
    );
}

#[test]
fn test_db_multiple() {
    let code = "DB 1, 2, 3";
    let res = asm(code);
    assert_eq!(res, vec![1, 2, 3]);
}

#[test]
fn test_db_string() {
    let code = "DB \"Hello\"";
    let res = asm(code);
    assert_eq!(res, vec![72, 101, 108, 108, 111]); // ASCII Hello
}

#[test]
fn test_dw_basic() {
    let code = "DW 0x1234, 0x5678";
    let res = asm(code);
    assert_eq!(res, vec![0x34, 0x12, 0x78, 0x56]);
}

#[test]
fn test_label_forward_ref_in_db() {
    let code = "
        DB END
        END: NOP
    ";
    let res = asm(code);
    assert_eq!(res, vec![0x01, 0x00]);
}

#[test]
fn test_org_gap() {
    let code = "
        EXX
        ORG 0x0004
        EXX
    ";
    let res = asm(code);
    assert_eq!(res, vec![0xD9, 0x00, 0x00, 0x00, 0xD9]);
}

#[test]
fn test_equ_constants() {
    let code = "
        TERM_DATA EQU 80h
        TERM_STATUS EQU 81h
        LD A, 2Ah
        OUT (TERM_DATA), A
        IN A, (TERM_STATUS)
        JP DONE
    DONE:
        RET
    ";
    let result = assemble_with_metadata(code).unwrap();

    assert_eq!(
        result.bytes,
        vec![0x3E, 0x2A, 0xD3, 0x80, 0xDB, 0x81, 0xC3, 0x09, 0x00, 0xC9]
    );
    assert_eq!(result.symbols["TERM_DATA"].address, 0x80);
    assert_eq!(result.symbols["TERM_STATUS"].address, 0x81);
}

#[test]
fn late_label_referenced_by_early_jump_assembles_with_many_symbols() {
    let mut code = String::from("JP TARGET\n");
    for i in 0..50 {
        code.push_str(&format!("SYM{}: DB {}\n", i, i));
    }
    code.push_str("TARGET: NOP\n");

    let result = assemble_with_metadata(&code).unwrap();

    assert_eq!(result.symbols.len(), 51);
    assert_eq!(result.symbols["TARGET"].address, 53);
    assert_eq!(result.bytes[0..3], [0xC3, 53, 0]);
    assert_eq!(result.bytes.len(), 54);
}

#[test]
fn db_continuation_with_three_rows_keeps_parent_and_row_symbols() {
    let code = "DATA: DB 0x11, 0x22\nDB 0x33\nDB 0x44\nDB 0x55";
    let result = assemble_with_metadata(code).unwrap();

    assert_eq!(result.bytes, vec![0x11, 0x22, 0x33, 0x44, 0x55]);

    // The labelled row owns the first two bytes; each continuation row gets a
    // `LABEL[line]` sub-symbol at its own address.
    assert_eq!(result.symbols["DATA"].address, 0);
    assert_eq!(result.symbols["DATA"].kind, SymbolType::Array(2));
    assert_eq!(result.symbols["DATA[2]"].address, 2);
    assert_eq!(result.symbols["DATA[2]"].kind, SymbolType::Byte);
    assert_eq!(result.symbols["DATA[3]"].address, 3);
    assert_eq!(result.symbols["DATA[3]"].kind, SymbolType::Byte);
    assert_eq!(result.symbols["DATA[4]"].address, 4);
    assert_eq!(result.symbols["DATA[4]"].kind, SymbolType::Byte);
}

#[test]
fn equ_referencing_a_later_label_is_an_error() {
    let error = assemble_with_metadata("VALUE EQU LATER\nLATER: NOP").unwrap_err();
    assert_eq!(error, "Line 1: Label not found: LATER");
}

#[test]
fn string_data_label_continuation_totals_three_bytes() {
    // The parent keeps the first row's `String(len)`; the continuation row is
    // recorded as `TEXT[line]` so the GUI can render it on its own row. Their
    // lengths still add up to the full three-byte string.
    let result = assemble_with_metadata("TEXT: DB \"Hi\"\nDB \"!\"").unwrap();

    assert_eq!(result.bytes, b"Hi!");
    assert_eq!(result.symbols["TEXT"].kind, SymbolType::String(2));
    assert_eq!(result.symbols["TEXT[2]"].address, 2);
    assert_eq!(result.symbols["TEXT[2]"].kind, SymbolType::String(1));

    let total_len: usize = ["TEXT", "TEXT[2]"]
        .iter()
        .map(|name| match result.symbols[*name].kind {
            SymbolType::String(len) => len,
            other => panic!("expected a string symbol, got {other:?}"),
        })
        .sum();
    assert_eq!(total_len, 3);
}
