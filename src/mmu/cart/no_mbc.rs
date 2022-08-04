use crate::mmu::cart::{Cartridge, CartridgeError, RamBank, RomBank, BLANK_RAM, BLANK_ROM};

pub struct NoMbc {
    rom0: RomBank,
    romx: RomBank,
    ram: Ram,
}

enum Ram {
    NONE,
    RAM(RamBank),
}

impl NoMbc {
    pub fn init(ram: bool) -> Self {
        if ram {
            NoMbc { rom0: BLANK_ROM, romx: BLANK_ROM, ram: Ram::RAM(BLANK_RAM) }
        } else {
            NoMbc { rom0: BLANK_ROM, romx: BLANK_ROM, ram: Ram::NONE }
        }
    }
}

impl Cartridge for NoMbc {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError> {
        if nbanks != 2 {
            return Err(CartridgeError::InvalidCombination {
                tp: "No MBC".to_string(),
                feat: "multiple ROM banks".to_string(),
            });
        }

        if raw_rom.len() >= 0x8000 {
            return Err(CartridgeError::OutOfRomBanks { nbanks, rom_size: raw_rom.len() / 1024 });
        }

        for i in 0..0x4000 {
            self.rom0[i] = raw_rom[i];
        }
        for i in 0..0x4000 {
            self.romx[i] = raw_rom[i + 0x4000];
        }

        Ok(())
    }

    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        match self.ram {
            Ram::NONE => {
                if nbanks != 0 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "No MBC without RAM".to_string(),
                        feat: "ROM banks".to_string(),
                    });
                }
            }
            Ram::RAM(_) => {
                if nbanks > 1 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "No MBC".to_string(),
                        feat: "multiple ROM banks".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn rom0_read(&self, addr: u16) -> u8 {
        self.rom0[addr as usize]
    }

    fn romx_read(&self, addr: u16) -> u8 {
        self.romx[(addr - 0x4000) as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        match self.ram {
            Ram::NONE => 0xFF,
            Ram::RAM(ram) => ram[(addr - 0xA000) as usize],
        }
    }

    fn rom0_write(&mut self, _: u16, _: u8) {}

    fn romx_write(&mut self, _: u16, _: u8) {}

    fn sram_write(&mut self, addr: u16, val: u8) {
        match self.ram {
            Ram::NONE => {}
            Ram::RAM(ref mut ram) => ram[(addr - 0xA000) as usize] = val,
        }
    }
}
