use super::*;

pub struct Background {
    pub fifo: PixelFifo,
    pub index: u8,
    pub data_lo: u8,
    pub data_hi: u8,
    pub scanline_x: u8,
}

impl Background {
    pub fn init() -> Self {
        Self { fifo: PixelFifo::init(), index: 0, data_lo: 0, data_hi: 0, scanline_x: 0 }
    }
}

impl Ppu {
    pub(super) fn bg_fifo_cycle(&mut self) {
        // verificando se está dentro da window, pra flushar o bg fifo e recomeçar fetching
        if !self.in_win && self.lcdc_bit(5) && self.wy_eq_ly && self.lx >= self.wx - 7 {
            self.bg.fifo.state = FifoState::INDEX;
            self.bg.fifo.clear();
            self.in_win = true;
        }

        match self.bg.fifo.state {
            FifoState::INDEX => {
                self.bg.index = self.get_tile();
                self.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATALOW => {
                self.bg.data_lo = self.get_data_lo();
                self.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATAHIGH => {
                self.bg.data_hi = self.get_data_hi();
                self.bg.fifo.state = FifoState::PUSH;
                self.bg.scanline_x += 8;
            }
            FifoState::PUSH => self.push(),
            FifoState::SLEEP => {}
        }
    }

    #[inline(always)]
    fn get_tile(&self) -> u8 {
        let addr_offset = {
            let fetcher_x;
            let fetcher_y;

            if self.in_win {
                fetcher_x = self.bg.scanline_x / 8;
                fetcher_y = self.win_y / 8;
            } else {
                fetcher_x = u8::wrapping_add(self.bg.scanline_x, self.scx / 8) & 0x1F;
                fetcher_y = u8::wrapping_add(self.ly, self.scy);
            }

            (32 * (fetcher_y as u16) + (fetcher_x as u16)) & 0x3FF
        };

        let addr = if (self.lcdc_bit(3) && !self.in_win) || (self.lcdc_bit(6) && self.in_win) {
            0x9C00 + addr_offset
        } else {
            0x9800 + addr_offset
        };

        self.read(addr)
    }

    #[inline(always)]
    fn get_data_lo(&mut self) -> u8 {
        let addr = if self.lcdc_bit(4) { 0x8000 } else { 0x8800 };
        self.read(addr + (self.bg.index as u16) * 16)
    }

    #[inline(always)]
    fn get_data_hi(&mut self) -> u8 {
        let addr = if self.lcdc_bit(4) { 0x8000 } else { 0x8800 };
        self.read(addr + (self.bg.index as u16) * 16 + 1)
    }

    #[inline(always)]
    fn push(&mut self) {
        if self.bg.fifo.empty() {
            self.bg.fifo.push(self.bg.data_lo, self.bg.data_hi, 8).unwrap();
        }
    }
}
