use crate::gameboy::GameBoy;

pub struct WRam0 {
    bytes: [u8; 0x1000],
}

impl WRam0 {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x1000],
        }
    }
}

impl GameBoy {
    pub fn wram0_read(&self, index: u16) -> u8 {
        self.wram0.bytes[(index & 0x0FFF) as usize]
    }

    pub fn wram0_write(&mut self, index: u16, val: u8) {
        self.wram0.bytes[(index & 0x0FFF) as usize] = val;
    }
}
