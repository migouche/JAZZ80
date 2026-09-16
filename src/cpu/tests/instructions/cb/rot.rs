use rstest::rstest;

use crate::cpu::tests::setup_cpu;
use crate::cpu::{Flag, GPR, RegisterPair};
use crate::traits::SynchronousComponent;

// this only tests carry flag because its old :C
// there are more extensive tests for the rotation functions in alu_tests.rs

#[rstest]
#[case::rlc_b(GPR::B, 0x00, (0b1000_0001, true), (0b0000_0011, true), 0x1000)]
#[case::rlc_c(GPR::C, 0x01, (0b0000_0001, false), (0b0000_0010, false), 0x1000)]
#[case::rlc_d(GPR::D, 0x02, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::rlc_e(GPR::E, 0x03, (0b0101_0101, false), (0b1010_1010, false), 0x1000)]
#[case::rlc_h(GPR::H, 0x04, (0b0000_0000, false), (0b0000_0000, false), 0x1000)]
#[case::rlc_l(GPR::L, 0x05, (0b1111_0000, true), (0b1110_0001, true), 0x1000)]
#[case::rlc_a(GPR::A, 0x07, (0b1001_1110, true), (0b0011_1101, true), 0x1000)]
#[case::rrc_b(GPR::B, 0x08, (0b0000_0011, true), (0b1000_0001, true), 0x1000)]
#[case::rrc_c(GPR::C, 0x09, (0b0000_0010, false), (0b0000_0001, false), 0x1000)]
#[case::rrc_d(GPR::D, 0x0A, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::rrc_e(GPR::E, 0x0B, (0b1010_1010, false), (0b0101_0101, false), 0x1000)]
#[case::rrc_h(GPR::H, 0x0C, (0b0000_0000, false), (0b0000_0000, false), 0x1000)]
#[case::rrc_l(GPR::L, 0x0D, (0b1110_0001, true), (0b1111_0000, true), 0x1000)]
#[case::rrc_a(GPR::A, 0x0F, (0b0011_1101, true), (0b1001_1110, true), 0x1000)]
#[case::rl_b(GPR::B, 0x10, (0b1000_0001, false), (0b0000_0010, true), 0x1000)]
#[case::rl_c(GPR::C, 0x11, (0b1000_0001, true), (0b0000_0011, true), 0x1000)]
#[case::rl_d(GPR::D, 0x12, (0b0000_0001, false), (0b0000_0010, false), 0x1000)]
#[case::rl_e(GPR::E, 0x13, (0b0000_0001, true), (0b0000_0011, false), 0x1000)]
#[case::rl_h(GPR::H, 0x14, (0b1111_1111, false), (0b1111_1110, true), 0x1000)]
#[case::rl_l(GPR::L, 0x15, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::rl_a(GPR::A, 0x17, (0b0101_0101, false), (0b1010_1010, false), 0x1000)]
#[case::rr_b(GPR::B, 0x18, (0b0000_0011, false), (0b0000_0001, true), 0x1000)]
#[case::rr_c(GPR::C, 0x19, (0b0000_0011, true), (0b1000_0001, true), 0x1000)]
#[case::rr_d(GPR::D, 0x1A, (0b0000_0010, false), (0b0000_0001, false), 0x1000)]
#[case::rr_e(GPR::E, 0x1B, (0b0000_0010, true), (0b1000_0001, false), 0x1000)]
#[case::rr_h(GPR::H, 0x1C, (0b1111_1111, false), (0b0111_1111, true), 0x1000)]
#[case::rr_l(GPR::L, 0x1D, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::rr_a(GPR::A, 0x1F, (0b1010_1010, false), (0b0101_0101, false), 0x1000)]
#[case::sla_b(GPR::B, 0x20, (0b1000_0001, true), (0b0000_0010, true), 0x1000)]
#[case::sla_c(GPR::C, 0x21, (0b0000_0001, false), (0b0000_0010, false), 0x1000)]
#[case::sla_d(GPR::D, 0x22, (0b1111_1111, true), (0b1111_1110, true), 0x1000)]
#[case::sla_e(GPR::E, 0x23, (0b0101_0101, false), (0b1010_1010, false), 0x1000)]
#[case::sla_h(GPR::H, 0x24, (0b0000_0000, false), (0b0000_0000, false), 0x1000)]
#[case::sla_l(GPR::L, 0x25, (0b1111_0000, true), (0b1110_0000, true), 0x1000)]
#[case::sla_a(GPR::A, 0x27, (0b1001_1110, true), (0b0011_1100, true), 0x1000)]
#[case::sra_b(GPR::B, 0x28, (0b1000_0001, true), (0b1100_0000, true), 0x1000)]
#[case::sra_c(GPR::C, 0x29, (0b0000_0001, false), (0b0000_0000, true), 0x1000)]
#[case::sra_d(GPR::D, 0x2A, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::sra_e(GPR::E, 0x2B, (0b0101_0101, false), (0b0010_1010, true), 0x1000)]
#[case::sra_h(GPR::H, 0x2C, (0b0000_0000, false), (0b0000_0000, false), 0x1000)]
#[case::sra_l(GPR::L, 0x2D, (0b1111_0000, true), (0b1111_1000, false), 0x1000)]
#[case::sra_a(GPR::A, 0x2F, (0b1001_1110, true), (0b1100_1111, false), 0x1000)]
#[case::sll_b(GPR::B, 0x30, (0b1000_0001, true), (0b0000_0011, true), 0x1000)]
#[case::sll_c(GPR::C, 0x31, (0b0000_0001, false), (0b0000_0011, false), 0x1000)]
#[case::sll_d(GPR::D, 0x32, (0b1111_1111, true), (0b1111_1111, true), 0x1000)]
#[case::sll_e(GPR::E, 0x33, (0b0101_0101, false), (0b1010_1011, false), 0x1000)]
#[case::sll_h(GPR::H, 0x34, (0b0000_0000, false), (0b0000_0001, false), 0x1000)]
#[case::sll_l(GPR::L, 0x35, (0b1111_0000, true), (0b1110_0001, true), 0x1000)]
#[case::sll_a(GPR::A, 0x37, (0b1001_1110, true), (0b0011_1101, true), 0x1000)]
#[case::srl_b(GPR::B, 0x38, (0b1000_0001, true), (0b0100_0000, true), 0x1000)]
#[case::srl_c(GPR::C, 0x39, (0b0000_0001, false), (0b0000_0000, true), 0x1000)]
#[case::srl_d(GPR::D, 0x3A, (0b1111_1111, true), (0b0111_1111, true), 0x1000)]
#[case::srl_e(GPR::E, 0x3B, (0b0101_0101, false), (0b0010_1010, true), 0x1000)]
#[case::srl_h(GPR::H, 0x3C, (0b0000_0000, false), (0b0000_0000, false), 0x1000)]
#[case::srl_l(GPR::L, 0x3D, (0b1111_0000, true), (0b0111_1000, false), 0x1000)]
#[case::srl_a(GPR::A, 0x3F, (0b1001_1110, true), (0b0100_1111, false), 0x1000)]
fn test_rot_register(
    #[case] reg: GPR,
    #[case] opcode: u8,
    #[case] src: (u8, bool),
    #[case] expected: (u8, bool),
    #[case] pc: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = pc;
    cpu.set_register(reg, src.0);
    cpu.set_flag(src.1, Flag::C);

    cpu.memory.borrow_mut().write(pc, 0xCB); // CB Prefix
    cpu.memory.borrow_mut().write(pc + 1, opcode); // Opcode

    cpu.tick();

    let result = (cpu.get_register(reg), cpu.get_flag(Flag::C));
    assert_eq!(result, expected);
}

