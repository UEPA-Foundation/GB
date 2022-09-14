
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

    framebuffer: [u8; NCOL * NLIN],
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

            framebuffer: [0; NLIN * NCOL],
        }
    }
}
