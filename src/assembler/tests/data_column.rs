use crate::assembler::assemble_with_metadata;

/// The listing Data column for a 1-based source line.
fn line(code: &str, n: usize) -> String {
    let result = assemble_with_metadata(code).expect("assembles");
    result
        .lines
        .get(&n)
        .unwrap_or_else(|| panic!("no assembly line {n} for {code:?}"))
        .data_column
        .clone()
}

#[test]
fn single_byte_instruction_renders_its_opcode() {
    assert_eq!(line("NOP", 1), "00");
}

#[test]
fn immediates_render_after_the_opcode() {
    assert_eq!(line("JP 0x1234", 1), "C3 34 12");
}

#[test]
fn parenthesised_second_operand_uses_a_comma() {
    assert_eq!(line("LD A, (0x1234)", 1), "3A, (34 12)");
}

#[test]
fn parenthesised_first_operand_has_no_comma() {
    assert_eq!(line("LD (0x1234), A", 1), "32 (34 12)");
}

#[test]
fn indexed_load_keeps_the_prefix_and_uses_a_comma() {
    assert_eq!(line("LD A, (IX+5)", 1), "DD7E, (05)");
}

#[test]
fn indexed_store_keeps_the_prefix_and_has_no_comma() {
    assert_eq!(line("LD (IX+5), A", 1), "DD77 (05)");
}

#[test]
fn register_indirect_instruction_has_no_argument() {
    assert_eq!(line("LD A, (HL)", 1), "7E");
}

#[test]
fn byte_directive_lists_bytes() {
    assert_eq!(line("DB 1, 2", 1), "01, 02");
}

#[test]
fn string_byte_directive_lists_the_string_bytes() {
    assert_eq!(line("DEFB \"AB\"", 1), "41, 42");
}

#[test]
fn single_word_directive_renders_little_endian() {
    assert_eq!(line("DW 0x1234", 1), "1234");
}

#[test]
fn multiple_word_directive_renders_each_word_little_endian() {
    assert_eq!(line("DW 0x1234, 0x5678", 1), "1234, 5678");
}

#[test]
fn storage_directive_renders_like_an_instruction() {
    assert_eq!(line("DS 3", 1), "00 00 00");
}

#[test]
fn data_column_shows_at_most_eight_bytes() {
    assert_eq!(
        line("DB 1,2,3,4,5,6,7,8,9", 1),
        "01, 02, 03, 04, 05, 06, 07, 08"
    );
}

#[test]
fn labelled_instruction_keeps_the_byte_formatting() {
    assert_eq!(line("START: NOP", 1), "00");
}

#[test]
fn labelled_data_directive_keeps_the_byte_formatting() {
    assert_eq!(line("START: DB 1, 2", 1), "01, 02");
}

#[test]
fn origin_has_no_data_column() {
    assert_eq!(line("ORG 0", 1), "");
}

#[test]
fn comment_only_line_has_no_assembly_entry() {
    let result = assemble_with_metadata("; note").expect("assembles");
    assert!(!result.lines.contains_key(&1));
}

#[test]
fn tab_separated_directive_is_classified_by_token() {
    assert_eq!(line("DB\t1, 2", 1), "01, 02");
}

#[test]
fn string_containing_equ_is_not_blanked_by_text_heuristic() {
    assert_eq!(line("DB \" EQU \"", 1), "20, 45, 51, 55, 20");
}

#[test]
fn directive_without_separating_whitespace_is_classified_by_token() {
    assert_eq!(line("DB\"AB\"", 1), "41, 42");
}
