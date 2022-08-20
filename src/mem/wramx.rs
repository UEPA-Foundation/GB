use crate::mem::MemoryUnit;

pub struct WRamX {
    // As only DMG is currently supported, there is only one RAM bank
    // TODO: Bank switching must be implemented for CGB support!
    bytes: [u8; 0x1000],
}

impl MemoryUnit for WRamX {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x1000],
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bytes[(addr & 0x0FFF) as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.bytes[(addr & 0x0FFF) as usize] = val;
    }
}
