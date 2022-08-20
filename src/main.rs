mod cart;
mod cpu;
mod debug;
mod gameboy;
mod hram;
mod io;
mod oam;
mod test;
mod unused;
mod vram;
mod wram0;
mod wramx;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate snafu;

const PATH: &str = "path";

fn main() {
    let mut gb = gameboy::GameBoy::init(PATH);
    gb.run(true);
}
