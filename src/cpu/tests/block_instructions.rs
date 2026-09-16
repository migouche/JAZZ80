use crate::cpu::tests::setup_cpu;
use crate::cpu::{Flag, GPR, RegisterPair};
use crate::traits::SynchronousComponent;

#[test]
fn test_ldi() {
    let mut cpu = setup_cpu();
    // LDI: (DE) <- (HL), DE++, HL++, BC--
    // HL = 0x1000, DE = 0x2000, BC = 0x0001
    // (0x1000) = 0xAA

    cpu.set_register_pair(RegisterPair::HL, 0x1000);
    cpu.set_register_pair(RegisterPair::DE, 0x2000);
    cpu.set_register_pair(RegisterPair::BC, 0x0001);

    cpu.memory.borrow_mut().write(0x1000, 0xAA);

    // ED A0 is LDI
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xA0);

    cpu.tick(); // ED A0 (4 bytes, 16 cycles - wait logic not fully simulated in single tick unless loop, but here block instruction is atomic in our impl)

    let val = cpu.memory.borrow().read(0x2000);
    assert_eq!(val, 0xAA);
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), 0x1001);
    assert_eq!(cpu.get_register_pair(RegisterPair::DE), 0x2001);
    assert_eq!(cpu.get_register_pair(RegisterPair::BC), 0x0000);
    assert!(!cpu.get_flag(Flag::PV)); // BC == 0 -> PV = 0
}

#[test]
fn test_ldir() {
    let mut cpu = setup_cpu();
    // LDIR: Copy block
    // HL = 0x1000, DE = 0x2000, BC = 3
    // 0x1000 -> 0xAA, 0x1001 -> 0xBB, 0x1002 -> 0xCC

    cpu.set_register_pair(RegisterPair::HL, 0x1000);
    cpu.set_register_pair(RegisterPair::DE, 0x2000);
    cpu.set_register_pair(RegisterPair::BC, 0x0003);

    cpu.memory.borrow_mut().write(0x1000, 0xAA);
    cpu.memory.borrow_mut().write(0x1001, 0xBB);
    cpu.memory.borrow_mut().write(0x1002, 0xCC);

    // ED B0 is LDIR
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xB0);

    // Loop until Done (PC > 2)
    // Our LDIR decrements PC by 2 if BC != 0.
    // So we just tick until BC is 0?
    // Safety limit
    for _ in 0..20 {
        if cpu.get_register_pair(RegisterPair::BC) == 0 {
            break;
        }
        cpu.tick();
    }

    assert_eq!(cpu.memory.borrow().read(0x2000), 0xAA);
    assert_eq!(cpu.memory.borrow().read(0x2001), 0xBB);
    assert_eq!(cpu.memory.borrow().read(0x2002), 0xCC);
    assert_eq!(cpu.get_register_pair(RegisterPair::BC), 0);
    assert!(!cpu.get_flag(Flag::PV));
}

#[test]
fn test_ldd() {
    let mut cpu = setup_cpu();
    // LDD: (DE) <- (HL), DE--, HL--, BC--

    cpu.set_register_pair(RegisterPair::HL, 0x1002);
    cpu.set_register_pair(RegisterPair::DE, 0x2002);
    cpu.set_register_pair(RegisterPair::BC, 0x0001);

    cpu.memory.borrow_mut().write(0x1002, 0xDD);

    // ED A8 is LDD
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xA8);

    cpu.tick();

    assert_eq!(cpu.memory.borrow().read(0x2002), 0xDD);
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), 0x1001);
    assert_eq!(cpu.get_register_pair(RegisterPair::DE), 0x2001);
    assert!(!cpu.get_flag(Flag::PV));
}

#[test]
fn test_lddr() {
    let mut cpu = setup_cpu();
    // LDDR: Reverse copy
    // HL = 0x1002, DE = 0x2002, BC = 3

    cpu.set_register_pair(RegisterPair::HL, 0x1002);
    cpu.set_register_pair(RegisterPair::DE, 0x2002);
    cpu.set_register_pair(RegisterPair::BC, 0x0003);

    cpu.memory.borrow_mut().write(0x1002, 0xCC);
    cpu.memory.borrow_mut().write(0x1001, 0xBB);
    cpu.memory.borrow_mut().write(0x1000, 0xAA);

    // ED B8 is LDDR
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xB8);

    for _ in 0..20 {
        if cpu.get_register_pair(RegisterPair::BC) == 0 {
            break;
        }
        cpu.tick();
    }

    assert_eq!(cpu.memory.borrow().read(0x2002), 0xCC);
    assert_eq!(cpu.memory.borrow().read(0x2001), 0xBB);
    assert_eq!(cpu.memory.borrow().read(0x2000), 0xAA);
    assert_eq!(cpu.get_register_pair(RegisterPair::BC), 0);
}

#[test]
fn test_cpi() {
    let mut cpu = setup_cpu();
    // CPI: A - (HL), HL++, BC--
    // A = 0x55, (HL) = 0x55. Z should be 1.

    cpu.set_register(GPR::A, 0x55);
    cpu.set_register_pair(RegisterPair::HL, 0x1000);
    cpu.set_register_pair(RegisterPair::BC, 10);

    cpu.memory.borrow_mut().write(0x1000, 0x55);

    // ED A1 is CPI
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xA1);

    cpu.tick();

    assert!(cpu.get_flag(Flag::Z));
    assert_eq!(cpu.get_register_pair(RegisterPair::BC), 9);
    assert!(cpu.get_flag(Flag::PV)); // BC != 0
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), 0x1001);
}

#[test]
fn test_cpir_found() {
    let mut cpu = setup_cpu();
    // CPIR: Find 0x99 in a block
    // 0x1000: 00, 0x1001: 99, 0x1002: 00

    cpu.set_register(GPR::A, 0x99);
    cpu.set_register_pair(RegisterPair::HL, 0x1000);
    cpu.set_register_pair(RegisterPair::BC, 5);

    cpu.memory.borrow_mut().write(0x1000, 0x00);
    cpu.memory.borrow_mut().write(0x1001, 0x99);

    // ED B1 is CPIR
    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0xB1);

    for _ in 0..10 {
        if cpu.get_flag(Flag::Z) || cpu.get_register_pair(RegisterPair::BC) == 0 {
            // Stop if found or exhausted (but PC loops so we check PC too in real emul)
            // Check if PC advanced past instruction
            if cpu.get_pc() > 0x0001 {
                break;
            }
        }
        cpu.tick();
    }

    assert!(cpu.get_flag(Flag::Z));
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), 0x1002); // Increment happened after compare
}
