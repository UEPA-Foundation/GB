use crate::{
    cpu::Cpu,
    intr::InterruptHandler,
    mmu::{
        cart,
        cart::Cartridge,
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

    pub cart: Box<dyn Cartridge>,
    pub wram0: WRam0,
    pub wramx: WRamX,
    pub _unused: Unused, // Currently unused, but will be needed for CGB implementation
    pub hram: HRam,

    pub ppu: Ppu,

    pub joypad: Joypad,
    pub serial: SerialLink,
    pub timer: Timer,

    pub cycles: u64,
    pub ahead_cycles: u64,
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

            cycles: 0,
            ahead_cycles: 0,
        }
    }

    pub fn step_ms(&mut self, time: u64) {
        let pending_cycles = (time << 22) / 1000 - self.ahead_cycles;
        let start_cycle = self.cycles;
        while pending_cycles >= self.cycles - start_cycle {
            self.cpu_step();
        }
        self.ahead_cycles = (self.cycles - start_cycle) - pending_cycles;
    }

    pub fn advance_cycles(&mut self, cycles: u8) {
        self.cycles += cycles as u64;
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
