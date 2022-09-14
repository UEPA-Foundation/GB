pub struct VRam {
    // As only DMG is currently supported, there is only one VRAM bank
    // TODO: Bank switching must be implemented for CGB support!
    bytes: [u8; 0x2000],
}

impl VRam {
    pub fn init() -> Self {
        Self {
            // WARN: memory is actually initialized with random garbage. There
            // are known patterns for this garbage. More research needed!
            bytes: [0; 0x2000],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.bytes[(addr & 0x1FFF) as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.bytes[(addr & 0x1FFF) as usize] = val;
    }
}
