#![allow(dead_code)]
#![allow(unused_variables)]

use crate::gameboy::{GameBoy, Opcode};

const Z_FLAG: u8 = 0b10000000;
const N_FLAG: u8 = 0b01000000;
const H_FLAG: u8 = 0b00100000;
const C_FLAG: u8 = 0b00010000;

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
    pub fn rd_bc(&self) -> u16 {
        (self.b as u16) << 8 + self.c
    }

    #[inline(always)]
    pub fn rd_de(&self) -> u16 {
        (self.d as u16) << 8 + self.e
    }

    #[inline(always)]
    pub fn rd_hl(&self) -> u16 {
        (self.h as u16) << 8 + self.l
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
        let opcode = self.mem[self.cpu.pc as usize];
        let handler = OPCODES[opcode as usize];
        handler(self, opcode);
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

pub fn cb_prefix(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    let opcode_cb = gb.mem[gb.cpu.pc as usize];
    let handler = OPCODES_CB[opcode_cb as usize];
    handler(gb, opcode_cb);
}

// 8-bit Arithmetic and Logic

macro_rules! adc {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_add(gb.cpu.a, val);
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, carry as u8);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (val & 0x0F) + carry as u8 > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a || (gb.cpu.a == old_a && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_add(gb.cpu.a, gb.cpu.$r8);
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, carry as u8);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (gb.cpu.$r8 & 0x0F) + carry as u8 > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a || (gb.cpu.a == old_a && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_add(gb.cpu.a, gb.mem[gb.cpu.rd_hl() as usize]);
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, carry as u8);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (gb.mem[gb.cpu.rd_hl() as usize] & 0x0F) + carry as u8 > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a || (gb.cpu.a == old_a && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! add {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            let old_a: u8 = gb.cpu.a;
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, val);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (val & 0x0F) > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, gb.cpu.$r8);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (gb.cpu.$r8 & 0x0F) > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            gb.cpu.a = u8::wrapping_add(gb.cpu.a, gb.mem[gb.cpu.rd_hl() as usize]);

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) + (gb.mem[gb.cpu.rd_hl() as usize] & 0x0F) > 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < old_a {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! and {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            gb.cpu.a &= val;

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.a &= gb.mem[gb.cpu.rd_hl() as usize];

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            gb.cpu.a &= val;

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! cp {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == val {
                gb.cpu.f |= Z_FLAG;
            }
            if gb.cpu.a & 0x0F < val & 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < val {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = N_FLAG;
            if gb.cpu.a == gb.cpu.$r8 {
                gb.cpu.f |= Z_FLAG;
            }
            if gb.cpu.a & 0x0F < gb.cpu.$r8 & 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < gb.cpu.$r8 {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = N_FLAG;
            if gb.cpu.a == gb.mem[addr] {
                gb.cpu.f |= Z_FLAG;
            }
            if gb.cpu.a & 0x0F < gb.mem[addr] & 0x0F {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < gb.mem[addr] {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! dec {
    ($r8: ident) => {
         |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                gb.cpu.$r8 = u8::wrapping_sub(gb.cpu.$r8, 1);

                gb.cpu.f &= !(Z_FLAG | H_FLAG) | N_FLAG;
                if (gb.cpu.$r8 == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if (u8::wrapping_sub(gb.cpu.$r8 & 0x0F, 1) > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };

    (d $r16: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let addr = gb.cpu.rd_hl() as usize;
                let val = u8::wrapping_sub(gb.mem[addr], 1);
                gb.mem[addr] = val;

                gb.cpu.f &= !(Z_FLAG | H_FLAG) | N_FLAG;
                if (val == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if (u8::wrapping_sub(val & 0x0F, 1) > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };
}

macro_rules! inc {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                gb.cpu.$r8 = u8::wrapping_add(gb.cpu.$r8, 1);

                gb.cpu.f &= !(Z_FLAG | N_FLAG | H_FLAG);
                if (gb.cpu.$r8 == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if ((gb.cpu.$r8 & 0x0F) + 1 > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };

    (d $r16: ident) => {
         |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let addr = gb.cpu.rd_hl() as usize;
                let val = u8::wrapping_add(gb.mem[addr], 1);
                gb.mem[addr] = val;

                gb.cpu.f &= !(Z_FLAG | N_FLAG | H_FLAG);
                if (val == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if ((val & 0x0F) + 1 > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };
}

macro_rules! or {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            gb.cpu.a |= val;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.a |= gb.cpu.$r8;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.a |= gb.mem[gb.cpu.rd_hl() as usize];

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! sbc {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < u8::wrapping_add(val, carry as u8) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.cpu.$r8);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.cpu.$r8 & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < u8::wrapping_add(gb.cpu.$r8, carry as u8) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            let carry: bool = gb.cpu.c_flag();
            let addr: usize = gb.cpu.rd_hl() as usize;

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.mem[addr]);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.mem[addr] & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < u8::wrapping_add(gb.mem[addr], carry as u8) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! sub {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            let old_a: u8 = gb.cpu.a;
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < val {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.cpu.$r8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.cpu.$r8 & 0x0F) as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < gb.cpu.$r8 {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_a: u8 = gb.cpu.a;
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.mem[addr]);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.mem[addr] & 0x0F) {
                gb.cpu.f |= H_FLAG;
            }
            if gb.cpu.a < gb.mem[addr] {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! xor {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            let val: u8 = gb.mem[gb.cpu.pc as usize];

            gb.cpu.a ^= val;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.a ^= gb.cpu.$r8;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.a ^= gb.mem[gb.cpu.rd_hl() as usize];

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

// 16-bit Arithmetic Instructions

macro_rules! add16 {
    (sp) => {
        |gb: &mut GameBoy, _: Opcode| {
            let old_hl = gb.cpu.rd_hl();
            let val = u16::wrapping_add(old_hl, gb.cpu.sp);
            gb.cpu.wr_hl(val);

            gb.cpu.f = !(N_FLAG | H_FLAG | C_FLAG);
            if (old_hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF {
                gb.cpu.f |= H_FLAG;
            }
            if val < old_hl {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r16: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let old_hl = gb.cpu.rd_hl();
                let val = u16::wrapping_add(old_hl, gb.cpu.[<rd_ $r16>]());
                gb.cpu.wr_hl(val);

                gb.cpu.f = !(N_FLAG | H_FLAG | C_FLAG);
                if (old_hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF {
                    gb.cpu.f |= H_FLAG;
                }
                if val < old_hl {
                    gb.cpu.f |= C_FLAG;
                }
            }
        }
    }
}

macro_rules! dec16 {
    ($r16: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let val = u16::wrapping_sub(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    }
}

macro_rules! inc16 {
    ($r16: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let val = u16::wrapping_add(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    }
}

// Bit Operations Instructions
macro_rules! bit {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f &= !(N_FLAG | Z_FLAG);
            gb.cpu.f |= H_FLAG;
            if gb.cpu.$r8 & (1 << $i) == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f &= !(N_FLAG | Z_FLAG);
            gb.cpu.f |= H_FLAG;
            if gb.mem[gb.cpu.rd_hl() as usize] & (1 << $i) == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! res {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.$r8 &= !(1 << $i);
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.mem[gb.cpu.rd_hl() as usize] &= !(1 << $i);
        }
    };
}

macro_rules! set {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.$r8 |= (1 << $i);
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.mem[gb.cpu.rd_hl() as usize] |= (1 << $i);
        }
    };
}

macro_rules! swap {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.$r8 = (gb.cpu.$r8 >> 4) | (gb.cpu.$r8 << 4);
            gb.cpu.f = 0;
            if gb.cpu.f == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.mem[addr] = (gb.mem[addr] >> 4) | (gb.mem[addr] << 4);
            gb.cpu.f = 0;
            if gb.cpu.f == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

// Bit Shift Instructions

macro_rules! rl {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let carry: u8 = gb.cpu.c_flag() as u8;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x80) >> 3;
            gb.cpu.$r8 = gb.cpu.$r8 << 1;
            gb.cpu.$r8 |= carry;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let carry: u8 = gb.cpu.c_flag() as u8;
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x80) >> 3;
            gb.mem[addr] = gb.mem[addr] << 1;
            gb.mem[addr] |= carry;
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

pub fn rla(gb: &mut GameBoy, opcode: Opcode) {
    let carry: u8 = gb.cpu.c_flag() as u8;
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x80) >> 3;
    gb.cpu.a = gb.cpu.a << 1;
    gb.cpu.a |= carry;
}

macro_rules! rlc {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x80) >> 3;
            gb.cpu.$r8 = u8::rotate_left(gb.cpu.$r8, 1);
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x80) >> 3;
            gb.mem[addr] = u8::rotate_left(gb.mem[addr], 1);
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

pub fn rlca(gb: &mut GameBoy, opcode: Opcode) {
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x80) >> 3;
    gb.cpu.a = u8::rotate_left(gb.cpu.a, 1);
}

macro_rules! rr {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            let carry: u8 = gb.cpu.c_flag() as u8;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 5;
            gb.cpu.$r8 = gb.cpu.$r8 >> 1;
            gb.cpu.$r8 |= carry << 7;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let carry: u8 = gb.cpu.c_flag() as u8;
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x01) << 5;
            gb.mem[addr] = gb.mem[addr] >> 1;
            gb.mem[addr] |= carry << 7;
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

pub fn rra(gb: &mut GameBoy, opcode: Opcode) {
    let carry: u8 = gb.cpu.c_flag() as u8;
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x01) << 5;
    gb.cpu.a = gb.cpu.a >> 1;
    gb.cpu.a |= carry << 7;
}

macro_rules! rrc {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 5;
            gb.cpu.$r8 = u8::rotate_right(gb.cpu.$r8, 1);
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x01) << 5;
            gb.mem[addr] = u8::rotate_right(gb.mem[addr], 1);
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

pub fn rrca(gb: &mut GameBoy, opcode: Opcode) {
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x01) << 5;
    gb.cpu.a = u8::rotate_right(gb.cpu.a, 1);

}

macro_rules! sla {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x80) >> 3;
            gb.cpu.$r8 = gb.cpu.$r8 << 1;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x80) >> 3;
            gb.mem[addr] = gb.mem[addr] << 1;
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! sra {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 5;
            gb.cpu.$r8 = (gb.cpu.$r8 as i8 >> 1) as u8;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x01) << 5;
            gb.mem[addr] = (gb.mem[addr] as i8 >> 1) as u8;
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

}

macro_rules! srl {
    ($r8: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 5;
            gb.cpu.$r8 >>= 1;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            let addr: usize = gb.cpu.rd_hl() as usize;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.mem[addr] & 0x01) << 5;
            gb.mem[addr] >>= 1;
            if gb.mem[addr] == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

// Load Instructions

macro_rules! ld {
    (d $targ: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                gb.cpu.pc += 1;
                let r16 = gb.cpu.[<rd_ $targ>]();
                gb.mem[r16 as usize] = gb.mem[gb.cpu.pc as usize];
            }
        }
    };

    ($targ: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc += 1;
            gb.cpu.$targ = gb.mem[gb.cpu.pc as usize];
        }
    };

    ($targ: ident, d $orig: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let r16 = gb.cpu.[<rd_ $orig>]();
                gb.cpu.$targ = gb.mem[r16 as usize];
            }
        }
    };

    (d $targ: ident, $orig: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let r16 = gb.cpu.[<rd_ $targ>]();
                gb.mem[r16 as usize] = gb.cpu.$orig;
            }
        }
    };

    ($targ: ident, $orig: ident) => {
        |gb: &mut GameBoy, _: Opcode| gb.cpu.$targ = gb.cpu.$orig
    };
}

macro_rules! ld16 {
    (sp) => {
        |gb: &mut GameBoy, _: Opcode| {
            let lsb = gb.mem[(gb.cpu.pc as usize) + 1] as u16;
            let msb = gb.mem[(gb.cpu.pc as usize) + 2] as u16;
            gb.cpu.sp = msb << 8 + lsb;
            gb.cpu.pc += 2;
        }
    };

    ($targ: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let lsb = gb.mem[(gb.cpu.pc as usize) + 1] as u16;
                let msb = gb.mem[(gb.cpu.pc as usize) + 2] as u16;
                gb.cpu.[<wr_ $targ>](msb << 8 + lsb);
                gb.cpu.pc += 2;
            }
        }
    };
}

pub fn ld_n16_a(gb: &mut GameBoy, _: Opcode) {
    let addr = {
        let lsb = gb.mem[(gb.cpu.pc as usize) + 1] as usize;
        let msb = gb.mem[(gb.cpu.pc as usize) + 2] as usize;
        msb << 8 + lsb
    };
    gb.mem[addr] = gb.cpu.a;
    gb.cpu.pc += 2;
}


pub fn ldh_n8_a(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    let addr = 0xFF00 + (gb.mem[gb.cpu.pc as usize] as usize);
    gb.mem[addr] = gb.cpu.a;
}

pub fn ldh_c_a(gb: &mut GameBoy, _: Opcode) {
    let addr = 0xFF00 + (gb.cpu.c as usize) as usize;
    gb.mem[addr] = gb.cpu.a;
}

pub fn ld_a_n16(gb: &mut GameBoy, _: Opcode) {
    let addr = {
        let lsb = gb.mem[(gb.cpu.pc as usize) + 1] as usize;
        let msb = gb.mem[(gb.cpu.pc as usize) + 2] as usize;
        msb << 8 + lsb
    };
    gb.mem[addr] = gb.cpu.a;
    gb.cpu.pc += 2;
}

pub fn ldh_a_n8(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    let addr = 0xFF00 + (gb.mem[gb.cpu.pc as usize] as usize);
    gb.cpu.a = gb.mem[addr];
}

pub fn ldh_a_c(gb: &mut GameBoy, _: Opcode) {
    let addr = 0xFF00 + (gb.cpu.c as usize) as usize;
    gb.cpu.a = gb.mem[addr];
}

fn ld_hli_a(gb: &mut GameBoy, _: Opcode) {
    let hl = gb.cpu.rd_hl() as usize;
    gb.cpu.a = gb.mem[hl];
    gb.mem[hl] += 1;
}

fn ld_hld_a(gb: &mut GameBoy, _: Opcode) {
    let hl = gb.cpu.rd_hl() as usize;
    gb.cpu.a = gb.mem[hl];
    gb.mem[hl] -= 1;
}

fn ld_a_hli(gb: &mut GameBoy, _: Opcode) {
    let hl = gb.cpu.rd_hl() as usize;
    gb.mem[hl] = gb.cpu.a;
    gb.mem[hl] += 1;
}

fn ld_a_hld(gb: &mut GameBoy, _: Opcode) {
    let hl = gb.cpu.rd_hl() as usize;
    gb.mem[hl] = gb.cpu.a;
    gb.mem[hl] -= 1;
}

// Jumps and Subroutines

#[inline(always)]
fn _call(gb: &mut GameBoy) {
    let addr = u16::to_le_bytes(gb.cpu.pc.wrapping_add(2));

    gb.cpu.sp.dec();
    gb.mem[gb.cpu.sp as usize] = addr[1];
    gb.cpu.sp.dec();
    gb.mem[gb.cpu.sp as usize] = addr[0];

    _jp(gb);
}

macro_rules! call {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            _call(gb);
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                if gb.cpu.[<$cc _flag>]() {
                    _call(gb);
                    return;
                }
                gb.cpu.pc = gb.cpu.pc.wrapping_add(2);
            }
        }
    };
}

