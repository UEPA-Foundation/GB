use fifo::BgFifo;

mod fifo;

pub enum State {
    INDEX,
    DATALOW,
    DATAHIGH,
    PUSH,
    SLEEP,
}

pub struct Background {
    tile_id: u8,
    tile_line: u16,
    tile_x: u8,

    num_scrolled: u8,

    pub win_mode: bool,
    in_win_y: bool,
    win_line: u16,

    data_lo: u8,
    data_hi: u8,

    fifo: BgFifo,
    state: State,
}

impl Background {
    pub fn init() -> Self {
        Self {
            tile_id: 0,
            tile_line: 0,
            tile_x: 0,
            num_scrolled: 0,
            win_mode: false,
            in_win_y: false,
            win_line: 0,
            data_lo: 0,
            data_hi: 0,
            fifo: BgFifo::init(),
            state: State::INDEX,
        }
    }

    pub fn pause(&mut self) {
        self.state = State::SLEEP;
    }

    pub fn resume(&mut self) {
        self.state = State::INDEX;
    }
}

impl super::Ppu {
    pub fn init_scanline_bg(&mut self) {
        self.bg.num_scrolled = 0;
        self.bg.tile_x = (self.scx / 8) % 32;
        self.bg.tile_line = (self.ly as u16 + self.scy as u16) % 8;
        self.bg.win_mode = false;
        self.bg.fifo.clear();
        self.bg.state = State::INDEX;
    }

    pub fn init_frame_bg(&mut self) {
        self.bg.in_win_y = false;
        self.bg.win_line = 0;
    }

    pub(super) fn cycle_bg(&mut self) {
        match self.bg.state {
            State::INDEX => {
                let addr = self.tilemap_addr();
                self.bg.tile_id = self.vram.read(addr);
                self.bg.state = State::DATALOW;
            }
            State::DATALOW => {
                self.bg.data_lo = self.vram.read(self.get_tile_addr());
                self.bg.state = State::DATAHIGH;
            }
            State::DATAHIGH => {
                self.bg.data_hi = self.vram.read(self.get_tile_addr() + 1);
                self.bg.state = State::PUSH;
            }
            State::PUSH => {
                if self.bg.fifo.empty() {
                    self.bg.fifo.push(self.bg.data_lo, self.bg.data_hi);
                    self.bg.tile_x = (self.bg.tile_x + 1) % 32;
                    self.bg.state = State::INDEX;
                }
            }
            State::SLEEP => {}
        }
    }

    #[inline(always)]
    fn tilemap_addr(&self) -> u16 {
        let (base, y_offset) = match (self.bg.win_mode, self.lcdc_bg_tm_sel(), self.lcdc_wn_tm_sel()) {
            (false, false, _) => (0x9800, (self.ly as u16 + self.scy as u16) & 0xFF),
            (false, true, _) => (0x9C00, (self.ly as u16 + self.scy as u16) & 0xFF),
            (true, _, false) => (0x9800, (self.bg.win_line - 1)),
            (true, _, true) => (0x9C00, (self.bg.win_line - 1)),
        };
        let offset = (y_offset / 8) * 32 + self.bg.tile_x as u16;
        base + offset
    }

    #[inline(always)]
    fn get_tile_addr(&self) -> u16 {
        let mut index = self.bg.tile_id as u16;
        let base_addr = match (self.lcdc_td_sel(), self.bg.tile_id >= 128) {
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

    #[inline(always)]
    pub(super) fn check_in_win(&mut self) -> bool {
        if !self.bg.win_mode && self.lcdc_wn_enbl() && self.in_win_x() && self.bg.in_win_y {
            self.bg.win_mode = true;
            self.bg.tile_line = self.bg.win_line % 8;
            self.bg.tile_x = 0;
            self.bg.win_line += 1;
            self.bg.fifo.clear();
            self.bg.state = State::INDEX;
            return true;
        }
        false
    }

    #[inline(always)]
    fn in_win_x(&self) -> bool {
        self.lx >= u8::saturating_sub(self.wx, 7)
    }

    #[inline(always)]
    pub(super) fn check_in_win_y(&mut self) {
        if self.ly == self.wy {
            self.bg.in_win_y = true;
        }
    }

    #[inline(always)]
    pub fn bg_pop(&mut self) -> Option<u8> {
        match (self.bg.fifo.pop(), self.lcdc_bg_enbl(), self.bg.win_mode || self.bg.num_scrolled >= self.scx % 8) {
            (Some(pixel), true, true) => Some(pixel),
            (Some(_), true, false) => {
                self.bg.num_scrolled += 1;
                None
            }
            (Some(_), false, _) => Some(0),
            (None, _, _) => None,
        }
    }
}
