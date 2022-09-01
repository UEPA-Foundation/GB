use crate::mmu::ppu::Ppu;

pub enum FifoState {
    INDEX,
    DATALOW,
    DATAHIGH,
    PUSH,
    SLEEP,
}

pub struct PixelFifo {
    len: u8,
    flags: u8,
    pixels: u16,
    pub state: FifoState,
}

#[derive(Debug)]
enum FifoError {
    Full,
    Empty,
}

impl PixelFifo {
    pub fn push(&mut self, pixel: u8) -> Result<(), FifoError> {
        if self.len >= 16 {
            Err(FifoError::Full)
        } else {
            self.pixels |= ((pixel & 0b11) as u16) << self.len;
            self.len += 2;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<u8, FifoError> {
        if self.len <= 0 {
            Err(FifoError::Empty)
        } else {
            self.len -= 2;
            Ok(((self.pixels >> self.len) & 0b11) as u8)
        }
    }

    pub fn peek(&self) -> Result<u8, FifoError> {
        if self.len <= 0 {
            Err(FifoError::Empty)
        } else {
            Ok(((self.pixels >> (self.len - 2)) & 0b11) as u8)
        }
    }

    pub fn empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.flags = 0;
    }
}