#[inline(always)]
fn _jp(gb: &mut GameBoy) {
    gb.cpu.sp = {
        let lsb = gb.mem[gb.cpu.pc as usize] as u16;
        let msb = gb.mem[gb.cpu.pc.wrapping_add(1) as usize] as u16;
        msb << 8 + lsb
    };
}

macro_rules! jp {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            _jp(gb);
        }
    };

    (hl) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.pc = gb.cpu.rd_hl();
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                if gb.cpu.[<$cc _flag>]() {
                    _jp(gb);
                    return;
                }
                gb.cpu.pc = gb.cpu.pc.wrapping_add(2);
            }
        }
    };
}

#[inline(always)]
fn _jr(gb: &mut GameBoy) {
    let addr = gb.mem[gb.cpu.pc as usize] as u16;
    gb.cpu.pc = gb.cpu.pc.wrapping_add(addr);
}

macro_rules! jr {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            _jr(gb);
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                if gb.cpu.[<$cc _flag>]() {
                    _jr(gb);
                    return;
                }
                gb.cpu.pc = gb.cpu.pc.wrapping_add(2);
            }
        }
    };
}

#[inline(always)]
fn _ret(gb: &mut GameBoy) {
    gb.cpu.pc = {
        let lo = gb.mem[gb.cpu.sp as usize] as u16;
        gb.cpu.sp.inc();
        let hi = gb.mem[gb.cpu.sp as usize] as u16;
        gb.cpu.sp.inc();
        hi << 8 + lo
    }
}

