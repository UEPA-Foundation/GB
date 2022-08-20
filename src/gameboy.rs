use crate::mem::{MemoryUnit, hram::HRam, oam::Oam, unused::Unused, vram::VRam, wram0::WRam0, wramx::WRamX};
use crate::{cart, cart::Cartridge, cpu::Cpu, debug, io::IoRegisters};

pub struct GameBoy {
    pub cpu: Cpu,
    pub ime: bool,
    pub enabling_int: bool,
    pub halt: bool,
    pub cart: Box<dyn Cartridge>,
    pub vram: VRam,
    pub wram0: WRam0,
    pub wramx: WRamX,
    pub oam: Oam,
    pub _unused: Unused, // Currently unused, but will be needed for CGB implementation
    pub io: IoRegisters,
    pub hram: HRam,
    pub ie: u8,
}

impl GameBoy {
    pub fn init(path: &str) -> Self {
        Self {
            cpu: Cpu { a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0, pc: 0x100 },
            ime: false,
            enabling_int: false,
            halt: false,
            cart: cart::load_rom_file(path),
            wram0: MemoryUnit::init(),
            wramx: MemoryUnit::init(),
            vram: MemoryUnit::init(),
            oam: MemoryUnit::init(),
            _unused: MemoryUnit::init(),
            io: IoRegisters::init(),
            hram: MemoryUnit::init(),
            ie: 0,
        }
    }

    pub fn run(&mut self, debug: bool) {
        if debug {
            let mut dgb = debug::DebugGB::init(self);
            loop {
                dgb.prompt();
            }
        }

        loop {
            self.fetch_exec();
        }
    }

    pub fn advance_cycles(&mut self, cycles: u8) {
        self.cycle_timer(cycles);
    }

    pub fn read(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.cart.rom0_read(index),
            0x4000..=0x7FFF => self.cart.romx_read(index),
            0x8000..=0x9FFF => self.vram.read(index),
            0xA000..=0xBFFF => self.cart.sram_read(index),
            0xC000..=0xCFFF => self.wram0.read(index),
            0xD000..=0xDFFF => self.wramx.read(index),
            0xE000..=0xEFFF => self.wram0.read(index), // echo 0
            0xF000..=0xFDFF => self.wramx.read(index), // echo X
            0xFE00..=0xFE9F => self.oam.read(index),
            0xFEA0..=0xFEFF => self._unused.read(index),
            0xFF00..=0xFF7F => self.io_read(index),
            0xFF80..=0xFFFE => self.hram.read(index),
            0xFFFF => self.ie,
        }
    }

    #[inline(always)]
    pub fn dpc(&self, offset: i8) -> u8 {
        self.read(u16::wrapping_add(self.cpu.pc, offset as u16))
    }

    pub fn write(&mut self, index: u16, val: u8) {
        match index {
            0x0000..=0x3FFF => self.cart.rom0_write(index, val),
            0x4000..=0x7FFF => self.cart.romx_write(index, val),
            0x8000..=0x9FFF => self.vram.write(index, val),
            0xA000..=0xBFFF => self.cart.sram_write(index, val),
            0xC000..=0xCFFF => self.wram0.write(index, val),
            0xD000..=0xDFFF => self.wramx.write(index, val),
            0xE000..=0xEFFF => self.wram0.write(index, val), // echo 0
            0xF000..=0xFDFF => self.wramx.write(index, val), // echo X
            0xFE00..=0xFE9F => self.oam.write(index, val),
            0xFEA0..=0xFEFF => self._unused.write(index, val),
            0xFF00..=0xFF7F => self.io_write(index, val),
            0xFF80..=0xFFFE => self.hram.write(index, val),
            0xFFFF => self.ie = val,
        }
    }

    #[inline(always)]
    pub fn cycle_read(&mut self, addr: u16) -> u8 {
        let val = self.read(addr);
        self.advance_cycles(4);
        val
    }

    #[inline(always)]
    pub fn cycle_dpc(&mut self, offset: i8) -> u8 {
        self.cycle_read(u16::wrapping_add(self.cpu.pc, offset as u16))
    }

    #[inline(always)]
    pub fn cycle_write(&mut self, addr: u16, val: u8) {
        self.write(addr, val);
        self.advance_cycles(4);
    }

    #[inline(always)]
    pub fn fetch_interrupt(&self) -> Option<u8> {
        match self.io_read(0xFF0F) & self.ie & 0x1F {
            0 => None,
            intrs => Some(intrs.trailing_zeros() as u8),
        }
    }

    #[inline(always)]
    pub fn set_if(&mut self, intr: u8) {
        let iflags = self.io_read(0xFF0F);
        self.io_write(0xFF0F, iflags | intr);
    }

    #[inline(always)]
    pub fn reset_if(&mut self, intr: u8) {
        let iflags = self.io_read(0xFF0F);
        self.io_write(0xFF0F, iflags & !intr);
    }
}
