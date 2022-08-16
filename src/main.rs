mod cpu;
mod debug;
mod gameboy;
mod mmu;
mod test;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate snafu;

const PATH: &str = "path";

fn main() {
    let mut gb = gameboy::GameBoy::init(PATH);
    gb.run(true);
}
