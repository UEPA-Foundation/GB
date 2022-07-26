mod cpu;
mod gameboy;

extern crate num;
extern crate num_derive;
extern crate paste;

fn main() {
    let mut gb = gameboy::GameBoy::init();
    gb.fetch_exec();
}
