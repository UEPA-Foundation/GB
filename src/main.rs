mod cpu;

extern crate num;
#[macro_use]
extern crate num_derive;

#[allow(dead_code)]
static CPU: cpu::Cpu = cpu::Cpu { a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0, pc: 0 };

fn main() {}