macro_rules! ret {
    () => {
        |gb: &mut GameBoy, _: Opcode| {
            _ret(gb);
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                if gb.cpu.[<$cc _flag>]() {
                    _ret(gb);
                    return;
                }
            }
        }
    };
}

fn reti(gb: &mut GameBoy, _: Opcode) {
    _ret(gb);
    gb.ime = true;
}

macro_rules! rst {
    ($hx: expr) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.cpu.sp = $hx;
        }
    }
}

// Stack Operations

fn add_sp_e8(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    let offset = gb.mem[gb.cpu.pc as usize] as i8;
    let old_sp = gb.cpu.sp;
    gb.cpu.sp = i32::wrapping_add(old_sp as i32, offset as i32) as u16;

    gb.cpu.f = 0;
    if (old_sp & 0x000F) + (offset & 0x000F) as u16 > 0x000F {
        gb.cpu.f |= H_FLAG;
    }
    if (old_sp & 0x00FF) + offset as u16 > 0x00FF {
        gb.cpu.f |= C_FLAG;
    }
}

fn dec_sp(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.sp = u16::wrapping_sub(gb.cpu.sp, 1);
}

fn inc_sp(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.sp = u16::wrapping_add(gb.cpu.sp, 1);
}

fn ld_n16_sp(gb: &mut GameBoy, _: Opcode) {
    let addr = {
        let lsb = gb.mem[(gb.cpu.pc as usize) + 1] as usize;
        let msb = gb.mem[(gb.cpu.pc as usize) + 2] as usize;
        msb << 8 + lsb
    };
    let bytes = u16::to_le_bytes(gb.cpu.sp);
    gb.mem[addr] = bytes[0];
    gb.mem[addr + 1] = bytes[1];
    gb.cpu.pc += 2;
}

