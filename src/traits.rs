pub trait SynchronousComponent {
    fn tick(&mut self);
}

pub trait MemoryMapper {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);

    fn read_word(&self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address.overflowing_add(1).0);
        ((high as u16) << 8) | (low as u16)
    }

    fn write_word(&mut self, address: u16, data: u16) {
        self.write(address, (data & 0xFF) as u8);
        self.write(address.overflowing_add(1).0, (data >> 8) as u8);
    }
}

pub trait IODevice {
    fn read_in(&mut self, port: u16) -> Option<u8>;
    fn write_out(&mut self, port: u16, data: u8) -> bool;

    fn poll_interrupt(&self) -> bool {
        false
    }
    fn ack_interrupt(&mut self) -> u8 {
        0xFF
    }

    fn poll_nmi(&self) -> bool {
        false
    }
}
