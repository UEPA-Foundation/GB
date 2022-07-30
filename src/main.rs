mod cpu;
mod gameboy;
mod mmu;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate snafu;

fn main() {
    let mut gb = gameboy::GameBoy::init();
    gb.fetch_exec();
}
