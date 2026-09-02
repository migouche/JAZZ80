use crate::cpu::tests::setup_cpu;
use crate::cpu::{AddressingMode, GPR, IndexRegister, RegisterPair, SynchronousComponent};
use rstest::rstest;

const ADD_HL_BC_OPCODE: u8 = 0x09;
const ADD_HL_DE_OPCODE: u8 = 0x19;
const ADD_HL_HL_OPCODE: u8 = 0x29;
const ADD_HL_SP_OPCODE: u8 = 0x39;

#[rstest]
// ADD HL, BC
#[case::add_hl_bc(
    ADD_HL_BC_OPCODE,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x1000,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x2000,
    0x00,
    0x3000,
    0b00100000
)]
// H=0, C=0, N=0. Result=0x3000. High byte 0x30 (0011 0000). X=0, Y=1. Flags: Y(0x20).

// ADD HL, HL (Double)
#[case::add_hl_hl(
    ADD_HL_HL_OPCODE,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x1000,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x1000,
    0x00,
    0x2000,
    0b00100000
)] // Result 0x2000. High 0x20. Y=1, X=0. Flags 0x20.

// ADD HL, DE (Overflow 16-bit)
#[case::add_hl_de_carry(
    ADD_HL_DE_OPCODE,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0xFFFF,
    AddressingMode::RegisterPair(RegisterPair::DE),
    0x0001,
    0x00,
    0x0000,
    0b00010001
)]
// Result 0x0000. High 0x00. Y=0, X=0. C=1. H=1 (0xFFF+1=0x1000). Flags 0x11.

// ADD HL, SP (Half Carry)
// 0x0FFF + 0x0001 = 0x1000. Carry from bit 11.
#[case::add_hl_sp_half_carry(
    ADD_HL_SP_OPCODE,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x0FFF,
    AddressingMode::RegisterPair(RegisterPair::SP),
    0x0001,
    0x00,
    0x1000,
    0b00010000
)] // Result 0x1000. High 0x10. Y=0, X=0. H=1. Flags 0x10.

// ADD IX, BC
#[case::add_ix_bc(
    ADD_HL_BC_OPCODE,
    AddressingMode::IndexRegister(IndexRegister::IX),
    0x1000,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x2000,
    0x00,
    0x3000,
    0b00100000
)]
// Note: Opcode for ADD IX, BC is DD 09. But test setup handles prefix if we pass it? No, we pass opcode.
// We need to handle prefix in test setup or pass full opcode sequence.
// The test helper usually writes opcode to memory.
// So for IX, we need to write 0xDD, 0x09.

// ADD IY, DE
// Opcode FD 19.

fn test_add_16(
    #[case] opcode: u8,
    #[case] dest: AddressingMode,
    #[case] initial_dest_val: u16,
    #[case] src: AddressingMode,
    #[case] initial_src_val: u16,
    #[case] initial_flags: u8,
    #[case] expected_val: u16,
    #[case] expected_flags: u8,
) {
    let mut cpu = setup_cpu();
    cpu.set_register(GPR::F, initial_flags);

    // Setup registers
    match dest {
        AddressingMode::RegisterPair(rp) => cpu.set_register_pair(rp, initial_dest_val),
        AddressingMode::IndexRegister(r) => cpu.set_index_register(r, initial_dest_val),
        _ => panic!("Invalid dest setup"),
    }

    match src {
        AddressingMode::RegisterPair(rp) => cpu.set_register_pair(rp, initial_src_val),
        AddressingMode::IndexRegister(r) => cpu.set_index_register(r, initial_src_val),
        _ => panic!("Invalid src setup"),
    }

    // Handle prefixes for IX/IY
    let mut pc_offset = 0;
    match dest {
        AddressingMode::IndexRegister(IndexRegister::IX) => {
            cpu.memory.borrow_mut().write(0x0000, 0xDD);
            pc_offset = 1;
        }
        AddressingMode::IndexRegister(IndexRegister::IY) => {
            cpu.memory.borrow_mut().write(0x0000, 0xFD);
            pc_offset = 1;
        }
        _ => {}
    }

    cpu.memory.borrow_mut().write(pc_offset, opcode);

    // Run
    cpu.tick(); // If prefix, tick consumes prefix? No, tick executes one instruction cycle.
    // If prefix is present, `fetch` gets prefix, `decode` calls `decode_dd`, which fetches next opcode.
    // So one `tick` should handle the whole prefixed instruction because `decode_dd` calls `fetch` recursively?
    // Let's check `decode_dd` implementation.
    // `decode_dd` calls `self.fetch()` and then `self.decode(next_opcode)`.
    // So yes, one `tick` handles it.

    let result_val = match dest {
        AddressingMode::RegisterPair(rp) => cpu.get_register_pair(rp),
        AddressingMode::IndexRegister(r) => cpu.get_index_register(r),
        _ => panic!("Invalid dest check"),
    };

    assert_eq!(result_val, expected_val, "Value mismatch");

    // Mask out preserved flags to check only affected ones?
    // Or check exact match including preserved ones.
    // The test cases assume initial_flags=0, so preserved flags are 0.
    // But we should verify preservation.
    // Let's add a case with non-zero initial flags.

    assert_eq!(
        cpu.get_register(GPR::F),
        expected_flags,
        "Flags mismatch. Expected {:08b}, Got {:08b}",
        expected_flags,
        cpu.get_register(GPR::F)
    );
}

#[rstest]
// ADD HL, BC with preserved flags
#[case::add_hl_bc_preserved(
    0x09,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x1000,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x2000,
    0b11000100,
    0x3000,
    0b11100100
)]
// Initial: S=1, Z=1, P/V=1. (0xC4).
// Result 0x3000. Y=1.
// Expected: S=1, Z=1, Y=1, P/V=1. -> 11100100 (0xE4).

fn test_add_16_preserved(
    #[case] opcode: u8,
    #[case] dest: AddressingMode,
    #[case] initial_dest_val: u16,
    #[case] src: AddressingMode,
    #[case] initial_src_val: u16,
    #[case] initial_flags: u8,
    #[case] expected_val: u16,
    #[case] expected_flags: u8,
) {
    test_add_16(
        opcode,
        dest,
        initial_dest_val,
        src,
        initial_src_val,
        initial_flags,
        expected_val,
        expected_flags,
    );
}
