use crate::cpu::{Reg, C_FLAG, H_FLAG, N_FLAG, Z_FLAG};
use crate::gameboy::GameBoy;

// CB prefix

fn cb_prefix(gb: &mut GameBoy) {
    let opcode_cb = gb.cycle_dpc(0);
    gb.cpu.pc.inc();

    let handler = OPCODES_CB[opcode_cb as usize];
    handler(gb);
}

// 8-bit Arithmetic and Logic

macro_rules! adc {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();

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
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();

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
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();
            let val = gb.cycle_read(gb.cpu.rd_hl());

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
}

macro_rules! add {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            let old_a = gb.cpu.a;
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
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
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
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let val = gb.cycle_read(gb.cpu.rd_hl());
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
}

macro_rules! and {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            gb.cpu.a &= val;

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.a &= gb.cpu.$r8;

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            gb.cpu.a &= gb.cycle_read(gb.cpu.rd_hl());

            gb.cpu.f = H_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! cp {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

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
        |gb: &mut GameBoy| {
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
        |gb: &mut GameBoy| {
            let val = gb.cycle_read(gb.cpu.rd_hl());
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
}

macro_rules! dec {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let old_r8 = gb.cpu.$r8;
                gb.cpu.$r8.dec();

                gb.cpu.f &= !(Z_FLAG | H_FLAG);
                gb.cpu.f |= N_FLAG;
                if (gb.cpu.$r8 == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if (u8::wrapping_sub(old_r8 & 0x0F, 1) > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let addr = gb.cpu.rd_hl();
                let val = gb.cycle_read(addr);
                let res = u8::wrapping_sub(val, 1);
                gb.cycle_write(addr, res);

                gb.cpu.f &= !(Z_FLAG | H_FLAG);
                gb.cpu.f |= N_FLAG;
                if (res == 0) {
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
        |gb: &mut GameBoy| {
            paste::paste! {
                let old_r8 = gb.cpu.$r8;
                gb.cpu.$r8.inc();

                gb.cpu.f &= !(Z_FLAG | N_FLAG | H_FLAG);
                if (gb.cpu.$r8 == 0) {
                    gb.cpu.f |= Z_FLAG;
                }
                if ((old_r8 & 0x0F) + 1 > 0x0F) {
                    gb.cpu.f |= H_FLAG;
                }
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let addr = gb.cpu.rd_hl();
                let val = gb.cycle_read(addr);
                let res = u8::wrapping_add(val, 1);
                gb.cycle_write(addr, res);

                gb.cpu.f &= !(Z_FLAG | N_FLAG | H_FLAG);
                if (res == 0) {
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
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            gb.cpu.a |= val;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.a |= gb.cpu.$r8;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            gb.cpu.a |= gb.cycle_read(gb.cpu.rd_hl());

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! sbc {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < u8::wrapping_add(val, (carry as u8)) || (val == 0xFF && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.cpu.$r8);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.cpu.$r8 & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < u8::wrapping_add(gb.cpu.$r8, carry as u8) || (gb.cpu.$r8 == 0xFF && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let carry = gb.cpu.c_flag();
            let val = gb.cycle_read(gb.cpu.rd_hl());

            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, carry as u8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) + carry as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < u8::wrapping_add(val, carry as u8) || (val == 0xFF && carry) {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! sub {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            let old_a = gb.cpu.a;
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < val {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, gb.cpu.$r8);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (gb.cpu.$r8 & 0x0F) as u8 {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < gb.cpu.$r8 {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let old_a = gb.cpu.a;
            let val = gb.cycle_read(gb.cpu.rd_hl());
            gb.cpu.a = u8::wrapping_sub(gb.cpu.a, val);

            gb.cpu.f = N_FLAG;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            if (old_a & 0x0F) < (val & 0x0F) {
                gb.cpu.f |= H_FLAG;
            }
            if old_a < val {
                gb.cpu.f |= C_FLAG;
            }
        }
    };
}

macro_rules! xor {
    () => {
        |gb: &mut GameBoy| {
            let val = gb.cycle_dpc(0);
            gb.cpu.pc.inc();

            gb.cpu.a ^= val;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.a ^= gb.cpu.$r8;

            gb.cpu.f = 0;
            if gb.cpu.a == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            gb.cpu.a ^= gb.cycle_read(gb.cpu.rd_hl());

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
        |gb: &mut GameBoy| {
            gb.advance_cycles(4);

            let old_hl = gb.cpu.rd_hl();
            let val = gb.cpu.sp;
            let res = u16::wrapping_add(old_hl, val);
            gb.cpu.wr_hl(res);

            gb.cpu.f &= !(N_FLAG | H_FLAG | C_FLAG);
            if (old_hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF {
                gb.cpu.f |= H_FLAG;
            }
            if res < old_hl {
                gb.cpu.f |= C_FLAG;
            }
        }
    };

    ($r16: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                gb.advance_cycles(4);

                let old_hl = gb.cpu.rd_hl();
                let val = gb.cpu.[<rd_ $r16>]();
                let res = u16::wrapping_add(old_hl, val);
                gb.cpu.wr_hl(res);

                gb.cpu.f &= !(N_FLAG | H_FLAG | C_FLAG);
                if (old_hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF {
                    gb.cpu.f |= H_FLAG;
                }
                if res < old_hl {
                    gb.cpu.f |= C_FLAG;
                }
            }
        }
    };
}

macro_rules! dec16 {
    ($r16: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                gb.advance_cycles(4);
                let val = u16::wrapping_sub(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    };
}

macro_rules! inc16 {
    ($r16: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                gb.advance_cycles(4);
                let val = u16::wrapping_add(gb.cpu.[<rd_ $r16>](), 1);
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    };
}

// Bit Operations Instructions

macro_rules! bit {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f &= !(N_FLAG | Z_FLAG);
            gb.cpu.f |= H_FLAG;
            if gb.cpu.$r8 & (1 << $i) == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy| {
            gb.cpu.f &= !(N_FLAG | Z_FLAG);
            gb.cpu.f |= H_FLAG;
            if gb.cycle_read(gb.cpu.rd_hl()) & (1 << $i) == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

macro_rules! res {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.$r8 &= !(1 << $i);
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy| {
            let hl = gb.cpu.rd_hl();
            let val = gb.cycle_read(hl);
            gb.cycle_write(hl, val & !(1 << $i));
        }
    };
}

macro_rules! set {
    ($i: expr, $r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.$r8 |= (1 << $i);
        }
    };

    ($i: expr, d hl) => {
        |gb: &mut GameBoy| {
            let hl = gb.cpu.rd_hl();
            let val = gb.cycle_read(hl);
            gb.cycle_write(hl, val | (1 << $i));
        }
    };
}

macro_rules! swap {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.$r8 = (gb.cpu.$r8 >> 4) | (gb.cpu.$r8 << 4);
            gb.cpu.f = 0;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let dhl = gb.cycle_read(addr);
            gb.cycle_write(addr, (dhl >> 4) | (dhl << 4));
            gb.cpu.f = 0;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };
}

// Bit Shift Instructions

macro_rules! rl {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            let carry = gb.cpu.c_flag() as u8;
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
        |gb: &mut GameBoy| {
            let carry = gb.cpu.c_flag() as u8;
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x80) >> 3;
            dhl = dhl << 1;
            dhl |= carry;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

fn rla(gb: &mut GameBoy) {
    let carry = gb.cpu.c_flag() as u8;
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x80) >> 3;
    gb.cpu.a = gb.cpu.a << 1;
    gb.cpu.a |= carry;
}

macro_rules! rlc {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x80) >> 3;
            gb.cpu.$r8 = u8::rotate_left(gb.cpu.$r8, 1);
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x80) >> 3;
            dhl = u8::rotate_left(dhl, 1);
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

fn rlca(gb: &mut GameBoy) {
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x80) >> 3;
    gb.cpu.a = u8::rotate_left(gb.cpu.a, 1);
}

macro_rules! rr {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            let carry = gb.cpu.c_flag() as u8;
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 4;
            gb.cpu.$r8 = gb.cpu.$r8 >> 1;
            gb.cpu.$r8 |= carry << 7;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let carry = gb.cpu.c_flag() as u8;
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x01) << 4;
            dhl = dhl >> 1;
            dhl |= carry << 7;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

fn rra(gb: &mut GameBoy) {
    let carry = gb.cpu.c_flag() as u8;
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x01) << 4;
    gb.cpu.a = gb.cpu.a >> 1;
    gb.cpu.a |= carry << 7;
}

macro_rules! rrc {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 4;
            gb.cpu.$r8 = u8::rotate_right(gb.cpu.$r8, 1);
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x01) << 4;
            dhl = u8::rotate_right(dhl, 1);
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

fn rrca(gb: &mut GameBoy) {
    gb.cpu.f = 0;
    gb.cpu.f |= (gb.cpu.a & 0x01) << 4;
    gb.cpu.a = u8::rotate_right(gb.cpu.a, 1);
}

macro_rules! sla {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x80) >> 3;
            gb.cpu.$r8 = gb.cpu.$r8 << 1;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x80) >> 3;
            dhl = dhl << 1;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

macro_rules! sra {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 4;
            gb.cpu.$r8 = (gb.cpu.$r8 as i8 >> 1) as u8;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x01) << 4;
            dhl = (dhl as i8 >> 1) as u8;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

macro_rules! srl {
    ($r8: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.f = 0;
            gb.cpu.f |= (gb.cpu.$r8 & 0x01) << 4;
            gb.cpu.$r8 >>= 1;
            if gb.cpu.$r8 == 0 {
                gb.cpu.f |= Z_FLAG;
            }
        }
    };

    (d hl) => {
        |gb: &mut GameBoy| {
            let addr = gb.cpu.rd_hl();
            let mut dhl = gb.cycle_read(addr);
            gb.cpu.f = 0;
            gb.cpu.f |= (dhl & 0x01) << 4;
            dhl >>= 1;
            if dhl == 0 {
                gb.cpu.f |= Z_FLAG;
            }
            gb.cycle_write(addr, dhl);
        }
    };
}

// Load Instructions

macro_rules! ld {
    (d $targ: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let r16 = gb.cpu.[<rd_ $targ>]();
                let val = gb.cycle_dpc(0);
                gb.cycle_write(r16, val);
                gb.cpu.pc.inc();
            }
        }
    };

    ($targ: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.$targ = gb.cycle_dpc(0);
            gb.cpu.pc.inc();
        }
    };

    ($targ: ident, d $orig: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let r16 = gb.cpu.[<rd_ $orig>]();
                gb.cpu.$targ = gb.cycle_read(r16);
            }
        }
    };

    (d $targ: ident, $orig: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let r16 = gb.cpu.[<rd_ $targ>]();
                gb.cycle_write(r16, gb.cpu.$orig);
            }
        }
    };

    ($targ: ident, $orig: ident) => {
        |gb: &mut GameBoy| {
            gb.cpu.$targ = gb.cpu.$orig;
        }
    };
}

macro_rules! ld16 {
    (sp) => {
        |gb: &mut GameBoy| {
            gb.cpu.sp = {
                let lsb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                let msb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                (msb << 8) + lsb
            }
        }
    };

    ($targ: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let val = {
                    let lsb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    let msb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    (msb << 8) + lsb
                };
                gb.cpu.[<wr_ $targ>](val);
            }
        }
    };
}

