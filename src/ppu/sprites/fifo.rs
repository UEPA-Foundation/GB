pub struct Fifo {
    pixels_lo: u8,
    pixels_hi: u8,
    bg_pri_flag: u8,
    palette_flag: u8,
}

impl Fifo {
    pub fn init() -> Self {
        Self { pixels_lo: 0, pixels_hi: 0, bg_pri_flag: 0, palette_flag: 0 }
    }

    pub fn push(&mut self, data_lo: u8, data_hi: u8, flags: u8, num_pixels: u8) {
        let mask = !(self.pixels_lo | self.pixels_hi);
        self.pixels_lo |= data_lo & mask;
        self.pixels_hi |= data_hi & mask;
        self.bg_pri_flag = if flags & 0x80 != 0 {self.bg_pri_flag | mask} else {self.bg_pri_flag & !mask};
        self.palette_flag = if flags & 0x10 != 0 {self.palette_flag | mask} else {self.palette_flag & !mask};
    }

    pub fn pop(&mut self) -> Option<(u8, bool, bool)> {
        let pixel = ((self.pixels_lo & 0x80) >> 7) | ((self.pixels_hi & 0x80) >> 6);
        let bg_priority = self.bg_pri_flag & 0x80 != 0;
        let palette = self.palette_flag & 0x80 != 0;
        self.pixels_lo <<= 1;
        self.pixels_hi <<= 1;
        self.bg_pri_flag <<= 1;
        self.palette_flag <<= 1;
        Some((pixel, bg_priority, palette))
    }

    pub fn clear(&mut self) {
        self.pixels_lo = 0;
        self.pixels_hi = 0;
        self.bg_pri_flag = 0;
        self.palette_flag = 0;
    }
}
