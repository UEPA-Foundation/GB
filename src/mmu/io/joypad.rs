use crate::gameboy::GameBoy;
use crate::intr::Interrupt;

pub struct Joypad {
    joyp: u8,
    buttons: u8, // upper nybble is DPAD, lower nybble is action
}

pub enum Button {
    A = 0x01,
    B = 0x02,
    SELECT = 0x04,
    START = 0x08,
    RIGHT = 0x10,
    LEFT = 0x20,
    UP = 0x40,
    DOWN = 0x80,
}

impl Joypad {
    pub fn init() -> Self {
        Self { joyp: 0x0F, buttons: 0xFF }
    }

    pub fn read(&self) -> u8 {
        self.joyp | 0xC0
    }

    pub fn write(&mut self, val: u8) {
        self.joyp = val & 0x30;
    }

    #[inline(always)]
    fn update_joyp(&mut self) {
        self.joyp &= 0x30;
        match self.joyp >> 4 {
            0b00 => {}
            0b01 => self.joyp |= self.buttons & 0x0F, // selecting action buttons
            0b10 => self.joyp |= self.buttons >> 4,   // selecting directional buttons
            0b11 => self.joyp |= 0x0F,
            how => println!("Somehow, amazingly, a two bit number equals {}.", how),
        }
    }
}

impl GameBoy {
    pub fn cycle_joypad(&mut self, _cycles: u8) {
        // TODO: Joypad has a lot of intricacies regarding bouncing and stability.
        // For development's sake, these are being ignored for now.
        let old_keys = self.joypad.joyp & 0x0F;
        self.joypad.update_joyp();
        let new_keys = self.joypad.joyp & 0x0F;
        if (old_keys == 0x0F) && (new_keys != 0x0F) {
            self.intr.request(Interrupt::JOYPAD);
        }
    }

    pub fn set_button(&mut self, button: Button, state: bool) {
        match state {
            true => self.joypad.buttons |= button as u8,
            false => self.joypad.buttons &= !(button as u8),
        }
    }
}
