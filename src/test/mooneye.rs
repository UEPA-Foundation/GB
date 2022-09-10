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

test_mooneye!(timer_div_write, "acceptance/timer/div_write.gb");
test_mooneye!(timer_rapid_toggle, "acceptance/timer/rapid_toggle.gb");
test_mooneye!(timer_tim00, "acceptance/timer/tim00.gb");
test_mooneye!(timer_tim00_div_trigger, "acceptance/timer/tim00_div_trigger.gb");
test_mooneye!(timer_tim01, "acceptance/timer/tim01.gb");
test_mooneye!(timer_tim01_div_trigger, "acceptance/timer/tim01_div_trigger.gb");
test_mooneye!(timer_tim10, "acceptance/timer/tim10.gb");
test_mooneye!(timer_tim10_div_trigger, "acceptance/timer/tim10_div_trigger.gb");
test_mooneye!(timer_tim11, "acceptance/timer/tim11.gb");
test_mooneye!(timer_tim11_div_trigger, "acceptance/timer/tim11_div_trigger.gb");
test_mooneye!(timer_tima_reload, "acceptance/timer/tima_reload.gb");
test_mooneye!(timer_tima_write_reloading, "acceptance/timer/tima_write_reloading.gb");
test_mooneye!(timer_tma_write_reloading, "acceptance/timer/tma_write_reloading.gb");
