use crate::mmu::cart::{Cartridge, CartridgeError, RomBank, BLANK_ROM};

pub struct Mbc2 {
    rom: Vec<RomBank>,
    ram: [u8; 512],

    ram_enable: bool,
    bank: u8,
}

impl Mbc2 {
    pub fn init() -> Self {
        Self { rom: vec![], ram: [0xF0; 512], ram_enable: false, bank: 1 }
    }
}

impl Cartridge for Mbc2 {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError> {
        if nbanks > 16 {
            return Err(CartridgeError::InvalidCombination {
                tp: "MBC2".to_string(),
                feat: "more than 16 ROM banks".to_string(),
            });
        }

        if (nbanks as usize) * 0x4000 < raw_rom.len() {
            return Err(CartridgeError::OutOfRomBanks { nbanks, rom_size: raw_rom.len() / 1024 });
        }

        self.rom = vec![BLANK_ROM; nbanks as usize];

        for i in 0..nbanks as usize {
            for j in 0..0x4000 {
                self.rom[i][j] = raw_rom[(i * 0x4000) + j];
            }
        }

        Ok(())
    }

    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        if nbanks > 0 {
            return Err(CartridgeError::InvalidCombination {
                tp: "MBC2".to_string(),
                feat: "external RAM banks".to_string(),
            });
        }
        Ok(())
    }

    fn rom0_read(&self, addr: u16) -> u8 {
        self.rom[0][addr as usize]
    }

    fn romx_read(&self, addr: u16) -> u8 {
        self.rom[self.bank as usize][(addr - 0x4000) as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        self.ram[(addr & 0x01FF) as usize] | 0xF0
    }

    fn rom0_write(&mut self, addr: u16, val: u8) {
        if (addr & 0x0100) != 0 {
            self.bank = val & 0x0F;
            if self.bank == 0 {
                self.bank = 1;
            }
        } else {
            self.ram_enable = val == 0x0A;
        }
    }

    fn romx_write(&mut self, _addr: u16, _val: u8) {}

    fn sram_write(&mut self, addr: u16, val: u8) {
        self.ram[(addr & 0x01FF) as usize] = val;
    }
}
