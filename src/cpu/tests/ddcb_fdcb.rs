use crate::cpu::tests::setup_cpu;
use crate::cpu::{Flag, GPR, IndexRegister, SynchronousComponent, Z80A};
use rstest::rstest;
// use crate::traits::MemoryMapper;

// Helper to set up CPU with IX/IY and memory
fn prepare_cpu(cpu: &mut Z80A, prefix: u8, displacement: u8, initial_val: u8) {
    let base_addr = 0x1000;
    let target_addr = base_addr + displacement as u16;

    if prefix == 0xDD {
        cpu.set_index_register(IndexRegister::IX, base_addr);
    } else {
        cpu.set_index_register(IndexRegister::IY, base_addr);
    }

    cpu.memory.borrow_mut().write(target_addr, initial_val);

    // Write instruction to instruction memory at PC=0
    // Format: [Prefix] [CB] [Displacement] [Opcode]
    cpu.memory.borrow_mut().write(0x0000, prefix);
    cpu.memory.borrow_mut().write(0x0001, 0xCB);
    cpu.memory.borrow_mut().write(0x0002, displacement);
    // Opcode will be written in the test function
}

#[rstest]
// RLC (IX+d), B -> Opcode 0x00 (z=0 -> B)
// Input 0x80 (10000000) -> RotLeft -> 0x01 (00000001), Carry=1
#[case::dd_rlc_b(0xDD, 0x00, 0x80, 0x01, Some(GPR::B), Some(Flag::C), true)]
// RLC (IX+d) -> Opcode 0x06 (z=6 -> No register copy)
#[case::dd_rlc_nofile(0xDD, 0x06, 0x80, 0x01, None, Some(Flag::C), true)]
// RRC (IY+d), C -> Opcode 0x09 (z=1 -> C). RRC is y=1. 00 001 001 = 0x09.
// Input 0x01 (00000001) -> RotRight -> 0x80 (10000000), Carry=1
#[case::fd_rrc_c(0xFD, 0x09, 0x01, 0x80, Some(GPR::C), Some(Flag::C), true)]
// SET 3, (IX+d), D -> Opcode 0xDA (11 011 010). y=3, z=2(D).
// Input 0x00 -> Bit 3 set -> 0x08.
#[case::dd_set_3_d(0xDD, 0xDA, 0x00, 0x08, Some(GPR::D), None, false)]
// RES 7, (IX+d), A -> Opcode 0xBF (10 111 111). y=7, z=7(A).
// Input 0xFF -> Bit 7 res -> 0x7F.
#[case::dd_res_7_a(0xDD, 0xBF, 0xFF, 0x7F, Some(GPR::A), None, false)]
fn test_ddcb_fdcb_ops(
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] initial_val: u8,
    #[case] expected_val: u8,
    #[case] check_reg: Option<GPR>,
    #[case] check_flag: Option<Flag>,
    #[case] flag_val: bool,
) {
    let mut cpu = setup_cpu();
    let displacement = 0x05;

    // Setup
    prepare_cpu(&mut cpu, prefix, displacement, initial_val);

    // Write opcode
    cpu.memory.borrow_mut().write(0x0003, opcode);

    // Execute
    cpu.pc = 0x0000;
    cpu.tick(); // Should execute the 4-byte instruction

    // Verification
    let base_addr = 0x1000;
    let target_addr = base_addr + displacement as u16;
    let actual_mem_val = cpu.memory.borrow().read(target_addr);

    assert_eq!(actual_mem_val, expected_val, "Memory value mismatch");

    if let Some(reg) = check_reg {
        let reg_val = cpu.get_register(reg);
        assert_eq!(
            reg_val, expected_val,
            "Register copy mismatch for {:?}",
            reg
        );
    }

    if let Some(flag) = check_flag {
        assert_eq!(cpu.get_flag(flag), flag_val, "Flag {:?} mismatch", flag);
    }
}

#[rstest]
// BIT 0, (IX+d) -> Opcode 0x46 (01 000 110). z=6.
// Input 0x01. Bit 0 is 1. Z flag should be 0 (Not Zero).
#[case::bit_0_nz(0xDD, 0x46, 0x01, false)]
// BIT 0, (IX+d) -> Opcode 0x46.
// Input 0xFE (11111110). Bit 0 is 0. Z flag should be 1 (Zero).
#[case::bit_0_z(0xDD, 0x46, 0xFE, true)]
// BIT 7, (IX+d) -> Opcode 0x7E (01 111 110).
// Input 0x7F (01111111). Bit 7 is 0. Z flag should be 1.
#[case::bit_7_z(0xDD, 0x7E, 0x7F, true)]
// BIT 7, (IX+d) -> Opcode 0x7E.
// Input 0x80 (10000000). Bit 7 is 1. Z flag should be 0.
#[case::bit_7_nz(0xDD, 0x7E, 0x80, false)]
fn test_ddcb_bit(
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] initial_val: u8,
    #[case] expected_z_flag: bool,
) {
    let mut cpu = setup_cpu();
    let displacement = 0x10;

    prepare_cpu(&mut cpu, prefix, displacement, initial_val);
    cpu.memory.borrow_mut().write(0x0003, opcode);

    // Clear B to ensure it's not written to if we happened to test 'z=0' case,
    // though for BIT we expect no write.
    cpu.set_register(GPR::B, 0);

    cpu.pc = 0x0000;
    cpu.tick();

    // Check Z flag
    assert_eq!(cpu.get_flag(Flag::Z), expected_z_flag, "Z Flag mismatch");

    // Check memory UNCHANGED
    let base_addr = 0x1000;
    let target_addr = base_addr + displacement as u16;
    let mem_val = cpu.memory.borrow().read(target_addr);
    assert_eq!(
        mem_val, initial_val,
        "BIT instruction should not modify memory"
    );
}

#[test]
fn test_ddcb_bit_undocumented_no_copy() {
    let mut cpu = setup_cpu();
    let prefix = 0xDD;
    let displacement = 0x05;
    let initial_val = 0xFF;
    let opcode = 0x40; // BIT 0, (IX+d), B (theoretically, but BIT has no copy)

    prepare_cpu(&mut cpu, prefix, displacement, initial_val);
    cpu.memory.borrow_mut().write(0x0003, opcode);

    // Set B to something specific
    cpu.set_register(GPR::B, 0x55);

    cpu.pc = 0x0000;
    cpu.tick();

    // B should NOT change
    assert_eq!(
        cpu.get_register(GPR::B),
        0x55,
        "BIT instruction modified register - it should not!"
    );
}
