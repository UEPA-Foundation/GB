use crate::gameboy::GameBoy;

pub struct WRamX {
    // As only DMG is currently supported, there is only one RAM bank
    // TODO: Bank switching must be implemented for CGB support!
    bytes: [u8; 0x1000],
}

impl WRamX {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x1000],
        }
    }
}

impl GameBoy {
    pub fn wramx_read(&self, index: u16) -> u8 {
        self.wramx.bytes[(index & 0x0FFF) as usize]
    }

    pub fn wramx_write(&mut self, index: u16, val: u8) {
        self.wramx.bytes[(index & 0x0FFF) as usize] = val;
    }
}