fn ld_n16_a(gb: &mut GameBoy) {
    let addr = {
        let lsb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        let msb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        (msb << 8) + lsb
    };
    gb.cycle_write(addr, gb.cpu.a);
}

fn ldh_n8_a(gb: &mut GameBoy) {
    let addr = 0xFF00 + gb.cycle_dpc(0) as u16;
    gb.cpu.pc.inc();
    gb.cycle_write(addr, gb.cpu.a);
}

fn ldh_c_a(gb: &mut GameBoy) {
    let addr = 0xFF00 + gb.cpu.c as u16;
    gb.cycle_write(addr, gb.cpu.a);
}

fn ld_a_n16(gb: &mut GameBoy) {
    let addr = {
        let lsb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        let msb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        (msb << 8) + lsb
    };
    gb.cpu.a = gb.cycle_read(addr);
}

fn ldh_a_n8(gb: &mut GameBoy) {
    let addr = 0xFF00 + gb.cycle_dpc(0) as u16;
    gb.cpu.pc.inc();
    gb.cpu.a = gb.cycle_read(addr);
}

fn ldh_a_c(gb: &mut GameBoy) {
    let addr = 0xFF00 + gb.cpu.c as u16;
    gb.cpu.a = gb.cycle_read(addr);
}

fn ld_hli_a(gb: &mut GameBoy) {
    let hl = gb.cpu.rd_hl();
    gb.cycle_write(hl, gb.cpu.a);
    gb.cpu.wr_hl(u16::wrapping_add(hl, 1));
}

