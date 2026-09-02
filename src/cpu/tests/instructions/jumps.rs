use rstest::rstest;

use crate::cpu::tests::setup_cpu;
use crate::cpu::{GPR, RegisterPair, flags};
use crate::traits::SyncronousComponent;

const DJNZ_OPCODE: u8 = 0x10;

const JR_D_OPCODE: u8 = 0x18;
const JR_NZ_D_OPCODE: u8 = 0x20;
const JR_Z_D_OPCODE: u8 = 0x28;
const JR_NC_D_OPCODE: u8 = 0x30;
const JR_C_D_OPCODE: u8 = 0x38;

const JP_OPCODE: u8 = 0xC3;
const JP_NZ_OPCODE: u8 = 0xC2;
const JP_Z_OPCODE: u8 = 0xCA;
const JP_NC_OPCODE: u8 = 0xD2;
const JP_C_OPCODE: u8 = 0xDA;
const JP_PO_OPCODE: u8 = 0xE2;
const JP_PE_OPCODE: u8 = 0xEA;
const JP_P_OPCODE: u8 = 0xF2;
const JP_M_OPCODE: u8 = 0xFA;

const JP_HL_OPCODE: u8 = 0xE9;

#[rstest]
#[case(0x1000, 5, -5i8 as u8, 4, 0x0FFD)]
#[case(0x1000, 10, 10, 9, 0x100C)]
#[case(0x1000, 1, -5i8 as u8, 0, 0x1002)]
#[case(0x1000, 0, -2i8 as u8, 0xFF, 0x1000)]
#[case(0xFFFE, 2, 5, 1, 0x0005)]
fn test_djnz(
    #[case] starting_pc: u16,
    #[case] initial_b: u8,
    #[case] displacement: u8,
    #[case] expected_b: u8,
    #[case] expected_pc: u16,
) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.set_register(crate::cpu::GPR::B, initial_b);
    cpu.memory.borrow_mut().write(cpu.pc, DJNZ_OPCODE);
    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(1), displacement);
    cpu.tick(); // Fetch DJNZ

    assert_eq!(
        cpu.get_register(crate::cpu::GPR::B),
        expected_b,
        "Wrong B register after DJNZ: expected {}, got {}",
        expected_b,
        cpu.get_register(crate::cpu::GPR::B)
    );
    assert_eq!(
        cpu.pc, expected_pc,
        "Wrong PC after DJNZ: expected {:04X}, got {:04X}",
        expected_pc, cpu.pc
    );
}

#[rstest]
#[case(0x2000, 5, 0x2007)]
#[case(0x2000, -3i8 as u8, 0x1FFF)]
#[case(0x2000, 0, 0x2002)]
#[case(0xFFFD, 4, 0x0003)]
fn test_jr_d(#[case] starting_pc: u16, #[case] displacement: u8, #[case] expected_pc: u16) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.memory.borrow_mut().write(cpu.pc, JR_D_OPCODE); // JR d opcode
    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(1), displacement);
    cpu.tick(); // Fetch JR d

    assert_eq!(
        cpu.pc, expected_pc,
        "Wrong PC after JR d: expected {:04X}, got {:04X}",
        expected_pc, cpu.pc
    );
}

#[rstest]
#[case(0x1000, JR_NZ_D_OPCODE, -2i8 as u8, 0, 0x1000)]
#[case(0x1000, JR_NZ_D_OPCODE, -2i8 as u8, flags::ZERO, 0x1002)]
#[case(0x2000, JR_Z_D_OPCODE, 5, flags::ZERO, 0x2007)]
#[case(0x2000, JR_Z_D_OPCODE, 5, 0, 0x2002)]
#[case(0x3000, JR_NC_D_OPCODE, 10, 0, 0x300C)]
#[case(0x3000, JR_NC_D_OPCODE, 10, flags::CARRY, 0x3002)]
#[case(0x4000, JR_C_D_OPCODE, -4i8 as u8, flags::CARRY, 0x3FFE)]
#[case(0x4000, JR_C_D_OPCODE, -4i8 as u8, 0, 0x4002)]
fn test_jr_cc_d(
    #[case] starting_pc: u16,
    #[case] opcode: u8,
    #[case] displacement: u8,
    #[case] flags: u8,
    #[case] expected_pc: u16,
) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.set_register(GPR::F, flags);
    cpu.memory.borrow_mut().write(cpu.pc, opcode); // JR cc[y-4], d opcode
    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(1), displacement);
    cpu.tick(); // Fetch JR cc[y-4], d

    assert_eq!(
        cpu.pc, expected_pc,
        "Wrong PC after JR cc[y-4], d: expected {:04X}, got {:04X}",
        expected_pc, cpu.pc
    );
}

