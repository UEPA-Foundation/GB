pub struct Fifo {
    len: u8,
    pixels_lo: u8,
    pixels_hi: u8,
}

impl Fifo {
    pub fn init() -> Self {
        Self { len: 0, pixels_lo: 0, pixels_hi: 0 }
    }

    pub fn push(&mut self, data_lo: u8, data_hi: u8, num_pixels: u8) {
        let num_pushed = std::cmp::min(num_pixels, u8::saturating_sub(8, self.len));
        let mask = match u8::checked_shl(1, num_pushed as u32) {
            Some(x) => x - 1,
            None => 0xFF,
        };
        self.pixels_lo |= data_lo & mask;
        self.pixels_hi |= data_hi & mask;
        self.len += num_pushed;
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            let pixel = ((self.pixels_lo & 0x80) >> 7) | ((self.pixels_hi & 0x80) >> 6);
            self.pixels_lo <<= 1;
            self.pixels_hi <<= 1;
            self.len -= 1;
            Some(pixel)
        }
    }

    pub fn clear(&mut self) {
        self.pixels_lo = 0;
        self.pixels_hi = 0;
        self.len = 0;
    }
}