fn ld_hld_a(gb: &mut GameBoy) {
    let hl = gb.cpu.rd_hl();
    gb.cycle_write(hl, gb.cpu.a);
    gb.cpu.wr_hl(u16::wrapping_sub(hl, 1));
}

fn ld_a_hli(gb: &mut GameBoy) {
    let hl = gb.cpu.rd_hl();
    gb.cpu.a = gb.cycle_read(hl);
    gb.cpu.wr_hl(u16::wrapping_add(hl, 1));
}

fn ld_a_hld(gb: &mut GameBoy) {
    let hl = gb.cpu.rd_hl();
    gb.cpu.a = gb.cycle_read(hl);
    gb.cpu.wr_hl(u16::wrapping_sub(hl, 1));
}

// Jumps and Subroutines

macro_rules! call {
    () => {
        |gb: &mut GameBoy| {
            let jp_addr = {
                let lsb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                let msb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                (msb << 8) + lsb
            };

            gb.advance_cycles(4);

            let ret = u16::to_le_bytes(gb.cpu.pc);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, ret[1]);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, ret[0]);

            gb.cpu.pc = jp_addr;
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let jp_addr = {
                    let lsb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    let msb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    (msb << 8) + lsb
                };

                if gb.cpu.[<$cc _flag>]() {
                    gb.advance_cycles(4);

                    let ret = u16::to_le_bytes(gb.cpu.pc);
                    gb.cpu.sp.dec();
                    gb.cycle_write(gb.cpu.sp, ret[1]);
                    gb.cpu.sp.dec();
                    gb.cycle_write(gb.cpu.sp, ret[0]);

                    gb.cpu.pc = jp_addr;
                }
            }
        }
    };
}

