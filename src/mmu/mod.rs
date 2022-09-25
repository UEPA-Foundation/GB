use crate::gameboy::GameBoy;
use cart::CartridgeTrait;
use mem::MemoryUnit;

pub mod cart;
pub mod io;
pub mod mem;

impl GameBoy {
    pub fn read(&self, addr: u16) -> u8 {
        match self.dma_conflict(addr) {
            Some(byte) => byte,
            None => self.pure_read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match self.dma_conflict(addr) {
            Some(_) => {},
            _ => self.pure_write(addr, val),
        }
    }

    pub fn pure_read(&self, addr: u16) -> u8 {
        match addr {
            // cart
            0x0000..=0x3FFF => self.cart.rom0_read(addr),
            0x4000..=0x7FFF => self.cart.romx_read(addr),
            // vram
            0x8000..=0x9FFF => self.ppu.vram_read(addr),
            // cart
            0xA000..=0xBFFF => self.cart.sram_read(addr),
            // wram
            0xC000..=0xCFFF => self.wram0.read(addr),
            0xD000..=0xDFFF => self.wramx.read(addr),
            // echo
            0xE000..=0xEFFF => self.wram0.read(addr),
            0xF000..=0xFDFF => self.wramx.read(addr),
            // oam
            0xFE00..=0xFE9F => self.ppu.oam_read(addr),
            // unused
            0xFEA0..=0xFEFF => self._unused.read(addr),
            // io
            0xFF00 => self.joypad.read(),
            0xFF01 => self.serial.read_data(),
            0xFF02 => self.serial.read_control(),
            0xFF03 => 0xFF,
            0xFF04 => self.timer.read_div(),
            0xFF05 => self.timer.read_tima(),
            0xFF06 => self.timer.read_tma(),
            0xFF07 => self.timer.read_tac(),
            0xFF08..=0xFF0E => 0xFF,
            0xFF0F => self.intr.read_if(),
            0xFF10..=0xFF26 => 0xFF, // sound
            0xFF27..=0xFF2F => 0xFF,
            0xFF30..=0xFF3F => 0xFF, // wave pattern
            0xFF40 => self.ppu.read_lcdc(),
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.read_scy(),
            0xFF43 => self.ppu.read_scx(),
            0xFF44 => self.ppu.read_ly(),
            0xFF45 => self.ppu.read_lyc(),
            0xFF46 => self.ppu.read_dma(),
            0xFF47 => self.ppu.read_bgp(),
            0xFF48 => self.ppu.read_obp0(),
            0xFF49 => self.ppu.read_obp1(),
            0xFF4A => self.ppu.read_wy(),
            0xFF4B => self.ppu.read_wx(),
            0xFF4C..=0xFF4E => 0xFF,
            0xFF4F => 0xFF,          // vram bank select (CGB)
            0xFF50 => 0xFF,          // disable boot ROM
            0xFF51..=0xFF55 => 0xFF, // vram dma (CGB)
            0xFF56..=0xFF67 => 0xFF,
            0xFF68..=0xFF69 => 0xFF, // bg/obj palletes (CGB)
            0xFF6A..=0xFF6F => 0xFF,
            0xFF70 => 0xFF, // wram bank select (CGB)
            0xFF71..=0xFF7F => 0xFF,
            0xFF80..=0xFFFE => self.hram.read(addr),
            0xFFFF => self.intr.read_ie(),
        }
    }

    pub fn pure_write(&mut self, addr: u16, val: u8) {
        match addr {
            // cart
            0x0000..=0x3FFF => self.cart.rom0_write(addr, val),
            0x4000..=0x7FFF => self.cart.romx_write(addr, val),
            // vram
            0x8000..=0x9FFF => self.ppu.vram_write(addr, val),
            // cart
            0xA000..=0xBFFF => self.cart.sram_write(addr, val),
            // wram
            0xC000..=0xCFFF => self.wram0.write(addr, val),
            0xD000..=0xDFFF => self.wramx.write(addr, val),
            // echo
            0xE000..=0xEFFF => self.wram0.write(addr, val),
            0xF000..=0xFDFF => self.wramx.write(addr, val),
            // oam
            0xFE00..=0xFE9F => self.ppu.oam_write(addr, val),
            // unused
            0xFEA0..=0xFEFF => self._unused.write(addr, val),
            // io
            0xFF00 => self.joypad.write(val),
            0xFF01 => self.serial.write_data(val),
            0xFF02 => self.serial.write_control(val),
            0xFF03 => {}
            0xFF04 => self.timer.write_div(),
            0xFF05 => self.timer.write_tima(val),
            0xFF06 => self.timer.write_tma(val),
            0xFF07 => self.timer.write_tac(val),
            0xFF08..=0xFF0E => {}
            0xFF0F => self.intr.write_if(val),
            0xFF10..=0xFF26 => {} // sound
            0xFF27..=0xFF2F => {}
            0xFF30..=0xFF3F => {} // wave pattern
            0xFF40 => self.ppu.write_lcdc(val),
            0xFF41 => self.ppu.write_stat(val),
            0xFF42 => self.ppu.write_scy(val),
            0xFF43 => self.ppu.write_scx(val),
            0xFF44 => self.ppu.write_ly(val),
            0xFF45 => self.ppu.write_lyc(val),
            0xFF46 => self.ppu.write_dma(val),
            0xFF47 => self.ppu.write_bgp(val),
            0xFF48 => self.ppu.write_obp0(val),
            0xFF49 => self.ppu.write_obp1(val),
            0xFF4A => self.ppu.write_wy(val),
            0xFF4B => self.ppu.write_wx(val),
            0xFF4C..=0xFF4E => {}
            0xFF4F => {}          // vram bank select (CGB)
            0xFF50 => {}          // disable boot ROM
            0xFF51..=0xFF55 => {} // vram dma (CGB)
            0xFF56..=0xFF67 => {}
            0xFF68..=0xFF69 => {} // bg/obj palletes (CGB)
            0xFF6A..=0xFF6F => {}
            0xFF70 => {} // wram bank select (CGB)
            0xFF71..=0xFF7F => {}
            0xFF80..=0xFFFE => self.hram.write(addr, val),
            0xFFFF => self.intr.write_ie(val),
        }
    }
}
