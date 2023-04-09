use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use sdl2::EventPump;

use crate::chip8::{Chip8, KeyBoard, GFX_SIZE_COL, GFX_SIZE_ROW};

const WHITE: Color = Color::RGB(0xe0, 0xf8, 0xd0);
const BLACK: Color = Color::RGB(0x08, 0x18, 0x20);
const PIXEL_SIZE: u32 = 10;

pub struct IO {
    canvas: WindowCanvas,
    event_pump: EventPump,
}
impl IO {
    pub fn setup() -> IO {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "rust-sdl2 demo",
                GFX_SIZE_COL as u32 * PIXEL_SIZE,
                GFX_SIZE_ROW as u32 * PIXEL_SIZE,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut _canvas = window.into_canvas().build().unwrap();
        let mut _event_pump = sdl_context.event_pump().unwrap();

        _canvas.set_draw_color(WHITE);
        _canvas.clear();
        _canvas.present();
        IO {
            canvas: _canvas,
            event_pump: _event_pump,
        }
    }
    pub fn draw_graphics(&mut self, chip8: &Chip8) {
        self.canvas.set_draw_color(WHITE);
        self.canvas.clear();
        self.canvas.set_draw_color(BLACK);
        for y in 0..GFX_SIZE_ROW {
            for x in 0..GFX_SIZE_COL {
                let _x = (x * PIXEL_SIZE as usize) as i32;
                let _y = (y * PIXEL_SIZE as usize) as i32;
                if chip8.gfx[y * GFX_SIZE_COL + x] == 1 {
                    self.canvas
                        .fill_rect(Rect::new(_x, _y, PIXEL_SIZE, PIXEL_SIZE))
                        .unwrap();
                } else {
                    self.canvas.draw_point(Point::new(_x, _y)).unwrap();
                }
            }
        }
        self.canvas.present();
    }
    pub fn set_key(&mut self, kb: &mut KeyBoard) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => kb.fin_flag = true,
                Event::KeyDown {
                    keycode: Some(key_code),
                    ..
                } => match key_code {
                    Keycode::X => kb.key[0x0] = 1,
                    Keycode::Num1 => kb.key[0x1] = 1,
                    Keycode::Num2 => kb.key[0x2] = 1,
                    Keycode::Num3 => kb.key[0x3] = 1,
                    Keycode::Q => kb.key[0x4] = 1,
                    Keycode::W => kb.key[0x5] = 1,
                    Keycode::E => kb.key[0x6] = 1,
                    Keycode::A => kb.key[0x7] = 1,
                    Keycode::S => kb.key[0x8] = 1,
                    Keycode::D => kb.key[0x9] = 1,
                    Keycode::Z => kb.key[0xa] = 1,
                    Keycode::C => kb.key[0xb] = 1,
                    Keycode::Num4 => kb.key[0xc] = 1,
                    Keycode::R => kb.key[0xd] = 1,
                    Keycode::F => kb.key[0xe] = 1,
                    Keycode::V => kb.key[0xf] = 1,
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(key_code),
                    ..
                } => match key_code {
                    Keycode::X => kb.key[0x0] = 0,
                    Keycode::Num1 => kb.key[0x1] = 0,
                    Keycode::Num2 => kb.key[0x2] = 0,
                    Keycode::Num3 => kb.key[0x3] = 0,
                    Keycode::Q => kb.key[0x4] = 0,
                    Keycode::W => kb.key[0x5] = 0,
                    Keycode::E => kb.key[0x6] = 0,
                    Keycode::A => kb.key[0x7] = 0,
                    Keycode::S => kb.key[0x8] = 0,
                    Keycode::D => kb.key[0x9] = 0,
                    Keycode::Z => kb.key[0xa] = 0,
                    Keycode::C => kb.key[0xb] = 0,
                    Keycode::Num4 => kb.key[0xc] = 0,
                    Keycode::R => kb.key[0xd] = 0,
                    Keycode::F => kb.key[0xe] = 0,
                    Keycode::V => kb.key[0xf] = 0,
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
