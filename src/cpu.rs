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
    pub fn set_z(&mut self) {
        self.f |= Z_FLAG
    }

    #[inline(always)]
    pub fn set_n(&mut self) {
        self.f |= N_FLAG
    }

    #[inline(always)]
    pub fn set_h(&mut self) {
        self.f |= H_FLAG
    }

    #[inline(always)]
    pub fn set_c(&mut self) {
        self.f |= C_FLAG
    }

    #[inline(always)]
    pub fn reset_z(&mut self) {
        self.f &= !Z_FLAG
    }

    #[inline(always)]
    pub fn reset_n(&mut self) {
        self.f &= !N_FLAG
    }

    #[inline(always)]
    pub fn reset_h(&mut self) {
        self.f &= !H_FLAG
    }

    #[inline(always)]
    pub fn reset_c(&mut self) {
        self.f &= !C_FLAG
    }
}

impl GameBoy {
    pub fn fetch_exec(&mut self) {
        let opcode = self.mem[self.cpu.pc as usize];
        let handler = OPCODES[opcode as usize];
        handler(self, opcode);
    }
}

pub fn cb_prefix(gb: &mut GameBoy, _: Opcode) {
    gb.cpu.pc += 1;
    todo!()
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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
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

pub fn xor_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn xor_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn xor_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
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
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
                let val = u16::wrapping_sub(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    }
}

