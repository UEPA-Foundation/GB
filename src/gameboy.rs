use crate::{
    cpu::Cpu,
    intr::InterruptHandler,
    mmu::{
        cart,
        cart::Cartridge,
        io::{joypad::Joypad, serial::SerialLink, timer::Timer},
        mem::{hram::HRam, unused::Unused, wram0::WRam0, wramx::WRamX, MemoryUnit, vram::VRam, oam::Oam},
        ppu::{PPU}
    },
};

pub struct GameBoy {
    pub cpu: Cpu,
    pub halt: bool,

    pub intr: InterruptHandler,

    pub cart: Box<dyn Cartridge>,
    pub wram0: WRam0,
    pub wramx: WRamX,
    pub _unused: Unused, // Currently unused, but will be needed for CGB implementation
    pub hram: HRam,

    pub ppu: PPU,
    pub vram: VRam,
    pub oam: Oam,

    pub joypad: Joypad,
    pub serial: SerialLink,
    pub timer: Timer,
}

impl GameBoy {
    pub fn init(path: &str) -> Self {
        Self {
            cpu: Cpu { a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0, pc: 0x100 },
            halt: false,
            intr: InterruptHandler::init(),

            cart: cart::load_rom_file(path),
            wram0: MemoryUnit::init(),
            wramx: MemoryUnit::init(),
            _unused: MemoryUnit::init(),
            hram: MemoryUnit::init(),

            ppu: PPU::init(),
            vram: MemoryUnit::init(),
            oam: MemoryUnit::init(),

            joypad: Joypad::init(),
            timer: Timer::init(),
            serial: SerialLink::init(),
        }
    }

    pub fn advance_cycles(&mut self, cycles: u8) {
        self.cycle_timer(cycles);
        self.cycle_joypad(cycles);
        self.cycle_ppu(cycles);
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
}
