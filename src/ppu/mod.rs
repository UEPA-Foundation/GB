use crate::gameboy::GameBoy;
use crate::intr::Interrupt;
use background::Background;

use oam::Oam;
use vram::VRam;

mod background;
mod fifo;
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

    // interrupts
    stat_line: bool,
    stat_intr: bool,
    vblank_intr: bool,

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

        if self.ppu.stat_intr {
            self.intr.request(Interrupt::STAT);
            self.ppu.stat_intr = false;
        }
        if self.ppu.vblank_intr {
            self.intr.request(Interrupt::VBLANK);
            self.ppu.vblank_intr = false;
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

            stat_line: false,
            stat_intr: false,
            vblank_intr: false,

            vram: VRam::init(),
            oam: Oam::init(),

            bg: Background::init(),

            mode: PpuMode::OAMSCAN,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
        }
    }

    #[inline(always)]
    fn lcdc_bit(&self, bit: u8) -> bool {
        self.lcdc & 1 << bit != 0
    }

    #[inline(always)]
    fn stat_bit(&self, bit: u8) -> bool {
        self.stat & 1 << bit != 0
    }

    #[inline(always)]
    fn set_mode(&mut self, mode: PpuMode) {
        self.stat &= !0x03;
        self.stat |= mode as u8;
        self.mode = mode;
    }

    fn update_stat(&mut self) {
        let old_stat_line = self.stat_line;
        let lyc_line = self.stat_bit(6) && self.ly == self.lyc;
        let mode_line = match self.mode {
            PpuMode::DRAW => false,
            _ => self.stat & (8 << (self.mode as u8)) != 0,
        };
        self.stat_line = lyc_line || mode_line;

        // rising edge detection
        if !old_stat_line && self.stat_line {
            self.stat_intr = true;
        }

        if lyc_line {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
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
                    self.ly += 1;
                    if self.ly == 144 {
                        self.init_frame_bg();
                        self.set_mode(PpuMode::VBLANK);
                    } else {
                        self.set_mode(PpuMode::OAMSCAN);
                    }
                    self.update_stat();
                }
            }
            PpuMode::VBLANK => {
                if self.ly == 144 && self.cycles == 4 {
                    self.vblank_intr = true;
                }
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    if self.ly == 154 {
                        self.ly = 0;
                        self.set_mode(PpuMode::OAMSCAN);
                    }
                    self.update_stat();
                }
            }
            PpuMode::OAMSCAN => {
                if self.cycles == 1 {
                    self.check_in_win_y();
                }
                if self.cycles == 80 {
                    self.init_scanline_bg();
                    self.set_mode(PpuMode::DRAW);
                }
            }
            PpuMode::DRAW => {
                if self.cycles % 2 == 0 {
                    self.cycle_bg();
                }

                _ = self.bg.pop().and_then(|pixel| {
                    if self.bg.win_mode || self.lx >= self.scx % 8 {
                        let idx = self.ly as usize * 160 + self.lx as usize;
                        self.framebuffer[idx] = pixel;
                    }
                    self.lx += 1;
                    self.check_in_win();
                    Ok(())
                });

                if self.lx == 160 {
                    self.set_mode(PpuMode::HBLANK);
                    self.update_stat();
                }
            }
        };
    }
}
