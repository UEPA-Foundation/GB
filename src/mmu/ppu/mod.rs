#![allow(dead_code)]

use crate::gameboy::GameBoy;

pub mod lcd;

#[derive(Copy, Clone)]
struct Pixel {
    val: u8,
}

impl Pixel {
    const COL: u8 = 0b00000011;
    const PAL: u8 = 0b00011100;
    const SPR: u8 = 0b00100000;
    const BPR: u8 = 0b01000000;

    fn get_color(&self) -> u8 {
        self.val & Pixel::COL
    }

    fn set_color(&mut self, val: u8) {
        self.val |= val & Pixel::COL;
    }

    fn get_pallete(&self) -> u8 {
        (self.val & Pixel::PAL) >> 2
    }

    fn set_pallete(&mut self, val: u8) {
        self.val |= (val << 2) & Pixel::PAL
    }

    fn get_sprite_priority(&self) -> bool {
        self.val & Pixel::SPR != 0
    }

    fn set_sprite_priority(&mut self, val: bool) {
        self.val |= (val as u8) << 5;
    }

    fn get_bg_priority(&self) -> bool {
        self.val & Pixel::BPR != 0
    }

    fn set_bg_priority(&mut self, val: bool) {
        self.val |= (val as u8) << 6;
    }
}

pub struct Ppu {
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    lx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    sprite_buf: [u8; 16], // TODO: isso aqui vai ser preenchido em oam scan

    sp_fifo: [Pixel; 16],
    bg_fifo: [Pixel; 16],
    bg_fifo_state: FifoState,
    sp_fifo_state: FifoState,
    bg_x: u8,
    in_win: bool,
    win_y: u8,
    bg_index: u8,
    bg_data_low: u8,
    bg_data_high: u8,
    wy_eq_ly: bool,

    mode: PpuMode,
    cycles: u32,

}

#[derive(Copy, Clone)]
enum PpuMode {
    HBLANK = 0,
    VBLANK = 1,
    OAMSCAN = 2,
    DRAW = 3,
}

enum FifoState {
    INDEX,
    DATALOW,
    DATAHIGH,
    PUSH,
    SLEEP,
}

impl Ppu {
    pub fn init() -> Self {
        Self {
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            lx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            sp_fifo: [Pixel { val: 0 }; 16],
            bg_fifo: [Pixel { val: 0 }; 16],
            bg_fifo_state: FifoState::INDEX,
            sp_fifo_state: FifoState::SLEEP,
            bg_x: 0,
            in_win: false,
            win_y: 0,

            mode: PpuMode::OAMSCAN,
            cycles: 0,
        }
    }
}

impl Ppu {
    fn lcdc_bit(&self, bit: u8) -> bool {
        self.lcdc & 1 << bit != 0
    }

    fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        self.ly &= !(0x03);
        self.ly |= mode as u8;
    }
}

