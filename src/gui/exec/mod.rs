#[cfg(target_arch = "wasm32")]
pub mod cooperative;
#[cfg(not(target_arch = "wasm32"))]
pub mod threaded;

use crate::emulator::{Machine, MachineSnapshot};
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub const SLICE_TICKS: u64 = 50_000;

#[cfg(not(target_arch = "wasm32"))]
pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Clone, Debug)]
pub enum Command {
    Nmi,
    SetInterrupt(bool),
    SetBreakpoints(HashSet<u16>),
    Resume,
    #[cfg(not(target_arch = "wasm32"))]
    Stop,
    Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    #[cfg(not(target_arch = "wasm32"))]
    Stopped,
    Breakpoint(u16),
}

#[derive(Debug)]
pub enum Event {
    Snapshot(MachineSnapshot),
    Finished(StopReason),
}

pub trait Runner {
    fn start(&mut self, machine: Machine, breakpoints: HashSet<u16>);
    fn poll(&mut self) -> Vec<Event>;
    fn send_command(&mut self, command: Command);
    fn take_machine(&mut self) -> Machine;
    fn is_running(&self) -> bool;
}

#[cfg(not(target_arch = "wasm32"))]
pub type RunnerImpl = threaded::Threaded;

#[cfg(target_arch = "wasm32")]
pub type RunnerImpl = cooperative::Cooperative;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_runner() -> RunnerImpl {
    threaded::Threaded::new()
}

#[cfg(target_arch = "wasm32")]
pub fn create_runner() -> RunnerImpl {
    cooperative::Cooperative::new()
}

#[cfg(test)]
mod tests;
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests;
