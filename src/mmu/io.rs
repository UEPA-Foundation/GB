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
    OVERFLOW(u8),
    LOADING(u8),
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
                TimaState::OVERFLOW(_) => {
                    self.mmu.io.timer.tima = val;
                    self.mmu.io.timer.tima_state = TimaState::RUNNING;
                }
                TimaState::LOADING(_) => {}
            },
            0xFF06 => self.mmu.io.timer.tma = val,
            0xFF07 => {
                let timer = &mut self.mmu.io.timer;
                let mask = timer.div_tima_mask();

                let bit = timer.div & mask != 0;
                let enabled = timer.is_enabled();
                let disabling = val & 0b100 == 0;

                // disabling the timer while the selected div bit is active increments TIMA
                // because of the falling edge circuitry
                if bit && enabled && disabling {
                    timer.increment_tima();
                }

                timer.tac = val;
            }
            0xFF0F => self.mmu.io.iflags = val,
            _ => (),
        }
    }

    pub fn cycle_timer(&mut self, cycles: u8) {
        for _ in 0..cycles {
            match self.mmu.io.timer.tima_state {
                TimaState::RUNNING => {}
                TimaState::OVERFLOW(ref mut count) => match count {
                    0 => self.mmu.io.timer.tima_state = TimaState::LOADING(3),
                    _ => *count -= 1,
                },
                TimaState::LOADING(count) => {
                    if count == 0 {
                        self.mmu.io.timer.tima_state = TimaState::RUNNING;
                        self.mmu.io.timer.tima = self.mmu.io.timer.tma;
                        return;
                    }
                    if count == 3 {
                        self.set_if(0x04);
                    }
                    self.mmu.io.timer.tima_state = TimaState::LOADING(count - 1);
                }
            }

            let timer = &mut self.mmu.io.timer;
            let mask = timer.div_tima_mask();
            let enabled = timer.is_enabled();
            let orig_bit = timer.div & mask != 0;

            timer.div = u16::wrapping_add(timer.div, 1);

            // falling edge detection
            if enabled && orig_bit && timer.div & mask == 0 {
                timer.increment_tima();
            }
        }
    }
}

impl Timer {
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
}
