use crate::gameboy::GameBoy;
use crate::mmu::{
    cart::Cartridge, hram::HRam, io::IoRegisters, oam::Oam, unused::Unused, vram::VRam, wram0::WRam0, wramx::WRamX,
};

mod cart;
mod hram;
mod io;
mod oam;
mod unused;
mod vram;
mod wram0;
mod wramx;

pub struct Mmu {
    cart: Box<dyn Cartridge>,
    vram: VRam,
    wram0: WRam0,
    wramx: WRamX,
    oam: Oam,
    _unused: Unused, // Currently unused, but will be needed for CGB implementation
    io: IoRegisters,
    hram: HRam,
    // ie: IeRegister,
}

impl Mmu {
    pub fn init(path: &str) -> Self {
        Self {
            cart: cart::load_rom_file(path),
            wram0: WRam0::init(),
            wramx: WRamX::init(),
            vram: VRam::init(),
            oam: Oam::init(),
            _unused: Unused::init(),
            io: IoRegisters::init(),
            hram: HRam::init(),
        }
    }
}

impl GameBoy {
    pub fn read(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.mmu.cart.rom0_read(index),
            0x4000..=0x7FFF => self.mmu.cart.romx_read(index),
            0x8000..=0x9FFF => self.vram_read(index),
            0xA000..=0xBFFF => self.mmu.cart.sram_read(index),
            0xC000..=0xCFFF => self.wram0_read(index),
            0xD000..=0xDFFF => self.wramx_read(index),
            0xE000..=0xEFFF => self.wram0_read(index), // echo 0
            0xF000..=0xFDFF => self.wramx_read(index), // echo X
            0xFE00..=0xFE9F => self.oam_read(index),
            0xFEA0..=0xFEFF => self.unused_read(index),
            0xFF00..=0xFF7F => self.io_read(index),
            0xFF80..=0xFFFE => self.hram_read(index),
            // 0xFFFF => self.ie_read(index),
            _ => panic!(),
        }
    }

    pub fn write(&mut self, index: u16, val: u8) {
        match index {
            0x0000..=0x3FFF => self.mmu.cart.rom0_write(index, val),
            0x4000..=0x7FFF => self.mmu.cart.romx_write(index, val),
            0x8000..=0x9FFF => self.vram_write(index, val),
            0xA000..=0xBFFF => self.mmu.cart.sram_write(index, val),
            0xC000..=0xCFFF => self.wram0_write(index, val),
            0xD000..=0xDFFF => self.wramx_write(index, val),
            0xE000..=0xEFFF => self.wram0_write(index, val), // echo 0
            0xF000..=0xFDFF => self.wramx_write(index, val), // echo X
            0xFE00..=0xFE9F => self.oam_write(index, val),
            0xFEA0..=0xFEFF => self.unused_write(index, val),
            0xFF00..=0xFF7F => self.io_write(index, val),
            0xFF80..=0xFFFE => self.hram_write(index, val),
            // 0xFFFF => self.ie_write(index, val),
            _ => panic!(),
        }
    }
}
