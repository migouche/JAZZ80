use crate::components::devices::Keypad;
use crate::components::memories::mem_64k::Mem64k;
use crate::cpu::Z80A;
use crate::traits::{MemoryMapper, SynchronousComponent};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_wake_from_halt_on_interrupt() {
    let memory = Rc::new(RefCell::new(Mem64k::new()));
    let mut cpu = Z80A::new(memory.clone());

    // Connect Keypad
    let keypad = Rc::new(RefCell::new(Keypad::new(0x01)));
    cpu.attach_device(keypad.clone());

    // Code:
    // EI       (FB)
    // HALT     (76)
    // NOP      (00)

    memory.borrow_mut().write(0x0000, 0xFB); // EI
    memory.borrow_mut().write(0x0001, 0x76); // HALT
    memory.borrow_mut().write(0x0002, 0x00); // NOP

    // Run EI
    cpu.tick();
    assert_eq!(cpu.get_pc(), 0x0001);
    assert!(!cpu.get_iff1()); // Delay 2->1

    // Run HALT
    cpu.tick();
    assert_eq!(cpu.get_pc(), 0x0002); // PC increments after fetch, then HALT executes
    // Wait... does fetch increment PC?
    // fetch() reads at PC, then increments PC.
    // So instruction at 0x0001 is execution. PC becomes 0x0002.
    // HALT executes. sets halted=true.
    // Delay 1->0. Sets IFF1=true.

    assert!(cpu.is_halted(), "CPU should be halted");
    assert!(
        cpu.get_iff1(),
        "Interrupts should be enabled after HALT instruction finishes delay"
    );

    // Tick again (should stay halted)
    cpu.tick();
    assert!(cpu.is_halted(), "CPU should remain halted if no interrupt");

    // ... previous code ...
    // Trigger Interrupt
    keypad.borrow_mut().press_key(0x42);

    // Tick again (Handle Interrupt)
    cpu.tick();

    // Should wake up and be at 0x0038
    assert!(!cpu.is_halted(), "CPU should wake up");
    assert_eq!(cpu.get_pc(), 0x0038);

    // Write ISR at 0x0038
    // IN A, (01h)  -> DB 01
    // OUT (02h), A -> D3 02
    // RETI         -> ED 4D
    memory.borrow_mut().write(0x0038, 0xDB);
    memory.borrow_mut().write(0x0039, 0x01);
    memory.borrow_mut().write(0x003A, 0xD3);
    memory.borrow_mut().write(0x003B, 0x02);
    memory.borrow_mut().write(0x003C, 0xED);
    memory.borrow_mut().write(0x003D, 0x4D);

    // Attach Display
    use crate::components::devices::SevenSegmentDisplay;
    let display = Rc::new(RefCell::new(SevenSegmentDisplay::new(0x02)));
    cpu.attach_device(display.clone());

    // Execute IN A, (01h)
    cpu.tick(); // Fetch DB
    // cpu.tick(); // Param 01? No, fetch executes.
    // Wait, execute_instruction fetches operands.
    // check pc.
    assert_eq!(cpu.get_pc(), 0x003A);
    assert_eq!(cpu.get_register(crate::cpu::GPR::A), 0x42);

    // Execute OUT (02h), A
    cpu.tick();
    assert_eq!(cpu.get_pc(), 0x003C);
    assert_eq!(display.borrow().get_value(), 0x42);

    // Execute RETI
    cpu.tick(); // ED
    // 4D is fetched inside.
    // Returns to 0x0002 (after HALT?)
    // Wait, HALT pc was 0x0002.
    // Interrupt pushed 0x0002.
    assert_eq!(cpu.get_pc(), 0x0002);
}
