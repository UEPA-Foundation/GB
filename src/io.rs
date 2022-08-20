use crate::gameboy::GameBoy;

pub struct IoRegisters {
    serial_data: u8,
    serial_ctrl: u8,
    iflags: u8,
}

impl IoRegisters {
    pub fn init() -> Self {
        Self { serial_data: 0, serial_ctrl: 1, iflags: 0 }
    }
}

impl GameBoy {
    pub fn io_read(&self, index: u16) -> u8 {
        match index {
            0xFF01 => self.io.serial_data,
            0xFF02 => self.io.serial_ctrl,
            0xFF0F => self.io.iflags | 0xE0, // 3 upper bits always return 1
            _ => 0,
        }
    }

    pub fn io_write(&mut self, index: u16, val: u8) {
        match index {
            0xFF01 => self.io.serial_data = val,
            0xFF02 => self.io.serial_ctrl = val,
            0xFF0F => self.io.iflags = val,
            _ => (),
        }
    }
}