macro_rules! jp {
    () => {
        |gb: &mut GameBoy| {
            let addr = {
                let lsb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                let msb = gb.cycle_dpc(0) as u16;
                gb.cpu.pc.inc();
                (msb << 8) + lsb
            };
            gb.advance_cycles(4);
            gb.cpu.pc = addr;
        }
    };

    (hl) => {
        |gb: &mut GameBoy| {
            gb.cpu.pc = gb.cpu.rd_hl();
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let addr = {
                    let lsb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    let msb = gb.cycle_dpc(0) as u16;
                    gb.cpu.pc.inc();
                    (msb << 8) + lsb
                };
                if gb.cpu.[<$cc _flag>]() {
                    gb.advance_cycles(4);
                    gb.cpu.pc = addr;
                    return;
                }
            }
        }
    };
}

macro_rules! jr {
    () => {
        |gb: &mut GameBoy| {
            let addr = (gb.cycle_dpc(0) as i8) as u16;
            gb.cpu.pc.inc();
            gb.cpu.pc = u16::wrapping_add(gb.cpu.pc, addr);
            gb.advance_cycles(4);
        }
    };

    ($cc: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let addr = (gb.cycle_dpc(0) as i8) as u16;
                gb.cpu.pc.inc();
                if gb.cpu.[<$cc _flag>]() {
                    gb.cpu.pc = u16::wrapping_add(gb.cpu.pc, addr);
                    gb.advance_cycles(4);
                }
            }
        }
    };
}

#[inline(always)]
fn _ret(gb: &mut GameBoy) {
    gb.cpu.pc = {
        let lo = gb.cycle_read(gb.cpu.sp) as u16;
        gb.cpu.sp.inc();
        let hi = gb.cycle_read(gb.cpu.sp) as u16;
        gb.cpu.sp.inc();
        (hi << 8) + lo
    };
    gb.advance_cycles(4);
}

macro_rules! ret {
    () => {
        |gb: &mut GameBoy| _ret(gb)
    };

    ($cc: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                gb.advance_cycles(4);
                if gb.cpu.[<$cc _flag>]() {
                    _ret(gb);
                }
            }
        }
    };
}

fn reti(gb: &mut GameBoy) {
    _ret(gb);
    gb.intr.enable_immediate();
}

macro_rules! rst {
    ($hx: expr) => {
        |gb: &mut GameBoy| {
            gb.advance_cycles(4);

            let addr = u16::to_le_bytes(gb.cpu.pc);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, addr[1]);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, addr[0]);
            gb.cpu.pc = $hx;
        }
    };
}

// Stack Operations

