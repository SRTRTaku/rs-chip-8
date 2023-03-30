use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::iter::Iterator;

const MEMORY_SIZE: usize = 4096;
const V_SIZE: usize = 16;
const GFX_SIZE_COL: usize = 64;
const GFX_SIZE_ROW: usize = 32;
const GFX_SIZE: usize = GFX_SIZE_COL * GFX_SIZE_ROW;
const STACK_SIZE: usize = 16;
const KEY_NUM: usize = 16;

const CHIP8_FONTSET: [u8; 80] = [
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

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    v: [u8; V_SIZE],
    i: u16,
    pc: u16,
    gfx: [u8; GFX_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
    sp: u16,
    key: [u8; KEY_NUM],
    draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut initial_memory = [0; MEMORY_SIZE];
        for i in 0..80 {
            initial_memory[i] = CHIP8_FONTSET[i];
        }
        Chip8 {
            memory: initial_memory,
            v: [0; V_SIZE],
            i: 0,      // Reset inex reister
            pc: 0x200, // Program cunter starts at 0x200
            gfx: [0; GFX_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            sp: 0, // Rese stack posinter
            key: [0; KEY_NUM],
            draw_flag: false,
        }
    }

    pub fn load_game(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let buf = BufReader::new(File::open(filename)?);
        for (i, byte_or_error) in buf.bytes().enumerate() {
            let byte = byte_or_error.unwrap();
            self.memory[0x200 + i] = byte;
        }
        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        // Fetch Opcode
        let opcode: u16 = {
            let m0 = self.memory[self.pc as usize] as u16;
            let m1 = self.memory[self.pc as usize + 1] as u16;
            m0 << 8 | m1
        };

        // Decode Opcode
        // Execute Opcode
        if let Err(e) = self.decode_execute(opcode) {
            self.dump();
            panic!("decode_execute: {}", e);
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }
    fn decode_execute(&mut self, opcode: u16) -> Result<(), String> {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => {
                    // 0x00E0: Clears the screen
                    self.gfx = [0; GFX_SIZE];
                    self.draw_flag = true;
                    self.pc += 2;
                }
                0x00EE => {
                    // 0x00EE: Returns from a subroutine
                    // pop
                    self.sp -= 1;
                    let pc = self.stack[self.sp as usize];
                    // update
                    self.pc = pc
                }
                _ => {
                    return Err(format!("unknown opcode(0x0000): 0x{:x}", opcode));
                }
            },
            0x1000 => {
                // 0x1NNN: Jumps to address NNN
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }
            0x2000 => {
                // 0x2NNN: Calls  subroutine at NNN
                // push
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                // update
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }
            0x3000 => {
                // 0x3XNN: Skips the next instrunction if VX == NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] == nn {
                    self.pc += 4; // skip
                } else {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // 0x4XNN: Skips the next instrunction if VX != NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] != nn {
                    self.pc += 4; // skip
                } else {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // 0x5XY0: Skips the next instrunction if VX == VY
                if opcode & 0x000F != 0x0000 {
                    return Err(format!("unknown opcode(0x5000): 0x{:x}", opcode));
                }
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] == self.v[y] {
                    self.pc += 4; // skip
                } else {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // 0x6XNN: Sets VX to NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = nn;
                self.pc += 2;
            }
            0x7000 => {
                // 0x7XNN: Adds NN to VX
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] += nn;
                self.pc += 2;
            }
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    0x0000 => {
                        // 0x8XY0: Sets VX to the value of VY
                        self.v[x] = self.v[y];
                    }
                    0x0001 => {
                        // 0x8XY1: Sets VX to VX or VY
                        self.v[x] |= self.v[y];
                    }
                    0x0002 => {
                        // 0x8XY2: Sets VX to VX and VY
                        self.v[x] &= self.v[y];
                    }
                    0x0003 => {
                        // 0x8XY3: Sets VX to VX xor VY
                        self.v[x] ^= self.v[y];
                    }
                    0x0004 => {
                        // 0x8XY4: Add VY to VX with carry
                        self.v[0xf] = if self.v[x] > 0xFF - self.v[y] { 1 } else { 0 };
                        self.v[x] += self.v[y];
                    }
                    0x0005 => {
                        // 0x8XY5: VY is subtracted from VX with carry
                        self.v[0xf] = if self.v[x] < self.v[y] { 1 } else { 0 };
                        self.v[x] -= self.v[y];
                    }
                    0x0006 => {
                        // 0x8XY6: Stores the least significant bit of VX in VF and VX >>= 1
                        self.v[0xf] = self.v[x] & 0x01;
                        self.v[x] >>= 1;
                    }
                    0x0007 => {
                        // 0x8XY7: Sets VX to VY minus VX with carry
                        self.v[0xf] = if self.v[y] < self.v[x] { 1 } else { 0 };
                        self.v[x] = self.v[y] - self.v[x];
                    }
                    0x000E => {
                        // 0x8XYE: Stores the most significant bit of VX in VF and VX <<= 1
                        self.v[0xf] = (self.v[x] & 0x80) >> 7;
                        self.v[x] <<= 1;
                    }
                    _ => {
                        return Err(format!("unknown opcode(0x8000): 0x{:x}", opcode));
                    }
                };
                self.pc += 2;
            }
            0x9000 => {
                todo!()
            }
            0xA000 => {
                // ANNN: Set I to the address NNN
                let nnn = opcode & 0x0FFF;
                self.i = nnn;
                self.pc += 2;
            }
            0xB000 => {
                todo!()
            }
            0xC000 => {
                todo!()
            }
            0xD000 => {
                todo!()
            }
            0xE000 => {
                todo!()
            }
            0xF000 => {
                todo!()
            }
            _ => {
                return Err(format!("unknown opcode: 0x{:x}", opcode));
            }
        };
        Ok(())
    }

    pub fn draw_flag(&mut self) -> bool {
        let f = self.draw_flag;
        self.draw_flag = false;
        f
    }

    pub fn draw_graphics(&self) {
        print!("\x1b[2;1H");
        print!("\x1b[0J");
        for _i in 0..GFX_SIZE_ROW {
            for _j in 0..GFX_SIZE_COL {
                let idx = _i * GFX_SIZE_ROW + _j;
                if self.gfx[idx] == 1 {
                    print!("*");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }

    pub fn dump(&self) {
        println!("memory:");
        let begin = 0x100;
        let end = 0x400;
        // print header
        print!("    |");
        for i in 0..16 {
            print!("{:3x}", i);
        }
        println!();

        for row in (begin / 16)..(MEMORY_SIZE / 16) {
            let offset = row * 16;
            print!("{:03x} |", offset);
            for i in 0..16 {
                print!(" {:02x}", self.memory[offset + i]);
            }
            println!();

            // omit
            if offset > end {
                break;
            }
        }

        println!("registers:");
        for i in 0..V_SIZE {
            print!("[{:2}] ", i);
        }
        println!();
        for i in 0..V_SIZE {
            print!("{:4} ", self.v[i]);
        }
        println!();

        println!("stack:");
        println!(" sp: {}", self.sp);
        for i in 0..STACK_SIZE {
            print!("[{:2}]  ", i);
        }
        println!();
        for i in 0..STACK_SIZE {
            print!("0x{:03x} ", self.stack[i]);
        }
        println!();

        // others
        println!("others:");
        print!("i: {}", self.i);
        print!(", pc: {} (0x{:x})", self.pc, self.pc);
        print!(", delay_timer: {}", self.delay_timer);
        print!(", sound_timer: {}", self.sound_timer);
        println!();
    }
}

pub fn setup_graphics() {
    print!("\x1b[2J");
}
