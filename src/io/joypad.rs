use crate::gameboy::GameBoy;

pub struct Joypad {
    joyp: u8,
}

impl Joypad {
    pub fn init() -> Self {
        Self { joyp: 0 }
    }

    pub fn read(&self) -> u8 {
        self.joyp
    }

    pub fn write(&mut self, val: u8) {
    }
}

impl GameBoy {
    pub fn cycle_joypad(&mut self, cycles: u8) {
        // TODO: Joypad has a lot of intricacies regarding bouncing and stability.
        // For development's sake, these are being ignored for now.
    }
}
