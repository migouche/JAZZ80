use super::{binary_hexdump, parse_hexdump};

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

#[test]
fn hexdump_round_trips() {
    for bytes in [
        vec![],
        vec![0x00],
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        (0..=255u8).collect::<Vec<_>>(),
    ] {
        let dump = binary_hexdump(&bytes);
        assert_eq!(parse_hexdump(&dump).unwrap_or_default(), bytes);
    }
}

#[test]
fn parse_hexdump_rejects_garbage() {
    assert!(parse_hexdump("0000: ZZ").is_none());
}

#[test]
fn parse_hexdump_accepts_an_empty_image() {
    assert_eq!(parse_hexdump(""), Some(Vec::new()));
}
