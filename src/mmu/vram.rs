use crate::mmu::MemoryUnit;

pub struct VRam {
    // As only DMG is currently supported, there is only one VRAM bank
    // TODO: Bank switching must be implemented for CGB support!
    bytes: [u8; 0x2000],
}

impl MemoryUnit for VRam {
    fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage. There
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x2000],
        }
    }

    fn read(&self, index: u16) -> u8 {
        self.bytes[(index & 0x1FFF) as usize]
    }

    fn write(&mut self, index: u16, val: u8) {
        self.bytes[(index & 0x1FFF) as usize] = val;
    }
}
