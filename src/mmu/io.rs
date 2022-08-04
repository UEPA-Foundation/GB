use crate::gameboy::GameBoy;

pub struct IoRegisters {
    serial_data: u8,
    serial_ctrl: u8,
}

impl IoRegisters {
    pub fn init() -> Self {
        Self { serial_data: 0, serial_ctrl: 1 }
    }
}

impl GameBoy {
    pub fn io_read(&self, index: u16) -> u8 {
        match index {
            0xFF01 => self.mmu.io.serial_data,
            0xFF02 => self.mmu.io.serial_ctrl,
            _ => panic!("Io Register not yet implemented!"),
        }
    }

    pub fn io_write(&mut self, index: u16, val: u8) {
        match index {
            0xFF01 => self.mmu.io.serial_data = val,
            0xFF02 => self.mmu.io.serial_ctrl = val,
            _ => panic!("Io Register not yet implemented!"),
        }
    }
}
