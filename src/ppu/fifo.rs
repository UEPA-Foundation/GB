#[derive(Debug)]
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
    pixels_lo: u8,
    pixels_hi: u8,
    pub state: FifoState,
}

#[derive(Debug)]
pub enum FifoError {
    Full,
    Empty,
}

impl PixelFifo {
    pub fn init() -> Self {
        Self { len: 0, flags: 0, pixels_lo: 0, pixels_hi: 0, state: FifoState::SLEEP }
    }

    pub fn push(&mut self, data_lo: u8, data_hi: u8, num_pixels: u8) -> Result<(), FifoError> {
        if self.len >= 8 {
            Err(FifoError::Full)
        } else {
            let num_pushed = std::cmp::min(num_pixels, 8 - self.len);
            let mask = match u8::checked_shl(1, num_pushed as u32) {
                Some(x) => x - 1,
                None => 0xFF,
            };
            self.pixels_lo |= data_lo & mask;
            self.pixels_hi |= data_hi & mask;
            self.len += num_pushed;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<u8, FifoError> {
        if self.len <= 0 {
            Err(FifoError::Empty)
        } else {
            let pixel = ((self.pixels_lo & 0x80) >> 7) | ((self.pixels_hi & 0x80) >> 6);
            self.pixels_lo <<= 1;
            self.pixels_hi <<= 1;
            self.len -= 1;
            Ok(pixel)
        }
    }

    pub fn peek(&self) -> Result<u8, FifoError> {
        if self.len <= 0 {
            Err(FifoError::Empty)
        } else {
            let pixel = ((self.pixels_lo & 0x80) >> 7) | ((self.pixels_hi & 0x80) >> 6);
            Ok(pixel)
        }
    }

    pub fn empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.pixels_lo = 0;
        self.pixels_hi = 0;
        self.flags = 0;
        self.len = 0;
        self.state = FifoState::INDEX;
    }
}
