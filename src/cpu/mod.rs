use crate::gameboy::GameBoy;
use instructions::OPCODES;

pub mod instructions;

pub const Z_FLAG: u8 = 0b10000000;
pub const N_FLAG: u8 = 0b01000000;
pub const H_FLAG: u8 = 0b00100000;
pub const C_FLAG: u8 = 0b00010000;

pub struct Cpu {
    pub a: u8, // Accumulator
    pub f: u8, // Flags
    pub b: u8, // BC: u16
    pub c: u8,
    pub d: u8, // DE: u16
    pub e: u8,
    pub h: u8, // HL: u16
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Cpu {
    #[inline(always)]
    pub fn rd_af(&self) -> u16 {
        ((self.a as u16) << 8) + self.f as u16
    }

    #[inline(always)]
    pub fn rd_bc(&self) -> u16 {
        ((self.b as u16) << 8) + self.c as u16
    }

    #[inline(always)]
    pub fn rd_de(&self) -> u16 {
        ((self.d as u16) << 8) + self.e as u16
    }

    #[inline(always)]
    pub fn rd_hl(&self) -> u16 {
        ((self.h as u16) << 8) + self.l as u16
    }

    #[inline(always)]
    pub fn wr_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0x00FF) as u8;
    }

    #[inline(always)]
    pub fn wr_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0x00FF) as u8;
    }

    #[inline(always)]
    pub fn wr_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0x00FF) as u8;
    }

    #[inline(always)]
    pub fn z_flag(&mut self) -> bool {
        self.f & Z_FLAG != 0
    }

    #[inline(always)]
    pub fn n_flag(&mut self) -> bool {
        self.f & N_FLAG != 0
    }

    #[inline(always)]
    pub fn h_flag(&mut self) -> bool {
        self.f & H_FLAG != 0
    }

    #[inline(always)]
    pub fn c_flag(&mut self) -> bool {
        self.f & C_FLAG != 0
    }

    #[inline(always)]
    pub fn nz_flag(&mut self) -> bool {
        self.f & Z_FLAG == 0
    }

    #[inline(always)]
    pub fn nc_flag(&mut self) -> bool {
        self.f & C_FLAG == 0
    }
}

impl GameBoy {
    pub fn fetch_exec(&mut self) {
        let current_ime = self.ime;
        if self.enabling_int {
            self.ime = true;
            self.enabling_int = false;
        }

        if current_ime {
            match self.fetch_interrupt() {
                Some(intr) => {
                    // WARN: timing here has a lot of intricacies, which are ignored for now

                    // store current pc addr in stack
                    let addr = u16::to_le_bytes(self.cpu.pc);
                    self.cpu.sp.dec();
                    self.write(self.cpu.sp, addr[1]);
                    self.cpu.sp.dec();
                    self.write(self.cpu.sp, addr[0]);

                    // jump to intr handler addr
                    self.cpu.pc = (intr * 8) as u16 + 0x40;

                    self.reset_if(intr);
                    self.ime = false;
                }
                None => {}
            }
        }

        let opcode = self.read_instr(0);
        self.cpu.pc.inc();

        let handler = OPCODES[opcode as usize];
        handler(self);
    }
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut z = "-";
        let mut n = "-";
        let mut h = "-";
        let mut c = "-";
        if self.f & Z_FLAG != 0 {
            z = "Z";
        }
        if self.f & N_FLAG != 0 {
            n = "N";
        }
        if self.f & H_FLAG != 0 {
            h = "H";
        }
        if self.f & C_FLAG != 0 {
            c = "C";
        }
        write!(
            f,
            "Cpu: (AF: ${:04X} BC: ${:04X} DE: ${:04X} HL: ${:04X} | PC: ${:04X} SP: ${:04X}) | Flags: ({}{}{}{})",
            ((self.a as u16) << 8) + self.f as u16,
            self.rd_bc(),
            self.rd_de(),
            self.rd_hl(),
            self.pc,
            self.sp,
            z,
            n,
            h,
            c,
        )
    }
}

trait Reg {
    fn inc(&mut self);
    fn dec(&mut self);
}

impl Reg for u8 {
    #[inline(always)]
    fn inc(&mut self) {
        *self = u8::wrapping_add(*self, 1);
    }

    #[inline(always)]
    fn dec(&mut self) {
        *self = u8::wrapping_sub(*self, 1);
    }
}

impl Reg for u16 {
    #[inline(always)]
    fn inc(&mut self) {
        *self = u16::wrapping_add(*self, 1);
    }

    #[inline(always)]
    fn dec(&mut self) {
        *self = u16::wrapping_sub(*self, 1);
    }
}
