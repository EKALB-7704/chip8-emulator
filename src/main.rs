mod cpu;

use cpu::Cpu;
use minifb::{Key, Window, WindowOptions};
use rodio::{Sink, source::SineWave, Source};
use std::env;
use std::fs;
use std::time::{Duration, Instant};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 10;

fn main() {

    let (_stream, _handle, sink) = create_beep();
    let mut beep_playing = false;

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



    // framebuffer: minifb wants a Vec<u32> of packed 0xRRGGBB pixels
    let mut fb = vec![0u32; WIDTH * SCALE * HEIGHT * SCALE];


    const CPU_HZ: u64 = 500;
    const TIMER_HZ: u64 = 60;

    let cpu_interval = Duration::from_nanos(1_000_000_000 / CPU_HZ);
    let timer_interval = Duration::from_nanos(1_000_000_000 / TIMER_HZ);

    let mut last_cpu = Instant::now();
    let mut last_timer = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        if last_cpu.elapsed() >= cpu_interval {
            cpu.tick();
            last_cpu = Instant::now();
        }
        
        if last_timer.elapsed() >= timer_interval {
            cpu.tick_timers();

            if cpu.sound_timer > 0 {
                if !beep_playing {
                    sink.append(SineWave::new(440.0).amplify(0.2).repeat_infinite());
                    beep_playing = true;
                }
            }
            else {
                sink.clear();
                beep_playing = false;
            }

            handle_input(&mut cpu, &window);
            draw(&cpu, &mut fb);
            window.update_with_buffer(&fb, WIDTH * SCALE, HEIGHT * SCALE).unwrap();
            last_timer = Instant::now();
        }
        std::thread::sleep(Duration::from_micros(100));
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

fn create_beep() -> (rodio::OutputStream, rodio::OutputStreamHandle, rodio::Sink) {
    let (stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    (stream, handle, sink)
}