fn ld_hl_sp_e8(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    let offset = gb.mem[gb.cpu.pc as usize] as i8;
    let val = i32::wrapping_add(gb.cpu.sp as i32, offset as i32) as u16;
    gb.cpu.wr_hl(val);

    gb.cpu.f = 0;
    if (gb.cpu.sp & 0x000F) + (offset & 0x0F) as u16 > 0x000F {
        gb.cpu.f |= H_FLAG;
    }
    if val < gb.cpu.sp {
        gb.cpu.f |= C_FLAG;
    }
}

fn ld_sp_hl(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.sp = gb.cpu.rd_hl();
}

fn pop_af(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.f = gb.mem[gb.cpu.sp as usize];
    gb.cpu.a = gb.mem[(gb.cpu.sp + 1) as usize];
    gb.cpu.sp = u16::wrapping_add(gb.cpu.sp, 2);
}

macro_rules! pop {
    ($r16: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            paste::paste! {
                let lsb = gb.mem[gb.cpu.sp as usize] as u16;
                let msb = gb.mem[(gb.cpu.sp + 1) as usize] as u16;
                gb.cpu.[<wr_ $r16>](msb << 8 + lsb);
                gb.cpu.sp = u16::wrapping_add(gb.cpu.sp, 2);
            }
        }
    }
}

