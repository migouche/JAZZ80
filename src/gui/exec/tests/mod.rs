use crate::cpu::Z80A;
use crate::gui::Machine;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests;

pub fn machine_with(bytes: &[u8]) -> Machine {
    let mut cpu = Z80A::new(crate::components::memories::mem_64k::Mem64k::new());
    for (i, b) in bytes.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    Machine { cpu }
}
