use super::pixel_from_palette;
use fifo::Fifo;
mod fifo;

pub struct Sprites {
    state: State,

    cur_obj: Object,
    fetcher_idx: u8,
    obj_buffer: Vec<Object>,

    data_lo: u8,
    data_hi: u8,

    fifo: Fifo,
}

#[derive(Clone, Copy)]
struct Object {
    x: u8,
    y: u8,
    id: u8,
    flags: u8,
}

enum State {
    INDEX,
    DATALOW,
    DATAHIGH,
    PUSH,
    SLEEP,
}

impl Sprites {
    pub fn init() -> Self {
        Self {
            state: State::SLEEP,
            cur_obj: Object { x: 0, y: 0, id: 0, flags: 0 },
            fetcher_idx: 0,
            obj_buffer: vec![Object { x: 0, y: 0, id: 0, flags: 0 }; 10],
            data_lo: 0,
            data_hi: 0,
            fifo: Fifo::init(),
        }
    }
}

impl super::Ppu {
    pub(super) fn fetch_object(&mut self) {
        let obj_addr = 0xFE00 + (self.sp.fetcher_idx as u16 * 4);
        self.sp.fetcher_idx += 1;

        let obj = Object {
            y: self.oam.read(obj_addr + 0),
            x: self.oam.read(obj_addr + 1),
            id: self.oam.read(obj_addr + 2),
            flags: self.oam.read(obj_addr + 3),
        };

        let obj_height = if self.lcdc_bit(2) { 16 } else { 8 };
        if (obj.x == 0) || (self.ly + 16 < obj.y) || (self.ly + 16 >= obj.y + obj_height) {
            return;
        }

        if self.sp.obj_buffer.len() < 10 {
            self.sp.obj_buffer.push(obj);
        }
    }

    pub(super) fn clear_sp_fetcher(&mut self) {
        self.sp.fetcher_idx = 0;
        self.sp.obj_buffer.clear();
    }

    pub(super) fn init_scanline_sp(&mut self) {
        self.sp.fifo.clear();
        self.sp.state = if self.sp.obj_buffer.len() == 0 { State::SLEEP } else { State::INDEX };
    }

    pub(super) fn cycle_sp(&mut self) {
        match self.sp.state {
            State::INDEX => {
                for obj in &self.sp.obj_buffer {
                    if self.lx + 8 <= obj.x && obj.x < self.lx + 16 {
                        self.sp.cur_obj = *obj;
                        self.sp.state = State::DATALOW;
                    }
                }
            }
            State::DATALOW => {
                self.sp.data_lo = self.vram.read(self.get_sprite_addr());
                self.sp.state = State::DATAHIGH;
            }
            State::DATAHIGH => {
                self.sp.data_hi = self.vram.read(self.get_sprite_addr() + 1);
                self.sp.state = State::PUSH;
            }
            State::PUSH => {
                let push_amnt = u8::min(u8::saturating_sub(self.sp.cur_obj.x, 8), 8);
                if self.sp.cur_obj.flags & 0x20 != 0 {
                    self.sp.data_lo = mirror_byte(self.sp.data_lo);
                    self.sp.data_hi = mirror_byte(self.sp.data_hi);
                }
                self.sp.fifo.push(self.sp.data_lo, self.sp.data_hi, self.sp.cur_obj.flags, push_amnt);
                self.sp.state = State::INDEX;
            }
            State::SLEEP => {}
        }
    }

    fn get_sprite_addr(&self) -> u16 {
        let tile_addr = 0x8000 + (self.sp.cur_obj.id as u16 * 16);
        let obj_height = if self.lcdc_bit(2) { 16 } else { 8 };

        let mut offset = (self.ly as u16 + 16 - self.sp.cur_obj.y as u16) % obj_height;
        if self.sp.cur_obj.flags & 0x40 != 0 {
            offset = obj_height - 1 - offset;
        }

        tile_addr + 2 * offset
    }

    #[inline(always)]
    pub(super) fn sp_pop(&mut self) -> Option<(u8, bool)> {
        let (col_id, flags) = self.sp.fifo.pop()?;

        if !self.lcdc_bit(1) {
            return Some((0, false));
        }

        let palette = if flags & 0x10 == 0 { self.obp0 } else { self.obp1 };
        let pixel = pixel_from_palette(col_id, palette);

        Some((pixel, flags & 0x80 != 0))
    }
}

fn mirror_byte(mut byte: u8) -> u8 {
    byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
    byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
    byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;
    byte
}
