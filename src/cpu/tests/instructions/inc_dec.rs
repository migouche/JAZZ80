use crate::cpu::tests::setup_cpu;
use crate::cpu::{AddressingMode, GPR, RegisterPair, SynchronousComponent};
use rstest::rstest;

#[rstest]
// 8-bit INC
#[case::inc_b_0(0x04, AddressingMode::Register(GPR::B), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_b_overflow(0x04, AddressingMode::Register(GPR::B), 0x7F, 0x00, 0x80, 0b10010100)] // S=1, H=1, PV=1
#[case::inc_b_zero(0x04, AddressingMode::Register(GPR::B), 0xFF, 0x01, 0x00, 0b01010001)] // Z=1, H=1, C=1 (preserved)
#[case::inc_c(0x0C, AddressingMode::Register(GPR::C), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_d(0x14, AddressingMode::Register(GPR::D), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_e(0x1C, AddressingMode::Register(GPR::E), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_h(0x24, AddressingMode::Register(GPR::H), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_l(0x2C, AddressingMode::Register(GPR::L), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_a(0x3C, AddressingMode::Register(GPR::A), 0x00, 0x00, 0x01, 0b00000000)]
#[case::inc_hl_indirect(
    0x34,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x10,
    0x00,
    0x11,
    0b00000000
)]
// 8-bit DEC
#[case::dec_b_0(0x05, AddressingMode::Register(GPR::B), 0x01, 0x00, 0x00, 0b01000010)] // Z=1, N=1
#[case::dec_b_underflow(0x05, AddressingMode::Register(GPR::B), 0x80, 0x00, 0x7F, 0b00111110)] // Y=1, H=1, X=1, PV=1, N=1
#[case::dec_b_half_carry(0x05, AddressingMode::Register(GPR::B), 0x10, 0x00, 0x0F, 0b00011010)] // Y=1, H=1, N=1
#[case::dec_c(0x0D, AddressingMode::Register(GPR::C), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_d(0x15, AddressingMode::Register(GPR::D), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_e(0x1D, AddressingMode::Register(GPR::E), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_h(0x25, AddressingMode::Register(GPR::H), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_l(0x2D, AddressingMode::Register(GPR::L), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_a(0x3D, AddressingMode::Register(GPR::A), 0x01, 0x00, 0x00, 0b01000010)]
#[case::dec_hl_indirect(
    0x35,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x11,
    0x00,
    0x10,
    0b00000010
)]

fn test_inc_dec_8(
    #[case] opcode: u8,
    #[case] target: AddressingMode,
    #[case] initial_val: u8,
    #[case] initial_flags: u8,
    #[case] expected_val: u8,
    #[case] expected_flags: u8,
) {
    let mut cpu = setup_cpu();
    cpu.set_register(GPR::F, initial_flags);

    match target {
        AddressingMode::Register(r) => cpu.set_register(r, initial_val),
        AddressingMode::RegisterIndirect(rp) => {
            let addr = 0x1000;
            cpu.set_register_pair(rp, addr);
            cpu.memory.borrow_mut().write(addr, initial_val);
        }
        _ => panic!("Unsupported addressing mode for test setup"),
    }

    cpu.memory.borrow_mut().write(0x0000, opcode);
    cpu.tick();

    let result_val = match target {
        AddressingMode::Register(r) => cpu.get_register(r),
        AddressingMode::RegisterIndirect(rp) => {
            let addr = cpu.get_register_pair(rp);
            cpu.memory.borrow().read(addr)
        }
        _ => panic!("Unsupported addressing mode for test check"),
    };

    assert_eq!(result_val, expected_val, "Value mismatch");
    assert_eq!(cpu.get_register(GPR::F), expected_flags, "Flags mismatch");
}

#[rstest]
// 16-bit INC
#[case::inc_bc(
    0x03,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x0000,
    0x00,
    0x0001,
    0x00
)]
#[case::inc_de(
    0x13,
    AddressingMode::RegisterPair(RegisterPair::DE),
    0x0000,
    0x00,
    0x0001,
    0x00
)]
#[case::inc_hl(
    0x23,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x0000,
    0x00,
    0x0001,
    0x00
)]
#[case::inc_sp(
    0x33,
    AddressingMode::RegisterPair(RegisterPair::SP),
    0x0000,
    0x00,
    0x0001,
    0x00
)]
#[case::inc_bc_overflow(
    0x03,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0xFFFF,
    0xFF,
    0x0000,
    0xFF
)]
// 16-bit DEC
#[case::dec_bc(
    0x0B,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x0001,
    0x00,
    0x0000,
    0x00
)]
#[case::dec_de(
    0x1B,
    AddressingMode::RegisterPair(RegisterPair::DE),
    0x0001,
    0x00,
    0x0000,
    0x00
)]
#[case::dec_hl(
    0x2B,
    AddressingMode::RegisterPair(RegisterPair::HL),
    0x0001,
    0x00,
    0x0000,
    0x00
)]
#[case::dec_sp(
    0x3B,
    AddressingMode::RegisterPair(RegisterPair::SP),
    0x0001,
    0x00,
    0x0000,
    0x00
)]
#[case::dec_bc_underflow(
    0x0B,
    AddressingMode::RegisterPair(RegisterPair::BC),
    0x0000,
    0xFF,
    0xFFFF,
    0xFF
)]

fn test_inc_dec_16(
    #[case] opcode: u8,
    #[case] target: AddressingMode,
    #[case] initial_val: u16,
    #[case] initial_flags: u8,
    #[case] expected_val: u16,
    #[case] expected_flags: u8,
) {
    let mut cpu = setup_cpu();
    cpu.set_register(GPR::F, initial_flags);

    match target {
        AddressingMode::RegisterPair(rp) => cpu.set_register_pair(rp, initial_val),
        _ => panic!("Unsupported addressing mode"),
    }

    cpu.memory.borrow_mut().write(0x0000, opcode);
    cpu.tick();

    let result_val = match target {
        AddressingMode::RegisterPair(rp) => cpu.get_register_pair(rp),
        _ => panic!("Unsupported addressing mode"),
    };

    assert_eq!(result_val, expected_val, "Value mismatch");
    assert_eq!(cpu.get_register(GPR::F), expected_flags, "Flags mismatch");
}
