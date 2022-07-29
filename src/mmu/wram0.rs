use crate::mmu::MemoryUnit;

pub struct WRam0 {
    bytes: [u8; 0x1000],
}

impl MemoryUnit for WRam0 {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x1000],
        }
    }

    fn read(&self, index: u16) -> u8 {
        self.bytes[(index & 0x0FFF) as usize]
    }

    fn write(&mut self, index: u16, val: u8) {
        self.bytes[(index & 0x0FFF) as usize] = val;
    }
}
