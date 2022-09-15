#![allow(unused)]
use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Sprites {
    tile_id: u8,
    tile_line: u16,
    tile_x: u8,

    data_lo: u8,
    data_hi: u8,

    fifo: PixelFifo,
}

impl Sprites {
    pub fn init() -> Self {
        Self { tile_id: 0, tile_line: 0, tile_x: 0, data_lo: 0, data_hi: 0, fifo: PixelFifo::init() }
    }
}

impl super::Ppu {
    pub(super) fn cycle_sp(&mut self) {
        match self.sp.fifo.state {
            FifoState::INDEX => {
                self.sp.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATALOW => {
                self.sp.fifo.state = FifoState::DATAHIGH;
            }
            FifoState::DATAHIGH => {
                self.sp.fifo.state = FifoState::PUSH;
            }
            FifoState::PUSH => {
                self.sp.fifo.state = FifoState::SLEEP;
            }
            FifoState::SLEEP => {}
        }
    }

    #[inline(always)]
    pub fn sp_pop(&mut self) -> Result<u8, FifoError> {
        self.sp.fifo.pop()
    }
}
