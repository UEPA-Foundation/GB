pub mod hram;
pub mod oam;
pub mod unused;
pub mod vram;
pub mod wram0;
pub mod wramx;

pub trait MemoryUnit {
    fn init() -> Self;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}