#[rstest]
#[case(0x1000, 0x55AA)]
#[case(0xFFFF, 0x0000)] // Wrapping/Boundary check
#[case(0x0000, 0xFFFF)]
fn test_jp(#[case] starting_pc: u16, #[case] target_addr: u16) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.memory.borrow_mut().write(cpu.pc, JP_OPCODE);

    // JP uses Little-Endian for the address (Low Byte first)
    let low_byte = (target_addr & 0xFF) as u8;
    let high_byte = ((target_addr >> 8) & 0xFF) as u8;

    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(1), low_byte);
    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(2), high_byte);

    cpu.tick();

    assert_eq!(
        cpu.pc, target_addr,
        "Wrong PC after JP: expected {:04X}, got {:04X}",
        target_addr, cpu.pc
    );
}

#[rstest]
// --- Zero Flag Tests ---
#[case(0x1000, JP_NZ_OPCODE, 0x5555, 0, 0x5555)] // Z=0, Jump Taken
#[case(0x1000, JP_NZ_OPCODE, 0x5555, flags::ZERO, 0x1003)] // Z=1, Not Taken (PC+3)
#[case(0x2000, JP_Z_OPCODE, 0xAAAA, flags::ZERO, 0xAAAA)] // Z=1, Jump Taken
#[case(0x2000, JP_Z_OPCODE, 0xAAAA, 0, 0x2003)] // Z=0, Not Taken (PC+3)

// --- Carry Flag Tests ---
#[case(0x3000, JP_NC_OPCODE, 0x1234, 0, 0x1234)] // C=0, Jump Taken
#[case(0x3000, JP_NC_OPCODE, 0x1234, flags::CARRY, 0x3003)] // C=1, Not Taken
#[case(0x4000, JP_C_OPCODE, 0x4321, flags::CARRY, 0x4321)] // C=1, Jump Taken
#[case(0x4000, JP_C_OPCODE, 0x4321, 0, 0x4003)] // C=0, Not Taken

// --- Parity/Overflow Flag Tests ---
#[case(0x5000, JP_PO_OPCODE, 0x9999, 0, 0x9999)] // PV=0 (Odd), Jump Taken
#[case(0x5000, JP_PO_OPCODE, 0x9999, flags::PARITY_OVERFLOW, 0x5003)] // PV=1 (Even), Not Taken
#[case(0x6000, JP_PE_OPCODE, 0x8888, flags::PARITY_OVERFLOW, 0x8888)] // PV=1 (Even), Jump Taken
#[case(0x6000, JP_PE_OPCODE, 0x8888, 0, 0x6003)] // PV=0 (Odd), Not Taken

// --- Sign Flag Tests ---
#[case(0x7000, JP_P_OPCODE, 0x7777, 0, 0x7777)] // S=0 (Positive), Jump Taken
#[case(0x7000, JP_P_OPCODE, 0x7777, flags::SIGN, 0x7003)] // S=1 (Minus), Not Taken
#[case(0x8000, JP_M_OPCODE, 0x6666, flags::SIGN, 0x6666)] // S=1 (Minus), Jump Taken
#[case(0x8000, JP_M_OPCODE, 0x6666, 0, 0x8003)] // S=0 (Positive), Not Taken
fn test_jp_cc(
    #[case] starting_pc: u16,
    #[case] opcode: u8,
    #[case] target_addr: u16,
    #[case] flags: u8,
    #[case] expected_pc: u16,
) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.set_register(GPR::F, flags);
    cpu.memory.borrow_mut().write(cpu.pc, opcode);

    // Write Little-Endian Address (Target)
    let low_byte = (target_addr & 0xFF) as u8;
    let high_byte = ((target_addr >> 8) & 0xFF) as u8;

    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(1), low_byte);
    cpu.memory
        .borrow_mut()
        .write(cpu.pc.wrapping_add(2), high_byte);

    cpu.tick();

    assert_eq!(
        cpu.pc, expected_pc,
        "Wrong PC after JP cc: expected {:04X}, got {:04X}",
        expected_pc, cpu.pc
    );
}

#[rstest]
#[case(0x1000, 0x8000)] // Standard jump: PC goes from 0x1000 -> 0x8000
#[case(0x0000, 0xFFFF)] // Jump to the very end of memory
#[case(0xFFFF, 0x0000)] // Jump to the very start (Reset)
#[case(0x2000, 0x2000)] // "Infinite Loop" (Jump to itself)
fn test_jp_hl(#[case] starting_pc: u16, #[case] target_addr: u16) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;

    // Load the target address into the HL register
    // (Splitting into High and Low bytes for the individual registers)

    cpu.set_register_pair(RegisterPair::HL, target_addr);

    // Write the opcode
    cpu.memory.borrow_mut().write(cpu.pc, JP_HL_OPCODE);

    cpu.tick();

    assert_eq!(
        cpu.pc, target_addr,
        "Wrong PC after JP (HL): expected {:04X}, got {:04X}",
        target_addr, cpu.pc
    );
}
