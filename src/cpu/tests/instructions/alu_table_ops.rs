use rstest::rstest;

use crate::cpu::GPR;
use crate::cpu::RegisterPair;
use crate::cpu::tests::setup_cpu;
use crate::cpu::{AddressingMode, SynchronousComponent};

#[rstest]
// ADD
#[case::add_a_b(
    0x3f91,
    0x80,
    0x7F,
    false,
    0x41,
    AddressingMode::Register(GPR::B),
    0xC0,
    0b10010100
)] // ADD A, B
#[case::add_a_c(
    0x3f91,
    0x81,
    0x01,
    false,
    0x02,
    AddressingMode::Register(GPR::C),
    0x03,
    0b00000000
)] // ADD A, C
#[case::add_a_d(
    0x3f91,
    0x82,
    0x10,
    false,
    0x10,
    AddressingMode::Register(GPR::D),
    0x20,
    0b00100000
)] // ADD A, D
#[case::add_a_e(
    0x3f91,
    0x83,
    0x00,
    false,
    0x00,
    AddressingMode::Register(GPR::E),
    0x00,
    0b01000000
)] // ADD A, E
#[case::add_a_h(
    0x3f91,
    0x84,
    0xFF,
    false,
    0x01,
    AddressingMode::Register(GPR::H),
    0x00,
    0b01010001
)] // ADD A, H
#[case::add_a_l(
    0x3f91,
    0x85,
    0x7F,
    false,
    0x01,
    AddressingMode::Register(GPR::L),
    0x80,
    0b10010100
)] // ADD A, L
#[case::add_a_hl(
    0x3f91,
    0x86,
    0x01,
    false,
    0x01,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x02,
    0b00000000
)] // ADD A, (HL)
#[case::add_a_a(
    0x3f91,
    0x87,
    0x10,
    false,
    0x10,
    AddressingMode::Register(GPR::A),
    0x20,
    0b00100000
)] // ADD A, A
#[case::add_a_n(
    0x3f91,
    0xC6,
    0x10,
    false,
    0x20,
    AddressingMode::Immediate(0),
    0x30,
    0b00100000
)] // ADD A, n

// ADC
#[case::adc_a_b(
    0x3f91,
    0x88,
    0x00,
    true,
    0x00,
    AddressingMode::Register(GPR::B),
    0x01,
    0b00000000
)] // ADC A, B
#[case::adc_a_c(
    0x3f91,
    0x89,
    0x01,
    true,
    0x01,
    AddressingMode::Register(GPR::C),
    0x03,
    0b00000000
)] // ADC A, C
#[case::adc_a_d(
    0x3f91,
    0x8A,
    0x10,
    true,
    0x10,
    AddressingMode::Register(GPR::D),
    0x21,
    0b00100000
)] // ADC A, D
#[case::adc_a_e(
    0x3f91,
    0x8B,
    0x00,
    true,
    0x00,
    AddressingMode::Register(GPR::E),
    0x01,
    0b00000000
)] // ADC A, E
#[case::adc_a_h(
    0x3f91,
    0x8C,
    0xFF,
    true,
    0x00,
    AddressingMode::Register(GPR::H),
    0x00,
    0b01010001
)] // ADC A, H
#[case::adc_a_l(
    0x3f91,
    0x8D,
    0x7F,
    true,
    0x00,
    AddressingMode::Register(GPR::L),
    0x80,
    0b10010100
)] // ADC A, L
#[case::adc_a_hl(
    0x3f91,
    0x8E,
    0x00,
    true,
    0x00,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x01,
    0b00000000
)] // ADC A, (HL)
#[case::adc_a_a(
    0x3f91,
    0x8F,
    0x10,
    true,
    0x10,
    AddressingMode::Register(GPR::A),
    0x21,
    0b00100000
)] // ADC A, A
#[case::adc_a_n(
    0x3f91,
    0xCE,
    0x00,
    true,
    0x00,
    AddressingMode::Immediate(0),
    0x01,
    0b00000000
)] // ADC A, n

// SUB
#[case::sub_a_b(
    0x3f91,
    0x90,
    0x05,
    false,
    0x02,
    AddressingMode::Register(GPR::B),
    0x03,
    0b00000010
)] // SUB A, B
#[case::sub_a_c(
    0x3f91,
    0x91,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::C),
    0x00,
    0b01000010
)] // SUB A, C
#[case::sub_a_d(
    0x3f91,
    0x92,
    0x05,
    false,
    0x02,
    AddressingMode::Register(GPR::D),
    0x03,
    0b00000010
)] // SUB A, D
#[case::sub_a_e(
    0x3f91,
    0x93,
    0x00,
    false,
    0x01,
    AddressingMode::Register(GPR::E),
    0xFF,
    0b10111011
)] // SUB A, E
#[case::sub_a_h(
    0x3f91,
    0x94,
    0x80,
    false,
    0x01,
    AddressingMode::Register(GPR::H),
    0x7F,
    0b00111110
)] // SUB A, H
#[case::sub_a_l(
    0x3f91,
    0x95,
    0x00,
    false,
    0x00,
    AddressingMode::Register(GPR::L),
    0x00,
    0b01000010
)] // SUB A, L
#[case::sub_a_hl(
    0x3f91,
    0x96,
    0x05,
    false,
    0x05,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x00,
    0b01000010
)] // SUB A, (HL)
#[case::sub_a_a(
    0x3f91,
    0x97,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::A),
    0x00,
    0b01000010
)] // SUB A, A
#[case::sub_a_n(
    0x3f91,
    0xD6,
    0x00,
    false,
    0x01,
    AddressingMode::Immediate(0),
    0xFF,
    0b10111011
)] // SUB A, n

