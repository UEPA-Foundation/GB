use crate::gameboy::GameBoy;

pub struct IoRegisters {
    serial_data: u8,
    serial_ctrl: u8,
    iflags: u8,
    timer: Timer,
}

struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    tima_state: TimaState,
}

enum TimaState {
    RUNNING,
    OVERFLOW,
    LOADING,
}

impl IoRegisters {
    pub fn init() -> Self {
        Self {
            serial_data: 0,
            serial_ctrl: 1,
            iflags: 0,
            timer: Timer { div: 0, tima: 0, tma: 0, tac: 0, tima_state: TimaState::RUNNING },
        }
    }
}

impl GameBoy {
    pub fn io_read(&self, index: u16) -> u8 {
        match index {
            0xFF01 => self.mmu.io.serial_data,
            0xFF02 => self.mmu.io.serial_ctrl,
            0xFF04 => (self.mmu.io.timer.div >> 8) as u8,
            0xFF05 => self.mmu.io.timer.tima,
            0xFF06 => self.mmu.io.timer.tma,
            0xFF07 => self.mmu.io.timer.tac | 0xF8,
            0xFF0F => self.mmu.io.iflags | 0xE0, // 3 upper bits always return 1
            _ => 0,
        }
    }

    pub fn io_write(&mut self, index: u16, val: u8) {
        match index {
            0xFF01 => self.mmu.io.serial_data = val,
            0xFF02 => self.mmu.io.serial_ctrl = val,
            0xFF04 => self.mmu.io.timer.div = 0,
            0xFF05 => match self.mmu.io.timer.tima_state {
                TimaState::RUNNING => self.mmu.io.timer.tima = val,
                TimaState::OVERFLOW => {
                    self.mmu.io.timer.tima = val;
                    self.mmu.io.timer.tima_state = TimaState::RUNNING;
                }
                TimaState::LOADING => return,
            },
            0xFF06 => {
                self.mmu.io.timer.tma = val;
                match self.mmu.io.timer.tima_state {
                    TimaState::LOADING => self.mmu.io.timer.tima = val,
                    _ => {}
                }
            }
            0xFF07 => {
                let mask = match self.mmu.io.timer.tac & 0b11 {
                    0 => 1 << 9,
                    1 => 1 << 3,
                    2 => 1 << 5,
                    3 => 1 << 7,
                    why => panic!("How the hell a two bit value equals {}?", why),
                };
                let bit = self.mmu.io.timer.div & mask != 0;

                if bit && self.mmu.io.timer.tac & 0b100 == 1 && val & 0b100 == 0 {
                    self.mmu.io.timer.tima = u8::wrapping_add(self.mmu.io.timer.tima, 1);
                    if self.mmu.io.timer.tima == 0 {
                        self.mmu.io.timer.tima_state = TimaState::OVERFLOW;
                    }
                }

                self.mmu.io.timer.tac = val;
            }
            0xFF0F => self.mmu.io.iflags = val,
            _ => (),
        }
    }

    pub fn cycle_timer(&mut self, cycles: u8) {
        for _ in 0..cycles {
            match self.mmu.io.timer.tima_state {
                TimaState::RUNNING => {}
                TimaState::OVERFLOW => {
                    self.set_if(0x02); // timer
                    self.mmu.io.timer.tima_state = TimaState::LOADING;
                }
                TimaState::LOADING => {
                    self.mmu.io.timer.tima = self.mmu.io.timer.tma;
                    self.mmu.io.timer.tima_state = TimaState::RUNNING;
                }
            }

            let timer = &mut self.mmu.io.timer;
            let mask = match timer.tac & 0b11 {
                0 => 1 << 9,
                1 => 1 << 3,
                2 => 1 << 5,
                3 => 1 << 7,
                why => panic!("How the hell a two bit value equals {}?", why),
            };

            if timer.tac & 0b100 == 0 {
                for _ in 0..4 {
                    let orig_bit = timer.div & mask != 0;
                    timer.div = u16::wrapping_add(timer.div, 1);
                    if orig_bit && timer.div & mask == 0 {
                        timer.tima = u8::wrapping_add(timer.tima, 1);
                        if timer.tima == 0 {
                            timer.tima_state = TimaState::OVERFLOW;
                        }
                    }
                }
            } else {
                timer.div = u16::wrapping_add(timer.div, 4);
            }
        }
    }
}
