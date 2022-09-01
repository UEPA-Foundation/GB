#![allow(dead_code)]

use crate::gameboy::GameBoy;
use background::*;
use fifo::{FifoState, PixelFifo};

mod background;
mod fifo;
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

    bg: Background,

    sprite_buf: [u8; 16], // TODO: isso aqui vai ser preenchido em oam scan

    sp_fifo: PixelFifo,
    in_win: bool,
    win_y: u8,
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
                PpuMode::HBLANK => self.hblank_cycle(),
                PpuMode::VBLANK => self.vblank_cycle(),
                PpuMode::OAMSCAN => self.oamscan_cycle(),
                PpuMode::DRAW => self.draw_cycle(),
            };
        }
    }

    fn hblank_cycle(&mut self) {
        if self.ppu.cycles >= 456 {
            self.ppu.cycles = 0;
            self.ppu.lx = 0;
            self.ppu.ly += 1;
            if self.ppu.ly < 144 {
                self.ppu.mode = PpuMode::OAMSCAN;
            } else {
                self.ppu.mode = PpuMode::VBLANK;
            }
        }
    }

    fn vblank_cycle(&mut self) {
        if self.ppu.cycles >= 456 {
            self.ppu.cycles = 0;
            self.ppu.lx = 0;
            self.ppu.ly += 1;
            if self.ppu.ly >= 154 {
                self.ppu.ly = 0;
                self.ppu.mode = PpuMode::OAMSCAN;
            }
        }
    }

    fn oamscan_cycle(&mut self) {
        if self.ppu.cycles >= 80 {
            self.ppu.mode = PpuMode::DRAW;
        }
    }

    fn draw_cycle(&mut self) {
        // inicialização de vars no começo da scanline, mover pra outro lugar mais inteligente
        if self.ppu.lx == 0 {
            self.ppu.bg.fifo.state = FifoState::INDEX;
            self.ppu.sp_fifo.state = FifoState::SLEEP;
            self.ppu.win_y = 0;
            self.ppu.in_win = false;
        }
        // inicialização de vars no começo do frame, mover pra outro lugar mais inteligente
        if self.ppu.ly == 0 {}

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

    fn sp_fifo_cycle(&mut self) {
        for sprite in self.ppu.sprite_buf {
            if sprite <= self.ppu.lx + 8 {
                self.ppu.sp_fifo.state = FifoState::INDEX;
                self.ppu.bg.fifo.state = FifoState::SLEEP;
            }
        }

        match self.ppu.sp_fifo.state {
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

    fn push_lcd(&mut self) {
        // ainda tem que levar em conta que bg e win podem estar off, e printa só sprite
        if self.ppu.bg.fifo.empty() {
            return;
        }

        if self.ppu.lx >= (self.ppu.scx % 8) {
            let pixel = self.ppu.mix_pixel();
            // TODO: write pixel to framebuffer
        }

        self.ppu.lx += 1;
    }
}

impl Ppu {
    fn mix_pixel(&mut self) -> u8 {
        let bg_pixel = self.ppu.bg_fifo.pop().unwrap();
        if self.ppu.sp_fifo.empty() {
            return bg_pixel;
        }

        let sp_pixel = self.ppu.sp_fifo.pop().unwrap();

        if (sp_pixel == 0 || bg_over_sprite_priority_bit) && bg_pixel != 0
        // eu não sei onde pega esse bit vtnc
        {
            return bg_pixel;
        }

        sp_pixel
    }
}
