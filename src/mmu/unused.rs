use crate::gameboy::GameBoy;

pub struct Unused {
    _bytes: [u8; 0x60], // Currently unused, but will be needed for CGB implementation
}

impl Unused {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            _bytes: [0; 0x60],
        }
    }
}

impl GameBoy {
    pub fn unused_read(&self, _index: u16) -> u8 {
        0
    }

    pub fn unused_write(&mut self, _index: u16, _val: u8) {
        // In DMG, writes are ignored.
    }
}
