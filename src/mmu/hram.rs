use crate::mmu::MemoryUnit;

pub struct HRam {
    bytes: [u8; 0x7F],
}

impl MemoryUnit for HRam {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x7F],
        }
    }

    fn read(&self, index: u16) -> u8 {
        self.bytes[(index - 0xFF80) as usize]
    }

    fn write(&mut self, index: u16, val: u8) {
        self.bytes[(index - 0xFF80) as usize] = val;
    }
}
