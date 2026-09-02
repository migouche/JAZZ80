use rstest::rstest;

const CPL_OPCODE: u8 = 0x2F;

#[rstest]
#[case(0x00, 0b00000000, 0xFF, 0b00111010)]
#[case(0x01, 0b00000000, 0xFE, 0b00111010)]
#[case(0xAA, 0b00000001, 0x55, 0b00010011)]
#[case(0x00, 0b11000101, 0xFF, 0b11111111)]
fn test_cpl_instruction(
    #[case] initial_a: u8,
    #[case] initial_f: u8,
    #[case] expected_a: u8,
    #[case] expected_f: u8,
) {
    use crate::{
        cpu::{GPR, tests::setup_cpu},
        traits::SynchronousComponent,
    };

    let mut cpu = setup_cpu();

    cpu.set_register(GPR::A, initial_a);
    cpu.set_register(GPR::F, initial_f);

    cpu.pc = 0x1000;
    cpu.memory.borrow_mut().write(cpu.pc, CPL_OPCODE);
    cpu.tick();

    assert_eq!(
        cpu.get_register(GPR::A),
        expected_a,
        "A mismatch: expected {:02X}, got {:02X}",
        expected_a,
        cpu.get_register(GPR::A)
    );

    assert_eq!(
        cpu.get_register(GPR::F),
        expected_f,
        "F mismatch: expected {:08b}, got {:08b}",
        expected_f,
        cpu.get_register(GPR::F)
    );
}
