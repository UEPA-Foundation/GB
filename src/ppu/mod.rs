const NCOL: usize = 160;
const NLIN: usize = 144;

pub struct Ppu {
    framebuffer: [u8; NCOL * NLIN],
}
impl Ppu {
    pub fn init() -> Self {
        Self {
            framebuffer: [0; NLIN * NCOL],
        }
    }
}
