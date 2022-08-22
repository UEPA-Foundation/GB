pub struct SerialLink {
    sb: u8,
    sc: u8,
}

impl SerialLink {
    pub fn init() -> Self {
        Self { sb: 0, sc: 0 }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => panic!(),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.sb = val,
            0xFF02 => self.sc = val,
            _ => panic!(),
        }
    }
}
