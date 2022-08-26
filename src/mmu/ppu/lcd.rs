#![allow(unused)] // TODO: REMOVE THIS

use crate::mmu::ppu::PPU;

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

impl PPU {
    read_simple!(lcdc, scy, scx, ly, lyc, dma, bgp, obp0, obp1, wy, wx);

    #[inline(always)]
    pub fn read_stat(&self) -> u8 {
        self.stat | 0x80
    }

    write_simple!(scy, scx, bgp, obp0, obp1, wx);

    #[inline(always)]
    pub fn write_lcdc(&mut self, val: u8) {
        () // TODO: lcdc
    }

    #[inline(always)]
    pub fn write_stat(&mut self, val: u8) {
        () // TODO: stat
    }

    #[inline(always)]
    pub fn write_ly(&mut self, val: u8) {
        () // LY is read only
    }

    #[inline(always)]
    pub fn write_lyc(&mut self, val: u8) {
        () // TODO: LYC
    }

    #[inline(always)]
    pub fn write_dma(&mut self, val: u8) {
        () // TODO: DMA
    }

    #[inline(always)]
    pub fn write_wy(&mut self, val: u8) {
        self.wy = val; // TODO: more behavior in wy
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }
}
