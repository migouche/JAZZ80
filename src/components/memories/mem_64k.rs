use crate::traits::MemoryMapper;

pub struct Mem64k {
    data: [u8; 0x10000],
}

impl Mem64k {
    pub fn new() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl MemoryMapper for Mem64k {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.data[address as usize] = data;
    }
}

#[cfg(test)]
mod tests;
