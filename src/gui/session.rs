use crate::emulator::{Machine, MachineSnapshot, SnapshotDelta};
use crate::gui::exec::{Command, Runner, RunnerImpl};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionState {
    #[default]
    Idle,
    Running,
    Paused {
        at: Option<u16>,
    },
}

impl SessionState {
    pub fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn last_stop_addr(self) -> Option<u16> {
        match self {
            Self::Paused { at } => at,
            Self::Idle | Self::Running => None,
        }
    }
}

/// Execution state: the machine, the cached snapshot, the runner and the
/// explicit lifecycle state that replaces the old `is_running` /
/// `last_stop_addr` pair.
pub struct DebugSession {
    machine: Option<Machine>,
    snapshot: MachineSnapshot,
    runner: RunnerImpl,
    breakpoint_addrs: HashSet<u16>,
    state: SessionState,
}

impl DebugSession {
    pub fn new() -> Self {
        Self {
            machine: Some(Machine::new()),
            snapshot: MachineSnapshot::default(),
            runner: crate::gui::exec::create_runner(),
            breakpoint_addrs: HashSet::new(),
            state: SessionState::Idle,
        }
    }

    pub fn refresh_snapshot(&mut self) {
        if let Some(machine) = self.machine.as_mut() {
            machine.snapshot_into(&mut self.snapshot);
        }
    }

    pub fn apply_delta(&mut self, delta: SnapshotDelta) {
        for (addr, value) in delta.memory {
            self.snapshot.memory[addr as usize] = value;
        }
        let mut state = delta.state;
        std::mem::swap(&mut state.memory, &mut self.snapshot.memory);
        self.snapshot = state;
    }

    pub fn ensure_machine_reclaimed(&mut self) {
        if self.machine.is_none() && !self.runner().is_running() {
            self.machine = Some(self.runner_mut().take_machine());
            self.refresh_snapshot();
        }
    }

    pub fn begin_run(&mut self, breakpoints: HashSet<u16>) {
        let machine = self.take_machine();
        self.state = SessionState::Running;
        self.runner.start(machine, breakpoints);
    }

    pub fn finish_run(&mut self, at: u16) {
        self.state = SessionState::Paused { at: Some(at) };
    }

    pub fn pause(&mut self) {
        self.state = SessionState::Paused {
            at: self.state.last_stop_addr(),
        };
    }

    pub fn reset(&mut self) {
        self.state = SessionState::Idle;
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state().is_running()
    }

    pub fn last_stop_addr(&self) -> Option<u16> {
        self.state().last_stop_addr()
    }

    pub fn snapshot(&self) -> &MachineSnapshot {
        &self.snapshot
    }

    pub fn machine(&self) -> Option<&Machine> {
        self.machine.as_ref()
    }

    pub fn machine_mut(&mut self) -> Option<&mut Machine> {
        self.machine.as_mut()
    }

    pub fn runner(&self) -> &RunnerImpl {
        &self.runner
    }

    pub fn runner_mut(&mut self) -> &mut RunnerImpl {
        &mut self.runner
    }

    pub fn set_machine(&mut self, machine: Option<Machine>) {
        self.machine = machine;
    }

    pub fn take_machine(&mut self) -> Machine {
        self.machine.take().unwrap_or_default()
    }

    pub fn breakpoints(&self) -> &HashSet<u16> {
        &self.breakpoint_addrs
    }

    pub fn set_breakpoints(&mut self, breakpoints: HashSet<u16>) {
        let changed = breakpoints != self.breakpoint_addrs;
        self.breakpoint_addrs = breakpoints;
        if changed && self.is_running() {
            self.runner
                .send_command(Command::SetBreakpoints(self.breakpoint_addrs.clone()));
        }
    }
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{DebugSession, SessionState};
    use crate::emulator::Machine;
    use crate::gui::exec::Runner;
    use std::collections::HashSet;

    #[test]
    fn run_state_transitions() {
        let mut session = DebugSession::new();

        session.begin_run(HashSet::new());
        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(session.last_stop_addr(), None);

        session.finish_run(0x1234);
        assert_eq!(session.state(), SessionState::Paused { at: Some(0x1234) });

        // Stopping keeps the address of the last breakpoint hit.
        session.pause();
        assert_eq!(session.last_stop_addr(), Some(0x1234));

        session.reset();
        assert_eq!(session.state(), SessionState::Idle);
        assert_eq!(session.last_stop_addr(), None);

        // The run left a machine inside the runner; reclaim it so the worker
        // thread terminates instead of living until the test process exits.
        let _ = session.runner_mut().take_machine();
    }

    #[test]
    fn applying_a_delta_keeps_unchanged_memory() {
        let mut session = DebugSession::new();
        let mut machine = Machine::new();
        machine.memory_mut().write(0x1000, 0x11);
        machine.memory_mut().write(0x1001, 0x22);
        session.set_machine(Some(machine));
        session.refresh_snapshot();

        let mut machine = Machine::new();
        machine.memory_mut().write(0x1001, 0x33);
        session.apply_delta(machine.snapshot_delta());

        assert_eq!(session.snapshot().memory[0x1000], 0x11);
        assert_eq!(session.snapshot().memory[0x1001], 0x33);
    }
}
