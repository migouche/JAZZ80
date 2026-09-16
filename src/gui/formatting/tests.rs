use super::binary_hexdump;

#[test]
fn empty_input_has_no_rows() {
    assert_eq!(binary_hexdump(&[]), "");
}

#[test]
fn formats_hex_and_ascii_columns() {
    assert_eq!(
        binary_hexdump(&[0x00, b'A', b' ', 0x7f]),
        "0000: 00 41 20 7F                                     .A ."
    );
}

#[test]
fn starts_each_row_at_a_16_byte_boundary() {
    let bytes: Vec<u8> = (0..=16).collect();
    let dump = binary_hexdump(&bytes);

    assert!(dump.starts_with("0000: "));
    assert!(dump.contains("0010: 10"));
    assert_eq!(dump.lines().count(), 2);
}
