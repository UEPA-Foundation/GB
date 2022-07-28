use crate::mmu::MemoryUnit;

pub struct WRamX {
    bytes: [u8; 0x1000],    //As only DMG is currently supported, there is only one RAM bank
}                           //bank switching must be implemented for CGB support!

impl MemoryUnit for WRamX {
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
