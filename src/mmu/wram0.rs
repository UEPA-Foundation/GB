use crate::mmu::MemoryUnit;

pub struct Wram0 {
    bytes: [u8; 0x1000],
}

impl MemoryUnit for Wram0 {
    fn init() -> Self {
        Self {
            bytes: [0; 0x1000] //TODO: memory is actually initialized with random garbage
        }                      //there are known patterns for this garbage
    }                          //more research needed!

    fn read(&self, index: u16) -> u8 {
        self.bytes[(index & 0x0FFF) as usize]
    }

    fn write(&mut self, index: u16, val: u8) {
        self.bytes[(index & 0x0FFF) as usize] = val;
    }
}
