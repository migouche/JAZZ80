use crate::components::memories::mem_64k::Mem64k;
use crate::cpu::{Flag, GPR, Z80A};
use crate::traits::SynchronousComponent;
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

    pub fn snapshot(&self) -> MachineSnapshot {
        let mut regs = [0u8; 8];
        for (i, gpr) in GPR_ORDER.iter().enumerate() {
            regs[i] = self.cpu.get_register(*gpr);
        }
        let mut shadow_regs = [0u8; 8];
        for (i, gpr) in GPR_ORDER.iter().enumerate() {
            shadow_regs[i] = self.cpu.get_shadow_register(*gpr);
        }

        let mut memory = vec![0u8; 0x10000];
        for (addr, byte) in memory.iter_mut().enumerate() {
            *byte = self.cpu.memory.read(addr as u16);
        }

        MachineSnapshot {
            pc: self.cpu.get_pc(),
            sp: self.cpu.get_sp(),
            ix: self.cpu.get_ix(),
            iy: self.cpu.get_iy(),
            regs,
            shadow_regs,
            iff1: self.cpu.get_iff1(),
            iff2: self.cpu.get_iff2(),
            interrupt_mode: self.cpu.get_interrupt_mode(),
            halted: self.cpu.is_halted(),
            flags: SnapshotFlags {
                s: self.cpu.get_flag(Flag::S),
                z: self.cpu.get_flag(Flag::Z),
                y: self.cpu.get_flag(Flag::Y),
                h: self.cpu.get_flag(Flag::H),
                x: self.cpu.get_flag(Flag::X),
                pv: self.cpu.get_flag(Flag::PV),
                n: self.cpu.get_flag(Flag::N),
                c: self.cpu.get_flag(Flag::C),
            },
            memory,
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
    pub memory: Vec<u8>,
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
            memory: vec![0u8; 0x10000],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceResult {
    BudgetExhausted { cycles: u64 },
    Halted { cycles: u64 },
    HitBreakpoint { cycles: u64 },
}

#[cfg(test)]
mod tests;
