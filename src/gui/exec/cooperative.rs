use super::{Command, Event, Machine, Runner, SLICE_TICKS};
use crate::traits::SynchronousComponent;
use std::collections::{HashSet, VecDeque};

pub struct Cooperative {
    machine: Option<Machine>,
    breakpoints: HashSet<u16>,
    finished: Option<u16>,
    pending: VecDeque<Command>,
}

impl Cooperative {
    pub fn new() -> Self {
        Self {
            machine: None,
            breakpoints: HashSet::new(),
            finished: None,
            pending: VecDeque::new(),
        }
    }
}

impl Runner for Cooperative {
    fn start(&mut self, machine: Machine, breakpoints: HashSet<u16>) {
        assert!(self.machine.is_none(), "runner already running");
        self.machine = Some(machine);
        self.breakpoints = breakpoints;
        self.finished = None;
        self.pending.clear();
    }

    fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        let machine = match self.machine.as_mut() {
            Some(m) => m,
            None => return events,
        };
        if self.finished.is_some() {
            return events;
        }

        while let Some(command) = self.pending.pop_front() {
            match command {
                Command::Nmi => machine.cpu.nmi(),
                Command::SetInterrupt(v) => machine.cpu.set_interrupt(v),
                Command::SetBreakpoints(bp) => self.breakpoints = bp,
                Command::Resume => machine.cpu.set_halted(false),
                Command::Tick => machine.cpu.tick(),
            }
        }

        let mut cycles = 0u64;
        loop {
            if self.breakpoints.contains(&machine.cpu.get_pc()) {
                let addr = machine.cpu.get_pc();
                self.finished = Some(addr);
                events.push(Event::Finished(addr));
                events.push(Event::Snapshot(machine.snapshot()));
                return events;
            }
            if cycles >= SLICE_TICKS {
                events.push(Event::Snapshot(machine.snapshot()));
                break;
            }
            machine.cpu.tick();
            cycles += 1;
            if machine.cpu.is_halted() {
                events.push(Event::Snapshot(machine.snapshot()));
                break;
            }
        }
        events
    }

    fn send_command(&mut self, command: Command) {
        if self.machine.is_some() && self.finished.is_none() {
            self.pending.push_back(command);
        }
    }

    fn take_machine(&mut self) -> Machine {
        self.finished = None;
        self.pending.clear();
        self.machine.take().unwrap_or_default()
    }

    fn is_running(&self) -> bool {
        self.machine.is_some() && self.finished.is_none()
    }
}

impl Default for Cooperative {
    fn default() -> Self {
        Self::new()
    }
}
