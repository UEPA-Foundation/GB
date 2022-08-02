use crate::gameboy::GameBoy;

pub struct Oam {
    bytes: [u8; 0xA0],
}

impl Oam {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage. There
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0xA0],
        }
    }
}

impl GameBoy {
    pub fn oam_read(&self, index: u16) -> u8 {
        self.mmu.oam.bytes[(index & 0x00FF) as usize]
    }

    pub fn oam_write(&mut self, index: u16, val: u8) {
        self.mmu.oam.bytes[(index & 0x00FF) as usize] = val;
    }
}
