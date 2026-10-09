use crate::gui::exec::cooperative::Cooperative;
use crate::gui::exec::tests::machine_with;
use crate::gui::exec::Runner;
use crate::gui::{Command, Event, Machine, MachineSnapshot};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[wasm_bindgen_test::wasm_bindgen_test]
fn cooperative_hits_breakpoint_and_returns_machine() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    let mut runner = Cooperative::new();
    runner.start(machine, breakpoints);

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut saw = false;
    while Instant::now() < deadline {
        for event in runner.poll() {
            if let Event::Finished(0x0002) = event {
                saw = true;
            }
        }
        if saw {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(saw, "cooperative runner never finished");
    assert!(!runner.is_running());

    let machine = runner.take_machine();
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(crate::cpu::GPR::A), 0x42);
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn cooperative_delivers_snapshots_while_running() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut runner = Cooperative::new();
    runner.start(machine, HashSet::new());

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut saw_snapshot = false;
    while Instant::now() < deadline {
        for event in runner.poll() {
            if let Event::Snapshot(s) = &event {
                if s.pc > 0 {
                    saw_snapshot = true;
                }
            }
        }
        if saw_snapshot {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(saw_snapshot, "no snapshot received");
    let _ = runner.take_machine();
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn cooperative_nmi_command_wakes_halted_cpu() {
    let mut machine = machine_with(&[0x76, 0xC3, 0x01, 0x00]);
    machine.cpu.memory.write(0x0066, 0xC9);
    let mut runner = Cooperative::new();
    runner.start(machine, HashSet::new());

    std::thread::sleep(Duration::from_millis(40));
    runner.send_command(Command::Nmi);
    std::thread::sleep(Duration::from_millis(40));

    let machine = runner.take_machine();
    assert!(!machine.cpu.is_halted(), "NMI did not wake the halted CPU");
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
