pub struct BgFifo {
    len: u8,
    pixels_lo: u8,
    pixels_hi: u8,
}

impl BgFifo {
    pub fn init() -> Self {
        Self { len: 0, pixels_lo: 0, pixels_hi: 0 }
    }

    pub fn push(&mut self, data_lo: u8, data_hi: u8) {
        self.pixels_lo = data_lo;
        self.pixels_hi = data_hi;
        self.len = 8;
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let pixel = ((self.pixels_lo & 0x80) >> 7) | ((self.pixels_hi & 0x80) >> 6);
        self.pixels_lo <<= 1;
        self.pixels_hi <<= 1;
        self.len -= 1;
        Some(pixel)
    }

    pub fn empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.pixels_lo = 0;
        self.pixels_hi = 0;
        self.len = 0;
    }
}
