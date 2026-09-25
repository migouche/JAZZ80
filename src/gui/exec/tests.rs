use super::*;
use crate::cpu::{GPR, Z80A};
use std::time::Instant;

fn machine_with(bytes: &[u8]) -> Machine {
    let mut cpu = Z80A::new(crate::components::memories::mem_64k::Mem64k::new());
    for (i, b) in bytes.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    Machine { cpu }
}

fn drain_until_finished(runner: &mut Runner, max: Duration) -> Event {
    let deadline = Instant::now() + max;
    loop {
        for event in runner.poll() {
            if matches!(&event, Event::Finished(_)) {
                return event;
            }
        }
        assert!(Instant::now() < deadline, "timed out waiting for finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn threaded_hits_breakpoint_and_returns_machine() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    let mut runner = Runner::threaded();
    runner.start(machine, breakpoints);

    match drain_until_finished(&mut runner, Duration::from_secs(5)) {
        Event::Finished(StopReason::Breakpoint(0x0002)) => {}
        other => panic!("unexpected event: {:?}", std::mem::discriminant(&other)),
    }

    let machine = runner.take_machine();
    assert!(!runner.is_running());
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(GPR::A), 0x42);
}

#[test]
fn threaded_stop_reclaims_machine() {
    let machine = machine_with(&[0x3E, 0x42]);
    let breakpoints = HashSet::new();

    let mut runner = Runner::threaded();
    runner.start(machine, breakpoints);
    std::thread::sleep(Duration::from_millis(50));
    assert!(runner.is_running());

    let machine = runner.take_machine();
    assert!(!runner.is_running());
    assert_ne!(machine.cpu.get_register(GPR::A), 0);
}

#[test]
fn threaded_delivers_snapshots_while_running() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut runner = Runner::threaded();
    runner.start(machine, HashSet::new());

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut saw_snapshot = false;
    while Instant::now() < deadline {
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

#[test]
fn threaded_nmi_command_wakes_halted_cpu() {
    // HALT at 0x0000, then `JP $` (infinite loop) so the woken CPU keeps
    // running instead of re-executing the HALT after wrapping around memory.
    let mut machine = machine_with(&[0x76, 0xC3, 0x01, 0x00]);
    machine.cpu.memory.write(0x0066, 0xC9);
    let mut runner = Runner::threaded();
    runner.start(machine, HashSet::new());

    std::thread::sleep(Duration::from_millis(40));
    runner.send_command(Command::Nmi);
    std::thread::sleep(Duration::from_millis(40));

    let machine = runner.take_machine();
    assert!(!machine.cpu.is_halted(), "NMI did not wake the halted CPU");
}

#[test]
fn cooperative_hits_breakpoint_and_returns_machine() {
    let machine = machine_with(&[0x3E, 0x42]);
    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    let mut runner = Runner::cooperative();
    runner.start(machine, breakpoints);

    let mut saw = false;
    for _ in 0..20 {
        for event in runner.poll() {
            if let Event::Finished(StopReason::Breakpoint(0x0002)) = event {
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

#[test]
fn cooperative_stop_and_commands() {
    let machine = machine_with(&[0x76]);

    let mut runner = Runner::cooperative();
    runner.start(machine, HashSet::new());

    let _ = runner.poll();
    let _ = runner.poll();

    runner.send_command(Command::SetInterrupt(true));
    let machine = runner.take_machine();
    assert!(machine.cpu.is_halted());
}