fn add_sp_e8(gb: &mut GameBoy) {
    let offset = (gb.cycle_dpc(0) as i8) as u16;
    gb.cpu.pc.inc();

    gb.advance_cycles(8);

    let old_sp = gb.cpu.sp;
    gb.cpu.sp = u16::wrapping_add(old_sp, offset);

    gb.cpu.f = 0;
    if (old_sp & 0x000F) + (offset & 0x000F) > 0x000F {
        gb.cpu.f |= H_FLAG;
    }
    if u16::wrapping_add(old_sp & 0x00FF, offset & 0x00FF) > 0x00FF {
        gb.cpu.f |= C_FLAG;
    }
}

fn dec_sp(gb: &mut GameBoy) {
    gb.cpu.sp.dec();
    gb.advance_cycles(4);
}

fn inc_sp(gb: &mut GameBoy) {
    gb.cpu.sp.inc();
    gb.advance_cycles(4);
}

fn ld_n16_sp(gb: &mut GameBoy) {
    let addr = {
        let lsb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        let msb = gb.cycle_dpc(0) as u16;
        gb.cpu.pc.inc();
        (msb << 8) + lsb
    };
    let bytes = u16::to_le_bytes(gb.cpu.sp);
    gb.cycle_write(addr, bytes[0]);
    gb.cycle_write(u16::wrapping_add(addr, 1), bytes[1]);
}

fn ld_hl_sp_e8(gb: &mut GameBoy) {
    let offset = (gb.cycle_dpc(0) as i8) as u16;
    gb.cpu.pc.inc();

    gb.advance_cycles(4);

    let val = u16::wrapping_add(gb.cpu.sp, offset);
    gb.cpu.wr_hl(val);

    gb.cpu.f = 0;
    if (gb.cpu.sp & 0x000F) + (offset & 0x000F) > 0x000F {
        gb.cpu.f |= H_FLAG;
    }
    if (gb.cpu.sp & 0x00FF) + (offset & 0x00FF) > 0xFF {
        gb.cpu.f |= C_FLAG;
    }
}

fn ld_sp_hl(gb: &mut GameBoy) {
    gb.cpu.sp = gb.cpu.rd_hl();
    gb.advance_cycles(4);
}

fn pop_af(gb: &mut GameBoy) {
    gb.cpu.f = gb.cycle_read(gb.cpu.sp) & 0xF0;
    gb.cpu.sp.inc();
    gb.cpu.a = gb.cycle_read(gb.cpu.sp);
    gb.cpu.sp.inc();
}

macro_rules! pop {
    ($r16: ident) => {
        |gb: &mut GameBoy| {
            paste::paste! {
                let val = {
                    let lsb = gb.cycle_read(gb.cpu.sp) as u16;
                    gb.cpu.sp.inc();
                    let msb = gb.cycle_read(gb.cpu.sp) as u16;
                    gb.cpu.sp.inc();
                    (msb << 8) + lsb
                };
                gb.cpu.[<wr_ $r16>](val);
            }
        }
    };
}

fn push_af(gb: &mut GameBoy) {
    gb.advance_cycles(4);
    gb.cpu.sp.dec();
    gb.cycle_write(gb.cpu.sp, gb.cpu.a);
    gb.cpu.sp.dec();
    gb.cycle_write(gb.cpu.sp, gb.cpu.f);
}

macro_rules! push {
    ($hi: ident, $lo: ident) => {
        |gb: &mut GameBoy| {
            gb.advance_cycles(4);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, gb.cpu.$hi);
            gb.cpu.sp.dec();
            gb.cycle_write(gb.cpu.sp, gb.cpu.$lo);
        }
    };
}

// Miscellaneous Instructions

fn ccf(gb: &mut GameBoy) {
    gb.cpu.f &= !(N_FLAG | H_FLAG);
    gb.cpu.f ^= C_FLAG;
}

fn cpl(gb: &mut GameBoy) {
    gb.cpu.a = !gb.cpu.a;
    gb.cpu.f |= N_FLAG | H_FLAG;
}

