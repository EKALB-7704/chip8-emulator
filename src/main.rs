mod cpu;

use cpu::Cpu;
use minifb::{Key, Window, WindowOptions};
use rodio::{Sink, source::SineWave, Source};
use std::env;
use std::fs;
use std::time::{Duration, Instant};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;


fn main() {

    let (_stream, _handle, sink) = create_beep();
    let mut beep_playing = false;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: chip8 <rom> [scale] [fg] [bg]");
        eprintln!(" scale: integer (default 10)");
        eprintln!(" fg/bg: hex colour e.g. FFFFFF (default FFFFFF/000000)");
        return;
    }

    let scale = args.get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let fg = u32::from_str_radix(args.get(3).map(|s| s.as_str()).unwrap_or("FFFFFF"), 16)
        .unwrap_or(0xFFFFFF);

    let bg = u32::from_str_radix(args.get(4).map(|s| s.as_str()).unwrap_or("000000"), 16)
        .unwrap_or(0x000000);

    let rom = fs::read(&args[1]).expect("failed to read ROM");
    let mut cpu = Cpu::new();
    cpu.load_rom(&rom);

    if args.iter().any(|a| a == "--chip48") {
        cpu.quirks = cpu::Quirks::chip48();
    }

    let mut window = Window::new(
        "CHIP-8",
        WIDTH * scale,
        HEIGHT * scale,
        WindowOptions::default(),
    ).expect("failed to create window");



    // framebuffer: minifb wants a Vec<u32> of packed 0xRRGGBB pixels
    let mut fb = vec![0u32; WIDTH * scale * HEIGHT * scale];


    const CPU_HZ: u64 = 500;
    const TIMER_HZ: u64 = 60;

    let cpu_interval = Duration::from_nanos(1_000_000_000 / CPU_HZ);
    let timer_interval = Duration::from_nanos(1_000_000_000 / TIMER_HZ);

    let mut last_cpu = Instant::now();
    let mut last_timer = Instant::now();

    let mut paused = false;



    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            paused = !paused;
        }

        if paused && window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            cpu.tick();
        }

        if !paused && last_cpu.elapsed() >= cpu_interval {
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
            draw(&cpu, &mut fb, scale, fg, bg);
            let title = if paused {"CHIP-8 [PAUSED]"} else { "CHIP-8" };
            window.set_title(title);
            window.update_with_buffer(&fb, WIDTH * scale, HEIGHT * scale).unwrap();
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

fn draw(cpu: &Cpu, fb: &mut Vec<u32>, scale: usize, fg: u32, bg: u32 ) {
    for (i, &pixel) in cpu.display.iter().enumerate() {
        let x = i % 64;
        let y = i / 64;

        let colour = if pixel { fg } else { bg };

        for dy in 0..scale {
            for dx in 0..scale {
                let fx = x * scale + dx;
                let fy = y * scale + dy;
                fb[fy * WIDTH * scale + fx] = colour;
            }
        }
    }
}

fn create_beep() -> (rodio::OutputStream, rodio::OutputStreamHandle, rodio::Sink) {
    let (stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    (stream, handle, sink)
}