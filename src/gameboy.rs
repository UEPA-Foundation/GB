use crate::{
    cpu::Cpu,
    intr::InterruptHandler,
    mmu::{
        cart,
        cart::CartridgeEnum,
        io::{joypad::Joypad, serial::SerialLink, timer::Timer},
        mem::{hram::HRam, unused::Unused, wram0::WRam0, wramx::WRamX, MemoryUnit},
    },
    ppu::Ppu,
};

pub struct GameBoy {
    pub cpu: Cpu,
    pub halt: bool,
    pub halt_bug: bool,

    pub intr: InterruptHandler,

    pub cart: CartridgeEnum,
    pub wram0: WRam0,
    pub wramx: WRamX,
    pub _unused: Unused, // Currently unused, but will be needed for CGB implementation
    pub hram: HRam,

    pub ppu: Ppu,

    pub joypad: Joypad,
    pub serial: SerialLink,
    pub timer: Timer,
}

impl GameBoy {
    pub fn init(path: &str) -> Self {
        Self {
            cpu: Cpu { a: 0x01, f: 0xB0, b: 0x00, c: 0x13, d: 0x00, e: 0xD8, h: 0x01, l: 0x4D, sp: 0xFFFE, pc: 0x100 },
            halt: false,
            halt_bug: false,
            intr: InterruptHandler::init(),

            cart: cart::load_rom_file(path),
            wram0: MemoryUnit::init(),
            wramx: MemoryUnit::init(),
            _unused: MemoryUnit::init(),
            hram: MemoryUnit::init(),

            ppu: Ppu::init(),

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
        self.advance_cycles(4);
        self.read(addr)
    }

    #[inline(always)]
    pub fn cycle_dpc(&mut self, offset: i8) -> u8 {
        self.cycle_read(u16::wrapping_add(self.cpu.pc, offset as u16))
    }

    #[inline(always)]
    pub fn cycle_write(&mut self, addr: u16, val: u8) {
        self.advance_cycles(4);
        self.write(addr, val);
    }
}
