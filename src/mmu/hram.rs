use crate::gameboy::GameBoy;

pub struct HRam {
    bytes: [u8; 0x7F],
}

impl HRam {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x7F],
        }
    }
}

impl GameBoy {
    pub fn hram_read(&self, index: u16) -> u8 {
        self.mmu.hram.bytes[(index - 0xFF80) as usize]
    }

    pub fn hram_write(&mut self, index: u16, val: u8) {
        self.mmu.hram.bytes[(index - 0xFF80) as usize] = val;
    }
}
