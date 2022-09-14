use crate::gameboy::GameBoy;

use oam::Oam;
use vram::VRam;

mod lcd;
mod oam;
mod vram;

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

            vram: VRam::init(),
            oam: Oam::init(),

            mode: PpuMode::OAMSCAN,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
        }
    }
    fn cycle(&mut self) {
        self.cycles += 1;
        match self.mode {
            PpuMode::HBLANK => {
            }
            PpuMode::VBLANK => {
            }
            PpuMode::OAMSCAN => {
            }
            PpuMode::DRAW => {
            }
        };
    }
}
