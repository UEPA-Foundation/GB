#![allow(dead_code)]

use crate::gameboy::GameBoy;
use background::*;
use sprites::*;
use fifo::{FifoState, PixelFifo};

mod background;
mod sprites;
mod fifo;
pub mod lcd;

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
    sp: Sprites,

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

            bg: Background::init(),
            sp: Sprites::init(),

            in_win: false,
            win_y: 0,
            wy_eq_ly: false,

            mode: PpuMode::VBLANK,
            cycles: 0,
        }
    }

    fn lcdc_bit(&self, bit: u8) -> bool {
        self.lcdc & 1 << bit != 0
    }

    fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        self.ly &= !(0x03);
        self.ly |= mode as u8;
    }

    fn mix_pixel(&mut self) -> u8 {
        let bg_pixel = self.bg.fifo.pop().unwrap();
        bg_pixel

        /*
        if self.sp_fifo.empty() {
            return bg_pixel;
        }

        let sp_pixel = self.sp_fifo.pop().unwrap();

        
        if (sp_pixel == 0 || bg_over_sprite_priority_bit) && bg_pixel != 0
        {
            return bg_pixel;
        }

        sp_pixel
        */
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
            self.ppu.sp.fifo.state = FifoState::SLEEP;
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