fn push_af(gb: &mut GameBoy, _: Opcode) {
    gb.mem[(gb.cpu.sp - 1) as usize] = gb.cpu.a;
    gb.mem[(gb.cpu.sp - 2) as usize] = gb.cpu.f;
    gb.cpu.sp = u16::wrapping_sub(gb.cpu.sp, 2);
}

macro_rules! push {
    ($hi: ident, $lo: ident) => {
        |gb: &mut GameBoy, _: Opcode| {
            gb.mem[(gb.cpu.sp - 1) as usize] = gb.cpu.$hi;
            gb.mem[(gb.cpu.sp - 2) as usize] = gb.cpu.$lo;
            gb.cpu.sp = u16::wrapping_sub(gb.cpu.sp, 2);
        }
    }
}

// Miscellaneous Instructions
pub fn ccf(gb: &mut GameBoy, opcode: Opcode) {
    gb.cpu.f &= !(N_FLAG | H_FLAG);
    gb.cpu.f ^= C_FLAG;
}

pub fn cpl(gb: &mut GameBoy, opcode: Opcode) {
    gb.cpu.a = !gb.cpu.a;
    gb.cpu.f |= N_FLAG | H_FLAG;
}

pub fn daa(gb: &mut GameBoy, opcode: Opcode) {
    let mut res: i16 = (gb.cpu.a as i16) & 0xFF;
    if gb.cpu.n_flag() {
        if gb.cpu.h_flag() {
            res = (res - 0x06) & 0xFF;
        }
        if gb.cpu.c_flag() {
            res -= 0x60;
        }
    }
    else {
        if gb.cpu.h_flag() || res & 0x0F > 0x09 {
            res += 0x06;
        }
        if gb.cpu.c_flag() || res > 0x9F {
            res += 0x60;
        }
    }

    gb.cpu.a = res as u8;

    gb.cpu.f &= !H_FLAG;
    if res & 0x100 == 0x100 {
        gb.cpu.f |= C_FLAG;
    }
    if gb.cpu.a == 0 {
        gb.cpu.f |= Z_FLAG;
    }
}

