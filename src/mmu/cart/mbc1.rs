use crate::mmu::cart::{CartridgeError, CartridgeTrait, RamBank, RomBank, BLANK_RAM, BLANK_ROM};

pub struct Mbc1 {
    rom: Vec<RomBank>,
    ram: Ram,

    mode: u8,
    ram_enable: bool,
    mask: u8,
    bank_lo: u8,
    bank_hi: u8,
}

enum Ram {
    NONE,
    RAM(Vec<RamBank>),
}

impl Mbc1 {
    pub fn init(ram: bool) -> Self {
        if ram {
            Self { rom: vec![], ram: Ram::RAM(vec![]), mode: 0, ram_enable: false, mask: 0, bank_lo: 1, bank_hi: 0 }
        } else {
            Self { rom: vec![], ram: Ram::NONE, mode: 0, ram_enable: false, mask: 0, bank_lo: 1, bank_hi: 0 }
        }
    }
}

impl CartridgeTrait for Mbc1 {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError> {
        if nbanks > 128 {
            return Err(CartridgeError::InvalidCombination {
                tp: "MBC1".to_string(),
                feat: "more than 128 banks of ROM".to_string(),
            });
        }

        if (nbanks as usize) * 0x4000 < raw_rom.len() {
            return Err(CartridgeError::OutOfRomBanks { nbanks, rom_size: raw_rom.len() / 1024 });
        }

        self.mask = 1 << (15 - nbanks.leading_zeros() as u8);
        self.mask -= 1;

        self.rom = vec![BLANK_ROM; nbanks as usize];

        for i in 0..nbanks as usize {
            for j in 0..0x4000 {
                self.rom[i][j] = raw_rom[(i * 0x4000) + j];
            }
        }

        Ok(())
    }

    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError> {
        match self.ram {
            Ram::NONE => {
                if nbanks != 0 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC1 without RAM".to_string(),
                        feat: "RAM banks".to_string(),
                    });
                }
            }
            Ram::RAM(ref mut ram) => {
                if nbanks > 4 {
                    return Err(CartridgeError::InvalidCombination {
                        tp: "MBC1 with RAM".to_string(),
                        feat: "more than 4 banks of RAM".to_string(),
                    });
                }
                *ram = match nbanks {
                    0 => vec![BLANK_RAM; 1], // allocate the minimum ammount to be safe
                    _ => vec![BLANK_RAM; nbanks as usize],
                }
            }
        }

        Ok(())
    }

    fn rom0_read(&self, addr: u16) -> u8 {
        let rom_bank = (self.bank_hi << 5) as usize;
        match (self.mode, self.rom.len() > rom_bank) {
            (0, _) | (1, false) => self.rom[0][addr as usize],
            (1, true) => self.rom[rom_bank][addr as usize],
            _ => panic!("MBC1 mode somehow isn't 0 or 1, wtf?"),
        }
    }

    fn romx_read(&self, addr: u16) -> u8 {
        let rom_bank = (self.bank_hi << 5) + self.bank_lo;
        self.rom[(rom_bank & self.mask) as usize][(addr - 0x4000) as usize]
    }

    fn sram_read(&self, addr: u16) -> u8 {
        if self.ram_enable {
            let ram_addr = (addr - 0xA000) as usize;
            match self.ram {
                Ram::NONE => return 0xFF,
                Ram::RAM(ref ram) => match self.mode {
                    0 => return ram[0][ram_addr],
                    1 => return ram[self.bank_hi as usize][ram_addr],
                    _ => panic!("MBC1 mode somehow isn't 0 or 1, wtf?"),
                },
            }
        }

        0xFF
    }

    fn rom0_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enable = (val & 0x0F) == 0xA,
            0x2000..=0x3FFF => self.bank_lo = val & 0b00011111,
            _ => panic!(),
        }

        if self.bank_lo == 0 {
            self.bank_lo = 1;
        }
    }

    fn romx_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000..=0x5FFF => self.bank_hi = val & 0b00000011,
            0x6000..=0x7FFF => self.mode = val & 1,
            _ => panic!(),
        }
    }

    fn sram_write(&mut self, addr: u16, val: u8) {
        if self.ram_enable {
            let ram_addr = (addr - 0xA000) as usize;
            match self.ram {
                Ram::NONE => {}
                Ram::RAM(ref mut ram) => match self.mode {
                    0 => ram[0][ram_addr] = val,
                    1 => ram[self.bank_hi as usize][ram_addr] = val,
                    _ => panic!("MBC1 mode somehow isn't 0 or 1, wtf?"),
                },
            }
        }
    }
}
