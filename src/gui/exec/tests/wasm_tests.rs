use crate::gui::Command;
use crate::gui::Event;
use crate::gui::GPR;

use crate::gui::exec::Runner;
use crate::gui::exec::cooperative::Cooperative;
use crate::gui::exec::tests::machine_with;
use std::collections::HashSet;

#[wasm_bindgen_test::wasm_bindgen_test]
fn cooperative_hits_breakpoint_and_returns_machine() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    let mut runner = Cooperative::new();
    runner.start(machine, breakpoints);

    let mut saw = false;
    for _ in 0..20 {
        for event in runner.poll() {
            if let Event::Finished(0x0002) = event {
                saw = true;
            }
        }
        if saw {
            break;
        }
    }
    assert!(saw, "cooperative runner never finished");
    assert!(!runner.is_running());

    let machine = runner.take_machine();
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(GPR::A), 0x42);
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn cooperative_stop_and_commands() {
    let machine = machine_with(&[0x76]);

    let mut runner = Cooperative::new();
    runner.start(machine, HashSet::new());

    let _ = runner.poll();
    let _ = runner.poll();

    runner.send_command(Command::SetInterrupt(true));
    let machine = runner.take_machine();
    assert!(machine.cpu.is_halted());
}
