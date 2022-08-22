use crate::gameboy::GameBoy;

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    tima_state: TimaState,
}

enum TimaState {
    RUNNING,
    OVERFLOW(u8),
    LOADING(u8),
}

impl Timer {
    pub fn init() -> Self {
        Self { div: 0, tima: 0, tma: 0, tac: 0, tima_state: TimaState::RUNNING }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => panic!(),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                let freq_bit = self.div & self.div_tima_mask() != 0;
                if self.is_enabled() && freq_bit {
                    self.increment_tima();
                }
                self.div = 0;
            }
            0xFF05 => match self.tima_state {
                TimaState::RUNNING => self.tima = val,
                TimaState::OVERFLOW(_) => {
                    self.tima = val;
                    self.tima_state = TimaState::RUNNING;
                }
                TimaState::LOADING(_) => {}
            },
            0xFF06 => self.tma = val,
            0xFF07 => {
                let mask = self.div_tima_mask();

                let bit = self.div & mask != 0;
                let enabled = self.is_enabled();
                let disabling = val & 0b100 == 0;

                // disabling the timer while the selected div bit is active increments TIMA
                // because of the falling edge circuitry
                if bit && enabled && disabling {
                    self.increment_tima();
                }

                self.tac = val;
            }
            _ => panic!(),
        }
    }

    #[inline(always)]
    fn div_tima_mask(&self) -> u16 {
        match self.tac & 0b11 {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            3 => 1 << 7,
            why => panic!("How the hell a two bit value equals {}?", why),
        }
    }

    #[inline(always)]
    fn increment_tima(&mut self) {
        self.tima = u8::wrapping_add(self.tima, 1);
        if self.tima == 0 {
            self.tima_state = TimaState::OVERFLOW(3);
        }
    }

    #[inline(always)]
    fn is_enabled(&mut self) -> bool {
        self.tac & 0b100 != 0
    }

    #[inline(always)]
    fn int_occurred(&mut self) -> bool {
        match self.tima_state {
            TimaState::LOADING(3) => true,
            _ => false,
        }
    }

    #[inline(always)]
    fn update_tima_state(&mut self) {
        match self.tima_state {
            TimaState::RUNNING => {}
            TimaState::OVERFLOW(ref mut count) => match count {
                0 => self.tima_state = TimaState::LOADING(3),
                _ => *count -= 1,
            },
            TimaState::LOADING(ref mut count) => match count {
                0 => {
                    self.tima_state = TimaState::RUNNING;
                    self.tima = self.tma;
                }
                _ => *count -= 1,
            },
        }
    }
}

impl GameBoy {
    pub fn cycle_timer(&mut self, cycles: u8) {
        for _ in 0..cycles {
            if self.timer.int_occurred() {
                self.set_if(0x04);
            }

            self.timer.update_tima_state();

            let mask = self.timer.div_tima_mask();
            let enabled = self.timer.is_enabled();
            let orig_bit = self.timer.div & mask != 0;

            self.timer.div = u16::wrapping_add(self.timer.div, 1);

            // falling edge detect
            if enabled && orig_bit && self.timer.div & mask == 0 {
                self.timer.increment_tima();
            }
        }
    }
}