impl GameBoy {
    pub fn cycle_ppu(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.cycles += 1;
            match self.ppu.mode {
                PpuMode::HBLANK => self.hblank_cycle();
                PpuMode::VBLANK => self.vblank_cycle();
                PpuMode::OAMSCAN => self.oamscan_cycle();
                PpuMode::DRAW => self.draw_cycle(),
            };
        }
    }

    fn draw_cycle(&mut self) {
        // inicialização de vars no começo da scanline, mover pra outro lugar mais inteligente
        if self.ppu.lx == 0 {
            self.ppu.bg_fifo_state = FifoState::INDEX;
            self.ppu.sp_fifo_state = FifoState::SLEEP;
            self.ppu.win_y = 0;
            self.ppu.bg_x = 0;
            self.ppu.in_win = false;
        }
        // inicialização de vars no começo do frame, mover pra outro lugar mais inteligente
        if self.ppu.ly == 0 {
        }

        // fetchers atualizam a cada dois ciclos
        if self.ppu.cycles % 2 == 0 {
            self.bg_fifo_cycle();
            self.sp_fifo_cycle();
        }
        // todo ciclo, tenta pushar dos fifos pra tela
        self.push_lcd();

        // setta flag que indica se wy já foi igual a ly ao menos uma vez neste frame
        if self.ppu.ly == self.ppu.wy {
            self.ppu.wy_eq_ly = true;
        }

        // passa pra proxima scanline ao chegar no final
        if self.ppu.lx >= 160 {
            self.ppu.set_mode(PpuMode::HBLANK);
        }
    }

    fn push_lcd(&mut self) {
        // ainda tem que levar em conta que bg e win podem estar off, e printa só sprite
        if self.ppu.bg_fifo.empty() {
            return;
        }

        let mut pixel = Pixel { val : 0 };
        if self.ppu.lx >= (self.ppu.scx % 8) {
            pixel = self.ppu.mix_pixel();
            // escreve pixel decodificado no buffer de gráfico
        }
        // pop pixel da fifo
        self.ppu.lx += 1;
    }

    fn mix_pixel(&mut self) -> Pixel {
        let bg_pixel = self.ppu.bg_fifo[0];
        if self.ppu.sp_fifo.empty() {
            return bg_pixel;
        }

        let sp_pixel = self.ppu.sp_fifo[0];
        if sp_pixel.val == 0 ||
           bg_over_sprite_priority_bit && bg_pixel.val != 0 // eu não sei onde pega esse bit vtnc
        {
            return bg_pixel;
        }

        sp_pixel
    }

    fn bg_fifo_cycle(&mut self) {
        // verificando se está dentro da window, pra flushar o bg fifo e recomeçar fetching 
        if !self.ppu.in_win && self.ppu.lcdc_bit(5) && self.ppu.wy_eq_ly && self.ppu.lx >= self.ppu.wx - 7 {
            self.ppu.in_win = true;
            self.ppu.bg_fifo_state = FifoState::INDEX;
            self.ppu.bg_x = 0;
            self.ppu.bg_fifo.clear();
        }

        match self.ppu.bg_fifo_state {
            FifoState::INDEX => self.bg_fetch_index(),
            FifoState::DATALOW => self.bg_fetch_data_low(),
            FifoState::DATAHIGH => self.bg_fetch_data_high(),
            FifoState::PUSH => self.bg_push(),
            FifoState::SLEEP => {},
        }
    }

    fn bg_fetch_index(&mut self) {
        let mut tile_x = 0;
        let mut tile_y = 0;
        if self.ppu.in_win {
            tile_x = self.ppu.bg_x / 8;
            tile_y = self.ppu.win_y / 8;
        } else {
            tile_x = u8::wrapping_add(self.ppu.bg_x, self.ppu.scx) / 8;
            tile_y = u8::wrapping_add(self.ppu.ly, self.ppu.scy) / 8;
        }
        let tile: u16 = 32 * (tile_y as u16) + (tile_x as u16);

        let mut addr: u16 = if (self.ppu.lcdc_bit(3) && !self.ppu.in_win) || (self.ppu.lcdc_bit(6) && self.ppu.in_win) { 0x9C00 } else { 0x9800 };
        addr += tile * 16;
        
        self.ppu.bg_index = self.read(addr);

        self.ppu.bg_fifo_state = FifoState::DATALOW;
    }

    fn bg_fetch_data_low(&mut self) {
        let mut addr = if self.ppu.lcdc_bit(4) {0x8000} else {0x8800};
        addr += (self.ppu.bg_index as u16) * 16;
        self.ppu.bg_data_low = self.read(addr);

        self.ppu.bg_fifo_state = FifoState::DATAHIGH;
    }

    fn bg_fetch_data_high(&mut self) {
        let mut addr = if self.ppu.lcdc_bit(4) {0x8000} else {0x8800};
        addr += (self.ppu.bg_index as u16) * 16 + 1;
        self.ppu.bg_data_high = self.read(addr);
        self.ppu.bg_x += 8;

        self.ppu.bg_fifo_state = FifoState::PUSH;
    }

    fn bg_push(&mut self) {
        if self.ppu.bg_fifo.not_empty() {
            return;
        }
        // self.ppu.bg_fifo.push((data low e data high decoded em pixels));
    }

    fn sp_fifo_cycle(&mut self) {
        for sprite in self.ppu.sprite_buf {
            if sprite.x <= self.ppu.lx + 8 {
                self.ppu.sp_fifo_state = FifoState::INDEX;
                self.ppu.bg_fifo_state = FifoState::SLEEP;
            }
        }

        match self.ppu.sp_fifo_state {
            FifoState::INDEX => self.sp_fetch_index(),
            FifoState::DATALOW => self.sp_fetch_data_low(),
            FifoState::DATAHIGH => self.sp_fetch_data_high(),
            FifoState::PUSH => self.sp_push(),
            FifoState::SLEEP => {},
        }
    }

    fn sp_fetch_index(&mut self) {
    }

    fn sp_fetch_data_low(&mut self) {
    }

    fn sp_fetch_data_high(&mut self) {
    }

    fn sp_push(&mut self) {
        self.ppu.bg_fifo_state = FifoState::INDEX;
    }
}
