#![cfg(test)]

macro_rules! test_mooneye {
    ($rom: ident, $path: expr) => {
        #[test]
        fn $rom() {
            let mut gb = crate::gameboy::GameBoy::init(concat!("./src/test/roms/mooneye/", $path));

            for _ in 0..10000000 {
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

#[test]
fn sprite_priority() {
    let mut gb = crate::gameboy::GameBoy::init("./src/test/roms/mooneye/manual-only/sprite_priority.gb");
    let img = image::io::Reader::open("./src/test/roms/mooneye/manual-only/sprite_priority-expected.png").unwrap().decode().unwrap().into_bytes();
    println!("{:?}", img);

    for _ in 0..10000000 {
        gb.cpu_step();
        if gb.dpc(0) == 0x40 {
            let fbuf = gb.borrow_framebuffer();
            let equ = img.iter().zip(fbuf.iter()).all(|(a, b)| a == b);
            match equ {
                true => return,
                false => panic!("Test failed."),
            }
        }
    }

    panic!("Test timed out at ${:04X}.", gb.cpu.pc);
}

mod intr {
    test_mooneye!(ei_sequence, "acceptance/ei_sequence.gb");
    test_mooneye!(ei_timing, "acceptance/ei_timing.gb");
    test_mooneye!(intr_timing, "acceptance/intr_timing.gb");
    test_mooneye!(if_ie_registers, "acceptance/if_ie_registers.gb");
    test_mooneye!(rapid_di_ei, "acceptance/rapid_di_ei.gb");
    test_mooneye!(ie_push, "acceptance/interrupts/ie_push.gb");
}

mod halt {
    test_mooneye!(halt_ime0_ei, "acceptance/halt_ime0_ei.gb");
    test_mooneye!(halt_ime0_nointr_timing, "acceptance/halt_ime0_nointr_timing.gb");
    test_mooneye!(halt_ime1_timing, "acceptance/halt_ime1_timing.gb");
    test_mooneye!(halt_ime1_timing2_gs, "acceptance/halt_ime1_timing2-GS.gb");
}

mod timing {
    test_mooneye!(call_cc_timing, "acceptance/call_cc_timing.gb");
    test_mooneye!(call_cc_timing2, "acceptance/call_cc_timing2.gb");
    test_mooneye!(call_timing, "acceptance/call_timing.gb");
    test_mooneye!(call_timing2, "acceptance/call_timing2.gb");

    test_mooneye!(ret_cc_timing, "acceptance/ret_cc_timing.gb");
    test_mooneye!(ret_timing, "acceptance/ret_timing.gb");
    test_mooneye!(reti_intr_timing, "acceptance/reti_intr_timing.gb");
    test_mooneye!(reti_timing, "acceptance/reti_timing.gb");

    test_mooneye!(add_sp_e_timing, "acceptance/add_sp_e_timing.gb");
    test_mooneye!(di_timing_gs, "acceptance/di_timing-GS.gb");
    test_mooneye!(div_timing, "acceptance/div_timing.gb");
    test_mooneye!(jp_cc_timing, "acceptance/jp_cc_timing.gb");
    test_mooneye!(jp_timing, "acceptance/jp_timing.gb");
    test_mooneye!(ld_hl_sp_e_timing, "acceptance/ld_hl_sp_e_timing.gb");
    test_mooneye!(oam_dma_timing, "acceptance/oam_dma_timing.gb");
    test_mooneye!(pop_timing, "acceptance/pop_timing.gb");
    test_mooneye!(push_timing, "acceptance/push_timing.gb");
    test_mooneye!(rst_timing, "acceptance/rst_timing.gb");
}

mod timer {
    test_mooneye!(div_write, "acceptance/timer/div_write.gb");
    test_mooneye!(rapid_toggle, "acceptance/timer/rapid_toggle.gb");
    test_mooneye!(tim00, "acceptance/timer/tim00.gb");
    test_mooneye!(tim00_div_trigger, "acceptance/timer/tim00_div_trigger.gb");
    test_mooneye!(tim01, "acceptance/timer/tim01.gb");
    test_mooneye!(tim01_div_trigger, "acceptance/timer/tim01_div_trigger.gb");
    test_mooneye!(tim10, "acceptance/timer/tim10.gb");
    test_mooneye!(tim10_div_trigger, "acceptance/timer/tim10_div_trigger.gb");
    test_mooneye!(tim11, "acceptance/timer/tim11.gb");
    test_mooneye!(tim11_div_trigger, "acceptance/timer/tim11_div_trigger.gb");
    test_mooneye!(tima_reload, "acceptance/timer/tima_reload.gb");
    test_mooneye!(tima_write_reloading, "acceptance/timer/tima_write_reloading.gb");
    test_mooneye!(tma_write_reloading, "acceptance/timer/tma_write_reloading.gb");
}

mod ppu {
    test_mooneye!(hblank_ly_scx_timing_gs, "acceptance/ppu/hblank_ly_scx_timing-GS.gb");
    test_mooneye!(intr_1_2_timing_gs, "acceptance/ppu/intr_1_2_timing-GS.gb");
    test_mooneye!(intr_2_0_timing, "acceptance/ppu/intr_2_0_timing.gb");
    test_mooneye!(intr_2_mode0_timing, "acceptance/ppu/intr_2_mode0_timing.gb");
    test_mooneye!(intr_2_mode0_timing_sprites, "acceptance/ppu/intr_2_mode0_timing_sprites.gb");
    test_mooneye!(intr_2_mode3_timing, "acceptance/ppu/intr_2_mode3_timing.gb");
    test_mooneye!(intr_2_oam_ok_timing, "acceptance/ppu/intr_2_oam_ok_timing.gb");
    test_mooneye!(lcdon_timing_gs, "acceptance/ppu/lcdon_timing-GS.gb");
    test_mooneye!(lcdon_write_timing_gs, "acceptance/ppu/lcdon_write_timing-GS.gb");
    test_mooneye!(stat_irq_blocking, "acceptance/ppu/stat_irq_blocking.gb");
    test_mooneye!(stat_lyc_onoff, "acceptance/ppu/stat_lyc_onoff.gb");
    test_mooneye!(vblank_stat_intr_gs, "acceptance/ppu/vblank_stat_intr-GS.gb");
}

mod oam {
    test_mooneye!(oam_dma_restart, "acceptance/oam_dma_restart.gb");
    test_mooneye!(oam_dma_start, "acceptance/oam_dma_start.gb");
    test_mooneye!(basic, "acceptance/oam_dma/basic.gb");
    test_mooneye!(reg_read, "acceptance/oam_dma/reg_read.gb");
    test_mooneye!(sources_gs, "acceptance/oam_dma/sources-GS.gb");
}

mod bits {
    test_mooneye!(mem_oam, "acceptance/bits/mem_oam.gb");
    test_mooneye!(reg_f, "acceptance/bits/reg_f.gb");
    test_mooneye!(unused_hwio_gs, "acceptance/bits/unused_hwio-GS.gb");
}

mod mbc1 {
    test_mooneye!(bits_bank1, "emulator-only/mbc1/bits_bank1.gb");
    test_mooneye!(bits_bank2, "emulator-only/mbc1/bits_bank2.gb");
    test_mooneye!(bits_mode, "emulator-only/mbc1/bits_mode.gb");
    test_mooneye!(bits_ramg, "emulator-only/mbc1/bits_ramg.gb");
    test_mooneye!(multicart_rom_8mb, "emulator-only/mbc1/multicart_rom_8Mb.gb");
    test_mooneye!(ram_256kb, "emulator-only/mbc1/ram_256kb.gb");
    test_mooneye!(ram_64kb, "emulator-only/mbc1/ram_64kb.gb");
    test_mooneye!(rom_16mb, "emulator-only/mbc1/rom_16Mb.gb");
    test_mooneye!(rom_1mb, "emulator-only/mbc1/rom_1Mb.gb");
    test_mooneye!(rom_2mb, "emulator-only/mbc1/rom_2Mb.gb");
    test_mooneye!(rom_4mb, "emulator-only/mbc1/rom_4Mb.gb");
    test_mooneye!(rom_512kb, "emulator-only/mbc1/rom_512kb.gb");
    test_mooneye!(rom_8mb, "emulator-only/mbc1/rom_8Mb.gb");
}

mod mbc2 {
    test_mooneye!(bits_ramg, "emulator-only/mbc2/bits_ramg.gb");
    test_mooneye!(rom_1mb, "emulator-only/mbc2/rom_1Mb.gb");
    test_mooneye!(bits_romb, "emulator-only/mbc2/bits_romb.gb");
    test_mooneye!(rom_2mb, "emulator-only/mbc2/rom_2Mb.gb");
    test_mooneye!(bits_unused, "emulator-only/mbc2/bits_unused.gb");
    test_mooneye!(rom_512kb, "emulator-only/mbc2/rom_512kb.gb");
    test_mooneye!(ram, "emulator-only/mbc2/ram.gb");
}   

mod mbc5 {
    test_mooneye!(rom_512kb, "emulator-only/mbc5/rom_512kb.gb");
    test_mooneye!(rom_1mb, "emulator-only/mbc5/rom_1Mb.gb");
    test_mooneye!(rom_2mb, "emulator-only/mbc5/rom_2Mb.gb");
    test_mooneye!(rom_4mb, "emulator-only/mbc5/rom_4Mb.gb");
    test_mooneye!(rom_8mb, "emulator-only/mbc5/rom_8Mb.gb");
    test_mooneye!(rom_16mb, "emulator-only/mbc5/rom_16Mb.gb");
    test_mooneye!(rom_32mb, "emulator-only/mbc5/rom_32Mb.gb");
    test_mooneye!(rom_64mb, "emulator-only/mbc5/rom_64Mb.gb");
}

test_mooneye!(instr_daa, "acceptance/instr/daa.gb");
