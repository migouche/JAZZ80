pub mod cooperative;
pub mod threaded;

use crate::emulator::{Machine, MachineSnapshot};
use std::collections::HashSet;
use std::time::Duration;

pub const SLICE_TICKS: u64 = 50_000;

pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Clone, Debug)]
pub enum Command {
    Nmi,
    SetInterrupt(bool),
    SetBreakpoints(HashSet<u16>),
    Resume,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Stopped,
    Breakpoint(u16),
}

#[derive(Debug)]
pub enum Event {
    Snapshot(MachineSnapshot),
    Finished(StopReason),
}

pub enum Runner {
    Threaded(threaded::Threaded),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Cooperative(cooperative::Cooperative),
}

impl Runner {
    pub fn threaded() -> Self {
        Runner::Threaded(threaded::Threaded::new())
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn cooperative() -> Self {
        Runner::Cooperative(cooperative::Cooperative::new())
    }

    pub fn start(&mut self, machine: Machine, breakpoints: HashSet<u16>) {
        match self {
            Runner::Threaded(r) => r.start(machine, breakpoints),
            Runner::Cooperative(r) => r.start(machine, breakpoints),
        }
    }

    pub fn poll(&mut self) -> Vec<Event> {
        match self {
            Runner::Threaded(r) => r.poll(),
            Runner::Cooperative(r) => r.poll(),
        }
    }

    pub fn send_command(&mut self, command: Command) {
        match self {
            Runner::Threaded(r) => r.send_command(command),
            Runner::Cooperative(r) => r.send_command(command),
        }
    }

    pub fn take_machine(&mut self) -> Machine {
        match self {
            Runner::Threaded(r) => r.take_machine(),
            Runner::Cooperative(r) => r.take_machine(),
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            Runner::Threaded(r) => r.is_running(),
            Runner::Cooperative(r) => r.is_running(),
        }
    }
}

impl Default for Runner {
    fn default() -> Self {
        Runner::threaded()
    }
}

#[cfg(test)]
mod tests;
