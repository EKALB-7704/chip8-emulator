mod cpu;

use cpu::Cpu;
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 10;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: chip8 <rom>");
        return;
    }

    let rom = fs::read(&args[1]).expect("failed to read ROM");
    let mut cpu = Cpu::new();
    cpu.load_rom(&rom);

    let mut window = Window::new(
        "CHIP-8",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    ).expect("failed to create window");

    window.set_target_fps(60);

    // framebuffer: minifb wants a Vec<u32> of packed 0xRRGGBB pixels
    let mut fb = vec![0u32; WIDTH * SCALE * HEIGHT * SCALE];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // run ~8 CPU ticks per frame (500Hz / 60 fps ~= 8)
        for _ in 0..8 {
            cpu.tick();
        }
        cpu.tick_timers();

        handle_input(&mut cpu, &window);
        draw(&cpu, &mut fb);
        window.update_with_buffer(&fb, WIDTH * SCALE, HEIGHT * SCALE).unwrap();
    }
}

fn handle_input(cpu: &mut Cpu, window: &Window) {
    let key_map = [
        Key::X,     // 0
        Key::Key1,  // 1
        Key::Key2,  // 2
        Key::Key3,  // 3
        Key::Q,     // 4
        Key::W,     // 5
        Key::E,     // 6
        Key::A,     // 7
        Key::S,     // 8
        Key::D,     // 9
        Key::Z,     // A
        Key::C,     // B
        Key::Key4,  // C
        Key::R,     // D
        Key::F,     // E
        Key::V,     // F
    ];

    for (chip8_key, host_key) in key_map.iter().enumerate() {
        cpu.keys[chip8_key] = window.is_key_down(*host_key);
    }
}

fn draw(cpu: &Cpu, fb: &mut Vec<u32>) {
    for (i, &pixel) in cpu.display.iter().enumerate() {
        let x = i % 64;
        let y = i / 64;

        let colour = if pixel { 0xFFFFFF } else { 0x000000 };

        for dy in 0..SCALE {
            for dx in 0..SCALE {
                let fx = x * SCALE + dx;
                let fy = y * SCALE + dy;
                fb[fy * WIDTH * SCALE + fx] = colour;
            }
        }
    }
}