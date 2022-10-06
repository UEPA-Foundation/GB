use crate::gameboy::GameBoy;
use crate::intr::Interrupt;
use background::Background;
use sprites::Sprites;

use oam::Oam;
use vram::VRam;

mod background;
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
    dma_cycles: u16,

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

#[derive(Copy, Clone, Debug)]
enum LcdStatus {
    ON,
    OFF,
    STARTUP,
}

#[derive(Copy, Clone, Debug)]
enum DmaStatus {
    INACTIVE,
    STARTING,
    RESTARTING(u8),
    FIRSTREAD,
    RESTARTFIRSTREAD,
    ACTIVE(u8),
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
        if let DmaStatus::INACTIVE = self.ppu.oam_dma {
            return;
        }

        self.ppu.dma_cycles += 1;
        if self.ppu.dma_cycles % 4 == 0 {
            let count = self.ppu.dma_cycles / 4;
            match self.ppu.oam_dma {
                DmaStatus::STARTING => {
                    self.ppu.oam_dma = DmaStatus::FIRSTREAD;
                }
                DmaStatus::RESTARTING(byte) => {
                    self.ppu.oam.write(count - 1, byte);
                    self.ppu.oam_dma = DmaStatus::RESTARTFIRSTREAD;
                }
                DmaStatus::FIRSTREAD => {
                    self.ppu.dma_cycles = 0;
                    let byte = self.oam_dma_fetch();
                    self.ppu.oam_dma = DmaStatus::ACTIVE(byte);
                }
                DmaStatus::RESTARTFIRSTREAD => {
                    self.ppu.dma_cycles = 0;
                    let byte = self.oam_dma_fetch();
                    self.ppu.oam_dma = DmaStatus::ACTIVE(byte);
                }
                DmaStatus::ACTIVE(byte) => {
                    self.ppu.oam.write(count - 1, byte);
                    if count >= 0xA0 {
                        self.ppu.dma_cycles = 0;
                        self.ppu.oam_dma = DmaStatus::INACTIVE;
                    } else {
                        let new_byte = self.oam_dma_fetch();
                        self.ppu.oam_dma = DmaStatus::ACTIVE(new_byte);
                    }
                }
                _ => {}
            }
        }
    }

    fn oam_dma_fetch(&self) -> u8 {
        match self.ppu.dma <= 0xDF {
            true => self.pure_read(((self.ppu.dma as u16) << 8) + self.ppu.dma_cycles / 4),
            false => self.pure_read((((self.ppu.dma as u16) << 8) + self.ppu.dma_cycles / 4) - 0x2000),
        }
    }

    pub fn dma_conflict(&self, addr: u16) -> Option<u8> {
        match (self.ppu.oam_dma, self.ppu.dma, addr) {
            (
                DmaStatus::ACTIVE(_) | DmaStatus::FIRSTREAD | DmaStatus::RESTARTFIRSTREAD | DmaStatus::RESTARTING(_),
                0x00..=0x7F | 0xA0..=0xFF,
                0x0000..=0x7FFF | 0xA000..=0xDFFF,
            ) => Some(self.oam_dma_fetch()), // rom/sram/wram bus conflict: the CPU reads the byte being read by DMA
            (
                DmaStatus::ACTIVE(_) | DmaStatus::FIRSTREAD | DmaStatus::RESTARTFIRSTREAD | DmaStatus::RESTARTING(_),
                0x80..=0x9F,
                0x8000..=0x9FFF,
            ) => Some(self.oam_dma_fetch()), // vram bus conflict: the CPU reads the byte being read by DMA
            _ => None,
        }
    }

    pub fn borrow_framebuffer(&self) -> &[u8; NCOL * NLIN] {
        &self.ppu.framebuffer
    }
}

macro_rules! bit_access {
    ($reg: ident, $bit_name: ident, $bit: expr) => {
        paste::paste! {
            #[inline(always)]
            fn [<$reg _ $bit_name>](&self) -> bool {
                self.$reg & (1 << $bit) != 0
            }
        }
    };
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
            bgp: 0b11100100,
            obp0: 0b11111111,
            obp1: 0b11111111,
            wy: 0,
            wx: 0,

            stat_line: false,
            stat_intr: false,
            vblank_intr: false,

            vram: VRam::init(),
            oam: Oam::init(),

            oam_dma: DmaStatus::INACTIVE,
            dma_cycles: 0,

            bg: Background::init(),
            sp: Sprites::init(),

            mode: PpuMode::OAMSCAN,
            cycles: 0,

            framebuffer: [0; NLIN * NCOL],
            lcd_status: LcdStatus::ON,
        }
    }

    bit_access!(lcdc, bg_enbl, 0);
    bit_access!(lcdc, sp_enbl, 1);
    bit_access!(lcdc, sp_size, 2);
    bit_access!(lcdc, bg_tm_sel, 3);
    bit_access!(lcdc, td_sel, 4);
    bit_access!(lcdc, wn_enbl, 5);
    bit_access!(lcdc, wn_tm_sel, 6);
    // bit_access!(lcdc, lcd_enbl, 7);

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
            PpuMode::HBLANK => self.stat & 0x08 != 0,
            PpuMode::VBLANK => (self.stat & 0x10 != 0) || (self.ly == 144 && self.cycles == 4 && self.stat & 0x20 != 0),
            PpuMode::OAMSCAN => self.stat & 0x20 != 0,
            PpuMode::DRAW => false,
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
        match (&self.lcd_status, self.mode, self.oam_dma) {
            (LcdStatus::ON, PpuMode::DRAW | PpuMode::OAMSCAN, _) => 0xFF,
            (_, _, DmaStatus::ACTIVE(_) | DmaStatus::RESTARTING(_) | DmaStatus::RESTARTFIRSTREAD) => 0xFF,
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
        match (&self.lcd_status, self.mode, self.oam_dma) {
            (LcdStatus::ON, PpuMode::DRAW | PpuMode::OAMSCAN, _) => (),
            (_, _, DmaStatus::ACTIVE(_) | DmaStatus::RESTARTING(_) | DmaStatus::RESTARTFIRSTREAD) => (),
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
                }
            }
        };
        self.update_stat();
    }

    fn draw_pixel(&mut self) {
        if self.sp.is_fetching() || self.check_in_win() {
            return;
        }
        _ = self.mix_pixel().and_then(|pixel| {
            let idx = self.ly as usize * 160 + self.lx as usize;
            // first frame after turning lcd on gets skipped
            if let LcdStatus::ON = self.lcd_status {
                self.framebuffer[idx] = pixel;
            }
            self.lx += 1;
            self.fetch_obj();
            Some(())
        });
    }

    fn mix_pixel(&mut self) -> Option<u8> {
        let bg_pixel = self.bg_pop()?;
        let (sp_pixel, bg_priority, sp_palette) = self.sp_pop().unwrap_or((0, false, 0));

        if sp_pixel == 0 || (bg_priority && bg_pixel != 0) {
            return Some(apply_palette(bg_pixel, self.bgp));
        }

        Some(apply_palette(sp_pixel, sp_palette))
    }
}

#[inline(always)]
fn apply_palette(color: u8, palette: u8) -> u8 {
    match color & 0x03 {
        0 => (palette & 0b00000011) >> 0,
        1 => (palette & 0b00001100) >> 2,
        2 => (palette & 0b00110000) >> 4,
        3 => (palette & 0b11000000) >> 6,
        wtf => panic!("How can a 2 bit number be {}?", wtf),
    }
}
