use crate::cpu::Flag;
use crate::cpu::GPR;
use crate::cpu::RegisterPair;
use crate::cpu::tests::setup_cpu;
use crate::traits::IODevice;
use crate::traits::SyncronousComponent;
use rstest::rstest;
use std::cell::RefCell;
use std::rc::Rc;

struct MockIODevice {
    port: u16,
    input_queue: Vec<u8>,
    output_log: Vec<u8>,
}

impl MockIODevice {
    fn new(port: u16, data: Vec<u8>) -> Self {
        Self {
            port,
            input_queue: data,
            output_log: Vec::new(),
        }
    }
}

impl IODevice for MockIODevice {
    fn read_in(&mut self, port: u16) -> Option<u8> {
        // Simple mock: if port matches lower 8 bits or full 16 bits depending on decoding (CPU logic passes full 16 bits)
        // Usually basic IO matches lower 8 bits.
        if (port & 0xFF) == (self.port & 0xFF) {
            if !self.input_queue.is_empty() {
                Some(self.input_queue.remove(0))
            } else {
                Some(0xFF) // Default bus value
            }
        } else {
            None
        }
    }

    fn write_out(&mut self, port: u16, data: u8) -> bool {
        if (port & 0xFF) == (self.port & 0xFF) {
            self.output_log.push(data);
            true
        } else {
            false
        }
    }
}

#[rstest]
#[case(0x10, 0x42)]
#[case(0x20, 0x00)]
#[case(0xFF, 0xAA)]
fn test_in_a_n(#[case] port: u8, #[case] value: u8) {
    let mut cpu = setup_cpu();
    let device = Rc::new(RefCell::new(MockIODevice::new(port as u16, vec![value])));
    cpu.attach_device(device.clone());
    cpu.set_register(GPR::A, 0xAA);

    // Opcode: DB n (IN A, (n))
    cpu.memory.borrow_mut().write(0x0000, 0xDB);
    cpu.memory.borrow_mut().write(0x0001, port);

    cpu.tick();

    assert_eq!(cpu.get_register(GPR::A), value);
}

#[rstest]
#[case(0x20, 0x55)]
#[case(0x10, 0x00)]
#[case(0xFF, 0xFF)]
fn test_out_n_a(#[case] port: u8, #[case] value: u8) {
    let mut cpu = setup_cpu();
    let device = Rc::new(RefCell::new(MockIODevice::new(port as u16, vec![])));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::A, value);

    // Opcode: D3 n (OUT (n), A)
    cpu.memory.borrow_mut().write(0x0000, 0xD3);
    cpu.memory.borrow_mut().write(0x0001, port);

    cpu.tick();

    assert_eq!(device.borrow().output_log[0], value);
}

#[rstest]
// Opcode map for IN r, (C):
// 40: B, 48: C, 50: D, 58: E, 60: H, 68: L, 78: A
#[case::in_b(0x40, GPR::B)]
#[case::in_c(0x48, GPR::C)]
#[case::in_d(0x50, GPR::D)]
#[case::in_e(0x58, GPR::E)]
#[case::in_h(0x60, GPR::H)]
#[case::in_l(0x68, GPR::L)]
#[case::in_a(0x78, GPR::A)]
fn test_in_r_c(#[case] opcode: u8, #[case] target: GPR) {
    let mut cpu = setup_cpu();
    let val = 0x99;
    let port = 0x30;

    let device = Rc::new(RefCell::new(MockIODevice::new(port, vec![val])));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::C, port as u8); // Port low byte
    cpu.set_register(GPR::B, 0x00); // BC high byte

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, opcode);

    cpu.tick();

    assert_eq!(cpu.get_register(target), val);

    // Check flags for 0x99 (1001 1001) -> S=1, Z=0, P=1(even)
    assert!(cpu.get_flag(Flag::S));
    assert!(!cpu.get_flag(Flag::Z));
    assert!(cpu.get_flag(Flag::PV));
}

#[rstest]
// Opcode map for OUT (C), r:
// 41: B, 49: C, 51: D, 59: E, 61: H, 69: L, 79: A
#[case::out_b(0x41, GPR::B)]
#[case::out_c(0x49, GPR::C)]
#[case::out_d(0x51, GPR::D)]
#[case::out_e(0x59, GPR::E)]
#[case::out_h(0x61, GPR::H)]
#[case::out_l(0x69, GPR::L)]
#[case::out_a(0x79, GPR::A)]
fn test_out_c_r(#[case] opcode: u8, #[case] source: GPR) {
    let val_to_write = 0x77;
    let port_addr = 0x40;

    let mut cpu = setup_cpu();

    if source == GPR::C {
        // Special case: OUT (C), C -> writes value of C to port C.
        // Let's use port_addr as both value and port.
        let device = Rc::new(RefCell::new(MockIODevice::new(port_addr, vec![])));
        cpu.attach_device(device.clone());
        cpu.set_register(GPR::C, port_addr as u8);

        cpu.memory.borrow_mut().write(0x0000, 0xED);
        cpu.memory.borrow_mut().write(0x0001, opcode);
        cpu.tick();

        assert_eq!(device.borrow().output_log[0], port_addr as u8);
    } else {
        let device = Rc::new(RefCell::new(MockIODevice::new(port_addr, vec![])));
        cpu.attach_device(device.clone());
        cpu.set_register(GPR::C, port_addr as u8);
        cpu.set_register(source, val_to_write);

        cpu.memory.borrow_mut().write(0x0000, 0xED);
        cpu.memory.borrow_mut().write(0x0001, opcode);
        cpu.tick();

        assert_eq!(device.borrow().output_log[0], val_to_write);
    }
}

