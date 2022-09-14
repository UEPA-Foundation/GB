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

    pub fn read(&self, addr: u16) -> u8 {
        self.bytes[(addr & 0x00FF) as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.bytes[(addr & 0x00FF) as usize] = val;
    }
}
