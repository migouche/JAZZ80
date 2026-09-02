use crate::cpu::GPR;
use crate::{cpu::tests::setup_cpu, traits::SynchronousComponent};
use rstest::rstest;

const SCF_OPCODE: u8 = 0x37;
const CCF_OPCODE: u8 = 0x3F;

#[rstest]
#[case(0b00000000, 0b00000001)]
#[case(0b00010010, 0b00000001)]
#[case(0b00000001, 0b00000001)]
#[case(0b11000100, 0b11000101)]
fn test_scf(#[case] initial_f: u8, #[case] expected_f: u8) {
    let mut cpu = setup_cpu();

    cpu.set_register(GPR::F, initial_f);

    cpu.pc = 0x1000;
    cpu.memory.borrow_mut().write(cpu.pc, SCF_OPCODE);
    cpu.tick();

    assert_eq!(
        cpu.get_register(GPR::F),
        expected_f,
        "F mismatch: expected {:08b}, got {:08b}",
        expected_f,
        cpu.get_register(GPR::F)
    );
}

#[rstest]
#[case(0b00101000, 0b00000001)] // A is 0, so x and y cleared
#[case(0b00000001, 0b00010000)]
#[case(0b00010000, 0b00000001)]
#[case(0b11000100, 0b11000101)]
#[case(0b11000101, 0b11010100)]
fn test_ccf(#[case] initial_f: u8, #[case] expected_f: u8) {
    let mut cpu = setup_cpu();

    cpu.set_register(GPR::F, initial_f);

    cpu.pc = 0x1000;
    cpu.memory.borrow_mut().write(cpu.pc, CCF_OPCODE);
    cpu.tick();

    assert_eq!(
        cpu.get_register(GPR::F),
        expected_f,
        "F mismatch: expected {:08b}, got {:08b}",
        expected_f,
        cpu.get_register(GPR::F)
    );
}
