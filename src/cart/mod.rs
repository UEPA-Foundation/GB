use snafu::prelude::*;

mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
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

    #[snafu(display("Cartridge expected a maximum of {} banks of ROM ({}KiB), but got {}KiB of ROM", nbanks, (*nbanks as usize) * 0x4000 / 1024, rom_size))]
    OutOfRomBanks { nbanks: u16, rom_size: usize },
}

pub trait Cartridge {
    fn init_rom_banks(&mut self, nbanks: u16, raw_rom: &Vec<u8>) -> Result<(), CartridgeError>;
    fn init_ram_banks(&mut self, nbanks: u16) -> Result<(), CartridgeError>;

    fn rom0_read(&self, addr: u16) -> u8;
    fn romx_read(&self, addr: u16) -> u8;
    fn sram_read(&self, addr: u16) -> u8;

    fn rom0_write(&mut self, addr: u16, val: u8);
    fn romx_write(&mut self, addr: u16, val: u8);
    fn sram_write(&mut self, addr: u16, val: u8);
}

pub fn load_rom_file(path: &str) -> Box<dyn Cartridge> {
    let raw_rom = match std::fs::read(path) {
        Ok(content) => content,
        Err(e) => panic!("{}", e),
    };

    let title = String::from_utf8_lossy(&raw_rom[0x0134..=0x0142]);
    println!("Cartridge title: {}", title);

    // TODO: Switch between gb modes
    let gcb = raw_rom[0x0143];
    if gcb == 0xC0 {
        panic!("CGB only ROM :(");
    }

    let new_licensee = &raw_rom[0x0144..=0x0145];
    println!("New licensee code: {:02X?}", new_licensee);

    let old_licensee = raw_rom[0x014B];
    // if old_licensee != 0x33 -> disable SGB functions
    println!("Old licensee code: {:02X?}", old_licensee);

    let _sgb = raw_rom[0x0146];
    // if sgb == 0x03 -> enable SGB functions

    let cartridge_type = raw_rom[0x0147];
    let mut rom = match boxed_cartridge(cartridge_type) {
        Ok(cart) => cart,
        Err(e) => panic!("{}", e),
    };

    let rom_size = raw_rom[0x0148];
    let rom_banks = 2 << rom_size;
    match rom.init_rom_banks(rom_banks, &raw_rom) {
        Ok(()) => {}
        Err(e) => panic!("{}", e),
    };

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
    match rom.init_ram_banks(ram_banks) {
        Ok(()) => {}
        Err(e) => panic!("{}", e),
    };

    let _destination = raw_rom[0x014A];

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

    let _global_checksum = ((raw_rom[0x014E] as u16) << 8) + raw_rom[0x014F] as u16;

    println!("");

    rom
}

fn boxed_cartridge(code: u8) -> Result<Box<dyn Cartridge>, CartridgeError> {
    // Some cartridges include batteries, but it doesn't seem to make a
    // difference from the emulator perspective, might be wise to keep an eye on
    // this if bugs arise.
    Ok(match code {
        0x00 => Box::new(no_mbc::NoMbc::init(false)),
        0x01 => Box::new(mbc1::Mbc1::init(false)),
        0x02 | 0x03 => Box::new(mbc1::Mbc1::init(true)),
        0x05 | 0x06 => Box::new(mbc2::Mbc2::init()),
        0x08 | 0x09 => Box::new(no_mbc::NoMbc::init(true)),
        0x0F => Box::new(mbc3::Mbc3::init(false, true)),
        0x10 => Box::new(mbc3::Mbc3::init(true, true)),
        0x11 => Box::new(mbc3::Mbc3::init(false, false)),
        0x12 | 0x13 => Box::new(mbc3::Mbc3::init(true, false)),
        0x19 => Box::new(mbc5::Mbc5::init(false, false)),
        0x1A | 0x1B => Box::new(mbc5::Mbc5::init(true, false)),
        0x1C => Box::new(mbc5::Mbc5::init(false, true)),
        0x1D | 0x1E => Box::new(mbc5::Mbc5::init(false, false)),
        val => return Err(CartridgeError::InvalidType { tp: val }),
    })
}
