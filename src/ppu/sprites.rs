#![allow(unused)]
use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Sprites {
    tile_id: u8,
    tile_line: u16,
    tile_x: u8,

    data_lo: u8,
    data_hi: u8,

    fifo: PixelFifo,
    fetcher: Fetcher,
}

struct Fetcher {
    cur: u8,
    buffer: [Object; 10],
    len: u8,
}

impl Fetcher {
    fn init() -> Self {
        Self { cur: 0, buffer: [Object { x: 0, y: 0, id: 0, flags: 0 }; 10], len: 0 }
    }
}

#[derive(Clone, Copy)]
struct Object {
    x: u8,
    y: u8,
    id: u8,
    flags: u8,
}

impl Sprites {
    pub fn init() -> Self {
        Self {
            tile_id: 0,
            tile_line: 0,
            tile_x: 0,
            data_lo: 0,
            data_hi: 0,
            fetcher: Fetcher::init(),
            fifo: PixelFifo::init(),
        }
    }
}

impl super::Ppu {
    pub(super) fn fetch_object(&mut self) {
        let obj_addr = 0xFE00 + (self.sp.fetcher.cur as u16 * 4);
        let obj = Object {
            y: self.read(obj_addr + 0),
            x: self.read(obj_addr + 1),
            id: self.read(obj_addr + 2),
            flags: self.read(obj_addr + 3),
        };

        let obj_height = 8; // TODO: check tall sprite mode
        if (obj.x == 0) || (self.ly + 16 < obj.y) || (self.ly + 16 >= obj.y + obj_height) {
            return;
        }

        let fetcher = &mut self.sp.fetcher;
        if fetcher.len < 10 {
            fetcher.buffer[fetcher.len as usize] = obj;
            fetcher.len += 1;
        }
    }

    pub(super) fn init_scanline_sp(&mut self) {
        self.sp.fifo.clear();
        if self.sp.fetcher.len == 0 {
            self.sp.fifo.state = FifoState::SLEEP;
        }
    }

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
    pub(super) fn sp_pop(&mut self) -> Result<u8, FifoError> {
        self.sp.fifo.pop()
    }
}
