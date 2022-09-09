use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Background {
    tile_id: u8,
    tile_line: u8,
    tile_index: u16,
    map_addr: u16,

    data_lo: u8,
    data_hi: u8,

    fifo: PixelFifo,
}

impl Background {
    pub fn init() -> Self {
        Self { tile_id: 0, tile_line: 0, tile_index: 0, map_addr: 0, data_lo: 0, data_hi: 0, fifo: PixelFifo::init() }
    }

    pub fn pop(&mut self) -> Result<u8, FifoError> {
        self.fifo.pop()
    }
}

impl super::Ppu {
    pub(super) fn init_scanline_bg(&mut self) {
        self.lx = 0;
        self.bg.tile_index = 0;
        self.bg.tile_line = self.ly % 8;
        self.bg.map_addr = 0x9800 + (self.ly as u16 / 8) * 32;
        self.bg.fifo.state = FifoState::INDEX;
        self.bg.fifo.clear();
    }

    pub(super) fn cycle_bg(&mut self) {
        match self.bg.fifo.state {
            FifoState::INDEX => {
                let addr = self.bg.map_addr + self.bg.tile_index;
                self.bg.tile_id = self.read(addr);
                self.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATALOW => {
                let offset = 0x8000 + self.bg.tile_id as u16 * 16;
                let addr = offset + self.bg.tile_line as u16 * 2;
                self.bg.data_lo = self.read(addr);
                self.bg.fifo.state = FifoState::DATAHIGH;
            }
            FifoState::DATAHIGH => {
                let offset = 0x8000 + self.bg.tile_id as u16 * 16;
                let addr = offset + self.bg.tile_line as u16 * 2;
                self.bg.data_lo = self.read(addr + 1);
                self.bg.fifo.state = FifoState::PUSH;
            }
            FifoState::PUSH => {
                if self.bg.fifo.empty() {
                    self.bg.fifo.push(self.bg.data_lo, self.bg.data_hi, 8).unwrap();
                    self.bg.tile_index += 1;
                    self.bg.fifo.state = FifoState::INDEX;
                }
            }
            FifoState::SLEEP => {}
        }
    }
}
