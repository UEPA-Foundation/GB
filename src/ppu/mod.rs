use crate::gameboy::GameBoy;
use crate::intr::Interrupt;
use background::Background;
use sprites::Sprites;

use oam::Oam;
use vram::VRam;

mod background;
mod fifo;
mod lcd;
mod oam;
mod sprites;
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

    oam_dma: DmaStatus,

    bg: Background,
    sp: Sprites,

    mode: PpuMode,
    cycles: u32,

    framebuffer: [u8; NCOL * NLIN],
    lcd_status: LcdStatus,
}

#[derive(Copy, Clone, Debug)]
enum PpuMode {
    HBLANK = 0,
    VBLANK = 1,
    OAMSCAN = 2,
    DRAW = 3,
}

enum LcdStatus {
    ON,
    OFF,
    STARTUP,
}

#[derive(Copy, Clone, Debug)]
enum DmaStatus {
    INACTIVE,
    ACTIVE(u16),
}

impl GameBoy {
    pub fn cycle_ppu(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.cycle_dma();
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

    fn cycle_dma(&mut self) {
        if let DmaStatus::ACTIVE(count) = self.ppu.oam_dma {
            let addr = ((self.ppu.dma as u16) << 8) + count;
            let val = self.read(addr);
            self.ppu.oam.write(count, val);
            if count + 1 > 0x9F {
                self.ppu.oam_dma = DmaStatus::INACTIVE;
            } else {
                self.ppu.oam_dma = DmaStatus::ACTIVE(count + 1);
            }
        };
    }

    pub fn borrow_framebuffer(&self) -> &[u8; NCOL * NLIN] {
        &self.ppu.framebuffer
    }
}

impl Ppu {
    pub fn init() -> Self {
        Self {
            lcdc: 0x91,
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

            oam_dma: DmaStatus::INACTIVE,

            bg: Background::init(),
            sp: Sprites::init(),

            mode: PpuMode::OAMSCAN,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
            lcd_status: LcdStatus::ON,
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

    pub fn vram_read(&self, addr: u16) -> u8 {
        match (&self.lcd_status, self.mode) {
            (LcdStatus::ON, PpuMode::DRAW) => 0xFF,
            _ => self.vram.read(addr),
        }
    }

    pub fn oam_read(&self, addr: u16) -> u8 {
        match (&self.lcd_status, self.mode) {
            (LcdStatus::ON, PpuMode::DRAW | PpuMode::OAMSCAN) => 0xFF,
            _ => self.oam.read(addr),
        }
    }

    pub fn vram_write(&mut self, addr: u16, val: u8) {
        match (&self.lcd_status, self.mode) {
            (LcdStatus::ON, PpuMode::DRAW) => (),
            _ => self.vram.write(addr, val),
        }
    }

    pub fn oam_write(&mut self, addr: u16, val: u8) {
        match (&self.lcd_status, self.mode) {
            (LcdStatus::ON, PpuMode::DRAW | PpuMode::OAMSCAN) => (),
            _ => self.oam.write(addr, val),
        }
    }

    fn cycle(&mut self) {
        if let LcdStatus::OFF = self.lcd_status {
            return;
        };
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
                        self.clear_sp_fetcher();
                        self.set_mode(PpuMode::OAMSCAN);
                    }
                    self.update_stat();
                }
            }
            PpuMode::VBLANK => {
                if self.ly == 144 && self.cycles == 4 {
                    self.vblank_intr = true;
                } else if self.cycles == 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    if self.ly == 154 {
                        self.ly = 0;
                        self.clear_sp_fetcher();
                        self.lcd_status = LcdStatus::ON;
                        self.set_mode(PpuMode::OAMSCAN);
                    }
                    self.update_stat();
                }
            }
            PpuMode::OAMSCAN => {
                if self.cycles % 2 == 0 {
                    self.fetch_object();
                }

                if self.cycles == 1 {
                    self.check_in_win_y();
                } else if self.cycles == 80 {
                    self.lx = 0;
                    self.init_scanline_bg();
                    self.init_scanline_sp();
                    self.set_mode(PpuMode::DRAW);
                }
            }
            PpuMode::DRAW => {
                if self.cycles % 2 == 0 {
                    self.cycle_bg();
                    self.cycle_sp();
                }

                self.draw_pixel();

                if self.lx == 160 {
                    self.set_mode(PpuMode::HBLANK);
                    self.update_stat();
                }
            }
        };
    }

    fn draw_pixel(&mut self) {
        _ = self.mix_pixel().and_then(|pixel| {
            let idx = self.ly as usize * 160 + self.lx as usize;
            //first frame after turning lcd on gets skipped
            if let LcdStatus::ON = self.lcd_status {
                self.framebuffer[idx] = pixel;
            }
            self.lx += 1;
            self.check_in_win();
            Some(())
        });
    }

    fn mix_pixel(&mut self) -> Option<u8> {
        let bg_pixel = self.bg_pop()?;
        let sp_pixel = self.sp_pop().unwrap_or(0);

        let bg_to_obj_priority = false; // TODO: actually fetch this
        if sp_pixel == 0 || (bg_to_obj_priority && bg_pixel != 0) {
            return Some(bg_pixel);
        }

        Some(sp_pixel)
    }
}