#[rstest]
#[case::rlc_hl_indirect(0x06, (0b0000_0001, true), (0b0000_0010, false), 0x2000, 0x1000)]
#[case::rrc_hl_indirect(0x0E, (0b0000_0010, false), (0b0000_0001, false), 0x2000, 0x1000)]
#[case::rl_hl_indirect(0x16, (0b1000_0001, false), (0b0000_0010, true), 0x2000, 0x1000)]
#[case::rr_hl_indirect(0x1E, (0b0000_0011, false), (0b0000_0001, true), 0x2000, 0x1000)]
#[case::sla_hl_indirect(0x26, (0b1000_0001, true), (0b0000_0010, true), 0x2000, 0x1000)]
#[case::sra_hl_indirect(0x2E, (0b1000_0001, true), (0b1100_0000, true), 0x2000, 0x1000)]
#[case::sll_hl_indirect(0x36, (0b1000_0001, true), (0b0000_0011, true), 0x2000, 0x1000)]
#[case::srl_hl_indirect(0x3E, (0b1000_0001, true), (0b0100_0000, true), 0x2000, 0x1000)]
fn test_rot_hl_indirect(
    #[case] opcode: u8,
    #[case] src: (u8, bool),
    #[case] expected: (u8, bool),
    #[case] hl: u16,
    #[case] pc: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = pc;
    cpu.set_register_pair(RegisterPair::HL, hl);
    cpu.memory.borrow_mut().write(hl, src.0);
    cpu.set_flag(src.1, Flag::C);

    cpu.memory.borrow_mut().write(pc, 0xCB); // CB Prefix
    cpu.memory.borrow_mut().write(pc + 1, opcode); // Opcode

    cpu.tick();

    let result = (cpu.memory.borrow().read(hl), cpu.get_flag(Flag::C));
    assert_eq!(result, expected);
}
