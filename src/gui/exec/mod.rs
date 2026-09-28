#![cfg_attr(target_arch = "wasm32", allow(dead_code, unused_imports))]

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
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Stop,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetPC(u16),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetSP(u16),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetIX(u16),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetIY(u16),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetRegister(crate::cpu::GPR, u8),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetShadowRegister(crate::cpu::GPR, u8),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetFlag(crate::cpu::Flag, bool),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Tick,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    SetHalted(bool),
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
