use std::ops::{Index, IndexMut};

pub struct Mmu {
    rom0: Rom0,
    romx: RomX,
    vram: VRam,
    sram: SRam,
    wram0: WRam0,
    wramx: WRamX,
    echo: Echo,
    oam: Oam,
    unused: Unused,
    io: IoRegisters,
    hram: HRam,
    ie: IeRegister,
}

impl Mmu {
    fn init() -> Self {
        todo!();
    }
}

impl Index<u16> for Mmu {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0x0000..=0x3FFF => self.rom0[index],
            0x4000..=0x7FFF => self.romx[index],
            0x8000..=0x9FFF => self.vram[index],
            0xA000..=0xBFFF => self.sram[index],
            0xC000..=0xCFFF => self.wram0[index],
            0xD000..=0xDFFF => self.wramx[index],
            0xE000..=0xFDFF => self.echo[index],
            0xFE00..=0xFE9F => self.oam[index],
            0xFFA0..=0xFEFF => self.unused[index],
            0xFF00..=0xFF7F => self.io[index],
            0xFF80..=0xFFFE => self.hram[index],
            0xFFFF => self.ie[0xFFFF],
            _ => todo!(),
        }
    }
}

impl IndexMut<u16> for Mmu {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0x0000..=0x3FFF => self.rom0[index],
            0x4000..=0x7FFF => self.romx[index],
            0x8000..=0x9FFF => self.vram[index],
            0xA000..=0xBFFF => self.sram[index],
            0xC000..=0xCFFF => self.wram0[index],
            0xD000..=0xDFFF => self.wramx[index],
            0xE000..=0xFDFF => self.echo[index],
            0xFE00..=0xFE9F => self.oam[index],
            0xFFA0..=0xFEFF => self.unused[index],
            0xFF00..=0xFF7F => self.io[index],
            0xFF80..=0xFFFE => self.hram[index],
            0xFFFF => self.ie[0xFFFF],
            _ => todo!(),
        }
    }
}
