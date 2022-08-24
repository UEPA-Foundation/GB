use crate::gameboy::GameBoy;

pub struct Joypad {
    joyp: u8,
    a: u8,
    b: u8,
    start: u8,
    select: u8,
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

impl Joypad {
    pub fn init() -> Self {
        Self { joyp: 0, a: 0, b: 0, start: 0, select: 0, up: 0, down: 0, left: 0, right: 0 }
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
            0b01 => self.joyp |= self.start << 3 | self.select << 2 | self.b << 1 | self.a,
            0b10 => self.joyp |= self.down << 3 | self.up << 2 | self.left << 1 | self.right,
            0b11 => self.joyp |= 0x0F,
            how => println!("Somehow, amazingly, a two bit number equals {}.", how),
        }
    }
}

macro_rules! set_key {
    ($key: ident) => {
        paste::paste! {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn [<set_ $key>](&mut self, state: bool) {
                self.joypad.$key = state as u8;
            }
        }
    };
}

impl GameBoy {
    pub fn cycle_joypad(&mut self, _cycles: u8) {
        // TODO: Joypad has a lot of intricacies regarding bouncing and stability.
        // For development's sake, these are being ignored for now.
        let old_keys = self.joypad.joyp & 0x0F;
        self.joypad.update_joyp();
        let new_keys = self.joypad.joyp & 0x0F;
        if (old_keys == 0x0F) && (new_keys != 0x0F) {
            self.set_if(0x10);
        }
    }

    set_key!(a);
    set_key!(b);
    set_key!(start);
    set_key!(select);
    set_key!(up);
    set_key!(down);
    set_key!(left);
    set_key!(right);
}
