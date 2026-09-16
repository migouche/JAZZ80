use super::PREFIX;
use crate::cpu::GPR;
use crate::cpu::{RegisterPair, flags, tests::setup_cpu};
use crate::traits::SynchronousComponent;
use rstest::rstest;

const RRD_OPCODE: u8 = 0x67;
const RLD_OPCODE: u8 = 0x6F;

#[rstest]
#[case::rrd_jzn_example(0x1000, RRD_OPCODE, 0x84, 0x2000, 0x20, 0x80, 0x42, flags::SIGN)]
#[case::rrd_parity_odd(0x1000, RRD_OPCODE, 0x00, 0x2000, 0x11, 0x01, 0x01, 0x00)]
#[case::rrd_parity_even(
    0x1000,
    RRD_OPCODE,
    0x00,
    0x2000,
    0x13,
    0x03,
    0x01,
    flags::PARITY_OVERFLOW
)]
#[case::rrd_to_zero(
    0x1000, RRD_OPCODE, 0x0F, 0x2000, 0x00,
    0x00, 0xF0, flags::ZERO | flags::PARITY_OVERFLOW
)]
#[case::rld_simple(0x1000, RLD_OPCODE, 0x12, 0x2000, 0x34, 0x13, 0x42, 0x00)]
#[case::rld_to_sign_bit(
    0x1000, RLD_OPCODE, 0x80, 0x2000, 0x11,
    0x81, 0x10, flags::SIGN | flags::PARITY_OVERFLOW
)]
#[case::rld_to_zero(
    0x1000, RLD_OPCODE, 0x01, 0x2000, 0x02,
    0x00, 0x21, flags::ZERO | flags::PARITY_OVERFLOW
)]
#[case::rld_parity(
    0x1000,
    RLD_OPCODE,
    0x00,
    0x2000,
    0x30,
    0x03,
    0x00,
    flags::PARITY_OVERFLOW
)]
fn test_rot(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] initial_a: u8,
    #[case] initial_hl: u16,
    #[case] initial_hl_val: u8,
    #[case] expected_a: u8,
    #[case] expected_hl_val: u8,
    #[case] expected_flags: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.set_register(GPR::A, initial_a);
    cpu.memory.borrow_mut().write(initial_pc, PREFIX);
    cpu.memory
        .borrow_mut()
        .write(initial_pc.overflowing_add(1).0, opcode);
    cpu.set_register_pair(RegisterPair::HL, initial_hl);
    cpu.memory.borrow_mut().write(initial_hl, initial_hl_val);

    // Execute instruction
    cpu.tick();

    // Check results
    let result_a = cpu.get_register(GPR::A);
    let result_hl_val = cpu.memory.borrow().read(initial_hl);
    let result_flags = cpu.get_register(GPR::F);

    assert_eq!(
        result_a, expected_a,
        "A mismatch: expected 0x{:02X}, got 0x{:02X}",
        expected_a, result_a
    );
    assert_eq!(
        result_hl_val, expected_hl_val,
        "HL value mismatch: expected 0x{:02X}, got 0x{:02X}",
        expected_hl_val, result_hl_val
    );
    assert_eq!(
        result_flags, expected_flags,
        "Flags mismatch: expected {:08b}, got {:08b}",
        expected_flags, result_flags
    );
}
