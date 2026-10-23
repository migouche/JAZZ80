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

#[test]
fn take_dirty_returns_each_written_byte_once() {
    let mut memory = Mem64k::new();
    assert_eq!(memory.take_dirty(), Some(Vec::new()));

    memory.write(0x0000, 0x11);
    memory.write(0x003F, 0x22);
    memory.write(0x0040, 0x33);
    memory.write(0x1234, 0x44);
    memory.write(0x1234, 0x55);
    memory.write(0xFFFF, 0x66);

    let mut changed = memory.take_dirty().unwrap();
    changed.sort_unstable();
    assert_eq!(
        changed,
        vec![
            (0x0000, 0x11),
            (0x003F, 0x22),
            (0x0040, 0x33),
            (0x1234, 0x55),
            (0xFFFF, 0x66),
        ]
    );
    assert_eq!(memory.take_dirty(), Some(Vec::new()));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn mark_dirty_requeues_a_byte() {
    let mut memory = Mem64k::new();
    memory.write(0x1234, 0xAB);
    let _ = memory.take_dirty();

    memory.mark_dirty(0x1234);

    assert_eq!(memory.take_dirty(), Some(vec![(0x1234, 0xAB)]));
}
