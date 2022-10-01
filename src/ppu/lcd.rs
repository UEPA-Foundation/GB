use super::{DmaStatus, LcdStatus, Ppu, PpuMode, NCOL, NLIN};

// generates read methods for regs with trivial reads
macro_rules! read_simple {
    ($($reg: ident),+) => {
        $(
            paste::paste! {
                #[inline(always)]
                pub fn [<read_ $reg>](&self) -> u8 {
                    self.$reg
                }
            }
        )+
    };
}

// generates write methods for regs with trivial writes
macro_rules! write_simple {
    ($($reg: ident),+) => {
        $(
            paste::paste! {
                #[inline(always)]
                pub fn [<write_ $reg>](&mut self, val: u8) {
                    self.$reg = val;
                }
            }
        )+
    };
}

// generates write methods for sprite palette regs with _almost_ trivial writes
macro_rules! write_sprite_palette {
    ($($reg: ident),+) => {
        $(
            paste::paste! {
                #[inline(always)]
                pub fn [<write_ $reg>](&mut self, val: u8) {
                    self.$reg = val & 0xFC;
                }
            }
        )+
    };
}

impl Ppu {
    read_simple!(lcdc, scy, scx, ly, lyc, dma, bgp, obp0, obp1, wy, wx);

    #[inline(always)]
    pub fn read_stat(&self) -> u8 {
        self.stat | 0x80
    }

    write_simple!(scy, scx, bgp, wx);
    write_sprite_palette!(obp0, obp1);

    #[inline(always)]
    pub fn write_lcdc(&mut self, val: u8) {
        // TODO: lots of behavior for each bit
        match (&self.lcd_status, val & 0x80 != 0) {
            (LcdStatus::OFF, true) => {
                // we leave stat alone: mode bits stay 0 during first OAM Scan
                self.lcd_status = LcdStatus::STARTUP;
            }
            (LcdStatus::ON | LcdStatus::STARTUP, false) => {
                self.lcd_status = LcdStatus::OFF;
                self.mode = PpuMode::OAMSCAN;
                self.clear_sp_fetcher();
                self.cycles = 0;
                self.ly = 0;
                self.stat &= !0x03; // stat's mode bits are 0 when off
                self.framebuffer = [0; NLIN * NCOL];
            }
            _ => {}
        }
        self.lcdc = val;
    }

    #[inline(always)]
    pub fn write_stat(&mut self, val: u8) {
        self.stat &= !0xF8;
        self.stat |= val & 0xF8;
    }

    #[inline(always)]
    pub fn write_ly(&mut self, _: u8) {
        () // LY is read only
    }

    #[inline(always)]
    pub fn write_lyc(&mut self, val: u8) {
        self.lyc = val;
    }

    #[inline(always)]
    pub fn write_dma(&mut self, val: u8) {
        self.dma = val;
        match self.oam_dma {
            DmaStatus::ACTIVE(byte) => self.oam_dma = DmaStatus::RESTARTING(byte),
            _ => self.oam_dma = DmaStatus::STARTING,
        }
    }

    #[inline(always)]
    pub fn write_wy(&mut self, val: u8) {
        self.wy = val; // TODO: more behavior in wy
    }
}
