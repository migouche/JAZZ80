use rstest::rstest;

use crate::cpu::tests::setup_cpu;
use crate::cpu::{Flag, GPR};
use crate::traits::SynchronousComponent;

#[rstest]
// RLCA
#[case::rlca_basic(0x07, 0b1000_0000, false, 0b0000_0001, true)]
#[case::rlca_no_carry(0x07, 0b0000_0001, false, 0b0000_0010, false)]
// RRCA
#[case::rrca_basic(0x0F, 0b0000_0001, false, 0b1000_0000, true)]
#[case::rrca_no_carry(0x0F, 0b1000_0000, false, 0b0100_0000, false)]
// RLA
#[case::rla_carry_in(0x17, 0b0000_0000, true, 0b0000_0001, false)]
#[case::rla_carry_out(0x17, 0b1000_0000, false, 0b0000_0000, true)]
// RRA
#[case::rra_carry_in(0x1F, 0b0000_0000, true, 0b1000_0000, false)]
#[case::rra_carry_out(0x1F, 0b0000_0001, false, 0b0000_0000, true)]
fn test_rot_accumulator(
    #[case] opcode: u8,
    #[case] initial_a: u8,
    #[case] initial_c: bool,
    #[case] expected_a: u8,
    #[case] expected_c: bool,
) {
    let mut cpu = setup_cpu();
    cpu.set_register(GPR::A, initial_a);
    cpu.set_flag(initial_c, Flag::C);

    // Set Z, S, P/V flags to true to verify they are preserved
    // These instructions (RLCA, RRCA, RLA, RRA) should NOT affect these flags.
    cpu.set_flag(true, Flag::Z);
    cpu.set_flag(true, Flag::S);
    cpu.set_flag(true, Flag::PV);

    // Set H and N to true to verify they are reset
    // These instructions should reset H and N.
    cpu.set_flag(true, Flag::H);
    cpu.set_flag(true, Flag::N);

    cpu.memory.borrow_mut().write(0x0000, opcode);

    cpu.tick();

    assert_eq!(
        cpu.get_register(GPR::A),
        expected_a,
        "Accumulator value mismatch"
    );
    assert_eq!(cpu.get_flag(Flag::C), expected_c, "Carry flag mismatch");

    // Verify Z, S, P are preserved (should still be true)
    assert!(cpu.get_flag(Flag::Z), "Z flag should be preserved");
    assert!(cpu.get_flag(Flag::S), "S flag should be preserved");
    assert!(cpu.get_flag(Flag::PV), "P/V flag should be preserved");

    // Verify H and N are reset
    assert!(!cpu.get_flag(Flag::H), "H flag should be reset");
    assert!(!cpu.get_flag(Flag::N), "N flag should be reset");
}
