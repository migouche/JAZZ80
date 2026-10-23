use crate::traits::MemoryMapper;

const DIRTY_WORDS: usize = 0x10000 / 64;

pub struct Mem64k {
    data: [u8; 0x10000],
    dirty: [u64; DIRTY_WORDS],
}

impl Mem64k {
    pub fn new() -> Self {
        Self {
            data: [0; 0x10000],
            dirty: [0; DIRTY_WORDS],
        }
    }
}

impl MemoryMapper for Mem64k {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        let addr = address as usize;
        self.data[addr] = data;
        self.dirty[addr / 64] |= 1 << (addr % 64);
    }

    fn take_dirty(&mut self) -> Option<Vec<(u16, u8)>> {
        let mut changed = Vec::new();
        for (word_index, word) in self.dirty.iter_mut().enumerate() {
            while *word != 0 {
                let bit = word.trailing_zeros() as usize;
                let addr = word_index * 64 + bit;
                changed.push((addr as u16, self.data[addr]));
                *word &= *word - 1;
            }
        }
        Some(changed)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn mark_dirty(&mut self, address: u16) {
        let addr = address as usize;
        self.dirty[addr / 64] |= 1 << (addr % 64);
    }
}

#[cfg(test)]
mod tests;
