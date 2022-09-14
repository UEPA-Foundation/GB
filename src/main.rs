use crate::{debug::Debugger, gameboy::GameBoy, mmu::io::joypad::Button};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureAccess},
    video::Window,
    Sdl,
};

mod cpu;
mod debug;
mod gameboy;
mod intr;
mod mmu;
mod ppu;
mod test;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate sdl2;
extern crate snafu;

const PATH: &str = "src/test/dmg-acid2.gb";
const DEBUG: bool = false;
const PALETTE: [[u8; 4]; 3] = [[15, 48, 139, 155], [56, 98, 172, 188], [15, 48, 15, 15]];

fn main() {
    let mut gb = GameBoy::init(PATH);
    let (sdl, mut canvas) = init_renderer();

    let tex_creator = canvas.texture_creator();
    let mut tex = tex_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, 160, 144).unwrap();
    update_tex(&mut tex, &gb);

    match DEBUG {
        true => {
            let mut dbg = Debugger::init();
            loop {
                dbg.prompt(&mut gb);
                handle_events(&sdl, &mut gb);
                update_tex(&mut tex, &gb);
                canvas.copy(&tex, None, None).unwrap();
                canvas.present();
            }
        }
        false => loop {
            gb.cpu_step();
            handle_events(&sdl, &mut gb);
            update_tex(&mut tex, &gb);
            canvas.copy(&tex, None, None).unwrap();
            canvas.present();
        },
    }
}

fn init_renderer() -> (Sdl, Canvas<Window>) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("UEPA-GB", 640, 576).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    (sdl, canvas)
}

#[inline(always)]
fn update_tex(tex: &mut Texture, gb: &GameBoy) {
    let fb = gb.borrow_framebuffer();
    let mut tex_buf = [0; 160 * 144 * 3];

    for (i, pixel) in fb.iter().enumerate() {
        tex_buf[(i * 3) + 0] = PALETTE[0][*pixel as usize];
        tex_buf[(i * 3) + 1] = PALETTE[1][*pixel as usize];
        tex_buf[(i * 3) + 2] = PALETTE[2][*pixel as usize];
    }

    tex.update(None, &tex_buf, 160 * 3).unwrap();
}

#[inline(always)]
fn handle_events(sdl: &Sdl, gb: &mut GameBoy) {
    for event in sdl.event_pump().unwrap().poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => std::process::exit(0),

            Event::KeyDown { keycode: Some(Keycode::Z), .. } => gb.set_button(Button::A, true),
            Event::KeyDown { keycode: Some(Keycode::X), .. } => gb.set_button(Button::B, true),
            Event::KeyDown { keycode: Some(Keycode::Return), .. } => gb.set_button(Button::START, true),
            Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => gb.set_button(Button::SELECT, true),
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => gb.set_button(Button::UP, true),
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => gb.set_button(Button::DOWN, true),
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => gb.set_button(Button::LEFT, true),
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => gb.set_button(Button::RIGHT, true),

            Event::KeyUp { keycode: Some(Keycode::Z), .. } => gb.set_button(Button::A, false),
            Event::KeyUp { keycode: Some(Keycode::X), .. } => gb.set_button(Button::B, false),
            Event::KeyUp { keycode: Some(Keycode::Return), .. } => gb.set_button(Button::START, false),
            Event::KeyUp { keycode: Some(Keycode::Backspace), .. } => gb.set_button(Button::SELECT, false),
            Event::KeyUp { keycode: Some(Keycode::Up), .. } => gb.set_button(Button::UP, false),
            Event::KeyUp { keycode: Some(Keycode::Down), .. } => gb.set_button(Button::DOWN, false),
            Event::KeyUp { keycode: Some(Keycode::Left), .. } => gb.set_button(Button::LEFT, false),
            Event::KeyUp { keycode: Some(Keycode::Right), .. } => gb.set_button(Button::RIGHT, false),

            _ => {}
        }
    }
}
