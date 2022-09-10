#![allow(dead_code)]

use crate::gameboy::GameBoy;
use crate::mmu::mem::{oam::Oam, vram::VRam, MemoryUnit};
use background::Background;

mod background;
mod fifo;
mod lcd;

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

    mode: PpuMode,
    cycles: u32,

    framebuffer: [u8; NCOL * NLIN],
}

#[derive(Copy, Clone, Debug)]
enum PpuMode {
    HBLANK = 0,
    VBLANK = 1,
    OAMSCAN = 2,
    DRAW = 3,
}

impl GameBoy {
    pub fn cycle_ppu(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.cycle();
        }
    }

    pub fn borrow_framebuffer(&self) -> &[u8; NCOL * NLIN] {
        &self.ppu.framebuffer
    }
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

            mode: PpuMode::OAMSCAN,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
        }
    }

    fn lcdc_bit(&self, bit: u8) -> bool {
        self.lcdc & 1 << bit != 0
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram.read(addr),
            0xFE00..=0xFE9F => self.oam.read(addr),
            _ => panic!("Addr {:02X} not owned by PPU", addr),
        }
    }

    fn cycle(&mut self) {
        self.cycles += 1;
        match self.mode {
            PpuMode::HBLANK => {
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.inc_ly();
                    if self.ly == 144 {
                        self.mode = PpuMode::VBLANK;
                    } else {
                        self.mode = PpuMode::OAMSCAN;
                    }
                }
            }
            PpuMode::VBLANK => {
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.inc_ly();
                    if self.ly == 154 {
                        self.set_ly(0);
                        self.mode = PpuMode::OAMSCAN;
                    }
                }
            }
            PpuMode::OAMSCAN => {
                if self.cycles == 80 {
                    self.init_scanline_bg();
                    self.mode = PpuMode::DRAW;
                }
            }
            PpuMode::DRAW => {
                if self.cycles % 2 == 0 {
                    self.cycle_bg();
                }

                _ = self.bg.pop().and_then(|pixel| {
                    let idx = self.ly as usize * 160 + self.lx as usize;
                    self.framebuffer[idx] = pixel;
                    self.lx += 1;
                    Ok(())
                });

                if self.lx == 160 {
                    self.mode = PpuMode::HBLANK;
                }
            }
        };
    }
}
