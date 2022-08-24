use crate::{debug::Debugger, gameboy::GameBoy, mmu::io::joypad::Button};
use sdl2::{event::Event, keyboard::Scancode, pixels::Color, render::Canvas, video::Window, Sdl, VideoSubsystem};

mod cpu;
mod debug;
mod gameboy;
mod mmu;
mod test;

extern crate num;
extern crate num_derive;
extern crate paste;
extern crate sdl2;
extern crate snafu;

const PATH: &str = "src/test/gb-test-roms/cpu_instrs/cpu_instrs.gb";

fn main() {
    let mut emu = Emulator::init();
    emu.run(false);
}

#[allow(unused)]
struct Emulator {
    gb: GameBoy,
    sdl: Sdl,
    video: VideoSubsystem,
    canvas: Canvas<Window>,
}

impl Emulator {
    fn init() -> Self {
        let gb = GameBoy::init(PATH);
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        let window = video.window("UEPA-GB", 640, 576).resizable().position_centered().build().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));

        Self { gb, sdl, video, canvas }
    }

    fn run(&mut self, debug: bool) {
        loop {
            match debug {
                true => {
                    let mut dbg = Debugger::init();
                    loop {
                        dbg.prompt(&mut self.gb);
                        self.sdl_update();
                    }
                }
                false => loop {
                    self.gb.cpu_step();
                    self.sdl_update();
                },
            }
        }
    }

    #[inline(always)]
    fn sdl_update(&mut self) {
        self.handle_events();
        self.collect_input();
        self.canvas.present();
    }

    #[inline(always)]
    fn handle_events(&self) {
        for event in self.sdl.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { timestamp: _ } => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    #[inline(always)]
    fn collect_input(&mut self) {
        let e = self.sdl.event_pump().unwrap();

        self.gb.set_button(Button::A, e.keyboard_state().is_scancode_pressed(Scancode::Z));
        self.gb.set_button(Button::B, e.keyboard_state().is_scancode_pressed(Scancode::X));
        self.gb.set_button(Button::START, e.keyboard_state().is_scancode_pressed(Scancode::Return));
        self.gb.set_button(Button::SELECT, e.keyboard_state().is_scancode_pressed(Scancode::Backspace));
        self.gb.set_button(Button::UP, e.keyboard_state().is_scancode_pressed(Scancode::Up));
        self.gb.set_button(Button::DOWN, e.keyboard_state().is_scancode_pressed(Scancode::Down));
        self.gb.set_button(Button::LEFT, e.keyboard_state().is_scancode_pressed(Scancode::Left));
        self.gb.set_button(Button::RIGHT, e.keyboard_state().is_scancode_pressed(Scancode::Right));
    }
}
