use crate::gameboy::GameBoy;
use crate::intr::Interrupt;

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

    #[inline(always)]
    pub fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    #[inline(always)]
    pub fn read_tima(&self) -> u8 {
        match self.tima_state {
            TimaState::LOADING(_) => self.tma,
            _ => self.tima,
        }
    }

    #[inline(always)]
    pub fn read_tma(&self) -> u8 {
        self.tma
    }

    #[inline(always)]
    pub fn read_tac(&self) -> u8 {
        self.tac | 0xF8
    }

    #[inline(always)]
    pub fn write_div(&mut self) {
        let freq_bit = self.div & self.div_tima_mask() != 0;
        if self.is_enabled() && freq_bit {
            self.increment_tima();
        }
        self.div = 0;
    }

    #[inline(always)]
    pub fn write_tima(&mut self, val: u8) {
        match self.tima_state {
            TimaState::RUNNING => self.tima = val,
            TimaState::OVERFLOW(_) => {
                self.tima = val;
                self.tima_state = TimaState::RUNNING;
            }
            TimaState::LOADING(_) => {}
        }
    }

    #[inline(always)]
    pub fn write_tma(&mut self, val: u8) {
        self.tma = val;
    }

    #[inline(always)]
    pub fn write_tac(&mut self, val: u8) {
        let old_freq_bit = self.is_enabled() && self.div & self.div_tima_mask() != 0;
        self.tac = val;
        let new_freq_bit = self.is_enabled() && self.div & self.div_tima_mask() != 0;

        // disabling or changing the timer rate while the selected div bit is active
        // increments TIMA because of the falling edge circuitry
        if old_freq_bit && !new_freq_bit {
            self.increment_tima();
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
                self.intr.request(Interrupt::TIMER);
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
