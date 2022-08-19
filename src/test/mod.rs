#[cfg(test)]
use crate::gameboy::GameBoy;

macro_rules! test_blargg {
    ($rom: ident, $path: expr) => {
        #[test]
        fn $rom() {
            std::panic::set_hook(Box::new(|_| {}));
            let mut gb = GameBoy::init(concat!("./src/test/gb-test-roms/", $path));
            let mut out = vec![];
            for _ in 0..30000000 {
                gb.fetch_exec();

                if gb.read(0xFF02) == 0x81 {
                    out.push(gb.read(0xFF01));
                }

                // if it has reached an infinite loop (jr -2 or jp pc), break
                let opcode = gb.dpc(0);
                let param1 = gb.dpc(1);
                let param2 = gb.dpc(2);
                let pc_hi = (gb.cpu.pc >> 8) as u8;
                let pc_lo = gb.cpu.pc as u8;
                if (opcode == 0x18 && param1 == 0xFE)  ||
                   (opcode == 0xC3 && param1 == pc_hi && param2 == pc_lo)
                {
                    break;
                }

                gb.write(0xFF02, 0); // This should be removed when serial works
            }

            let out_str = std::str::from_utf8(&out);
            match out_str {
                Ok(s) => {
                    if !s.contains("Passed") {
                        println!("{}", s);
                        panic!();
                    }
                }
                Err(_) => {
                    println!("Test failed to produce valid output.");
                    panic!();
                }
            }
        }
    };
}

test_blargg!(cpu_instrs_01_special, "cpu_instrs/individual/01-special.gb");
test_blargg!(cpu_instrs_02_int, "cpu_instrs/individual/02-interrupts.gb");
test_blargg!(cpu_instrs_03_op_sp_hl, "cpu_instrs/individual/03-op sp,hl.gb");
test_blargg!(cpu_instrs_04_op_r_imm, "cpu_instrs/individual/04-op r,imm.gb");
test_blargg!(cpu_instrs_05_op_rp, "cpu_instrs/individual/05-op rp.gb");
test_blargg!(cpu_instrs_06_ld_r_r, "cpu_instrs/individual/06-ld r,r.gb");
test_blargg!(cpu_instrs_07_jp, "cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
test_blargg!(cpu_instrs_08_misc, "cpu_instrs/individual/08-misc instrs.gb");
test_blargg!(cpu_instrs_09_op_r_r, "cpu_instrs/individual/09-op r,r.gb");
test_blargg!(cpu_instrs_10_bit, "cpu_instrs/individual/10-bit ops.gb");
test_blargg!(cpu_instrs_11_op_a_dhl, "cpu_instrs/individual/11-op a,(hl).gb");
