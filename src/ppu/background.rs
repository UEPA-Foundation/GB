use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Background {
    tile_id: u8,
    tile_line: u16,
    tile_x: u8,

    pub win_mode: bool,
    in_win_y: bool,
    win_line: u16,

    data_lo: u8,
    data_hi: u8,

    fifo: PixelFifo,
}

impl Background {
    pub fn init() -> Self {
        Self { tile_id: 0, tile_line: 0, tile_x: 0, win_mode: false, in_win_y: false, win_line: 0, data_lo: 0, data_hi: 0, fifo: PixelFifo::init() }
    }

    pub fn pop(&mut self) -> Result<u8, FifoError> {
        self.fifo.pop()
    }
}

impl super::Ppu {
    pub(super) fn init_scanline_bg(&mut self) {
        self.lx = 0;
        self.bg.tile_x = (self.scx / 8) % 32;
        self.bg.tile_line = (self.ly as u16 + self.scy as u16) % 8;
        self.bg.win_mode = false;
        self.bg.fifo.clear();
    }

    pub(super) fn init_frame_bg(&mut self) {
        self.bg.in_win_y = false;
        self.bg.win_line = 0;
    }

    pub(super) fn cycle_bg(&mut self) {
        match self.bg.fifo.state {
            FifoState::INDEX => {
                let addr = match self.bg.win_mode {
                    false => self.bg_tilemap_addr() + ((self.ly as u16 + self.scy as u16) / 8) * 32 + self.bg.tile_x as u16,
                    true => self.win_tilemap_addr() + ((self.bg.win_line - 1) / 8) * 32 + self.bg.tile_x as u16,
                };
                self.bg.tile_id = self.read(addr);
                self.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATALOW => {
                self.bg.data_lo = self.read(self.get_tile_addr());
                self.bg.fifo.state = FifoState::DATAHIGH;
            }
            FifoState::DATAHIGH => {
                self.bg.data_hi = self.read(self.get_tile_addr() + 1);
                self.bg.fifo.state = FifoState::PUSH;
            }
            FifoState::PUSH => {
                if self.bg.fifo.empty() {
                    self.bg.fifo.push(self.bg.data_lo, self.bg.data_hi, 8).unwrap();
                    self.bg.tile_x = (self.bg.tile_x + 1) % 32;
                    self.bg.fifo.state = FifoState::INDEX;
                }
            }
            FifoState::SLEEP => {}
        }
    }

    #[inline(always)]
    fn bg_tilemap_addr(&self) -> u16 {
        match self.lcdc_bit(3) {
            false => 0x9800,
            true => 0x9C00,
        }
    }

    #[inline(always)]
    fn win_tilemap_addr(&self) -> u16 {
        match self.lcdc_bit(6) {
            false => 0x9800,
            true => 0x9C00,
        }
    }

    fn get_tile_addr(&self) -> u16 {
        let mut index = self.bg.tile_id as u16;
        let base_addr = match (self.lcdc_bit(4), self.bg.tile_id >= 128) {
            (true, _) => 0x8000,
            (false, false) => 0x9000,
            (false, true) => {
                index -= 128;
                0x8800
            }
        };
        let offset = index * 16 + self.bg.tile_line * 2;
        base_addr + offset
    }

    pub(super) fn check_in_win(&mut self) {
        if !self.bg.win_mode && self.lcdc_bit(5) && self.in_win_x() && self.bg.in_win_y {
            self.bg.win_mode = true;
            self.bg.tile_line = self.bg.win_line % 8;
            self.bg.tile_x = 0;
            self.bg.win_line += 1;
            self.bg.fifo.clear();
        }
    }

    #[inline(always)]
    fn in_win_x(&self) -> bool {
        self.lx >= u8::wrapping_sub(self.wx, 7)
    }

    #[inline(always)]
    pub(super) fn check_in_win_y(&mut self) {
        if self.ly == self.wy {
            self.bg.in_win_y = true;
        }
    }
}
