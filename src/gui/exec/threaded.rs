use super::{Command, Event, Machine, Runner, SLICE_TICKS, SNAPSHOT_INTERVAL};
use crate::emulator::SliceResult;
use crate::traits::SynchronousComponent;
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const EVENT_CHANNEL_CAPACITY: usize = 4;

const BREATHER: Duration = Duration::from_micros(500);

pub struct Threaded {
    active: Option<Active>,
}

struct Active {
    cmd_tx: Sender<Command>,
    event_rx: Receiver<Event>,
    machine_rx: Receiver<Machine>,
    handle: JoinHandle<()>,
}

impl Threaded {
    pub fn new() -> Self {
        Self { active: None }
    }
}

impl Runner for Threaded {
    fn start(&mut self, machine: Machine, breakpoints: HashSet<u16>) {
        assert!(self.active.is_none(), "runner already running");
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::sync_channel(EVENT_CHANNEL_CAPACITY);
        let (machine_tx, machine_rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            worker_loop(machine, breakpoints, cmd_rx, event_tx, machine_tx);
        });
        self.active = Some(Active {
            cmd_tx,
            event_rx,
            machine_rx,
            handle,
        });
    }

    fn poll(&mut self) -> Vec<Event> {
        let Some(active) = &self.active else {
            return Vec::new();
        };
        active.event_rx.try_iter().collect()
    }

    fn send_command(&mut self, command: Command) {
        if let Some(active) = &self.active {
            let _ = active.cmd_tx.send(command);
        }
    }

    fn take_machine(&mut self) -> Machine {
        let Some(active) = self.active.take() else {
            return Machine::new();
        };
        let _ = active.cmd_tx.send(Command::Stop);
        // Wait for the machine with a timeout to avoid blocking the UI thread
        // indefinitely if the worker is in the middle of a long slice.
        match active.machine_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(machine) => machine,
            Err(_) => {
                // Timeout or channel closed; try to join the thread
                let _ = active.handle.join();
                active
                    .machine_rx
                    .try_recv()
                    .unwrap_or_else(|_| Machine::new())
            }
        }
    }

    fn is_running(&self) -> bool {
        self.active
            .as_ref()
            .map(|a| !a.handle.is_finished())
            .unwrap_or(false)
    }
}

impl Default for Threaded {
    fn default() -> Self {
        Self::new()
    }
}

fn worker_loop(
    mut machine: Machine,
    mut breakpoints: HashSet<u16>,
    cmd_rx: Receiver<Command>,
    event_tx: SyncSender<Event>,
    machine_tx: Sender<Machine>,
) {
    let mut last_snap = Instant::now();
    let stop_addr: Option<u16> = 'outer: loop {
        let mut stop = false;
        for command in cmd_rx.try_iter() {
            match command {
                Command::Nmi => machine.cpu.nmi(),
                Command::SetInterrupt(v) => machine.cpu.set_interrupt(v),
                Command::SetBreakpoints(bp) => breakpoints = bp,
                Command::Resume => machine.cpu.set_halted(false),
                Command::Stop => stop = true,
                Command::Tick => machine.cpu.tick(),
            }
        }
        if stop {
            break 'outer None;
        }
        match machine.run_slice(SLICE_TICKS, &breakpoints) {
            SliceResult::BudgetExhausted { .. } => {
                if last_snap.elapsed() >= SNAPSHOT_INTERVAL {
                    let _ = event_tx.try_send(Event::Snapshot(machine.snapshot()));
                    last_snap = Instant::now();
                }
                std::thread::sleep(BREATHER);
            }
            SliceResult::Halted { .. } => {
                let _ = event_tx.try_send(Event::Snapshot(machine.snapshot()));
                last_snap = Instant::now();
                std::thread::sleep(SNAPSHOT_INTERVAL);
            }
            SliceResult::HitBreakpoint { .. } => {
                let addr = machine.cpu.get_pc();
                let _ = event_tx.try_send(Event::Snapshot(machine.snapshot()));
                break 'outer Some(addr);
            }
        }
    };
    if let Some(addr) = stop_addr {
        let _ = event_tx.try_send(Event::Finished(addr));
    }
    let _ = event_tx.try_send(Event::Snapshot(machine.snapshot()));
    let _ = machine_tx.send(machine);
}