// SBC
#[case::sbc_a_b(
    0x3f91,
    0x98,
    0x05,
    true,
    0x02,
    AddressingMode::Register(GPR::B),
    0x02,
    0b00000010
)] // SBC A, B
#[case::sbc_a_c(
    0x3f91,
    0x99,
    0x05,
    true,
    0x05,
    AddressingMode::Register(GPR::C),
    0xFF,
    0b10111011
)] // SBC A, C
#[case::sbc_a_d(
    0x3f91,
    0x9A,
    0x05,
    true,
    0x02,
    AddressingMode::Register(GPR::D),
    0x02,
    0b00000010
)] // SBC A, D
#[case::sbc_a_e(
    0x3f91,
    0x9B,
    0x05,
    true,
    0x02,
    AddressingMode::Register(GPR::E),
    0x02,
    0b00000010
)] // SBC A, E
#[case::sbc_a_h(
    0x3f91,
    0x9C,
    0x80,
    true,
    0x00,
    AddressingMode::Register(GPR::H),
    0x7F,
    0b00111110
)] // SBC A, H
#[case::sbc_a_l(
    0x3f91,
    0x9D,
    0x00,
    true,
    0x00,
    AddressingMode::Register(GPR::L),
    0xFF,
    0b10111011
)] // SBC A, L
#[case::sbc_a_hl(
    0x3f91,
    0x9E,
    0x05,
    true,
    0x05,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0xFF,
    0b10111011
)] // SBC A, (HL)
#[case::sbc_a_a(
    0x3f91,
    0x9F,
    0x05,
    true,
    0x05,
    AddressingMode::Register(GPR::A),
    0xFF,
    0b10111011
)] // SBC A, A
#[case::sbc_a_n(
    0x3f91,
    0xDE,
    0x00,
    true,
    0x00,
    AddressingMode::Immediate(0),
    0xFF,
    0b10111011
)] // SBC A, n

// AND
#[case::and_a_b(
    0x3f91,
    0xA0,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::B),
    0x0F,
    0b00011100
)] // AND A, B
#[case::and_a_c(
    0x3f91,
    0xA1,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::C),
    0x0F,
    0b00011100
)] // AND A, C
#[case::and_a_d(
    0x3f91,
    0xA2,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::D),
    0x0F,
    0b00011100
)] // AND A, D
#[case::and_a_e(
    0x3f91,
    0xA3,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::E),
    0x0F,
    0b00011100
)] // AND A, E
#[case::and_a_h(
    0x3f91,
    0xA4,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::H),
    0x0F,
    0b00011100
)] // AND A, H
#[case::and_a_l(
    0x3f91,
    0xA5,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::L),
    0x0F,
    0b00011100
)] // AND A, L
#[case::and_a_hl(
    0x3f91,
    0xA6,
    0xFF,
    false,
    0xAA,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0xAA,
    0b10111100
)] // AND A, (HL)
#[case::and_a_a(
    0x3f91,
    0xA7,
    0xFF,
    false,
    0xFF,
    AddressingMode::Register(GPR::A),
    0xFF,
    0b10111100
)] // AND A, A
#[case::and_a_n(
    0x3f91,
    0xE6,
    0x55,
    false,
    0xAA,
    AddressingMode::Immediate(0),
    0x00,
    0b01010100
)] // AND A, n

// XOR
#[case::xor_a_b(
    0x3f91,
    0xA8,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::B),
    0xF0,
    0b10100100
)] // XOR A, B
#[case::xor_a_c(
    0x3f91,
    0xA9,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::C),
    0xF0,
    0b10100100
)] // XOR A, C
#[case::xor_a_d(
    0x3f91,
    0xAA,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::D),
    0xF0,
    0b10100100
)] // XOR A, D
#[case::xor_a_e(
    0x3f91,
    0xAB,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::E),
    0xF0,
    0b10100100
)] // XOR A, E
#[case::xor_a_h(
    0x3f91,
    0xAC,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::H),
    0xF0,
    0b10100100
)] // XOR A, H
#[case::xor_a_l(
    0x3f91,
    0xAD,
    0xFF,
    false,
    0x0F,
    AddressingMode::Register(GPR::L),
    0xF0,
    0b10100100
)] // XOR A, L
#[case::xor_a_hl(
    0x3f91,
    0xAE,
    0xFF,
    false,
    0xFF,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x00,
    0b01000100
)] // XOR A, (HL)
#[case::xor_a_a(
    0x3f91,
    0xAF,
    0xFF,
    false,
    0xFF,
    AddressingMode::Register(GPR::A),
    0x00,
    0b01000100
)] // XOR A, A
#[case::xor_a_n(
    0x3f91,
    0xEE,
    0x55,
    false,
    0x55,
    AddressingMode::Immediate(0),
    0x00,
    0b01000100
)] // XOR A, n

