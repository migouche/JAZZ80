use crate::cpu::tests::setup_cpu;
use crate::traits::{IODevice, SyncronousComponent};
use std::cell::RefCell;
use std::rc::Rc;

struct MockInterruptDevice {
    active: bool,
    vector: u8,
}

impl MockInterruptDevice {
    fn new(vector: u8) -> Self {
        Self {
            active: false,
            vector,
        }
    }
    fn trigger(&mut self) {
        self.active = true;
    }
}

impl IODevice for MockInterruptDevice {
    fn read_in(&mut self, _port: u16) -> Option<u8> {
        None
    }
    fn write_out(&mut self, _port: u16, _data: u8) -> bool {
        false
    }
    fn poll_interrupt(&self) -> bool {
        self.active
    }
    fn ack_interrupt(&mut self) -> u8 {
        self.active = false;
        self.vector
    }
}

#[test]
fn test_interrupt_mode_0() {
    let mut cpu = setup_cpu();
    let dev = Rc::new(RefCell::new(MockInterruptDevice::new(0xFF))); // RST 38H
    cpu.attach_device(dev.clone());

    cpu.sp = 0xFFFF;
    cpu.memory.borrow_mut().write(0x0038, 0x00); // NOP at ISR

    // 0000: ED 46 (IM 0)
    // 0002: FB (EI)
    // 0003: 00 (NOP)
    // 0004: 00 (NOP)

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0x46);
    cpu.memory.borrow_mut().write(0x0002, 0xFB);
    cpu.memory.borrow_mut().write(0x0003, 0x00);
    cpu.memory.borrow_mut().write(0x0004, 0x00);

    cpu.tick(); // IM 0
    cpu.tick(); // EI

    dev.borrow_mut().trigger();

    cpu.tick(); // NOP (Enable takes effect after)

    cpu.tick(); // Interrupt -> RST 38H (pushes 0004)

    assert_eq!(cpu.pc, 0x0038);
    assert_eq!(cpu.pop(), 0x0004);
}

#[test]
fn test_interrupt_mode_1() {
    let mut cpu = setup_cpu();
    let dev = Rc::new(RefCell::new(MockInterruptDevice::new(0x00)));
    cpu.attach_device(dev.clone());
    cpu.sp = 0xFFFF;

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0x56); // IM 1
    cpu.memory.borrow_mut().write(0x0002, 0xFB); // EI
    cpu.memory.borrow_mut().write(0x0003, 0x00);
    cpu.memory.borrow_mut().write(0x0004, 0x00);

    cpu.tick();
    cpu.tick();
    dev.borrow_mut().trigger();
    cpu.tick();

    cpu.tick();

    assert_eq!(cpu.pc, 0x0038);
    assert_eq!(cpu.pop(), 0x0004);
}

#[test]
fn test_interrupt_mode_2() {
    let mut cpu = setup_cpu();
    let dev = Rc::new(RefCell::new(MockInterruptDevice::new(0x04)));
    cpu.attach_device(dev.clone());
    cpu.sp = 0xFFFF;

    // I = 0x20. ISR table at 0x2004 -> points to 0x3000
    cpu.memory.borrow_mut().write_word(0x2004, 0x3000);

    // LD A, 20H; LD I, A; IM 2; EI; NOP; NOP
    cpu.memory.borrow_mut().write(0x0000, 0x3E);
    cpu.memory.borrow_mut().write(0x0001, 0x20);
    cpu.memory.borrow_mut().write(0x0002, 0xED);
    cpu.memory.borrow_mut().write(0x0003, 0x47);
    cpu.memory.borrow_mut().write(0x0004, 0xED);
    cpu.memory.borrow_mut().write(0x0005, 0x5E);
    cpu.memory.borrow_mut().write(0x0006, 0xFB);
    cpu.memory.borrow_mut().write(0x0007, 0x00);
    cpu.memory.borrow_mut().write(0x0008, 0x00);

    cpu.tick(); // LD A
    cpu.tick(); // LD I
    cpu.tick(); // IM 2
    cpu.tick(); // EI

    dev.borrow_mut().trigger();
    cpu.tick(); // NOP

    cpu.tick(); // Interrupt logic

    assert_eq!(cpu.pc, 0x3000);
    assert_eq!(cpu.pop(), 0x0008);
}

#[test]
fn test_reti() {
    let mut cpu = setup_cpu();
    cpu.sp = 0xFFFD;
    cpu.memory.borrow_mut().write_word(0xFFFD, 0x1234);

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0x4D); // RETI

    cpu.tick();

    assert_eq!(cpu.pc, 0x1234);
}

#[test]
fn test_retn() {
    let mut cpu = setup_cpu();
    cpu.sp = 0xFFFD;
    cpu.memory.borrow_mut().write_word(0xFFFD, 0x5678);
    cpu.iff1 = false;
    cpu.iff2 = true;

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0x45); // RETN

    cpu.tick();

    assert_eq!(cpu.pc, 0x5678);
    assert!(cpu.iff1);
}
