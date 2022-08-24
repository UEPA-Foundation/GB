use crate::mem::{hram::HRam, oam::Oam, unused::Unused, vram::VRam, wram0::WRam0, wramx::WRamX, MemoryUnit};
use crate::{
    cpu::Cpu,
    mmu::{
        cart,
        cart::Cartridge,
        io::{joypad::Joypad, serial::SerialLink, timer::Timer},
    },
};

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

    pub joypad: Joypad,
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
            joypad: Joypad::init(),
            timer: Timer::init(),
            serial: SerialLink::init(),
            iflags: 0,
            ie: 0,
        }
    }

    pub fn advance_cycles(&mut self, cycles: u8) {
        self.cycle_timer(cycles);
        self.cycle_joypad(cycles);
    }

    #[inline(always)]
    pub fn dpc(&self, offset: i8) -> u8 {
        self.read(u16::wrapping_add(self.cpu.pc, offset as u16))
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
