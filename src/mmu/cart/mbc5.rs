use crate::mmu::cart::{Cartridge, CartridgeError, RamBank, RomBank, BLANK_RAM, BLANK_ROM};

pub struct Mbc5 {
    rom: Vec<RomBank>,
    extras: Extras,

    rom_bank_lo: u8,
    rom_bank_hi: u8,
    ram_bank: u8,
    ram_enable: bool,
}

enum Extras {
    None,
    Ram(Vec<RamBank>),
    Rumble,
    RamRumble(Vec<RamBank>),
}

impl Mbc5 {
    pub fn init(has_ram: bool, has_rumble: bool) -> Self {
        let extras = match (has_ram, has_rumble) {
            (false, false) => Extras::None,
            (true, false) => Extras::Ram(vec![]),
            (false, true) => Extras::Rumble,
            (true, true) => Extras::RamRumble(vec![]),
        };
        Self { rom: vec![], extras, rom_bank_lo: 0, rom_bank_hi: 0, ram_bank: 0, ram_enable: false }
    }
}

impl Cartridge for Mbc5 {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError> {
        if nbanks > 512 {
            return Err(CartridgeError::InvalidCombination {
                tp: "MBC5".to_string(),
                feat: "more than 512 banks of ROM".to_string(),
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
            Extras::None | Extras::Rumble => {
                if nbanks != 0 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC5 without RAM".to_string(),
                        feat: "RAM banks".to_string(),
                    });
                }
            }
            Extras::Ram(ref mut ram) | Extras::RamRumble(ref mut ram) => {
                if nbanks != 1 && nbanks != 4 && nbanks != 16 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC5 with RAM".to_string(),
                        feat: format!("{} banks of RAM", nbanks),
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
        let rom_bank = (((self.rom_bank_hi as u16) << 8) | (self.rom_bank_lo as u16)) as usize;
        self.rom[rom_bank][(addr - 0x4000) as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        if self.ram_enable {
            match self.extras {
                Extras::None | Extras::Rumble => return 0xFF,
                Extras::Ram(ref ram) | Extras::RamRumble(ref ram) => {
                    return ram[self.ram_bank as usize][(addr & 0x1FFF) as usize]
                }
            }
        }

        0xFF
    }

    fn rom0_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enable = (val & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.rom_bank_lo = val,
            0x3000..=0x3FFF => self.rom_bank_hi = val & 0x01,
            _ => panic!(),
        }
    }

    fn romx_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000..=0x5FFF => {
                match self.extras {
                    Extras::None | Extras::Ram(_) => self.ram_bank = val & 0x0F,
                    Extras::Rumble | Extras::RamRumble(_) => {
                        self.ram_bank = (val & 0x0F) & !0x08
                        // TODO: set rumble
                    }
                }
            }
            _ => {}
        }
    }

    fn sram_write(&mut self, addr: u16, val: u8) {
        if self.ram_enable {
            match self.extras {
                Extras::None | Extras::Rumble => {}
                Extras::Ram(ref mut ram) | Extras::RamRumble(ref mut ram) => {
                    ram[self.ram_bank as usize][(addr & 0x1FFF) as usize] = val
                }
            }
        }
    }
}
