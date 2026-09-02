use crate::cpu::RegisterPair;
use crate::cpu::tests::setup_cpu;
use crate::traits::SynchronousComponent;
use rstest::rstest;

#[rstest]
// Case 1: Standard POP BC
// We put 0x44 at 0x1000 and 0x33 at 0x1001.
// POP should load Low=0x44, High=0x33 -> Result 0x3344.
#[case(
    0x0000,                         // Starting PC
    0x1000,                         // Starting SP
    &[(0x1000, 0x44), (0x1001, 0x33)], // Memory Setup (Little Endian!)
    RegisterPair::BC,               // Dest Register
    0x3344,                         // Expected Value in Register
    0x1002                          // Expected Final SP (+2)
)]
// Case 2: POP HL with Address Wrapping
// SP starts at 0xFFFF.
// 1. Reads Low byte at 0xFFFF.
// 2. Increments SP to 0x0000.
// 3. Reads High byte at 0x0000.
// 4. Increments SP to 0x0001.
#[case(
    0x0200,
    0xFFFF,                         // SP at the very top
    &[(0xFFFF, 0xAA), (0x0000, 0xBB)], // Wraps around memory end
    RegisterPair::HL,
    0xBBAA,                         // Expected HL (High=BB, Low=AA)
    0x0001                          // Final SP
)]
// Case 3: POP AF (The Flags Register)
// Validating that the F register accepts values from the stack.
// Stack: 0xC0 (F) and 0x12 (A).
// F=0xC0 means Sign(S) and Zero(Z) flags are set (bits 7 and 6).
#[case(
    0x0000,
    0x2000,
    &[(0x2000, 0xC0), (0x2001, 0x12)], // F=C0, A=12
    RegisterPair::AF,
    0x12C0,
    0x2002
)]
// Case 4: POP DE - "Circular" Stack
// If SP is 0x0000, it pops from 0x0000 then 0x0001.
// This ensures your emulator treats 0x0000 as a valid start point for POP.
#[case(
    0x0010,
    0x0000,
    &[(0x0000, 0xEF), (0x0001, 0xBE)],
    RegisterPair::DE,
    0xBEEF,
    0x0002
)]

fn test_pop(
    #[case] starting_pc: u16,
    #[case] starting_sp: u16,
    #[case] memory_contents: &[(u16, u8)],
    #[case] dest_reg: RegisterPair,
    #[case] expected_value: u16,
    #[case] expected_sp: u16,
) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.sp = starting_sp;

    for (addr, value) in memory_contents {
        cpu.memory.borrow_mut().write(*addr, *value);
    }

    // Write the POP instruction at the current PC
    let pop_opcode = match dest_reg {
        RegisterPair::BC => 0xC1,
        RegisterPair::DE => 0xD1,
        RegisterPair::HL => 0xE1,
        RegisterPair::AF => 0xF1,
        _ => panic!("Invalid register for POP instruction"),
    };
    cpu.memory.borrow_mut().write(cpu.pc, pop_opcode);

    cpu.tick();

    let result = cpu.get_register_pair(dest_reg);
    assert_eq!(
        result, expected_value,
        "Wrong value in {:?} after POP: expected {:04X}, got {:04X}",
        dest_reg, expected_value, result
    );
    assert_eq!(
        cpu.sp, expected_sp,
        "Wrong SP after POP: expected {:04X}, got {:04X}",
        expected_sp, cpu.sp
    );
}

#[rstest]
// Case 1: Standard PUSH BC
// BC = 0x1234. SP starts at 0x2000.
// Should write 0x12 to 0x1FFF and 0x34 to 0x1FFE.
#[case(
    0x0000,           // starting_pc
    0x2000,           // starting_sp
    RegisterPair::BC, // reg_to_push
    0x1234,           // value_in_reg
    0x1FFE,           // expected_sp
    &[(0x1FFF, 0x12), (0x1FFE, 0x34)] // expected_mem (Addr, Val)
)]
// Case 2: PUSH HL with Address Wrapping (The "Zero Boundary")
// SP starts at 0x0001.
// 1. SP -> 0x0000: Writes High Byte (0xAA)
// 2. SP -> 0xFFFF: Writes Low Byte (0xBB)
#[case(
    0x0100,
    0x0001,
    RegisterPair::HL,
    0xAABB,
    0xFFFF,
    &[(0x0000, 0xAA), (0xFFFF, 0xBB)]
)]
// Case 3: PUSH AF (Verifying Flags)
// AF = 0x55FF.
// A = 0x55, F = 0xFF.
#[case(
    0x0000,
    0x4000,
    RegisterPair::AF,
    0x55FF,
    0x3FFE,
    &[(0x3FFF, 0x55), (0x3FFE, 0xFF)]
)]
// Case 4: PUSH DE at the very top of memory
// SP starts at 0x0000.
// 1. SP -> 0xFFFF: Writes High Byte (0xDE)
// 2. SP -> 0xFFFE: Writes Low Byte (0xAD)
#[case(
    0xC001,
    0x0000,
    RegisterPair::DE,
    0xDEAD,
    0xFFFE,
    &[(0xFFFF, 0xDE), (0xFFFE, 0xAD)]
)]

fn test_push(
    #[case] starting_pc: u16,
    #[case] starting_sp: u16,
    #[case] reg_to_push: RegisterPair,
    #[case] value_in_reg: u16,
    #[case] expected_sp: u16,
    #[case] expected_mem: &[(u16, u8)],
) {
    let mut cpu = setup_cpu();

    cpu.pc = starting_pc;
    cpu.sp = starting_sp;
    cpu.set_register_pair(reg_to_push, value_in_reg);

    // Write the PUSH instruction at the current PC
    let push_opcode = match reg_to_push {
        RegisterPair::BC => 0xC5,
        RegisterPair::DE => 0xD5,
        RegisterPair::HL => 0xE5,
        RegisterPair::AF => 0xF5,
        _ => panic!("Invalid register for PUSH instruction"),
    };
    cpu.memory.borrow_mut().write(cpu.pc, push_opcode);

    cpu.tick();

    // Verify SP moved correctly
    assert_eq!(
        cpu.sp, expected_sp,
        "Wrong SP after PUSH {:?}: expected {:04X}, got {:04X}",
        reg_to_push, expected_sp, cpu.sp
    );

    // Verify Memory contents
    for (addr, expected_val) in expected_mem {
        let actual_val = cpu.memory.borrow().read(*addr);
        assert_eq!(
            actual_val, *expected_val,
            "Memory mismatch at {:04X}: expected {:02X}, got {:02X}",
            addr, expected_val, actual_val
        );
    }
}