fn daa(gb: &mut GameBoy) {
    let mut res = (gb.cpu.a as i16) & 0xFF;

    if gb.cpu.n_flag() {
        if gb.cpu.h_flag() {
            res = (res - 0x06) & 0xFF;
        }
        if gb.cpu.c_flag() {
            res -= 0x60;
        }
    } else {
        if gb.cpu.h_flag() || res & 0x0F > 0x09 {
            res += 0x06;
        }
        if gb.cpu.c_flag() || res > 0x9F {
            res += 0x60;
        }
    }

    gb.cpu.a = res as u8;

    gb.cpu.f &= !(Z_FLAG | H_FLAG);
    if res & 0x100 == 0x100 {
        gb.cpu.f |= C_FLAG;
    }
    if gb.cpu.a == 0 {
        gb.cpu.f |= Z_FLAG;
    }
}

fn di(gb: &mut GameBoy) {
    gb.intr.disable();
}

fn ei(gb: &mut GameBoy) {
    gb.intr.enable();
}

fn halt(gb: &mut GameBoy) {
    match (gb.intr.current_ime(), gb.intr.fetch()) {
        (false, Some(_)) => gb.halt_bug = true,
        _ => gb.halt = true,
    }
}

fn nop(_gb: &mut GameBoy) {}

fn scf(gb: &mut GameBoy) {
    gb.cpu.f &= !(N_FLAG | H_FLAG);
    gb.cpu.f |= C_FLAG;
}

fn stop(_gb: &mut GameBoy) {}

fn undefined(_gb: &mut GameBoy) {}

#[rustfmt::skip]
pub const OPCODES: [fn(&mut GameBoy); 256] = [
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
              ld!(c, b),    nop,          ld!(c, d),    ld!(c, e),    ld!(c, h),    ld!(c, l),    ld!(c, d hl), ld!(c, a),
/* 5X */      ld!(d, b),    ld!(d, c),    nop,          ld!(d, e),    ld!(d, h),    ld!(d, l),    ld!(d, d hl), ld!(d, a),
              ld!(e, b),    ld!(e, c),    ld!(e, d),    nop,          ld!(e, h),    ld!(e, l),    ld!(e, d hl), ld!(e, a),
/* 6X */      ld!(h, b),    ld!(h, c),    ld!(h, d),    ld!(h, e),    nop,          ld!(h, l),    ld!(h, d hl), ld!(h, a),
              ld!(l, b),    ld!(l, c),    ld!(l, d),    ld!(l, e),    ld!(l, h),    nop,          ld!(l, d hl), ld!(l, a),
/* 7X */      ld!(d hl, b), ld!(d hl, c), ld!(d hl, d), ld!(d hl, e), ld!(d hl, h), ld!(d hl, l), halt,         ld!(d hl, a),
              ld!(a, b),    ld!(a, c),    ld!(a, d),    ld!(a, e),    ld!(a, h),    ld!(a, l),    ld!(a, d hl), nop,
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
/* FX */      ldh_a_n8,     pop_af,       ldh_a_c,      di,           undefined,    push_af,      or!(),        rst!(0x30),
              ld_hl_sp_e8,  ld_sp_hl,     ld_a_n16,     ei,           undefined,    undefined,    cp!(),        rst!(0x38),
];

