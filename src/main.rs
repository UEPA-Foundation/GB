mod cpu;
mod gameboy;
mod mmu;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate snafu;

const PATH: &str = "path";

fn main() {
    let mut gb = gameboy::GameBoy::init(PATH);
    gb.fetch_exec();
}
