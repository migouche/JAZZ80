use crate::traits::MemoryMapper;
use std::cell::Cell;

pub struct Mem64k {
    data: [u8; 0x10000],
    dirty: Vec<Cell<bool>>,
}

impl Mem64k {
    pub fn new() -> Self {
        Self {
            data: [0; 0x10000],
            dirty: (0..0x10000).map(|_| Cell::new(false)).collect(),
        }
    }
}

impl MemoryMapper for Mem64k {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.data[address as usize] = data;
        self.dirty[address as usize].set(true);
    }

    fn get_dirty(&self) -> Option<Vec<bool>> {
        Some(self.dirty.iter().map(|c| c.get()).collect())
    }

    fn clear_dirty(&self) {
        for c in &self.dirty {
            c.set(false);
        }
    }
}

#[cfg(test)]
mod tests;
