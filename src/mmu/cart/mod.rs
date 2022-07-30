use snafu::prelude::*;

mod no_mbc;

pub type RomBank = [u8; 0x4000];
pub const BLANK_ROM: RomBank = [0; 0x4000];

pub type RamBank = [u8; 0x2000];
pub const BLANK_RAM: RamBank = [0; 0x2000];

#[derive(Snafu, Debug)]
pub enum CartridgeError {
    #[snafu(display("Cartridge type {:02X?} is not supported", tp))]
    InvalidType { tp: u8 },

    #[snafu(display("{} Cartridge doesn't support {}", tp, feat))]
    InvalidCombination { tp: String, feat: String },
}

pub trait Cartridge {
    fn init_rom_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError>;
    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError>;

    fn rom0_read(&self, addr: u16) -> u8;
    fn romx_read(&self, addr: u16) -> u8;
    fn sram_read(&self, addr: u16) -> u8;

    fn rom0_write(&mut self, addr: u16, val: u8);
    fn romx_write(&mut self, addr: u16, val: u8);
    fn sram_write(&mut self, addr: u16, val: u8);
}

pub fn read_rom(path: &str) -> Box<dyn Cartridge> {
    let raw_rom = std::fs::read(path).unwrap();

    let title = String::from_utf8_lossy(&raw_rom[0x0134..=0x0142]);
    println!("Cartridge title: {}", title);

    // TODO: Switch between gb modes
    let gcb = raw_rom[0x0143];
    if gcb == 0xC0 {
        panic!("CGB only ROM :(");
    }

    let licensee = &raw_rom[0x0144..=0x0145];
    println!("Licensee code: {:02X?}", licensee);

    let _sgb = raw_rom[0x0146];
    // if sgb == 0x03 -> enable SGB functions

    let cartridge_type = raw_rom[0x0147];
    let mut rom = boxed_cartridge(cartridge_type).unwrap();

    let rom_size = raw_rom[0x0148];
    let rom_banks = 2 << rom_size;
    rom.init_rom_banks(rom_banks).unwrap();

    let ram_size = raw_rom[0x0149];
    let ram_banks = match ram_size {
        0x00 => 0,
        // The 0x01 ram code is weird as it isn't listed in official docs and
        // supposedly uses a quarter of a normal RAM bank (2KiB vs 8KiB), but we
        // can support it by allocating a normal RAM bank for it and leaving the
        // upper portion of the bank unused.
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        _ => panic!("Too much RAM?"),
    };
    rom.init_ram_banks(ram_banks).unwrap();

    let _destination = raw_rom[0x014A];

    let _old_licensee = raw_rom[0x014B];
    // if old_licensee != 0x33 -> disable SGB functions

    let mask_version = raw_rom[0x014C];
    println!("Mask ROM version: {}", mask_version);

    let mut checksum = 0;
    for addr in 0x0134..=0x014C {
        checksum = u8::wrapping_sub(checksum, u8::wrapping_sub(raw_rom[addr], 1));
    }
    let header_checksum = raw_rom[0x014D];

    if checksum == header_checksum {
        panic!("ROM checksum does not match");
    }

    let _global_checksum = (raw_rom[0x014E] as u16) << 8 + raw_rom[0x014F] as u16;

    rom
}

fn boxed_cartridge(code: u8) -> Result<Box<dyn Cartridge>, CartridgeError> {
    let res = match code {
        0x00 => Ok(no_mbc::NoMbc::init(false)),
        0x08 => Ok(no_mbc::NoMbc::init(true)),
        // 0x09 cartridges include batteries, but it doesn't seem to make a
        // difference from the emulator perspective, might be wise to keep an
        // eye on this if bugs arise.
        0x09 => Ok(no_mbc::NoMbc::init(true)),
        val => Err(CartridgeError::InvalidType { tp: val }),
    };
    Ok(Box::new(res?))
}
