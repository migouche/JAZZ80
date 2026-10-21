use super::machine_with;
use crate::cpu::GPR;
use crate::gui::exec::threaded::Threaded;
use crate::gui::exec::{Command, Event, Runner};
use std::collections::HashSet;
use std::time::Duration;

pub fn drain_until_finished<R: Runner>(runner: &mut R, max: Duration) -> Event {
    let deadline = std::time::Instant::now() + max;
    loop {
        for event in runner.poll() {
            if matches!(&event, Event::Finished(_)) {
                return event;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for finish"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub fn hits_breakpoint_and_returns_machine<R: Runner>(mut runner: R) {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    runner.start(machine, breakpoints);

    match drain_until_finished(&mut runner, Duration::from_secs(5)) {
        Event::Finished(0x0002) => {}
        other => panic!("unexpected event: {:?}", std::mem::discriminant(&other)),
    }

    let machine = runner.take_machine();
    assert!(!runner.is_running());
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(crate::cpu::GPR::A), 0x42);
}

pub fn delivers_snapshots_while_running<R: Runner>(mut runner: R) {
    let machine = machine_with(&[0x3E, 0x42]);
    runner.start(machine, HashSet::new());

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut saw_snapshot = false;
    while std::time::Instant::now() < deadline {
        for event in runner.poll() {
            if matches!(&event, Event::Snapshot(s) if s.pc > 0) {
                saw_snapshot = true;
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

pub fn nmi_command_wakes_halted_cpu<R: Runner>(mut runner: R) {
    let mut machine = machine_with(&[0x76, 0xC3, 0x01, 0x00]);
    machine.cpu.memory.write(0x0066, 0xC9);
    runner.start(machine, HashSet::new());

    std::thread::sleep(Duration::from_millis(40));
    runner.send_command(Command::Nmi);
    std::thread::sleep(Duration::from_millis(40));

    let machine = runner.take_machine();
    assert!(!machine.cpu.is_halted(), "NMI did not wake the halted CPU");
}

#[test]
fn threaded_hits_breakpoint_and_returns_machine() {
    hits_breakpoint_and_returns_machine(Threaded::new());
}

#[test]
fn threaded_delivers_snapshots_while_running() {
    delivers_snapshots_while_running(Threaded::new());
}

#[test]
fn threaded_nmi_command_wakes_halted_cpu() {
    nmi_command_wakes_halted_cpu(Threaded::new());
}

#[test]
fn threaded_stop_reclaims_machine() {
    let machine = super::machine_with(&[0x3E, 0x42]);
    let breakpoints = HashSet::new();

    let mut runner = Threaded::new();
    runner.start(machine, breakpoints);
    std::thread::sleep(Duration::from_millis(50));
    assert!(runner.is_running());

    let machine = runner.take_machine();
    assert!(!runner.is_running());
    assert_ne!(machine.cpu.get_register(GPR::A), 0);
}
