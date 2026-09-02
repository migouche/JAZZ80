use crate::cpu::tests::setup_cpu;
use crate::traits::SyncronousComponent;

#[test]
fn test_nmi_basic() {
    let mut cpu = setup_cpu();

    // Set PC to some known location
    cpu.pc = 0x1000;

    // Trigger NMI
    cpu.nmi();

    // NMI is edge triggered, so we need to tick to process it.
    // The previous instruction completes, checking for NMI.
    // Let's assume we are between instructions.

    // We can't just tick without memory being set up or it will fetch 0x00 and execute NOP or HALT.
    // But we want to see NMI processing.
    // NMI happens *before* fetch.

    // cpu.tick() -> check NMI -> handle_nmi -> return (end of tick) or proceed?
    // Looking at tick():
    // if self.nmi_pending { handle_nmi(); return; }

    cpu.tick();

    // After NMI:
    // PC should be 0x0066
    assert_eq!(cpu.pc, 0x0066);
    // Old PC (0x1000) should be on stack
    let sp = cpu.sp;
    let ret_addr_low = cpu.memory.borrow().read(sp);
    let ret_addr_high = cpu.memory.borrow().read(sp.wrapping_add(1));
    let ret_addr = u16::from_le_bytes([ret_addr_low, ret_addr_high]);

    assert_eq!(ret_addr, 0x1000);
}

#[test]
fn test_nmi_disables_maskable_interrupts() {
    let mut cpu = setup_cpu();

    cpu.iff1 = true;
    cpu.iff2 = true;

    cpu.nmi();
    cpu.tick(); // Process NMI

    // NMI Implementation: iff2 = iff1; iff1 = false;
    assert!(!cpu.iff1);
    assert!(cpu.iff2);

    // And PC check
    assert_eq!(cpu.pc, 0x0066);
}

#[test]
fn test_nmi_preserves_iff2_status() {
    let mut cpu = setup_cpu();

    cpu.iff1 = false; // Maskable interrupts disabled
    cpu.iff2 = false;

    cpu.nmi();
    cpu.tick();

    assert!(!cpu.iff1);
    assert!(!cpu.iff2);
}

#[test]
fn test_retn_restores_iff1() {
    let mut cpu = setup_cpu();

    // Start with maskable interrupts enabled
    cpu.iff1 = true;
    cpu.iff2 = true;

    // Fire NMI
    cpu.nmi();
    cpu.tick();

    // Now at 0x0066, IFF1=false, IFF2=true
    assert!(!cpu.iff1);
    assert!(cpu.iff2);

    // Execute RETN (ED 45)
    // Write RETN at 0x0066
    cpu.memory.borrow_mut().write(0x0066, 0xED);
    cpu.memory.borrow_mut().write(0x0067, 0x45);

    cpu.tick(); // Fetch and execute RETN

    // Should have returned to where we were (0x0000 default start)
    assert_eq!(cpu.pc, 0x0000);

    // IFF1 should be restored from IFF2
    assert!(cpu.iff1);
    assert!(cpu.iff2);
}

#[test]
fn test_retn_restores_iff1_disabled() {
    let mut cpu = setup_cpu();

    // Start with maskable interrupts disabled
    cpu.iff1 = false;
    cpu.iff2 = false;

    // Fire NMI
    cpu.nmi();
    cpu.tick();

    // Now at 0x0066, IFF1=false, IFF2=false
    assert!(!cpu.iff1);
    assert!(!cpu.iff2);

    // Execute RETN (ED 45) at 0x0066
    cpu.memory.borrow_mut().write(0x0066, 0xED);
    cpu.memory.borrow_mut().write(0x0067, 0x45);

    cpu.tick();

    // Should have returned to check PC
    assert_eq!(cpu.pc, 0x0000);

    // IFF1 should be restored from IFF2 (false)
    assert!(!cpu.iff1);
    assert!(!cpu.iff2);
}

#[test]
fn test_nmi_wakes_halt() {
    let mut cpu = setup_cpu();

    cpu.halted = true;
    cpu.nmi(); // should set pending

    cpu.tick(); // Should wake up and process NMI

    assert!(!cpu.halted);
    assert_eq!(cpu.pc, 0x0066);
}