pub fn di(gb: &mut GameBoy, opcode: Opcode) {
    gb.ime = false;
}

pub fn ei(gb: &mut GameBoy, opcode: Opcode) {}

pub fn halt(gb: &mut GameBoy, opcode: Opcode) {}

pub fn nop(gb: &mut GameBoy, opcode: Opcode) {}

pub fn scf(gb: &mut GameBoy, opcode: Opcode) {
    gb.cpu.f &= !(N_FLAG | H_FLAG);
    gb.cpu.f |= C_FLAG;
}

pub fn stop(gb: &mut GameBoy, opcode: Opcode) {}

pub fn undefined(gb: &mut GameBoy, opcode: Opcode) {}

#[rustfmt::skip]
pub const OPCODES: [fn(&mut GameBoy, u8); 256] = [
/*            X0            X1            X2            X3            X4            X5            X6            X7            */
/*            X8            X9            XA            XB            XC            XD            XE            XF            */
/* 0X */      nop,          ld16!(bc),    ld!(d bc, a), inc16!(bc),   inc!(b),      dec!(b),      ld!(b),       rlca,
              ld_n16_sp,    add16!(bc),   ld!(a, d bc), dec16!(bc),   inc!(c),      dec!(c),      ld!(c),       rrca,
/* 1X */      stop,         ld16!(de),    ld!(d de, a), inc16!(de),   inc!(d),      dec!(d),      ld!(d),       rla,
              jr!(),        add16!(de),   ld!(a, d de), dec16!(de),   inc!(e),      dec!(e),      ld!(e),       rra,
/* 2X */      jr!(nz),      ld16!(hl),    ld_hli_a,     inc16!(hl),   inc!(h),      dec!(h),      ld!(h),       daa,
              jr!(z),       add16!(hl),   ld_a_hli,     dec16!(hl),   inc!(l),      dec!(l),      ld!(l),       cpl,
/* 3X */      jr!(nc),      ld16!(sp),    ld_hld_a,     inc_sp,       inc!(d hl),   dec!(d hl),   ld!(d hl),    scf,
              jr!(c),       add16!(sp),   ld_a_hld,     dec_sp,       inc!(a),      dec!(a),      ld!(a),       ccf,
/* 4X */      nop,          ld!(b, c),    ld!(b, d),    ld!(b, e),    ld!(b, h),    ld!(b, l),    ld!(b, d hl), ld!(b, a),
              ld!(c, b),    nop,          ld!(c, d),    ld!(c, e),    ld!(b, h),    ld!(b, l),    ld!(b, d hl), ld!(b, a),
/* 5X */      ld!(d, b),    ld!(d, c),    nop,          ld!(d, e),    ld!(d, h),    ld!(d, l),    ld!(d, d hl), ld!(d, a),
              ld!(e, b),    ld!(e, c),    ld!(e, d),    nop,          ld!(e, h),    ld!(e, l),    ld!(e, d hl), ld!(e, a),
/* 6X */      ld!(h, b),    ld!(h, c),    ld!(h, d),    ld!(h, e),    nop,          ld!(h, l),    ld!(h, d hl), ld!(h, a),
              ld!(l, b),    ld!(l, c),    ld!(l, d),    ld!(l, e),    ld!(l, h),    nop,          ld!(h, d hl), ld!(h, a),
/* 7X */      ld!(d hl, b), ld!(d hl, c), ld!(d hl, d), ld!(d hl, e), ld!(d hl, h), ld!(d hl, l), halt,         ld!(d hl, a),
              ld!(a, b),    ld!(a, c),    ld!(a, d),    ld!(a, e),    ld!(a, h),    ld!(a, h),    ld!(a, d hl), nop,
/* 8X */      add!(b),      add!(c),      add!(d),      add!(e),      add!(h),      add!(l),      add!(d hl),   add!(a),
              adc!(b),      adc!(c),      adc!(d),      adc!(e),      adc!(h),      adc!(l),      adc!(d hl),   adc!(a),
/* 9X */      sub!(b),      sub!(c),      sub!(d),      sub!(e),      sub!(h),      sub!(l),      sub!(d hl),   sub!(a),
              sbc!(b),      sbc!(c),      sbc!(d),      sbc!(e),      sbc!(h),      sbc!(l),      sbc!(d hl),   sbc!(a),
/* AX */      and!(b),      and!(c),      and!(d),      and!(e),      and!(h),      and!(l),      and!(d hl),   and!(a),
              xor!(b),      xor!(c),      xor!(d),      xor!(e),      xor!(h),      xor!(l),      xor!(d hl),   xor!(a),
/* BX */      or!(b),       or!(c),       or!(d),       or!(e),       or!(h),       or!(l),       or!(d hl),    or!(a),
              cp!(b),       cp!(c),       cp!(d),       cp!(e),       cp!(h),       cp!(l),       cp!(d hl),    cp!(a),
/* CX */      ret!(nz),     pop!(bc),     jp!(nz),      jp!(),        call!(nz) ,   push!(b, c),  add!(),       rst!(0x00),
              ret!(z),      ret!(),       jp!(z),       cb_prefix,    call!(z),     call!(),      adc!(),       rst!(0x08),
/* DX */      ret!(nc),     pop!(de),     jp!(nc),      undefined,    call!(nc) ,   push!(d, e),  sub!(),       rst!(0x10),
              ret!(c),      reti,         jp!(c),       undefined,    call!(c),     undefined,    sbc!(),       rst!(0x18),
/* EX */      ldh_n8_a,     pop!(hl),     ldh_c_a,      undefined,    undefined,    push!(h, l),  and!(),       rst!(0x20),
              add_sp_e8,    jp!(hl),      ld_n16_a,     undefined,    undefined,    undefined,    xor!(),       rst!(0x28),
/* fX */      ldh_a_n8,     pop_af,       ldh_a_c,      di,           undefined,    push_af,      or!(),        rst!(0x30),
              ld_hl_sp_e8,  ld_sp_hl,     ld_a_n16,     ei,           undefined,    undefined,    cp!(),        rst!(0x38),
];

