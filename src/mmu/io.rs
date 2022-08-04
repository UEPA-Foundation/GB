use crate::gameboy::GameBoy;

pub struct IoRegisters {
    serial_data: u8,
    serial_ctrl: u8,
    timer: Timer,
}

struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    control: u8,
}

impl IoRegisters {
    pub fn init() -> Self {
        Self { serial_data: 0, serial_ctrl: 1, timer: Timer { divider: 0, counter: 0, modulo: 0, control: 0 } }
    }
}

impl GameBoy {
    pub fn io_read(&self, index: u16) -> u8 {
        match index {
            0xFF01 => self.mmu.io.serial_data,
            0xFF02 => self.mmu.io.serial_ctrl,
            0xFF04 => (self.mmu.io.timer.divider >> 8) as u8,
            0xFF05 => self.mmu.io.timer.counter,
            0xFF06 => self.mmu.io.timer.modulo,
            0xFF07 => self.mmu.io.timer.control | 0xF8,
            _ => 0, // panic!("Io Register not yet implemented!"),
        }
    }

    pub fn io_write(&mut self, index: u16, val: u8) {
        match index {
            0xFF01 => self.mmu.io.serial_data = val,
            0xFF02 => self.mmu.io.serial_ctrl = val,
            0xFF04 => self.mmu.io.timer.divider = 0,
            0xFF05 => {
                // WARN: writes to the timer counter are supposed to be ignored
                // in the cycle it is being written to from the modulo
                // if being_written_to_from_mod { return; }
                self.mmu.io.timer.counter = val
            }
            0xFF06 => {
                // WARN: when writing to the timer modulo in the same cycle it
                // is loaded to the counter, the counter must also be written
                // if mod_writing_to_count { self.mmu.io.timer.counter = val; }
                self.mmu.io.timer.modulo = val
            }
            0xFF07 => {
                // WARN: apparently there is a hardware glitch that needs to be
                // emulated here. More research needed!
                self.mmu.io.timer.control = val
            }
            _ => (), // panic!("Io Register not yet implemented!"),
        }
    }
}
