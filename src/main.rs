use crate::{debug::Debugger, gameboy::GameBoy, mmu::io::joypad::Button};
use sdl2::{
    controller,
    controller::GameController,
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureAccess},
    video::Window,
    GameControllerSubsystem, Sdl,
};
use std::collections::HashMap;

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
const PALETTE: [u8; 4] = [0xFF, 0xA9, 0x54, 0x00];

fn main() {
    let mut gb = GameBoy::init(PATH);
    let (sdl, mut canvas) = init_renderer();

    let tex_creator = canvas.texture_creator();
    let mut tex = tex_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, 160, 144).unwrap();
    update_tex(&mut tex, &gb);

    let (ctrl, mut controllers) = init_ctrl(&sdl);

    let timer = sdl.timer().unwrap();
    let mut old_t = timer.ticks();
    let mut dt = 0;
    let mut cycle_count = 0;

    match DEBUG {
        true => {
            let mut dbg = Debugger::init();
            loop {
                dbg.prompt(&mut gb);
                handle_events(&sdl, &ctrl, &mut gb, &mut controllers);
                update_tex(&mut tex, &gb);
                canvas.copy(&tex, None, None).unwrap();
                canvas.present();
            }
        }
        false => loop {
            gb.cycles = 0;
            while gb.cycles < 70224 {
                gb.cpu_step();
            }
            handle_events(&sdl, &ctrl, &mut gb, &mut controllers);
            update_tex(&mut tex, &gb);
            canvas.copy(&tex, None, None).unwrap();
            canvas.present();

            cycle_count += gb.cycles;
            let t = timer.ticks();
            dt += t - old_t;
            old_t = t;
            if dt >= 1000 {
                println!(
                    "Clock rate: {} ({:.1}%)",
                    cycle_count,
                    100.0 * (cycle_count as f64 / ((1 << 22) as f64) - 1.0)
                );
                dt = 0;
                cycle_count = 0;
            }
        },
    }
}

fn init_renderer() -> (Sdl, Canvas<Window>) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("UEPA-GB", 640, 576).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    (sdl, canvas)
}

fn init_ctrl(sdl: &Sdl) -> (GameControllerSubsystem, HashMap<u32, GameController>) {
    let ctrl = sdl.game_controller().unwrap();
    ctrl.load_mappings("gamecontrollerdb.txt").unwrap();
    let controllers = HashMap::new();
    (ctrl, controllers)
}

#[inline(always)]
fn update_tex(tex: &mut Texture, gb: &GameBoy) {
    let fb = gb.borrow_framebuffer();
    let mut tex_buf = [0; 160 * 144 * 3];

    for (i, pixel) in fb.iter().enumerate() {
        tex_buf[(i * 3) + 0] = PALETTE[*pixel as usize];
        tex_buf[(i * 3) + 1] = PALETTE[*pixel as usize];
        tex_buf[(i * 3) + 2] = PALETTE[*pixel as usize];
    }

    tex.update(None, &tex_buf, 160 * 3).unwrap();
}

#[inline(always)]
fn handle_events(
    sdl: &Sdl,
    ctrl: &GameControllerSubsystem,
    gb: &mut GameBoy,
    controllers: &mut HashMap<u32, GameController>,
) {
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

            Event::ControllerButtonDown { button: controller::Button::A, .. } => gb.set_button(Button::A, true),
            Event::ControllerButtonDown { button: controller::Button::B, .. } => gb.set_button(Button::B, true),
            Event::ControllerButtonDown { button: controller::Button::Start, .. } => gb.set_button(Button::START, true),
            Event::ControllerButtonDown { button: controller::Button::Back, .. } => gb.set_button(Button::SELECT, true),
            Event::ControllerButtonDown { button: controller::Button::DPadUp, .. } => gb.set_button(Button::UP, true),
            Event::ControllerButtonDown { button: controller::Button::DPadDown, .. } => {
                gb.set_button(Button::DOWN, true)
            }
            Event::ControllerButtonDown { button: controller::Button::DPadLeft, .. } => {
                gb.set_button(Button::LEFT, true)
            }
            Event::ControllerButtonDown { button: controller::Button::DPadRight, .. } => {
                gb.set_button(Button::RIGHT, true)
            }

            Event::ControllerButtonUp { button: controller::Button::A, .. } => gb.set_button(Button::A, false),
            Event::ControllerButtonUp { button: controller::Button::B, .. } => gb.set_button(Button::B, false),
            Event::ControllerButtonUp { button: controller::Button::Start, .. } => gb.set_button(Button::START, false),
            Event::ControllerButtonUp { button: controller::Button::Back, .. } => gb.set_button(Button::SELECT, false),
            Event::ControllerButtonUp { button: controller::Button::DPadUp, .. } => gb.set_button(Button::UP, false),
            Event::ControllerButtonUp { button: controller::Button::DPadDown, .. } => {
                gb.set_button(Button::DOWN, false)
            }
            Event::ControllerButtonUp { button: controller::Button::DPadLeft, .. } => {
                gb.set_button(Button::LEFT, false)
            }
            Event::ControllerButtonUp { button: controller::Button::DPadRight, .. } => {
                gb.set_button(Button::RIGHT, false)
            }

            Event::ControllerDeviceAdded { which, .. } => {
                controllers.insert(which, ctrl.open(which).unwrap());
                println!("Inserted controller {}", which);
            }
            Event::ControllerDeviceRemoved { which, .. } => {
                controllers.remove(&which);
                println!("Removed controller {}", which);
            }

            _ => {}
        }
    }
}
