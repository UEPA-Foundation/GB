#[cfg(test)]
use crate::gameboy::GameBoy;

macro_rules! test_mooneye {
    ($rom: ident, $path: expr) => {
        #[test]
        fn $rom() {
            let mut gb = GameBoy::init(concat!("./src/test/mooneye/", $path));
            for _ in 0..100000000 {
                gb.cpu_step();

                if gb.dpc(0) == 0x40 {
                    match (gb.cpu.b, gb.cpu.c, gb.cpu.d, gb.cpu.e, gb.cpu.h, gb.cpu.l) {
                        (3, 5, 8, 13, 21, 34) => return,
                        _ => panic!("Test failed."),
                    }
                }
            }

            panic!("Test timed out at ${:04X}.", gb.cpu.pc);
        }
    };
}
test_mooneye!(daa, "acceptance/instr/daa.gb");