#[rustfmt::skip]
pub const OPCODES_CB: [fn(&mut GameBoy); 256] = [
/*             X0             X1             X2             X3             X4             X5             X6             X7             */
/*             X8             X9             XA             XB             XC             XD             XE             XF             */
/* 0X */       rlc!(b),       rlc!(c),       rlc!(d),       rlc!(e),       rlc!(h),       rlc!(l),       rlc!(d hl),    rlc!(a),
               rrc!(b),       rrc!(c),       rrc!(d),       rrc!(e),       rrc!(h),       rrc!(l),       rrc!(d hl),    rrc!(a),
/* 1X */       rl!(b),        rl!(c),        rl!(d),        rl!(e),        rl!(h),        rl!(l),        rl!(d hl),     rl!(a),
               rr!(b),        rr!(c),        rr!(d),        rr!(e),        rr!(h),        rr!(l),        rr!(d hl),     rr!(a),
/* 2X */       sla!(b),       sla!(c),       sla!(d),       sla!(e),       sla!(h),       sla!(l),       sla!(d hl),    sla!(a),
               sra!(b),       sra!(c),       sra!(d),       sra!(e),       sra!(h),       sra!(l),       sra!(d hl),    sra!(a),
/* 3X */       swap!(b),      swap!(c),      swap!(d),      swap!(e),      swap!(h),      swap!(l),      swap!(d hl),   swap!(a),
               srl!(b),       srl!(c),       srl!(d),       srl!(e),       srl!(h),       srl!(l),       srl!(d hl),    srl!(a),
/* 4X */       bit!(0, b),    bit!(0, c),    bit!(0, d),    bit!(0, e),    bit!(0, h),    bit!(0, l),    bit!(0, d hl), bit!(0, a),
               bit!(1, b),    bit!(1, c),    bit!(1, d),    bit!(1, e),    bit!(1, h),    bit!(1, l),    bit!(1, d hl), bit!(1, a),
/* 5X */       bit!(2, b),    bit!(2, c),    bit!(2, d),    bit!(2, e),    bit!(2, h),    bit!(2, l),    bit!(2, d hl), bit!(2, a),
               bit!(3, b),    bit!(3, c),    bit!(3, d),    bit!(3, e),    bit!(3, h),    bit!(3, l),    bit!(3, d hl), bit!(3, a),
/* 6X */       bit!(4, b),    bit!(4, c),    bit!(4, d),    bit!(4, e),    bit!(4, h),    bit!(4, l),    bit!(4, d hl), bit!(4, a),
               bit!(5, b),    bit!(5, c),    bit!(5, d),    bit!(5, e),    bit!(5, h),    bit!(5, l),    bit!(5, d hl), bit!(5, a),
/* 7X */       bit!(6, b),    bit!(6, c),    bit!(6, d),    bit!(6, e),    bit!(6, h),    bit!(6, l),    bit!(6, d hl), bit!(6, a),
               bit!(7, b),    bit!(7, c),    bit!(7, d),    bit!(7, e),    bit!(7, h),    bit!(7, l),    bit!(7, d hl), bit!(7, a),
/* 8X */       res!(0, b),    res!(0, c),    res!(0, d),    res!(0, e),    res!(0, h),    res!(0, l),    res!(0, d hl), res!(0, a),
               res!(1, b),    res!(1, c),    res!(1, d),    res!(1, e),    res!(1, h),    res!(1, l),    res!(1, d hl), res!(1, a),
/* 9X */       res!(2, b),    res!(2, c),    res!(2, d),    res!(2, e),    res!(2, h),    res!(2, l),    res!(2, d hl), res!(2, a),
               res!(3, b),    res!(3, c),    res!(3, d),    res!(3, e),    res!(3, h),    res!(3, l),    res!(3, d hl), res!(3, a),
/* AX */       res!(4, b),    res!(4, c),    res!(4, d),    res!(4, e),    res!(4, h),    res!(4, l),    res!(4, d hl), res!(4, a),
               res!(5, b),    res!(5, c),    res!(5, d),    res!(5, e),    res!(5, h),    res!(5, l),    res!(5, d hl), res!(5, a),
/* BX */       res!(6, b),    res!(6, c),    res!(6, d),    res!(6, e),    res!(6, h),    res!(6, l),    res!(6, d hl), res!(6, a),
               res!(7, b),    res!(7, c),    res!(7, d),    res!(7, e),    res!(7, h),    res!(7, l),    res!(7, d hl), res!(7, a),
/* CX */       set!(0, b),    set!(0, c),    set!(0, d),    set!(0, e),    set!(0, h),    set!(0, l),    set!(0, d hl), set!(0, a),
               set!(1, b),    set!(1, c),    set!(1, d),    set!(1, e),    set!(1, h),    set!(1, l),    set!(1, d hl), set!(1, a),
/* DX */       set!(2, b),    set!(2, c),    set!(2, d),    set!(2, e),    set!(2, h),    set!(2, l),    set!(2, d hl), set!(2, a),
               set!(3, b),    set!(3, c),    set!(3, d),    set!(3, e),    set!(3, h),    set!(3, l),    set!(3, d hl), set!(3, a),
/* EX */       set!(4, b),    set!(4, c),    set!(4, d),    set!(4, e),    set!(4, h),    set!(4, l),    set!(4, d hl), set!(4, a),
               set!(5, b),    set!(5, c),    set!(5, d),    set!(5, e),    set!(5, h),    set!(5, l),    set!(5, d hl), set!(5, a),
/* FX */       set!(6, b),    set!(6, c),    set!(6, d),    set!(6, e),    set!(6, h),    set!(6, l),    set!(6, d hl), set!(6, a),
               set!(7, b),    set!(7, c),    set!(7, d),    set!(7, e),    set!(7, h),    set!(7, l),    set!(7, d hl), set!(7, a),
];
