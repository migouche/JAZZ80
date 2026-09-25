use super::{Command, Event, Machine, SLICE_TICKS, StopReason};
use crate::emulator::SliceResult;
use std::collections::{HashSet, VecDeque};

pub struct Cooperative {
    machine: Option<Machine>,
    breakpoints: HashSet<u16>,
    finished: Option<StopReason>,
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

    pub fn start(&mut self, machine: Machine, breakpoints: HashSet<u16>) {
        assert!(self.machine.is_none(), "runner already running");
        self.machine = Some(machine);
        self.breakpoints = breakpoints;
        self.finished = None;
        self.pending.clear();
    }

    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        let machine = match self.machine.as_mut() {
            Some(m) => m,
            None => return events,
        };
        if self.finished.is_some() {
            return events;
        }

        let mut stop = false;
        while let Some(command) = self.pending.pop_front() {
            if stop {
                self.pending.clear();
                break;
            }
            match command {
                Command::Nmi => machine.cpu.nmi(),
                Command::SetInterrupt(v) => machine.cpu.set_interrupt(v),
                Command::SetBreakpoints(bp) => self.breakpoints = bp,
                Command::Resume => machine.cpu.set_halted(false),
                Command::Stop => stop = true,
            }
        }
        if stop {
            let reason = StopReason::Stopped;
            self.finished = Some(reason);
            events.push(Event::Finished(reason));
            events.push(Event::Snapshot(machine.snapshot()));
            return events;
        }

        match machine.run_slice(SLICE_TICKS, &self.breakpoints) {
            SliceResult::BudgetExhausted { .. } | SliceResult::Halted { .. } => {
                events.push(Event::Snapshot(machine.snapshot()));
            }
            SliceResult::HitBreakpoint { .. } => {
                let reason = StopReason::Breakpoint(machine.cpu.get_pc());
                self.finished = Some(reason);
                events.push(Event::Finished(reason));
                events.push(Event::Snapshot(machine.snapshot()));
            }
        }
        events
    }

    pub fn send_command(&mut self, command: Command) {
        if self.machine.is_some() && self.finished.is_none() {
            self.pending.push_back(command);
        }
    }

    pub fn take_machine(&mut self) -> Machine {
        self.finished = None;
        self.pending.clear();
        self.machine.take().unwrap_or_default()
    }

    pub fn is_running(&self) -> bool {
        self.machine.is_some() && self.finished.is_none()
    }
}

impl Default for Cooperative {
    fn default() -> Self {
        Self::new()
    }
}
