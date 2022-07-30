use crate::mmu::cart::{Cartridge, CartridgeError, RamBank, RomBank, BLANK_RAM, BLANK_ROM};

pub struct NoMbc {
    rom0: RomBank,
    romx: RomBank,
    extras: Extras,
}

enum Extras {
    NONE,
    RAM { ram: RamBank },
}

impl NoMbc {
    pub fn init(ram: bool) -> Self {
        match ram {
            false => NoMbc { rom0: BLANK_ROM, romx: BLANK_ROM, extras: Extras::NONE },
            true => NoMbc { rom0: BLANK_ROM, romx: BLANK_ROM, extras: Extras::RAM { ram: BLANK_RAM } },
        }
    }
}

impl Cartridge for NoMbc {
    fn init_rom_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        if nbanks != 2 {
            Err(invalid_combination("multiple ROM banks".to_string()))
        } else {
            Ok(())
        }
    }

    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        match self.extras {
            Extras::NONE => {
                if nbanks != 0 {
                    Err(invalid_combination("RAM banks".to_string()))
                } else {
                    Ok(())
                }
            }
            Extras::RAM { .. } => {
                if nbanks > 1 {
                    Err(invalid_combination("multiple RAM banks".to_string()))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn rom0_read(&self, addr: u16) -> u8 {
        self.rom0[addr as usize]
    }

    fn romx_read(&self, addr: u16) -> u8 {
        self.romx[addr as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        match self.extras {
            Extras::NONE => 0,
            Extras::RAM { ram } => ram[addr as usize],
        }
    }

    fn rom0_write(&mut self, _: u16, _: u8) {}

    fn romx_write(&mut self, _: u16, _: u8) {}

    fn sram_write(&mut self, addr: u16, val: u8) {
        match self.extras {
            Extras::NONE => {}
            Extras::RAM { ref mut ram } => {
                ram[addr as usize] = val;
            }
        }
    }
}

fn invalid_combination(desc: String) -> CartridgeError {
    CartridgeError::InvalidCombination { tp: "No MBC".to_string(), feat: desc }
}
