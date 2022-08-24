pub struct LCD {
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
}

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

impl LCD {
    pub fn init() -> Self {
        Self { lcdc: 0, stat: 0, scy: 0, scx: 0, ly: 0, lyc: 0, dma: 0, bgp: 0, obp0: 0, obp1: 0, wy: 0, wx: 0 }
    }

    read_simple!(lcdc, scy, scx, ly, lyc, dma, bgp, obp0, obp1, wy, wx);

    #[inline(always)]
    pub fn read_stat(&self) -> u8 {
        // TODO: stat logic
        self.stat
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
}
