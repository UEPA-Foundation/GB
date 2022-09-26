pub struct SerialLink {
    sb: u8,
    sc: u8,
}

impl SerialLink {
    pub fn init() -> Self {
        Self { sb: 0, sc: 0 }
    }

    pub fn read_data(&self) -> u8 {
        self.sb
    }

    pub fn read_control(&self) -> u8 {
        self.sc | 0x7E
    }

    pub fn write_data(&mut self, val: u8) {
        self.sb = val;
    }

    pub fn write_control(&mut self, val: u8) {
        self.sc = val;
    }
}
