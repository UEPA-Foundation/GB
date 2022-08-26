#![allow(dead_code)]

use crate::gameboy::GameBoy;

pub mod lcd;

pub struct PPU {
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    mode: PPUMode,
    cycles: u32,
}

#[derive(Copy, Clone)]
enum PPUMode {
    HBLANK, // mode 0
    VBLANK, // mode 1
    OAMSCAN, // mode 2
    DRAW, // mode 3
}

impl PPU {
    pub fn init() -> Self {
        Self { lcdc: 0, stat: 0, scy: 0, scx: 0, ly: 0, lyc: 0, dma: 0, bgp: 0, obp0: 0, obp1: 0, wy: 0, wx: 0, mode: PPUMode::OAMSCAN, cycles: 0 }
    }
}

impl GameBoy {
    pub fn cycle_ppu(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.cycles += 1;
            match self.ppu.mode {
                PPUMode::HBLANK => {},
                PPUMode::VBLANK => {},
                PPUMode::OAMSCAN => {
                    match self.ppu.cycles % 8 {
                        0..=1 => {}, // get tile
                        2..=3 => {}, // get tile data low
                        4..=5 => {}, // get tile data high
                        6..=7 => {}, // sleep
                        _ => panic!(),
                    }
                    // try push
                },
                PPUMode::DRAW => {},
            };
        }
    }
}
