use super::*;
use crate::gameboy::GameBoy;

pub struct Background {
    pub fifo: PixelFifo,
    pub index: u8,
    pub data_lo: u8,
    pub data_hi: u8,
    pub x: u8,
}

impl GameBoy {
    pub(super) fn bg_fifo_cycle(&mut self) {
        // verificando se está dentro da window, pra flushar o bg fifo e recomeçar fetching
        if !self.ppu.in_win && self.ppu.lcdc_bit(5) && self.ppu.wy_eq_ly && self.ppu.lx >= self.ppu.wx - 7 {
            self.ppu.bg.fifo.state = FifoState::INDEX;
            self.ppu.bg.fifo.clear();
            self.ppu.in_win = true;
        }

        match self.ppu.bg.fifo.state {
            FifoState::INDEX => {
                let addr = self.ppu.get_tile_addr();
                self.ppu.bg.index = self.read(addr);
                self.ppu.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATALOW => {
                let addr = self.ppu.get_data_addr();
                self.ppu.bg.data_lo = self.read(addr);
                self.ppu.bg.fifo.state = FifoState::DATALOW;
            }
            FifoState::DATAHIGH => {
                let addr = self.ppu.get_data_addr() + 1;
                self.ppu.bg.data_hi = self.read(addr);
                self.ppu.bg.fifo.state = FifoState::PUSH;
                self.ppu.bg.x += 8;
            }
            FifoState::PUSH => self.ppu.push(),
            FifoState::SLEEP => {}
        }
    }
}

impl Ppu {
    #[inline(always)]
    fn get_tile_addr(&self) -> u16 {
        let tile = {
            let mut tile_x = 0;
            let mut tile_y = 0;

            if self.in_win {
                tile_x = self.bg.x / 8;
                tile_y = self.win_y / 8;
            } else {
                tile_x = u8::wrapping_add(self.bg.x, self.scx) / 8;
                tile_y = u8::wrapping_add(self.ly, self.scy) / 8;
            }

            (32 * (tile_y as u16) + (tile_x as u16)) & 0x3FF
        };

        let addr =
            if (self.lcdc_bit(3) && !self.in_win) || (self.lcdc_bit(6) && self.in_win) { 0x9C00 } else { 0x9800 };
        addr + (tile * 16)
    }

    fn get_data_addr(&mut self) -> u16 {
        let addr = if self.lcdc_bit(4) { 0x8000 } else { 0x8800 };
        addr + (self.bg.index as u16) * 16
    }

    fn push(&mut self) {
        if !self.bg.fifo.empty() {
            return;
        }
        // self.ppu.bg_fifo.push((data low e data high decoded em pixels));
    }
}
