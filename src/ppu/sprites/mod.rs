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
            obj_buffer: vec![],
            data_lo: 0,
            data_hi: 0,
            fifo: Fifo::init(),
        }
    }

    pub fn is_fetching(&self) -> bool {
        match self.state {
            State::SLEEP => false,
            _ => true,
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

        let obj_height = if self.lcdc_sp_size() { 16 } else { 8 };
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
        self.sp.state = State::SLEEP;
    }

    pub(super) fn cycle_sp(&mut self) {
        match self.sp.state {
            State::DATALOW => {
                self.sp.data_lo = self.vram.read(self.get_sprite_addr());
                self.sp.state = State::DATAHIGH;
            }
            State::DATAHIGH => {
                self.sp.data_hi = self.vram.read(self.get_sprite_addr() + 1);
                self.sp.state = State::PUSH;
            }
            State::PUSH => {
                let push_amnt = u8::min(self.sp.cur_obj.x, 8);
                if self.sp.cur_obj.flags & 0x20 != 0 {
                    self.sp.data_lo = mirror_byte(self.sp.data_lo);
                    self.sp.data_hi = mirror_byte(self.sp.data_hi);
                }
                self.sp.fifo.push(self.sp.data_lo, self.sp.data_hi, self.sp.cur_obj.flags, push_amnt);
                self.sp.state = State::SLEEP;
                self.bg.resume();
                self.fetch_obj();
            }
            State::SLEEP => {}
        }
    }

    pub fn fetch_obj(&mut self) {
        for i in 0..self.sp.obj_buffer.len() {
            let obj = &self.sp.obj_buffer[i];
            if self.lx <= obj.x && obj.x <= self.lx + 8 {
                self.sp.cur_obj = *obj;
                self.sp.obj_buffer.remove(i);
                self.sp.state = State::DATALOW;
                self.bg.pause();
                break;
            }
        }
    }

    fn get_sprite_addr(&self) -> u16 {
        let (obj_height, obj_id) = match self.lcdc_sp_size() {
            false => (8, self.sp.cur_obj.id),
            true => (16, self.sp.cur_obj.id & !0x01),
        };
        let tile_addr = 0x8000 + (obj_id as u16 * 16);

        let mut offset = (self.ly as u16 + 16 - self.sp.cur_obj.y as u16) % obj_height;
        if self.sp.cur_obj.flags & 0x40 != 0 {
            offset = obj_height - 1 - offset;
        }

        tile_addr + 2 * offset
    }

    #[inline(always)]
    pub(super) fn sp_pop(&mut self) -> Option<(u8, bool, u8)> {
        let (pixel, bg_priority, palette_flag) = self.sp.fifo.pop()?;

        if !self.lcdc_sp_enbl() {
            return Some((0, false, 0));
        }

        let palette = if palette_flag { self.obp1 } else { self.obp0 };
        Some((pixel, bg_priority, palette))
    }
}

#[inline(always)]
fn mirror_byte(mut byte: u8) -> u8 {
    byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
    byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
    byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;
    byte
}