#[test]
fn test_out_c_0() {
    // ED 71 -> OUT (C), 0
    let mut cpu = setup_cpu();
    let device = Rc::new(RefCell::new(MockIODevice::new(0x50, vec![])));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::C, 0x50);

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, 0x71);

    cpu.tick();

    assert_eq!(device.borrow().output_log[0], 0x00);
}

#[rstest]
// Block I/O single step
#[case::ini(0xA2, true, true)] // INI:  IN,  Inc HL
#[case::ind(0xAA, true, false)] // IND:  IN,  Dec HL
#[case::outi(0xA3, false, true)] // OUTI: OUT, Inc HL
#[case::outd(0xAB, false, false)] // OUTD: OUT, Dec HL
fn test_block_io_step(#[case] opcode: u8, #[case] is_in: bool, #[case] inc_hl: bool) {
    let mut cpu = setup_cpu();
    let port = 0x60;
    let val = 0xAA;

    let device = Rc::new(RefCell::new(MockIODevice::new(port, vec![val])));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::C, port as u8);
    cpu.set_register(GPR::B, 0x05);
    cpu.set_register_pair(RegisterPair::HL, 0x1000);

    if !is_in {
        // Setup memory for OUT
        cpu.memory.borrow_mut().write(0x1000, val);
    }

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, opcode);

    cpu.tick();

    // Verify
    if is_in {
        assert_eq!(cpu.memory.borrow().read(0x1000), val);
    } else {
        assert_eq!(device.borrow().output_log[0], val);
    }

    assert_eq!(cpu.get_register(GPR::B), 0x04);

    let expected_hl = if inc_hl { 0x1001 } else { 0x0FFF };
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), expected_hl);

    assert!(!cpu.get_flag(Flag::Z));
    assert!(cpu.get_flag(Flag::N)); // Block IO instructions set N
}

#[rstest]
// Test Z flag when B reaches 0
#[case::ini_z(0xA2)]
fn test_block_io_zero_flag(#[case] opcode: u8) {
    let mut cpu = setup_cpu();
    let device = Rc::new(RefCell::new(MockIODevice::new(0x60, vec![0xBB])));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::C, 0x60);
    cpu.set_register(GPR::B, 0x01); // Will decrement to 0
    cpu.set_register_pair(RegisterPair::HL, 0x2000);

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, opcode);

    cpu.tick();

    assert_eq!(cpu.get_register(GPR::B), 0x00);
    assert!(cpu.get_flag(Flag::Z));
}

#[rstest]
// Block Repeat Instructions
// INDR (ED BA), OTDR (ED BB), INIR (ED B2), OTIR (ED B3)
#[case::indr(0xBA, true, false)] // INDR: IN, Dec
#[case::otdr(0xBB, false, false)] // OTDR: OUT, Dec
#[case::inir(0xB2, true, true)] // INIR: IN, Inc
#[case::otir(0xB3, false, true)] // OTIR: OUT, Inc
fn test_block_io_repeat(#[case] opcode: u8, #[case] is_in: bool, #[case] inc_hl: bool) {
    let mut cpu = setup_cpu();
    let port = 0x80;

    // For IN: queue 2 bytes
    // For OUT: we will inspect log
    let device_vals = vec![0x11, 0x22];
    let device = Rc::new(RefCell::new(MockIODevice::new(port, device_vals.clone())));
    cpu.attach_device(device.clone());

    cpu.set_register(GPR::C, port as u8);
    cpu.set_register(GPR::B, 0x02);
    cpu.set_register_pair(RegisterPair::HL, 0x4000);

    if !is_in {
        // Setup memory for OUT (0x4000, 0x3FFF or 0x4001 depending on inc/dec)
        cpu.memory.borrow_mut().write(0x4000, 0x11);
        if inc_hl {
            cpu.memory.borrow_mut().write(0x4001, 0x22);
        } else {
            cpu.memory.borrow_mut().write(0x3FFF, 0x22);
        }
    }

    cpu.memory.borrow_mut().write(0x0000, 0xED);
    cpu.memory.borrow_mut().write(0x0001, opcode);

    // Iteration 1
    cpu.tick();

    assert_eq!(cpu.get_register(GPR::B), 0x01);
    assert!(!cpu.get_flag(Flag::Z));
    assert_eq!(cpu.pc, 0x0000, "Should repeat");

    if is_in {
        assert_eq!(cpu.memory.borrow().read(0x4000), 0x11);
    } else {
        assert_eq!(device.borrow().output_log[0], 0x11);
    }

    // Iteration 2
    cpu.tick();

    assert_eq!(cpu.get_register(GPR::B), 0x00);
    assert!(cpu.get_flag(Flag::Z));
    assert_ne!(cpu.pc, 0x0000, "Should advance");

    // Address check
    let expected_final_hl = if inc_hl { 0x4002 } else { 0x3FFE };
    assert_eq!(cpu.get_register_pair(RegisterPair::HL), expected_final_hl);

    if is_in {
        let addr2 = if inc_hl { 0x4001 } else { 0x3FFF };
        assert_eq!(cpu.memory.borrow().read(addr2), 0x22);
    } else {
        assert_eq!(device.borrow().output_log[1], 0x22);
    }
}
