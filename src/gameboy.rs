use crate::cpu;

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub mmu: [u8; 4096],
    pub ime: bool,
    pub enabling_int: bool,
}

impl GameBoy {
    pub fn init() -> Self {
        Self { cpu: cpu::Cpu { a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0, pc: 0 }, mmu: [0;4096], ime: false , enabling_int: false }
    }
}
