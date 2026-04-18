use rand;


const FONT_START: usize = 0x050;

// Each digit 0-F is 5 bytes. Each byte is one row; only the high 4 bits matter.
// Example: '0' draws as a 4-wide, 5-tall rectangle.
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Cpu {
    // 4KB of RAM. Programs load at 0x200; 0x000–0x1FF reserved for interpreter.
    pub ram: [u8; 4096],

    // 16 general-purpose 8-bit registers: V0 through VF.
    // VF doubles as a flag register (carry, borrow, collision).
    pub v: [u8; 16],

    // 16-bit index register — points at memory addresses (used by draw/load ops).
    pub i: u16,

    // Program counter — address of the next instruction to fetch.
    pub pc: u16,

    // Call stack. CHIP-8 supports 16 levels of nested subroutine calls.
    pub stack: [u16; 16],
    pub sp: u8, // stack pointer

    // Timers count down at 60Hz. Sound buzzes while sound_timer > 0.
    pub delay_timer: u8,
    pub sound_timer: u8,

    // 64x32 display. true = pixel on.
    pub display: [bool; 64 * 32],

    // 16-key hex keypad state. keys[0xF] = key F, etc.
    pub keys: [bool; 16],
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            ram: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keys: [false; 16],
        };
        cpu.ram[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);
        cpu
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.ram[0x200..0x200 + data.len()].copy_from_slice(data);
    }

    pub fn tick(&mut self) {
        // Fetch read 2 bytes and combine into one u16 opcode
        let hi = self.ram[self.pc as usize] as u16;
        let lo = self.ram[self.pc as usize + 1] as u16;
        let opcode = (hi << 8) | lo;
        self.pc += 2;

        // Decode: crack the opcode into its parts
        let nnn = opcode & 0x0FFF; // 12-bit address
        let nn  = (opcode & 0x00FF) as u8; // 8-bit constant
        let n = (opcode & 0x000F) as u8; // 4-bit nibble
        let x = ((opcode >> 8) & 0xF) as usize; // register index
        let y = ((opcode >> 4) & 0xF) as usize; // register index

        // Execute 
        match (opcode >> 12) & 0xF {

            0x0 => match opcode{
                0x00E0 => self.display = [false; 64 * 32], // clear screen
                0x00EE => {                                // return from subroutine
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {} //0NNN (call machine code) - safe to ignore

            },
            0x1 => self.pc = nnn,               // jump to NNN
            0x2 => {                            // call subroutine at NNN
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            0x3 => if self.v[x] == nn { self.pc += 2 },        //skip if Vx == NN
            0x4 => if self.v[x] != nn { self.pc += 2 },        //skip if Vx != NN
            0x5 => if self.v[x] == self.v[y] { self.pc += 2 }, //skip if Vx == Vy
            0x9 => if self.v[x] != self.v[y] { self.pc += 2 }, //skip if Vx != Vy
            0x6 => self.v[x] = nn,                             //set Vx = NN
            0x7 => self.v[x] = self.v[x].wrapping_add(nn),     //Vx += NN (no carry flag)
            0x8 => match n {
                    0x0 => self.v[x] = self.v[y],                    // Vx = Vy
                    0x1 => self.v[x] |= self.v[y],                   // Vx |= Vy
                    0x2 => self.v[x] &= self.v[y],                   // Vx &= Vy
                    0x3 => self.v[x] ^= self.v[y],                   // Vx ^= Vy
                    0x4 => {                                          // Vx += Vy, VF = carry
                        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = result;
                        self.v[0xF] = carry as u8;
                    }
                    0x5 => {                                          // Vx -= Vy, VF = NOT borrow
                        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = result;
                        self.v[0xF] = !borrow as u8;
                    }
                    0x6 => {                                          // Vx >>= 1, VF = shifted bit
                        self.v[0xF] = self.v[x] & 0x1;
                        self.v[x] >>= 1;
                    }
                    0x7 => {                                          // Vx = Vy - Vx, VF = NOT borrow
                        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = result;
                        self.v[0xF] = !borrow as u8;
                    }
                    0xE => {                                          // Vx <<= 1, VF = shifted bit
                        self.v[0xF] = (self.v[x] >> 7) & 0x1;
                        self.v[x] <<= 1;
                    }
                    _ => {}
                },
            0xA => self.i = nnn,                            // set I = NNN
            0xB => self.pc = nnn + self.v[0] as u16,        // jump to NNN + V0
            0xC => self.v[x] = rand::random::<u8>() & nn,   // Vx = random & NN
            0xD => {
                let x_pos = self.v[x] as usize % 64;
                let y_pos = self.v[y] as usize % 32;
                self.v[0xF] = 0;

                for row in 0..n as usize {
                    let sprite_byte = self.ram[self.i as usize + row];
                    let py = (y_pos + row) % 32;

                    for col in 0..8 {
                        let px = (x_pos + col) % 64;
                        let sprite_bit = (sprite_byte >> (7 - col)) & 0x1;

                        if sprite_bit == 1 {
                            let idx = py * 64 + px;
                            if self.display[idx] {
                                self.v[0xF] = 1; // collision
                            }
                            self.display[idx] ^= true;
                        }
                    }
                }
            },
            0xE => match nn {
                0x9E => if self.keys[self.v[x] as usize] { self.pc += 2},  // skip if key Vx pressed
                0xA1 => if !self.keys[self.v[x] as usize] { self.pc += 2}, // skip if key Vx not pressed
                _ => {} 
            },
            0xF => match nn {
                0x07 => self.v[x] = self.delay_timer,                   // Vx = delay timer
                0x15 => self.delay_timer = self.v[x],                   // delay timer = Vx
                0x18 => self.sound_timer = self.v[x],                   // sound timer = Vx
                0x1E => self.i += self.v[x] as u16,                     // I += Vx
                0x0A => {                                               // wait for key press, store in Vx
                    match self.keys.iter().position(|&k| k) {
                        Some(key) => self.v[x] = key as u8,
                        None => self.pc -= 2,                           // rwind PC to repeat this instruction
                    }
                }
                0x29 => self.i = FONT_START as u16 + self.v[x] as u16 * 5, // I = font sprite for Vx
                0x33 => {                                                  // BCD encode Vx into ram[I..I+3]
                    self.ram[self.i as usize]     = self.v[x] / 100;
                    self.ram[self.i as usize + 1] = (self.v[x] / 10)%10;
                    self.ram[self.i as usize + 2] = self.v[x] % 10;
                }
                0x55 => {                                                  // store V0..Vx in RAM starting at I
                    for idx in 0..=x {
                        self.ram[self.i as usize + idx] = self.v[idx];
                    }
                }
                0x65 => {                                                  // load V0..Vx from RAM starting at I
                    for idx in 0..=x {
                        self.v[idx] = self.ram[self.i as usize + idx];
                    }
                }
                _ => {}
            },
            _ => todo!("opcode {opcode:#06X}"),
        }
    }
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {self.delay_timer -= 1 }
        if self.sound_timer > 0 {self.sound_timer -= 1 }
    }
}