// OR
#[case::or_a_b(
    0x3f91,
    0xB0,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::B),
    0xF0,
    0b10100100
)] // OR A, B
#[case::or_a_c(
    0x3f91,
    0xB1,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::C),
    0xF0,
    0b10100100
)] // OR A, C
#[case::or_a_d(
    0x3f91,
    0xB2,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::D),
    0xF0,
    0b10100100
)] // OR A, D
#[case::or_a_e(
    0x3f91,
    0xB3,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::E),
    0xF0,
    0b10100100
)] // OR A, E
#[case::or_a_h(
    0x3f91,
    0xB4,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::H),
    0xF0,
    0b10100100
)] // OR A, H
#[case::or_a_l(
    0x3f91,
    0xB5,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::L),
    0xF0,
    0b10100100
)] // OR A, L
#[case::or_a_hl(
    0x3f91,
    0xB6,
    0x0F,
    false,
    0xF0,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0xFF,
    0b10101100
)] // OR A, (HL)
#[case::or_a_a(
    0x3f91,
    0xB7,
    0xF0,
    false,
    0xF0,
    AddressingMode::Register(GPR::A),
    0xF0,
    0b10100100
)] // OR A, A
#[case::or_a_n(
    0x3f91,
    0xF6,
    0x00,
    false,
    0x00,
    AddressingMode::Immediate(0),
    0x00,
    0b01000100
)] // OR A, n

// CP
#[case::cp_a_b(
    0x3f91,
    0xB8,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::B),
    0x05,
    0b01000010
)] // CP A, B
#[case::cp_a_c(
    0x3f91,
    0xB9,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::C),
    0x05,
    0b01000010
)] // CP A, C
#[case::cp_a_d(
    0x3f91,
    0xBA,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::D),
    0x05,
    0b01000010
)] // CP A, D
#[case::cp_a_e(
    0x3f91,
    0xBB,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::E),
    0x05,
    0b01000010
)] // CP A, E
#[case::cp_a_h(
    0x3f91,
    0xBC,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::H),
    0x05,
    0b01000010
)] // CP A, H
#[case::cp_a_l(
    0x3f91,
    0xBD,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::L),
    0x05,
    0b01000010
)] // CP A, L
#[case::cp_a_hl(
    0x3f91,
    0xBE,
    0x00,
    false,
    0x01,
    AddressingMode::RegisterIndirect(RegisterPair::HL),
    0x00,
    0b10111011
)] // CP A, (HL)
#[case::cp_a_a(
    0x3f91,
    0xBF,
    0x05,
    false,
    0x05,
    AddressingMode::Register(GPR::A),
    0x05,
    0b01000010
)] // CP A, A
#[case::cp_a_n(
    0x3f91,
    0xFE,
    0x05,
    false,
    0x02,
    AddressingMode::Immediate(0),
    0x05,
    0b00000010
)] // CP A, n
fn test_alu_op(
    #[case] pc: u16,
    #[case] op_code: u8,
    #[case] initial_a: u8,
    #[case] initial_carry: bool,
    #[case] initial_b_val: u8,
    #[case] initial_b_src: AddressingMode,
    #[case] expected_a: u8,
    #[case] expected_flags: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = pc;
    cpu.set_register(crate::cpu::GPR::A, initial_a);
    cpu.set_flag(initial_carry, crate::cpu::Flag::C);

    match initial_b_src {
        AddressingMode::Immediate(_) => {
            cpu.memory.borrow_mut().write(pc + 1, initial_b_val);
        }
        AddressingMode::Register(r) => {
            cpu.set_register(r, initial_b_val);
        }
        AddressingMode::RegisterIndirect(rp) => {
            let addr = cpu.get_register_pair(rp);
            cpu.memory.borrow_mut().write(addr, initial_b_val);
        }
        _ => panic!("Unsupported addressing mode in test"),
    }

    cpu.memory.borrow_mut().write(pc, op_code);

    cpu.tick();

    let result_a = cpu.get_register(crate::cpu::GPR::A);
    let result_flags = cpu.get_register(GPR::F);

    assert_eq!(
        result_a, expected_a,
        "A register mismatch, expected: {:#04X}, got: {:#04X}",
        expected_a, result_a
    );
    assert_eq!(
        result_flags, expected_flags,
        "Flags mismatch, expected: {:#08b}, got: {:#08b}",
        expected_flags, result_flags
    );
}
