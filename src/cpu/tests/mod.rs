use crate::{cpu::Z80A, traits::MemoryMapper};

pub mod alu_tests;
pub mod block_instructions;
pub mod ddcb_fdcb;
pub mod decoding;
pub mod instructions;
pub mod interrupts;
pub mod io;
pub mod registers;
pub mod stack;

struct MockMemory {
    data: [u8; 0x10000],
}

impl MockMemory {
    fn new() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl MemoryMapper for MockMemory {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.data[address as usize] = data;
    }
}

fn setup_cpu() -> Z80A {
    Z80A::new(MockMemory::new())
}

#[test]
fn test_mock_memory_read_write() {
    let mut memory = MockMemory::new();
    memory.write(0x1234, 0xAB);
    assert_eq!(memory.read(0x1234), 0xAB);

    memory.write_word(0x1234, 0xCDEF);
    assert_eq!(memory.read_word(0x1234), 0xCDEF);
}
mod halt_test;
pub mod nmi_tests;
