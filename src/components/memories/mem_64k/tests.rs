use super::*;

#[test]
fn test_mem_64k_read_write() {
    let mut memory = Mem64k::new();
    memory.write(0x1234, 0xAB);
    assert_eq!(memory.read(0x1234), 0xAB);

    memory.write_word(0x1234, 0xCDEF);
    assert_eq!(memory.read_word(0x1234), 0xCDEF);

    // test little-endian behavior
    memory.write_word(0x2000, 0x3412);
    assert_eq!(memory.read(0x2000), 0x12);
    assert_eq!(memory.read(0x2001), 0x34);

    memory.write(0x3000, 0x78);
    memory.write(0x3001, 0x56);
    assert_eq!(memory.read_word(0x3000), 0x5678);
}
