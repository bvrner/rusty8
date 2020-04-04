use rusty8::cpu::CPU;

use std::env;
use std::fs::File;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::TextureAccess;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No rom passed in.");
        return Ok(());
    }

    let rom_path = &args[1];

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video
        .window("Rusty 8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.set_logical_size(800, 600).unwrap();

    let mut texture = texture_creator
        .create_texture(PixelFormatEnum::ARGB8888, TextureAccess::Streaming, 64, 32)
        .unwrap();

    let mut file = File::open(rom_path)?;
    let mut chip8 = CPU::init();
    chip8.load_rom(&mut file)?;

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut buf = [0_u32; 2048]; // temp texture buffer

    'running: loop {
        chip8.cycle();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Num1) => chip8.keyboard[0x1] = true,
                    Some(Keycode::Num2) => chip8.keyboard[0x2] = true,
                    Some(Keycode::Num3) => chip8.keyboard[0x3] = true,
                    Some(Keycode::Num4) => chip8.keyboard[0xC] = true,
                    Some(Keycode::Q) => chip8.keyboard[0x4] = true,
                    Some(Keycode::W) => chip8.keyboard[0x5] = true,
                    Some(Keycode::E) => chip8.keyboard[0x6] = true,
                    Some(Keycode::R) => chip8.keyboard[0xD] = true,
                    Some(Keycode::A) => chip8.keyboard[0x7] = true,
                    Some(Keycode::S) => chip8.keyboard[0x8] = true,
                    Some(Keycode::D) => chip8.keyboard[0x9] = true,
                    Some(Keycode::F) => chip8.keyboard[0xE] = true,
                    Some(Keycode::Z) => chip8.keyboard[0xA] = true,
                    Some(Keycode::X) => chip8.keyboard[0x0] = true,
                    Some(Keycode::C) => chip8.keyboard[0xB] = true,
                    Some(Keycode::V) => chip8.keyboard[0xF] = true,
                    _ => {}
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(Keycode::Num1) => chip8.keyboard[0x1] = false,
                    Some(Keycode::Num2) => chip8.keyboard[0x2] = false,
                    Some(Keycode::Num3) => chip8.keyboard[0x3] = false,
                    Some(Keycode::Num4) => chip8.keyboard[0xC] = false,
                    Some(Keycode::Q) => chip8.keyboard[0x4] = false,
                    Some(Keycode::W) => chip8.keyboard[0x5] = false,
                    Some(Keycode::E) => chip8.keyboard[0x6] = false,
                    Some(Keycode::R) => chip8.keyboard[0xD] = false,
                    Some(Keycode::A) => chip8.keyboard[0x7] = false,
                    Some(Keycode::S) => chip8.keyboard[0x8] = false,
                    Some(Keycode::D) => chip8.keyboard[0x9] = false,
                    Some(Keycode::F) => chip8.keyboard[0xE] = false,
                    Some(Keycode::Z) => chip8.keyboard[0xA] = false,
                    Some(Keycode::X) => chip8.keyboard[0x0] = false,
                    Some(Keycode::C) => chip8.keyboard[0xB] = false,
                    Some(Keycode::V) => chip8.keyboard[0xF] = false,
                    _ => {}
                },
                _ => {}
            }
        }

        if chip8.draw_flag {
            canvas.clear();

            for (i, p) in buf.iter_mut().enumerate() {
                *p = (0x00FF_FFFF * chip8.gfx[i] as u32) | 0xFF00_0000;
            }

            // let the gambiarra reign supreme
            let ptr = buf.as_ptr().cast::<u8>();
            let nbuf = unsafe { std::slice::from_raw_parts(ptr, buf.len()) };

            texture
                .update(None, &nbuf, 64 * std::mem::size_of::<u32>())
                .unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            chip8.draw_flag = false;
        }

        std::thread::sleep(Duration::from_micros(1200));
    }

    Ok(())
}
