#![allow(dead_code)]

use crate::gameboy::GameBoy;
use crate::mmu::mem::{oam::Oam, vram::VRam, MemoryUnit};
use background::Background;
use fifo::{FifoState, PixelFifo};
use sprites::Sprites;

mod background;
mod fifo;
mod lcd;
mod sprites;

const NCOL: usize = 160;
const NLIN: usize = 144;

pub struct Ppu {
    // Registers
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

    // Mem controlled by PPU
    pub vram: VRam,
    pub oam: Oam,

    bg: Background,
    sp: Sprites,

    in_win: bool,
    win_y: u8,
    wy_eq_ly: bool,

    mode: PpuMode,
    cycles: u32,

    pub framebuffer: [u8; NCOL * NLIN],
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

            vram: MemoryUnit::init(),
            oam: MemoryUnit::init(),

            bg: Background::init(),
            sp: Sprites::init(),

            in_win: false,
            win_y: 0,
            wy_eq_ly: false,

            mode: PpuMode::VBLANK,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
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

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram.read(addr),
            0xFE00..=0xFE9F => self.oam.read(addr),
            _ => panic!("Addr {:02X} not owned by PPU", addr),
        }
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
                PpuMode::HBLANK => self.ppu.hblank_cycle(),
                PpuMode::VBLANK => self.ppu.vblank_cycle(),
                PpuMode::OAMSCAN => self.ppu.oamscan_cycle(),
                PpuMode::DRAW => self.ppu.draw_cycle(),
            };
        }
    }

    pub fn borrow_framebuffer(&self) -> &[u8; NCOL * NLIN] {
        &self.ppu.framebuffer
    }
}

impl Ppu {
    fn hblank_cycle(&mut self) {
        if self.cycles >= 456 {
            self.cycles = 0;
            self.lx = 0;
            self.ly += 1;
            if self.ly < 144 {
                self.mode = PpuMode::OAMSCAN;
            } else {
                self.mode = PpuMode::VBLANK;
            }
        }
    }

    fn vblank_cycle(&mut self) {
        if self.cycles >= 456 {
            self.cycles = 0;
            self.lx = 0;
            self.ly += 1;
            if self.ly >= 154 {
                self.ly = 0;
                self.mode = PpuMode::OAMSCAN;
            }
        }
    }

    fn oamscan_cycle(&mut self) {
        if self.cycles >= 80 {
            self.mode = PpuMode::DRAW;
        }
    }

    fn draw_cycle(&mut self) {
        // inicialização de vars no começo da scanline, mover pra outro lugar mais inteligente
        if self.lx == 0 {
            self.bg.fifo.state = FifoState::INDEX;
            self.sp.fifo.state = FifoState::SLEEP;
            self.win_y = 0;
            self.in_win = false;
        }
        // inicialização de vars no começo do frame, mover pra outro lugar mais inteligente
        if self.ly == 0 {}

        // fetchers atualizam a cada dois ciclos
        if self.cycles % 2 == 0 {
            self.bg_fifo_cycle();
            self.sp_fifo_cycle();
        }
        // todo ciclo, tenta pushar dos fifos pra tela
        self.push_lcd();

        // setta flag que indica se wy já foi igual a ly ao menos uma vez neste frame
        if self.ly == self.wy {
            self.wy_eq_ly = true;
        }

        // passa pra proxima scanline ao chegar no final
        if self.lx >= 160 {
            self.set_mode(PpuMode::HBLANK);
        }
    }
    fn push_lcd(&mut self) {
        // ainda tem que levar em conta que bg e win podem estar off, e printa só sprite
        if self.bg.fifo.empty() {
            return;
        }

        if self.lx >= (self.scx % 8) {
            let idx = self.ly * NCOL as u8 + self.lx;
            self.framebuffer[idx as usize] = self.mix_pixel();
        }

        self.lx += 1;
    }
}
