#![allow(dead_code)]

use crate::gameboy::GameBoy;

pub mod lcd;

#[derive(Copy, Clone)]
struct Pixel {
    val: u8,
}

impl Pixel {
    const COL: u8 = 0b00000011;
    const PAL: u8 = 0b00011100;
    const SPR: u8 = 0b00100000;
    const BPR: u8 = 0b01000000;

    fn get_color(&self) -> u8 {
        self.val & Pixel::COL
    }

    fn set_color(&mut self, val: u8) {
        self.val |= val & Pixel::COL;
    }

    fn get_pallete(&self) -> u8 {
        (self.val & Pixel::PAL) >> 2
    }

    fn set_pallete(&mut self, val: u8) {
        self.val |= (val << 2) & Pixel::PAL
    }

    fn get_sprite_priority(&self) -> bool {
        self.val & Pixel::SPR != 0
    }

    fn set_sprite_priority(&mut self, val: bool) {
        self.val |= (val as u8) << 5;
    }

    fn get_bg_priority(&self) -> bool {
        self.val & Pixel::BPR != 0
    }

    fn set_bg_priority(&mut self, val: bool) {
        self.val |= (val as u8) << 6;
    }
}

pub struct Ppu {
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

    sp_fifo: [Pixel; 16],
    bg_fifo: [Pixel; 16],

    mode: PpuMode,
    cycles: u32,
}

#[derive(Copy, Clone)]
enum PpuMode {
    HBLANK,  // mode 0
    VBLANK,  // mode 1
    OAMSCAN, // mode 2
    DRAW,    // mode 3
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

            sp_fifo: [Pixel { val: 0 }; 16],
            bg_fifo: [Pixel { val: 0 }; 16],

            mode: PpuMode::OAMSCAN,
            cycles: 0,
        }
    }
}

impl GameBoy {
    pub fn cycle_ppu(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.cycles += 1;
            match self.ppu.mode {
                PpuMode::HBLANK => {}
                PpuMode::VBLANK => {}
                PpuMode::OAMSCAN => {
                    match self.ppu.cycles % 8 {
                        0..=1 => {} // get tile
                        2..=3 => {} // get tile data low
                        4..=5 => {} // get tile data high
                        6..=7 => {} // sleep
                        _ => panic!(),
                    }
                    // try push
                }
                PpuMode::DRAW => {}
            };
        }
    }
}

impl Ppu {
    pub fn lcdc_bit(&self, bit: u8) -> bool {
        self.lcdc & 1 << bit != 0
    }

    pub fn fetch_background(&mut self) {
        let bg_x = u8::wrapping_add(self.lx, self.scx);
        let bg_y = u8::wrapping_add(self.ly, self.scy);

        let in_x = (self.wx as i16) - 7 < bg_x as i16 && (bg_x as u16) < (self.wx as u16) + 249;
        let in_y = self.wy < bg_y && (bg_y as u16) < (self.wy as u16) + 256;
        let in_window = in_x && in_y;

        let mut addr: u16 = if (self.lcdc_bit(3) && !in_x) || (self.lcdc_bit(6) && in_x) { 0x9C00 } else { 0x9800 };

        let x_tile: u8 = if in_window {
            let win_x = bg_x - self.wx + 7;
            win_x / 8
        } else {
            bg_x / 8
        };

        let y_tile: u8 = if in_window {
            let win_y = bg_y - self.wy;
            win_y / 8
        } else {
            bg_y / 8
        };

        let tile: u16 = 32 * (y_tile as u16) + (x_tile as u16);
        addr += tile * 16;

        println!("{}", addr);
    }
}