macro_rules! inc16 {
    ($r16: ident) => {
        paste::paste! {
            |gb: &mut GameBoy, _: Opcode| {
                let val = u16::wrapping_add(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    }
}

// Bit Operations Instructions

pub fn bit_u3_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn bit_u3_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn res_u3_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn res_u3_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn set_u3_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn set_u3_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn swap_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn swap_hl(gb: &mut GameBoy, opcode: Opcode) {}

// Bit Shift Instructions

pub fn rl_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rl_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rla(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rlc_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rlc_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rlca(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rr_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rr_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rra(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rrc_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rrc_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rrca(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sla_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sla_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sra_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sra_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn srl_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn srl_hl(gb: &mut GameBoy, opcode: Opcode) {}

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

// Stack Operations

pub fn add_hl_sp(gb: &mut GameBoy, opcode: Opcode) {}

pub fn add_sp_e8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn dec_sp(gb: &mut GameBoy, opcode: Opcode) {
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
    gb.mem[addr] = bytes[1];
    gb.cpu.pc += 2;
}

pub fn ld_hl_sp_e8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_sp_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn pop_af(gb: &mut GameBoy, opcode: Opcode) {}

pub fn pop_r16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn push_af(gb: &mut GameBoy, opcode: Opcode) {}

pub fn push_r16(gb: &mut GameBoy, opcode: Opcode) {}

// Jumps and Subroutines

pub fn call_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn call_cc_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn jp_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn jp_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn jp_cc_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn jr_e8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn jr_cc_e8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ret_cc(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ret(gb: &mut GameBoy, opcode: Opcode) {}

pub fn reti(gb: &mut GameBoy, opcode: Opcode) {}

pub fn rst_vec(gb: &mut GameBoy, opcode: Opcode) {}

// Miscellaneous Instructions
pub fn ccf(gb: &mut GameBoy, opcode: Opcode) {}

pub fn cpl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn daa(gb: &mut GameBoy, opcode: Opcode) {}

pub fn di(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ei(gb: &mut GameBoy, opcode: Opcode) {}

pub fn halt(gb: &mut GameBoy, opcode: Opcode) {}

pub fn nop(gb: &mut GameBoy, opcode: Opcode) {}

pub fn scf(gb: &mut GameBoy, opcode: Opcode) {}

pub fn stop(gb: &mut GameBoy, opcode: Opcode) {}

pub fn undefined(gb: &mut GameBoy, opcode: Opcode) {}

#[rustfmt::skip]
pub const OPCODES: [fn(&mut GameBoy, u8); 256] = [
/*            X0            X1            X2            X3            X4            X5            X6            X7            */
/*            X8            X9            Xa            Xb            Xc            Xd            Xe            Xf            */
/* 0X */      nop,          ld16!(bc),    ld!(d bc, a), inc16!(bc),   inc!(b),      dec!(b),      ld!(b),       rlca,
              ld_n16_sp,    add16!(bc),   ld!(a, d bc), dec16!(bc),   inc!(c),      dec!(c),      ld!(c),       rrca,
/* 1X */      stop,         ld16!(de),    ld!(d de, a), inc16!(de),   inc!(d),      dec!(d),      ld!(d),       rla,
              jr_e8,        add16!(de),   ld!(a, d de), dec16!(de),   inc!(e),      dec!(e),      ld!(e),       rra,
/* 2X */      jr_cc_e8,     ld16!(hl),    ld_hli_a,     inc16!(hl),   inc!(h),      dec!(h),      ld!(h),       daa,
              jr_cc_e8,     add16!(hl),   ld_a_hli,     dec16!(hl),   inc!(l),      dec!(l),      ld!(l),       cpl,
/* 3X */      jr_cc_e8,     ld16!(sp),    ld_hld_a,     inc_sp,       inc!(d hl),   dec!(d hl),   ld!(d hl),    scf,
              jr_cc_e8,     add16!(sp),   ld_a_hld,     dec_sp,       inc!(a),      dec!(a),      ld!(a),       ccf,
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
              xor_a_r8,     xor_a_r8,     xor_a_r8,     xor_a_r8,     xor_a_r8,     xor_a_r8,     xor_a_hl,     xor_a_r8,
/* BX */      or!(b),       or!(c),       or!(d),       or!(e),       or!(h),       or!(l),       or!(d hl),    or!(a),
              cp!(b),       cp!(c),       cp!(d),       cp!(e),       cp!(h),       cp!(l),       cp!(d hl),    cp!(a),
/* CX */      ret_cc,       pop_r16,      jp_cc_n16,    jp_n16,       call_cc_n16,  push_r16,     add!(),       rst_vec,
              ret_cc,       ret,          jp_cc_n16,    cb_prefix,    call_cc_n16,  call_n16,     adc!(),       rst_vec,
/* DX */      ret_cc,       pop_r16,      jp_cc_n16,    undefined,    call_cc_n16,  push_r16,     sub!(),       rst_vec,
              ret_cc,       reti,         jp_cc_n16,    undefined,    call_cc_n16,  undefined,    sbc!(),       rst_vec,
/* EX */      ldh_n8_a,     pop_r16,      ldh_c_a,      undefined,    undefined,    push_r16,     and!(),       rst_vec,
              add_sp_e8,    jp_hl,        ld_n16_a,     undefined,    undefined,    undefined,    xor_a_n8,     rst_vec,
/* fX */      ldh_a_n8,     pop_af,       ldh_a_c,      di,           undefined,    push_af,      or!(),        rst_vec,
              ld_hl_sp_e8,  ld_sp_hl,     ld_a_n16,     ei,           undefined,    undefined,    cp!(),        rst_vec,
];

#[rustfmt::skip]
pub const OPCODES_CB: [fn(&mut GameBoy, u8); 256] = [
/*           X0           X1           X2           X3           X4           X5           X6           X7           */
/*           X8           X9           XA           XB           XC           XD           XE           XF           */
/* 0X */     rlc_r8,      rlc_r8,      rlc_r8,      rlc_r8,      rlc_r8,      rlc_r8,      rlc_hl,      rlc_r8,
             rrc_r8,      rrc_r8,      rrc_r8,      rrc_r8,      rrc_r8,      rrc_r8,      rrc_hl,      rrc_r8,
/* 1X */     rl_r8,       rl_r8,       rl_r8,       rl_r8,       rl_r8,       rl_r8,       rl_hl,       rl_r8,
             rr_r8,       rr_r8,       rr_r8,       rr_r8,       rr_r8,       rr_r8,       rr_hl,       rr_r8,
/* 2X */     sla_r8,      sla_r8,      sla_r8,      sla_r8,      sla_r8,      sla_r8,      sla_hl,      sla_r8,
             sra_r8,      sra_r8,      sra_r8,      sra_r8,      sra_r8,      sra_r8,      sra_hl,      sra_r8,
/* 3X */     swap_r8,     swap_r8,     swap_r8,     swap_r8,     swap_r8,     swap_r8,     swap_hl,     swap_r8,
             srl_r8,      srl_r8,      srl_r8,      srl_r8,      srl_r8,      srl_r8,      srl_hl,      srl_r8,
/* 4X */     bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
             bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
/* 5X */     bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
             bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
/* 6X */     bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
             bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
/* 7X */     bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
             bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_r8,   bit_u3_hl,   bit_u3_r8,
/* 8X */     res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
             res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
/* 9X */     res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
             res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
/* AX */     res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
             res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
/* BX */     res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
             res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_r8,   res_u3_hl,   res_u3_r8,
/* CX */     set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
             set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
/* DX */     set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
             set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
/* EX */     set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
             set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
/* FX */     set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
             set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_r8,   set_u3_hl,   set_u3_r8,
];
