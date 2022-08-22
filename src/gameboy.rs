use crate::io::{serial::SerialLink, timer::Timer};
use crate::mem::{hram::HRam, oam::Oam, unused::Unused, vram::VRam, wram0::WRam0, wramx::WRamX, MemoryUnit};
use crate::{cart, cart::Cartridge, cpu::Cpu, debug};

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
    pub hram: HRam,

    pub serial: SerialLink,
    pub timer: Timer,
    pub iflags: u8,
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
            hram: MemoryUnit::init(),
            timer: Timer::init(),
            serial: SerialLink::init(),
            iflags: 0,
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

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.cart.rom0_read(addr),
            0x4000..=0x7FFF => self.cart.romx_read(addr),
            0x8000..=0x9FFF => self.vram.read(addr),
            0xA000..=0xBFFF => self.cart.sram_read(addr),
            0xC000..=0xCFFF => self.wram0.read(addr),
            0xD000..=0xDFFF => self.wramx.read(addr),
            0xE000..=0xEFFF => self.wram0.read(addr), // echo 0
            0xF000..=0xFDFF => self.wramx.read(addr), // echo X
            0xFE00..=0xFE9F => self.oam.read(addr),
            0xFEA0..=0xFEFF => self._unused.read(addr),
            0xFF00 => 0, // TODO
            0xFF01 | 0xFF02 => self.serial.read(addr),
            0xFF03 => 0, // TODO
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF08..=0xFF0E => 0,         // TODO
            0xFF0F => self.iflags | 0xE0, // 3 upper bits always return 1,
            0xFF10..=0xFF7F => 0,         // TODO
            0xFF80..=0xFFFE => self.hram.read(addr),
            0xFFFF => self.ie,
        }
    }

    #[inline(always)]
    pub fn dpc(&self, offset: i8) -> u8 {
        self.read(u16::wrapping_add(self.cpu.pc, offset as u16))
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF => self.cart.rom0_write(addr, val),
            0x4000..=0x7FFF => self.cart.romx_write(addr, val),
            0x8000..=0x9FFF => self.vram.write(addr, val),
            0xA000..=0xBFFF => self.cart.sram_write(addr, val),
            0xC000..=0xCFFF => self.wram0.write(addr, val),
            0xD000..=0xDFFF => self.wramx.write(addr, val),
            0xE000..=0xEFFF => self.wram0.write(addr, val), // echo 0
            0xF000..=0xFDFF => self.wramx.write(addr, val), // echo X
            0xFE00..=0xFE9F => self.oam.write(addr, val),
            0xFEA0..=0xFEFF => self._unused.write(addr, val),
            0xFF00 => (), // TODO
            0xFF01 | 0xFF02 => self.serial.write(addr, val),
            0xFF03 => (), // TODO
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF08..=0xFF0E => (), // TODO
            0xFF0F => self.iflags = val,
            0xFF10..=0xFF7F => (), // TODO
            0xFF80..=0xFFFE => self.hram.write(addr, val),
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
        match self.iflags & self.ie & 0x1F {
            0 => None,
            intrs => Some(intrs.trailing_zeros() as u8),
        }
    }

    #[inline(always)]
    pub fn set_if(&mut self, intr: u8) {
        self.iflags = self.iflags | intr;
    }

    #[inline(always)]
    pub fn reset_if(&mut self, intr: u8) {
        self.iflags = self.iflags & !intr;
    }
}
