use crate::assembler::assemble_with_metadata;
use rstest::rstest;

fn asm(code: &str) -> Vec<u8> {
    assemble_with_metadata(code)
        .map(|result| result.bytes)
        .unwrap_or_else(|e| panic!("Failed to assemble '{}': {}", code, e))
}

#[rstest]
#[case("RLC (IX+5)",        vec![0xDD, 0xCB, 0x05, 0x06])] // RLC (IX+d)
#[case("RRC (IY-2)",        vec![0xFD, 0xCB, 0xFE, 0x0E])] // RRC (IY+d)
#[case("RLC (IX+5), B",     vec![0xDD, 0xCB, 0x05, 0x00])] // Undocumented copy to B
#[case("BIT 1, (IX+0)",     vec![0xDD, 0xCB, 0x00, 0x4E])] // BIT b, (IX+d)
#[case("SET 7, (IY+10), A", vec![0xFD, 0xCB, 0x0A, 0xFF])] // SET b, (IY+d), r
#[case("SLL (IX+2)",        vec![0xDD, 0xCB, 0x02, 0x36])] // SLL (IX+d)
#[case("SLL (IX+2), H",     vec![0xDD, 0xCB, 0x02, 0x34])] // SLL (IX+d), H
#[case("RES 0, (IX-1)",     vec![0xDD, 0xCB, 0xFF, 0x86])] // RES b, (IX+d)
fn test_asm_single_cases(#[case] code: &str, #[case] expected: Vec<u8>) {
    let res = asm(code);
    assert_eq!(res, expected);
}

#[rstest]
// RLC (00), RRC (08), RL (10), RR (18), SLA (20), SRA (28), SLL (30), SRL (38)
// All with suffix 06 (memory target)
#[case("RLC (IX+0)", vec![0xDD, 0xCB, 0x00, 0x06])]
#[case("RRC (IX+1)", vec![0xDD, 0xCB, 0x01, 0x0E])]
#[case("RL  (IX+2)", vec![0xDD, 0xCB, 0x02, 0x16])]
#[case("RR  (IX+3)", vec![0xDD, 0xCB, 0x03, 0x1E])]
#[case("SLA (IX+4)", vec![0xDD, 0xCB, 0x04, 0x26])]
#[case("SRA (IX+5)", vec![0xDD, 0xCB, 0x05, 0x2E])]
#[case("SLL (IX+6)", vec![0xDD, 0xCB, 0x06, 0x36])] // Undocumented SLL/SL1
#[case("SRL (IX+7)", vec![0xDD, 0xCB, 0x07, 0x3E])]
fn test_all_rot_shift_mnemonics_ix(#[case] code: &str, #[case] expected: Vec<u8>) {
    let res = asm(code);
    assert_eq!(res, expected);
}

#[rstest]
// Test undocumented versions on IY with various registers
#[case("RLC (IY+0), B", vec![0xFD, 0xCB, 0x00, 0x00])]
#[case("RRC (IY+0), C", vec![0xFD, 0xCB, 0x00, 0x09])]
#[case("RL  (IY+0), D", vec![0xFD, 0xCB, 0x00, 0x12])]
#[case("RR  (IY+0), E", vec![0xFD, 0xCB, 0x00, 0x1B])]
#[case("SLA (IY+0), H", vec![0xFD, 0xCB, 0x00, 0x24])]
#[case("SRA (IY+0), L", vec![0xFD, 0xCB, 0x00, 0x2D])]
#[case("SLL (IY+0), A", vec![0xFD, 0xCB, 0x00, 0x37])]
#[case("SRL (IY+0), B", vec![0xFD, 0xCB, 0x00, 0x38])]
fn test_all_rot_shift_mnemonics_iy_undocumented(#[case] code: &str, #[case] expected: Vec<u8>) {
    let res = asm(code);
    assert_eq!(res, expected);
}

#[rstest]
#[case(0, 0x46)] // 40 + 00 + 6
#[case(1, 0x4E)] // 40 + 08 + 6
#[case(2, 0x56)]
#[case(3, 0x5E)]
#[case(4, 0x66)]
#[case(5, 0x6E)]
#[case(6, 0x76)]
#[case(7, 0x7E)] // 40 + 38 + 6
fn test_bit_all_positions(#[case] bit: u8, #[case] expected_suffix: u8) {
    let code = format!("BIT {}, (IX-1)", bit);
    let res = asm(&code);
    assert_eq!(res, vec![0xDD, 0xCB, 0xFF, expected_suffix]);
}

#[rstest]
#[case("RES 0, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0x86])]
#[case("RES 3, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0x9E])]
#[case("RES 7, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0xBE])]
#[case("SET 0, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0xC6])]
#[case("SET 4, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0xE6])]
#[case("SET 7, (IY+10)", vec![0xFD, 0xCB, 0x0A, 0xFE])]
fn test_set_res_all_positions_iy(#[case] code: &str, #[case] expected: Vec<u8>) {
    let res = asm(code);
    assert_eq!(res, expected);
}

#[rstest]
// Test SET 5, (IX+0), r
// SET base C0. Bit 5 -> 5<<3 = 28 (40). Opcode base = C0 + 28 = E8.
#[case("B", 0)]
#[case("C", 1)]
#[case("D", 2)]
#[case("E", 3)]
#[case("H", 4)]
#[case("L", 5)]
#[case("A", 7)]
fn test_set_res_undocumented_all_regs(#[case] r_name: &str, #[case] r_code: u8) {
    let code = format!("SET 5, (IX+0), {}", r_name);
    let expected_opcode = 0xE8 | r_code;
    let res = asm(&code);
    assert_eq!(res, vec![0xDD, 0xCB, 0x00, expected_opcode]);
}

#[rstest]
#[case("RLC (IX+127)",  vec![0xDD, 0xCB, 0x7F, 0x06])]
#[case("RLC (IX-128)",  vec![0xDD, 0xCB, 0x80, 0x06])]
#[case("RLC (IX-1)",    vec![0xDD, 0xCB, 0xFF, 0x06])]
#[case("RLC (IX+0)",    vec![0xDD, 0xCB, 0x00, 0x06])]
fn test_offsets_boundary(#[case] code: &str, #[case] expected: Vec<u8>) {
    let res = asm(code);
    assert_eq!(res, expected);
}

#[rstest]
#[case("BIT 0, (IX+0), B", "BIT does not support 3 operands")]
#[case("RLC (IX+A)", "Expected number offset")]
#[case("SET 1, (IX+0), IX", "Invalid register 2")]
fn test_errors(#[case] code: &str, #[case] error_msg: &str) {
    let result = std::panic::catch_unwind(|| asm(code));
    match result {
        Ok(_) => panic!("Code '{}' should have panicked but didn't", code),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.as_str()
            } else {
                panic!("Unknown panic type");
            };
            assert!(
                msg.contains(error_msg),
                "Panic message '{}' did not contain '{}'",
                msg,
                error_msg
            );
        }
    }
}
