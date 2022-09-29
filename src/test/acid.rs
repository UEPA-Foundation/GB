#[cfg(test)]
use crate::gameboy::GameBoy;

macro_rules! test_acid {
    ($rom: ident, $path: expr) => {
        #[test]
        fn $rom() {
            let mut gb = GameBoy::init(concat!("./src/test/roms/acid/", $path));
            for _ in 0..10000000 {
                gb.cpu_step();

                // if it has software breakpoint (ld b, b), return
                if gb.dpc(0) == 0x40 {
                    return;
                }
            }

            panic!("Test timed out at ${:04X}.", gb.cpu.pc);
        }
    };
}

test_acid!(dmg_acid2, "dmg-acid2.gb");
