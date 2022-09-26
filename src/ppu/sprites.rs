#![allow(unused)]
use super::fifo::{FifoError, FifoState, PixelFifo};

pub struct Sprites {
    obj: Object,

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
            obj: Object { x: 0, y: 0, id: 0, flags: 0 },
            data_lo: 0,
            data_hi: 0,
            fetcher: Fetcher::init(),
            fifo: PixelFifo::init(),
        }
    }

    pub fn bg_priority(&self) -> bool {
        self.obj.flags & 0x80 != 0
    }
}

impl super::Ppu {
    pub(super) fn fetch_object(&mut self) {
        let obj_addr = 0xFE00 + (self.sp.fetcher.cur as u16 * 4);
        let obj = Object {
            y: self.oam.read(obj_addr + 0),
            x: self.oam.read(obj_addr + 1),
            id: self.oam.read(obj_addr + 2),
            flags: self.oam.read(obj_addr + 3),
        };
        self.sp.fetcher.cur += 1;

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

    pub(super) fn clear_sp_fetcher(&mut self) {
        self.sp.fetcher.cur = 0;
        self.sp.fetcher.len = 0;
        self.sp.fetcher.buffer = [Object { x: 0, y: 0, id: 0, flags: 0 }; 10];
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
                for obj in self.sp.fetcher.buffer[..self.sp.fetcher.len as usize].into_iter() {
                    if self.lx + 8 <= obj.x && obj.x < self.lx + 16 {
                        self.sp.obj = *obj;
                        self.sp.fifo.state = FifoState::DATALOW;
                    }
                }
            }
            FifoState::DATALOW => {
                self.sp.data_lo = self.vram.read(self.get_sprite_addr());
                self.sp.fifo.state = FifoState::DATAHIGH;
            }
            FifoState::DATAHIGH => {
                self.sp.data_hi = self.vram.read(self.get_sprite_addr() + 1);
                self.sp.fifo.state = FifoState::PUSH;
            }
            FifoState::PUSH => {
                let push_amnt = u8::min(u8::saturating_sub(self.sp.obj.x, 8), 8);
                self.sp.fifo.push(self.sp.data_lo, self.sp.data_hi, push_amnt);
                self.sp.fifo.state = FifoState::INDEX;
            }
            FifoState::SLEEP => {}
        }
    }

    fn get_sprite_addr(&self) -> u16 {
        let tile_addr = 0x8000 + (self.sp.obj.id as u16 * 16);
        tile_addr + 2 * ((self.ly as u16 + self.scy as u16) % 8)
    }

    #[inline(always)]
    pub(super) fn sp_pop(&mut self) -> Result<u8, FifoError> {
        self.sp.fifo.pop()
    }
}
