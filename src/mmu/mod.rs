use crate::mmu::{cart::Cartridge, hram::HRam, unused::Unused, wram0::WRam0, wramx::WRamX};

mod cart;
mod hram;
mod unused;
mod wram0;
mod wramx;

pub struct Mmu {
    cart: Box<dyn Cartridge>,
    // vram: VRam,
    wram0: WRam0,
    wramx: WRamX,
    // echo: Echo,
    // oam: Oam,
    unused: Unused,
    // io: IoRegisters,
    hram: HRam,
    // ie: IeRegister,
}

pub trait MemoryUnit {
    fn init() -> Self
    where
        Self: Sized;
    fn read(&self, index: u16) -> u8;
    fn write(&mut self, index: u16, val: u8);
}

impl Mmu {
    pub fn init(path: &str) -> Self {
        Self {
            cart: cart::read_rom(path),
            wram0: <WRam0 as MemoryUnit>::init(),
            wramx: <WRamX as MemoryUnit>::init(),
            hram: <HRam as MemoryUnit>::init(),
            unused: <Unused as MemoryUnit>::init(),
        }
    }

    pub fn read(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.cart.rom0_read(index),
            0x4000..=0x7FFF => self.cart.romx_read(index),
            // 0x8000..=0x9FFF => self.vram.read(index),
            0xA000..=0xBFFF => self.cart.sram_read(index),
            0xC000..=0xCFFF => self.wram0.read(index),
            0xD000..=0xDFFF => self.wramx.read(index),
            // 0xE000..=0xFDFF => self.echo.read(index),
            // 0xFE00..=0xFE9F => self.oam.read(index),
            0xFEA0..=0xFEFF => self.unused.read(index),
            // 0xFF00..=0xFF7F => self.io.read(index),
            0xFF80..=0xFFFE => self.hram.read(index),
            // 0xFFFF => self.ie.read(index),
            _ => panic!(),
        }
    }

    pub fn write(&mut self, index: u16, val: u8) {
        match index {
            0x0000..=0x3FFF => self.cart.rom0_write(index, val),
            0x4000..=0x7FFF => self.cart.romx_write(index, val),
            // 0x8000..=0x9FFF => self.vram.write(index, val),
            0xA000..=0xBFFF => self.cart.sram_write(index, val),
            0xC000..=0xCFFF => self.wram0.write(index, val),
            0xD000..=0xDFFF => self.wramx.write(index, val),
            // 0xE000..=0xFDFF => self.echo.write(index, val),
            // 0xFE00..=0xFE9F => self.oam.write(index, val),
            0xFEA0..=0xFEFF => self.unused.write(index, val),
            // 0xFF00..=0xFF7F => self.io.write(index, val),
            0xFF80..=0xFFFE => self.hram.write(index, val),
            // 0xFFFF => self.ie.write(index, val),
            _ => panic!(),
        }
    }
}
