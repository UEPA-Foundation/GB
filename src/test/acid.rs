#[cfg(test)]
use crate::gameboy::GameBoy;

macro_rules! test_acid {
    ($rom: ident, $path: expr) => {
        #[test]
        fn $rom() {
            let mut gb = GameBoy::init(concat!("./src/test/", $path));
            for _ in 0..10000000 {
                gb.cpu_step();

                println!("{:#04X}, {:#04X}", gb.cpu.pc, gb.dpc(0));
                // if it has reached an infinite loop (jr -3), break
                if gb.dpc(0) == 0x18 && gb.dpc(1) == 0xFD {
                    return;
                }
            }

            panic!("Test timed out at ${:04X}.", gb.cpu.pc);
        }
    };
}

test_acid!(dmg_acid2, "dmg-acid2.gb");
