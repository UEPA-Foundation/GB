pub struct InterruptHandler {
    ime: ImeState,
    iflags: u8,
    ienable: u8,
}

#[derive(Clone, Copy, num_derive::FromPrimitive)]
pub enum Interrupt {
    VBLANK = 0,
    STAT = 1,
    TIMER = 2,
    SERIAL = 3,
    JOYPAD = 4,
}

enum ImeState {
    DISABLED,
    ENABLING,
    ENABLED,
}

impl InterruptHandler {
    pub fn init() -> Self {
        Self { ime: ImeState::DISABLED, iflags: 0, ienable: 0 }
    }

    #[inline(always)]
    pub fn enable(&mut self) {
        match self.ime {
            ImeState::DISABLED => self.ime = ImeState::ENABLING,
            _ => {}
        }
    }

    #[inline(always)]
    pub fn enable_immediate(&mut self) {
        self.ime = ImeState::ENABLED;
    }

    #[inline(always)]
    pub fn disable(&mut self) {
        self.ime = ImeState::DISABLED;
    }

    #[inline(always)]
    pub fn current_ime(&mut self) -> bool {
        match self.ime {
            ImeState::DISABLED => false,
            ImeState::ENABLING => {
                self.ime = ImeState::ENABLED;
                false
            }
            ImeState::ENABLED => true,
        }
    }
}

impl InterruptHandler {
    #[inline(always)]
    pub fn fetch(&self) -> Option<Interrupt> {
        num::FromPrimitive::from_u32((self.iflags & self.ienable & 0x1F).trailing_zeros())
    }

    #[inline(always)]
    pub fn request(&mut self, intr: Interrupt) {
        self.iflags |= 1 << (intr as u8);
    }

    #[inline(always)]
    pub fn reset(&mut self, intr: Interrupt) {
        self.iflags &= !(1 << (intr as u8));
    }
}

impl InterruptHandler {
    #[inline(always)]
    pub fn read_if(&self) -> u8 {
        self.iflags | 0xE0
    }

    #[inline(always)]
    pub fn read_ie(&self) -> u8 {
        self.ienable
    }

    #[inline(always)]
    pub fn write_if(&mut self, val: u8) {
        self.iflags = val;
    }

    #[inline(always)]
    pub fn write_ie(&mut self, val: u8) {
        self.ienable = val;
    }
}
