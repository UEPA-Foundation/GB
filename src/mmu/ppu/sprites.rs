use super::*;
use crate::gameboy::GameBoy;

pub struct Sprites {
    pub fifo: PixelFifo,
    pub index: u8,
    pub data_lo: u8,
    pub data_hi: u8,
    pub x: u8,
    pub buf: [u8; 10],
}

impl Sprites {
    pub fn init() -> Self {
        Self {
            fifo: PixelFifo::init(),
            index: 0,
            data_lo: 0,
            data_hi: 0,
            x: 0,
            buf: [0; 10],
        }
    }
}

impl GameBoy {
    pub(super) fn sp_fifo_cycle(&mut self) {
        /*  TODO: start sprite drawing logic
        for sprite in self.sp.buf {
            if sprite <= self.lx + 8 {
                self.sp.fifo.state = FifoState::INDEX;
                self.bg.fifo.state = FifoState::SLEEP;
            }
        }
        */

        match self.ppu.sp.fifo.state {
            FifoState::INDEX => self.sp_fetch_index(),
            FifoState::DATALOW => self.sp_fetch_data_low(),
            FifoState::DATAHIGH => self.sp_fetch_data_high(),
            FifoState::PUSH => self.sp_push(),
            FifoState::SLEEP => {}
        }
    }

    fn sp_fetch_index(&mut self) {}

    fn sp_fetch_data_low(&mut self) {}

    fn sp_fetch_data_high(&mut self) {}

    fn sp_push(&mut self) {
        self.ppu.bg.fifo.state = FifoState::INDEX;
    }
}
