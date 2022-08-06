use crate::debug;
use crate::{cpu, mmu};

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub mmu: mmu::Mmu,
    pub ime: bool,
    pub enabling_int: bool,
}

impl GameBoy {
    pub fn init(path: &str) -> Self {
        Self {
            cpu: cpu::Cpu { a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0, pc: 0x100 },
            mmu: mmu::Mmu::init(path),
            ime: false,
            enabling_int: false,
        }
    }

    pub fn run(&mut self, debug: bool) {
        if debug {
            let mut dgb = debug::DebugGB::init(self);
            loop {
                let cmd = dgb.prompt();
                dgb.exec(cmd);
            }
        }

        loop {
            self.fetch_exec();
        }
    }
}
