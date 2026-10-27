use crate::components::memories::mem_64k::Mem64k;
use crate::cpu::{Flag, GPR, Z80A};
#[cfg(not(target_arch = "wasm32"))]
use crate::traits::SynchronousComponent;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;

pub struct Machine {
    pub cpu: Z80A,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Z80A::new(Mem64k::new()),
        }
    }

    pub fn memory_mut(&mut self) -> &mut dyn crate::traits::MemoryMapper {
        self.cpu.memory.as_mut()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_slice(&mut self, max_ticks: u64, breakpoints: &HashSet<u16>) -> SliceResult {
        let mut cycles = 0u64;
        loop {
            if breakpoints.contains(&self.cpu.get_pc()) {
                return SliceResult::HitBreakpoint { cycles };
            }
            if cycles >= max_ticks {
                return SliceResult::BudgetExhausted { cycles };
            }
            self.cpu.tick();
            cycles += 1;
            if self.cpu.is_halted() {
                return SliceResult::Halted { cycles };
            }
        }
    }
    #[cfg(test)]
    pub fn snapshot(&mut self) -> MachineSnapshot {
        let mut out = MachineSnapshot::default();
        self.snapshot_into(&mut out);
        out
    }

    pub fn snapshot_into(&mut self, out: &mut MachineSnapshot) {
        let _ = self.cpu.memory.take_dirty();
        out.state_from(self);
        for (addr, slot) in out.memory.iter_mut().enumerate() {
            *slot = self.cpu.memory.read(addr as u16);
        }
    }

    pub fn snapshot_delta(&mut self) -> SnapshotDelta {
        let memory = self.cpu.memory.take_dirty().unwrap_or_else(|| {
            (0..=u16::MAX)
                .map(|addr| (addr, self.cpu.memory.read(addr)))
                .collect()
        });
        let mut state = MachineSnapshot::default();
        state.state_from(self);
        SnapshotDelta { state, memory }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn mark_dirty(&mut self, addresses: impl IntoIterator<Item = u16>) {
        for address in addresses {
            self.cpu.memory.mark_dirty(address);
        }
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

const GPR_ORDER: [GPR; 8] = [
    GPR::A,
    GPR::F,
    GPR::B,
    GPR::C,
    GPR::D,
    GPR::E,
    GPR::H,
    GPR::L,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotFlags {
    pub s: bool,
    pub z: bool,
    pub y: bool,
    pub h: bool,
    pub x: bool,
    pub pv: bool,
    pub n: bool,
    pub c: bool,
}

#[derive(Clone, Debug)]
pub struct MachineSnapshot {
    pub pc: u16,
    pub sp: u16,
    pub ix: u16,
    pub iy: u16,
    pub regs: [u8; 8],
    pub shadow_regs: [u8; 8],
    pub iff1: bool,
    pub iff2: bool,
    pub interrupt_mode: u8,
    pub halted: bool,
    pub flags: SnapshotFlags,
    pub memory: Box<[u8; 0x10000]>,
}

impl Default for MachineSnapshot {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0,
            ix: 0,
            iy: 0,
            regs: [0; 8],
            shadow_regs: [0; 8],
            iff1: false,
            iff2: false,
            interrupt_mode: 0,
            halted: false,
            flags: SnapshotFlags {
                s: false,
                z: false,
                y: false,
                h: false,
                x: false,
                pv: false,
                n: false,
                c: false,
            },
            memory: Box::new([0u8; 0x10000]),
        }
    }
}

impl MachineSnapshot {
    fn state_from(&mut self, machine: &Machine) {
        self.pc = machine.cpu.get_pc();
        self.sp = machine.cpu.get_sp();
        self.ix = machine.cpu.get_ix();
        self.iy = machine.cpu.get_iy();
        for (i, gpr) in GPR_ORDER.iter().enumerate() {
            self.regs[i] = machine.cpu.get_register(*gpr);
            self.shadow_regs[i] = machine.cpu.get_shadow_register(*gpr);
        }
        self.iff1 = machine.cpu.get_iff1();
        self.iff2 = machine.cpu.get_iff2();
        self.interrupt_mode = machine.cpu.get_interrupt_mode();
        self.halted = machine.cpu.is_halted();
        self.flags = SnapshotFlags {
            s: machine.cpu.get_flag(Flag::S),
            z: machine.cpu.get_flag(Flag::Z),
            y: machine.cpu.get_flag(Flag::Y),
            h: machine.cpu.get_flag(Flag::H),
            x: machine.cpu.get_flag(Flag::X),
            pv: machine.cpu.get_flag(Flag::PV),
            n: machine.cpu.get_flag(Flag::N),
            c: machine.cpu.get_flag(Flag::C),
        };
    }
}

#[derive(Clone, Debug)]
pub struct SnapshotDelta {
    pub state: MachineSnapshot,
    pub memory: Vec<(u16, u8)>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceResult {
    BudgetExhausted { cycles: u64 },
    Halted { cycles: u64 },
    HitBreakpoint { cycles: u64 },
}

#[cfg(test)]
mod tests;
