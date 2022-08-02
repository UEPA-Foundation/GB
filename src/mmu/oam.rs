use crate::mmu::MemoryUnit;

pub struct Oam {
    bytes: [u8; 0xA0],
}

impl MemoryUnit for Oam {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage. There
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0xA0],
        }
    }

    fn read(&self, index: u16) -> u8 {
        self.bytes[(index & 0x00FF) as usize]
    }

    fn write(&mut self, index: u16, val: u8) {
        self.bytes[(index & 0x00FF) as usize] = val;
    }
}