#[rustfmt::skip]
pub const OPCODES_CB: [fn(&mut GameBoy, u8); 256] = [
/*           X0           X1           X2           X3           X4           X5           X6              X7           */
/*           X8           X9           XA           XB           XC           XD           XE              XF           */
/* 0X */     rlc!(b),     rlc!(c),     rlc!(d),     rlc!(e),     rlc!(h),     rlc!(l),     rlc!(d hl),     rlc!(a),
             rrc!(b),     rrc!(c),     rrc!(d),     rrc!(e),     rrc!(h),     rrc!(l),     rrc!(d hl),     rrc!(a),
/* 1X */     rl!(b),      rl!(c),      rl!(d),      rl!(e),      rl!(h),      rl!(l),      rl!(d hl),      rl!(a),
             rr!(b),      rr!(c),      rr!(d),      rr!(e),      rr!(h),      rr!(l),      rr!(d hl),      rr!(a),
/* 2X */     sla!(b),     sla!(c),     sla!(d),     sla!(e),     sla!(h),     sla!(l),     sla!(d hl),     sla!(a),
             sra!(b),     sra!(c),     sra!(d),     sra!(e),     sra!(h),     sra!(l),     sra!(d hl),     sra!(a),
/* 3X */     swap!(b),    swap!(c),    swap!(d),    swap!(e),    swap!(h),    swap!(l),    swap!(d hl),    swap!(a),
             srl!(b),     srl!(c),     srl!(d),     srl!(e),     srl!(h),     srl!(l),     srl!(d hl),     srl!(a),
/* 4X */     bit!(0, b),  bit!(0, c),  bit!(0, d),  bit!(0, e),  bit!(0, h),  bit!(0, l),  bit!(0, d hl),  bit!(0, a),
             bit!(1, b),  bit!(1, c),  bit!(1, d),  bit!(1, e),  bit!(1, h),  bit!(1, l),  bit!(1, d hl),  bit!(1, a),
/* 5X */     bit!(2, b),  bit!(2, c),  bit!(2, d),  bit!(2, e),  bit!(2, h),  bit!(2, l),  bit!(2, d hl),  bit!(2, a),
             bit!(3, b),  bit!(3, c),  bit!(3, d),  bit!(3, e),  bit!(3, h),  bit!(3, l),  bit!(3, d hl),  bit!(3, a),
/* 6X */     bit!(4, b),  bit!(4, c),  bit!(4, d),  bit!(4, e),  bit!(4, h),  bit!(4, l),  bit!(4, d hl),  bit!(4, a),
             bit!(5, b),  bit!(5, c),  bit!(5, d),  bit!(5, e),  bit!(5, h),  bit!(5, l),  bit!(5, d hl),  bit!(5, a),
/* 7X */     bit!(6, b),  bit!(6, c),  bit!(6, d),  bit!(6, e),  bit!(6, h),  bit!(6, l),  bit!(6, d hl),  bit!(6, a),
             bit!(7, b),  bit!(7, c),  bit!(7, d),  bit!(7, e),  bit!(7, h),  bit!(7, l),  bit!(7, d hl),  bit!(7, a),
/* 8X */     res!(0, b),  res!(0, c),  res!(0, d),  res!(0, e),  res!(0, h),  res!(0, l),  res!(0, d hl),  res!(0, a),
             res!(1, b),  res!(1, c),  res!(1, d),  res!(1, e),  res!(1, h),  res!(1, l),  res!(1, d hl),  res!(1, a),
/* 9X */     res!(2, b),  res!(2, c),  res!(2, d),  res!(2, e),  res!(2, h),  res!(2, l),  res!(2, d hl),  res!(2, a),
             res!(3, b),  res!(3, c),  res!(3, d),  res!(3, e),  res!(3, h),  res!(3, l),  res!(3, d hl),  res!(3, a),
/* AX */     res!(4, b),  res!(4, c),  res!(4, d),  res!(4, e),  res!(4, h),  res!(4, l),  res!(4, d hl),  res!(4, a),
             res!(5, b),  res!(5, c),  res!(5, d),  res!(5, e),  res!(5, h),  res!(5, l),  res!(5, d hl),  res!(5, a),
/* BX */     res!(6, b),  res!(6, c),  res!(6, d),  res!(6, e),  res!(6, h),  res!(6, l),  res!(6, d hl),  res!(6, a),
             res!(7, b),  res!(7, c),  res!(7, d),  res!(7, e),  res!(7, h),  res!(7, l),  res!(7, d hl),  res!(7, a),
/* CX */     set!(0, b),  set!(0, c),  set!(0, d),  set!(0, e),  set!(0, h),  set!(0, l),  set!(0, d hl),  set!(0, a),
             set!(1, b),  set!(1, c),  set!(1, d),  set!(1, e),  set!(1, h),  set!(1, l),  set!(1, d hl),  set!(1, a),
/* DX */     set!(2, b),  set!(2, c),  set!(2, d),  set!(2, e),  set!(2, h),  set!(2, l),  set!(2, d hl),  set!(2, a),
             set!(3, b),  set!(3, c),  set!(3, d),  set!(3, e),  set!(3, h),  set!(3, l),  set!(3, d hl),  set!(3, a),
/* EX */     set!(4, b),  set!(4, c),  set!(4, d),  set!(4, e),  set!(4, h),  set!(4, l),  set!(4, d hl),  set!(4, a),
             set!(5, b),  set!(5, c),  set!(5, d),  set!(5, e),  set!(5, h),  set!(5, l),  set!(5, d hl),  set!(5, a),
/* FX */     set!(6, b),  set!(6, c),  set!(6, d),  set!(6, e),  set!(6, h),  set!(6, l),  set!(6, d hl),  set!(6, a),
             set!(7, b),  set!(7, c),  set!(7, d),  set!(7, e),  set!(7, h),  set!(7, l),  set!(7, d hl),  set!(7, a),
];
