use crate::gameboy::{GameBoy, Opcode};

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
    pub fn rd_hl(&self) -> u16 {
        (self.h as u16) << 8 + self.l
    }

    #[inline(always)]
    pub fn wr_hl(&mut self, val: u16) {
        todo!()
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

pub fn adc_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn adc_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn adc_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn add_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn add_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn add_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn and_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn and_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn and_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn cp_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn cp_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn cp_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn dec_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn dec_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn inc_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn inc_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn or_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn or_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn or_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sbc_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sbc_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sbc_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sub_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sub_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn sub_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn xor_a_r8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn xor_a_hl(gb: &mut GameBoy, opcode: Opcode) {}

pub fn xor_a_n8(gb: &mut GameBoy, opcode: Opcode) {}

// 16-bit Arithmetic Instructions

pub fn add_hl_r16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn dec_r16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn inc_r16(gb: &mut GameBoy, opcode: Opcode) {}

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
    ($targ: ident, hl) => {
        |gb: &mut GameBoy, _: Opcode| gb.cpu.$targ = gb.mem[gb.cpu.rd_hl() as usize]
    };
    (hl, $orig: ident) => {
        |gb: &mut GameBoy, _: Opcode| gb.mem[gb.cpu.rd_hl() as usize] = gb.cpu.$orig
    };
    ($targ: ident, $orig: ident) => {
        |gb: &mut GameBoy, _: Opcode| gb.cpu.$targ = gb.cpu.$orig
    };
}

