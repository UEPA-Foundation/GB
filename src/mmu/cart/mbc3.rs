// TODO: Implement the MBC30 variant. Required if we want to play Pokémon Crystal.
// We want to play Pokémon Crystal.
use crate::mmu::cart::{Cartridge, CartridgeError, RamBank, RomBank, BLANK_RAM, BLANK_ROM};

pub struct Mbc3 {
    rom: Vec<RomBank>,
    extras: Extras,

    rom_bank: u8,
    ram_rtc_sel: u8, // RAM bank or RTC reg number
    ram_enable: bool,
}

enum Extras {
    None,
    Ram(Vec<RamBank>),
    Timer(Rtc),
    RamTimer((Vec<RamBank>, Rtc)),
}

struct Rtc {
    sec: u8,
    min: u8,
    hour: u8,
    day_lo: u8,
    day_hi: u8,

    latch_sec: u8,
    latch_min: u8,
    latch_hour: u8,
    latch_day_lo: u8,
    latch_day_hi: u8,

    last_latch_val: u8,
}

impl Mbc3 {
    pub fn init(has_ram: bool, has_rtc: bool) -> Self {
        let extras = match (has_ram, has_rtc) {
            (false, false) => Extras::None,
            (true, false) => Extras::Ram(vec![]),
            (false, true) => Extras::Timer(Rtc {
                sec: 0,
                min: 0,
                hour: 0,
                day_lo: 0,
                day_hi: 0,
                latch_sec: 0,
                latch_min: 0,
                latch_hour: 0,
                latch_day_lo: 0,
                latch_day_hi: 0,
                last_latch_val: 1,
            }),
            (true, true) => Extras::RamTimer((
                vec![],
                Rtc {
                    sec: 0,
                    min: 0,
                    hour: 0,
                    day_lo: 0,
                    day_hi: 0,
                    latch_sec: 0,
                    latch_min: 0,
                    latch_hour: 0,
                    latch_day_lo: 0,
                    latch_day_hi: 0,
                    last_latch_val: 1,
                },
            )),
        };
        Self { rom: vec![], extras, rom_bank: 1, ram_rtc_sel: 0, ram_enable: false }
    }
}

impl Cartridge for Mbc3 {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError> {
        if nbanks > 128 {
            return Err(CartridgeError::InvalidCombination {
                tp: "MBC3".to_string(),
                feat: "more than 128 banks of ROM".to_string(),
            });
        }

        self.rom = vec![BLANK_ROM; nbanks as usize];

        if (nbanks as usize) * 0x4000 < raw_rom.len() {
            return Err(CartridgeError::OutOfRomBanks { nbanks, rom_size: raw_rom.len() / 1024 });
        }

        for i in 0..nbanks as usize {
            for j in 0..0x4000 {
                self.rom[i][j] = raw_rom[(i * 0x4000) + j];
            }
        }

        Ok(())
    }

    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        match self.extras {
            Extras::None | Extras::Timer(_) => {
                if nbanks != 0 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC1 without RAM".to_string(),
                        feat: "RAM banks".to_string(),
                    });
                }
            }
            Extras::Ram(ref mut ram) | Extras::RamTimer((ref mut ram, _)) => {
                if nbanks > 4 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC1 with RAM".to_string(),
                        feat: "more than 4 banks of RAM".to_string(),
                    });
                }
                *ram = vec![BLANK_RAM; nbanks as usize];
            }
        }

        Ok(())
    }

    fn rom0_read(&self, addr: u16) -> u8 {
        self.rom[0][addr as usize]
    }

    fn romx_read(&self, addr: u16) -> u8 {
        self.rom[self.rom_bank as usize][(addr - 0x4000) as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        if self.ram_enable {
            match self.extras {
                Extras::None => return 0xFF,
                Extras::Ram(ref ram) => return ram[(self.ram_rtc_sel & 0x03) as usize][(addr & 0x1FFF) as usize],
                Extras::Timer(ref rtc) => match self.ram_rtc_sel {
                    0x08 => return rtc.latch_sec,
                    0x09 => return rtc.latch_min,
                    0x0A => return rtc.latch_hour,
                    0x0B => return rtc.latch_day_lo,
                    0x0C => return rtc.latch_day_hi,
                    _ => return 0xFF,
                },
                Extras::RamTimer((ref ram, ref rtc)) => match self.ram_rtc_sel {
                    0x00..=0x03 => return ram[self.ram_rtc_sel as usize][(addr & 0x1FFF) as usize],
                    0x08 => return rtc.latch_sec,
                    0x09 => return rtc.latch_min,
                    0x0A => return rtc.latch_hour,
                    0x0B => return rtc.latch_day_lo,
                    0x0C => return rtc.latch_day_hi,
                    _ => return 0xFF,
                },
            }
        }

        0xFF
    }

    fn rom0_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enable = (val & 0x0F) == 0x0A,
            0x2000..=0x3FFF => self.rom_bank = val & 0x7F,
            _ => panic!(),
        }

        if self.rom_bank == 0 {
            self.rom_bank = 1;
        }
    }

    fn romx_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000..=0x5FFF => {
                self.ram_rtc_sel = val & 0x0F;
            }
            0x6000..=0x7FFF => match self.extras {
                Extras::Timer(ref mut rtc) | Extras::RamTimer((_, ref mut rtc)) => {
                    if rtc.last_latch_val == 0 && val == 1 {
                        rtc.latch_sec = rtc.sec;
                        rtc.latch_min = rtc.min;
                        rtc.latch_hour = rtc.hour;
                        rtc.latch_day_lo = rtc.day_lo;
                        rtc.latch_day_hi = rtc.day_hi;
                    }
                    rtc.last_latch_val = val;
                }
                _ => {}
            },
            _ => panic!(),
        }
    }

    fn sram_write(&mut self, addr: u16, val: u8) {
        if self.ram_enable {
            match self.extras {
                Extras::None => {}
                Extras::Ram(ref mut ram) => ram[(self.ram_rtc_sel & 0x03) as usize][(addr & 0x1FFF) as usize] = val,
                Extras::Timer(ref mut rtc) => match self.ram_rtc_sel {
                    0x08 => rtc.sec = val,
                    0x09 => rtc.min = val,
                    0x0A => rtc.hour = val,
                    0x0B => rtc.day_lo = val,
                    0x0C => rtc.day_hi = val,
                    _ => return,
                },
                Extras::RamTimer((ref mut ram, ref mut rtc)) => match self.ram_rtc_sel {
                    0x00..=0x03 => ram[self.ram_rtc_sel as usize][(addr & 0x1FFF) as usize] = val,
                    0x08 => rtc.sec = val,
                    0x09 => rtc.min = val,
                    0x0A => rtc.hour = val,
                    0x0B => rtc.day_lo = val,
                    0x0C => rtc.day_hi = val,
                    _ => return,
                },
            }
        }
    }
}
