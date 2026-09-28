use super::{Command, Event, Machine, Runner, SLICE_TICKS, StopReason};
use crate::traits::SynchronousComponent;
use std::collections::{HashSet, VecDeque};

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct Cooperative {
    machine: Option<Machine>,
    breakpoints: HashSet<u16>,
    finished: Option<StopReason>,
    pending: VecDeque<Command>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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
                Command::SetPC(v) => machine.cpu.set_pc(v),
                Command::SetSP(v) => machine.cpu.set_sp(v),
                Command::SetIX(v) => machine.cpu.set_ix(v),
                Command::SetIY(v) => machine.cpu.set_iy(v),
                Command::SetRegister(r, v) => machine.cpu.set_register(r, v),
                Command::SetShadowRegister(r, v) => machine.cpu.set_shadow_register(r, v),
                Command::SetFlag(f, v) => machine.cpu.set_flag(v, f),
                Command::Tick => machine.cpu.tick(),
                Command::SetHalted(v) => machine.cpu.set_halted(v),
            }
        }
        if stop {
            let reason = StopReason::Stopped;
            self.finished = Some(reason);
            events.push(Event::Finished(reason));
            events.push(Event::Snapshot(machine.snapshot()));
            return events;
        }

        let mut cycles = 0u64;
        loop {
            if self.breakpoints.contains(&machine.cpu.get_pc()) {
                let reason = StopReason::Breakpoint(machine.cpu.get_pc());
                self.finished = Some(reason);
                events.push(Event::Finished(reason));
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
