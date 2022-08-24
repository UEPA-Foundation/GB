use crate::mmu::mem::MemoryUnit;

pub struct Unused {
    _bytes: [u8; 0x60], // Currently unused, but will be needed for CGB implementation
}

impl MemoryUnit for Unused {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage there
            // are known patterns for this garbage. More research needed!
            _bytes: [0; 0x60],
        }
    }

    fn read(&self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _val: u8) {
        // In DMG, writes are ignored.
    }
}