pub fn ld_r8_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_r16_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_hl_n8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_r16_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_n16_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ldh_n16_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ldh_c_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_a_r16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_a_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ldh_a_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ldh_a_c(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_hli_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_hld_a(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_a_hli(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_a_hld(gb: &mut GameBoy, opcode: Opcode) {}

// Stack Operations

pub fn add_hl_sp(gb: &mut GameBoy, opcode: Opcode) {}

pub fn add_sp_e8(gb: &mut GameBoy, opcode: Opcode) {}

pub fn dec_sp(gb: &mut GameBoy, opcode: Opcode) {}

pub fn inc_sp(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_sp_n16(gb: &mut GameBoy, opcode: Opcode) {}

pub fn ld_n16_sp(gb: &mut GameBoy, opcode: Opcode) {}

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
/*           X0           X1           X2           X3           X4           X5           X6           X7           */
/*           X8           X9           Xa           Xb           Xc           Xd           Xe           Xf           */
/* 0X */     nop,         ld_r16_n16,  ld_r16_a,    inc_r16,     inc_r8,      dec_r8,      ld_r8_n8,    rlca,
             ld_n16_sp,   add_hl_r16,  ld_a_r16,    dec_r16,     inc_r8,      dec_r8,      ld_r8_n8,    rrca,
/* 1X */     stop,        ld_r16_n16,  ld_r16_a,    inc_r16,     inc_r8,      dec_r8,      ld_r8_n8,    rla,
             jr_e8,       add_hl_r16,  ld_a_r16,    dec_r16,     inc_r8,      dec_r8,      ld_r8_n8,    rra,
/* 2X */     jr_cc_e8,    ld_r16_n16,  ld_hld_a,    inc_r16,     inc_r8,      dec_r8,      ld_r8_n8,    daa,
             jr_cc_e8,    add_hl_r16,  ld_a_hli,    dec_r16,     inc_r8,      dec_r8,      ld_r8_n8,    cpl,
/* 3X */     jr_cc_e8,    ld_sp_n16,   ld_hli_a,    inc_sp,      inc_hl,      dec_hl,      ld_hl_n8,    scf,
             jr_cc_e8,    add_hl_sp,   ld_a_hld,    dec_sp,      inc_r8,      dec_r8,      ld_r8_n8,    ccf,
/* 4X */     nop,         ld!(b, c),   ld!(b, d),   ld!(b, e),   ld!(b, h),   ld!(b, l),   ld!(b, hl),  ld!(b, a),
             ld!(c, b),   nop,         ld!(c, d),   ld!(c, e),   ld!(b, h),   ld!(b, l),   ld!(b, hl),  ld!(b, a),
/* 5X */     ld!(d, b),   ld!(d, c),   nop,         ld!(d, e),   ld!(d, h),   ld!(d, l),   ld!(d, hl),  ld!(d, a),
             ld!(e, b),   ld!(e, c),   ld!(e, d),   nop,         ld!(e, h),   ld!(e, l),   ld!(e, hl),  ld!(e, a),
/* 6X */     ld!(h, b),   ld!(h, c),   ld!(h, d),   ld!(h, e),   nop,         ld!(h, l),   ld!(h, hl),  ld!(h, a),
             ld!(l, b),   ld!(l, c),   ld!(l, d),   ld!(l, e),   ld!(l, h),   nop,         ld!(h, hl),  ld!(h, a),
/* 7X */     ld!(hl, b),  ld!(hl, c),  ld!(hl, d),  ld!(hl, e),  ld!(hl, h),  ld!(hl, l),  halt,        ld!(hl, a),
             ld!(a, b),   ld!(a, c),   ld!(a, d),   ld!(a, e),   ld!(a, h),   ld!(a, h),   ld!(a, hl),  nop,
/* 8X */     add_a_r8,    add_a_r8,    add_a_r8,    add_a_r8,    add_a_r8,    add_a_r8,    add_a_hl,    add_a_r8,
             adc_a_r8,    adc_a_r8,    adc_a_r8,    adc_a_r8,    adc_a_r8,    adc_a_r8,    adc_a_hl,    adc_a_r8,
/* 9X */     sub_a_r8,    sub_a_r8,    sub_a_r8,    sub_a_r8,    sub_a_r8,    sub_a_r8,    sub_a_hl,    sub_a_r8,
             sbc_a_r8,    sbc_a_r8,    sbc_a_r8,    sbc_a_r8,    sbc_a_r8,    sbc_a_r8,    sbc_a_hl,    sbc_a_r8,
/* aX */     and_a_r8,    and_a_r8,    and_a_r8,    and_a_r8,    and_a_r8,    and_a_r8,    and_a_hl,    and_a_r8,
             xor_a_r8,    xor_a_r8,    xor_a_r8,    xor_a_r8,    xor_a_r8,    xor_a_r8,    xor_a_hl,    xor_a_r8,
/* bX */     or_a_r8,     or_a_r8,     or_a_r8,     or_a_r8,     or_a_r8,     or_a_r8,     or_a_hl,     or_a_r8,
             cp_a_r8,     cp_a_r8,     cp_a_r8,     cp_a_r8,     cp_a_r8,     cp_a_r8,     cp_a_hl,     cp_a_r8,
/* cX */     ret_cc,      pop_r16,     jp_cc_n16,   jp_n16,      call_cc_n16, push_r16,    add_a_n8,    rst_vec,
             ret_cc,      ret,         jp_cc_n16,   cb_prefix,   call_cc_n16, call_n16,    adc_a_n8,    rst_vec,
/* dX */     ret_cc,      pop_r16,     jp_cc_n16,   undefined,   call_cc_n16, push_r16,    sub_a_n8,    rst_vec,
             ret_cc,      reti,        jp_cc_n16,   undefined,   call_cc_n16, undefined,   sbc_a_n8,    rst_vec,
/* eX */     ldh_n16_a,   pop_r16,     ldh_c_a,     undefined,   undefined,   push_r16,    and_a_n8,    rst_vec,
             add_sp_e8,   jp_hl,       ld_n16_a,    undefined,   undefined,   undefined,   xor_a_n8,    rst_vec,
/* fX */     ldh_a_n16,   pop_af,      ldh_a_c,     di,          undefined,   push_af,     or_a_n8,     rst_vec,
             ld_hl_sp_e8, ld_sp_hl,    ld_a_n16,    ei,          undefined,   undefined,   cp_a_n8,     rst_vec,
];
