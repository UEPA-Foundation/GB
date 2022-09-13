use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Background {
    tile_id: u8,
    tile_line: u8,
    tile_x: u8,

    data_lo: u8,
    data_hi: u8,

    fifo: PixelFifo,
}

impl Background {
    pub fn init() -> Self {
        Self { tile_id: 0, tile_line: 0, tile_x: 0, data_lo: 0, data_hi: 0, fifo: PixelFifo::init() }
    }

    pub fn pop(&mut self) -> Result<u8, FifoError> {
        self.fifo.pop()
    }
}

impl super::Ppu {
    pub(super) fn init_scanline_bg(&mut self) {
        self.lx = 0;
        self.bg.tile_x = (self.scx / 8) % 32;
        self.bg.tile_line = (self.ly + self.scy) % 8;
        self.bg.fifo.state = FifoState::INDEX;
        self.bg.fifo.clear();
    }

    pub(super) fn cycle_bg(&mut self) {
        match self.bg.fifo.state {
            FifoState::INDEX => {
                let addr = self.bg_tilemap_addr() + ((self.ly + self.scy) as u16 / 8) * 32 + self.bg.tile_x as u16;
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
        match self.lcdc_bit(4) {
            true => 0x8000 + (self.bg.tile_id as u16) * 16 + self.bg.tile_line as u16 * 2,
            false => (0x9000 + (self.bg.tile_id as i8) as i32 * 16) as u16 + self.bg.tile_line as u16 * 2,
        }
    }
}
