use getch_rs::{Getch, Key};
use rand::Rng;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

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
    draw_flag: bool,
}

pub struct KeyBoard {
    pub fin_flag: bool,
    pub key: [u8; KEY_NUM],
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

    pub fn emulate_cycle(&mut self, kb: &KeyBoard) {
        // Fetch Opcode
        let opcode: u16 = {
            let m0 = self.memory[self.pc as usize] as u16;
            let m1 = self.memory[self.pc as usize + 1] as u16;
            m0 << 8 | m1
        };

        // Decode Opcode
        // Execute Opcode
        if let Err(e) = self.decode_execute(opcode, kb) {
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
    fn decode_execute(&mut self, opcode: u16, kb: &KeyBoard) -> Result<(), String> {
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
                    self.pc = pc + 2;
                }
                _ => return Err(format!("unknown opcode(0x0000): 0x{:x}", opcode)),
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
                let (ans, _) = self.v[x].overflowing_add(nn);

                self.v[x] = ans;
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
                        let (ans, ovfl) = self.v[x].overflowing_add(self.v[y]);
                        self.v[0xf] = if ovfl { 1 } else { 0 };
                        self.v[x] = ans;
                    }
                    0x0005 => {
                        // 0x8XY5: VY is subtracted from VX with carry
                        let (ans, ovfl) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[0xf] = if ovfl { 1 } else { 0 };
                        self.v[x] = ans;
                    }
                    0x0006 => {
                        // 0x8XY6: Stores the least significant bit of VX in VF and VX >>= 1
                        self.v[0xf] = self.v[x] & 0x01;
                        self.v[x] >>= 1;
                    }
                    0x0007 => {
                        // 0x8XY7: Sets VX to VY minus VX with carry
                        let (ans, ovfl) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[0xf] = if ovfl { 1 } else { 0 };
                        self.v[x] = ans;
                    }
                    0x000E => {
                        // 0x8XYE: Stores the most significant bit of VX in VF and VX <<= 1
                        self.v[0xf] = (self.v[x] & 0x80) >> 7;
                        self.v[x] <<= 1;
                    }
                    _ => return Err(format!("unknown opcode(0x8000): 0x{:x}", opcode)),
                }
                self.pc += 2;
            }
            0x9000 => {
                // 0x9XY0: Skips the next instrunction if VX != VY
                if opcode & 0x000F != 0x0000 {
                    return Err(format!("unknown opcode(0x9000): 0x{:x}", opcode));
                }
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 4; // skip
                } else {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // 0xANNN: Set I to the address NNN
                let nnn = opcode & 0x0FFF;
                self.i = nnn;
                self.pc += 2;
            }
            0xB000 => {
                // 0xBNNN: Jumps to address NNN plus V0
                let nnn = opcode & 0x0FFF;
                let v0 = self.v[0] as u16;
                self.pc = v0 + nnn;
            }
            0xC000 => {
                // 0xCXNN: Sets VX to the bitwise and operation on an random number and NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                let r = rand::thread_rng().gen_range(1..=255);
                self.v[x] = r & nn;
                self.pc += 2;
            }
            0xD000 => {
                // 0xDXYN:
                // Draws a sprite at coordinate (VX, VY)
                // that has a width of 8 pixels and a height of N pixels.
                // Each row of 8 pixels is read as bit-coded starting
                // from memory location I;
                // I value does not change after the execution of this instruction.
                // As described above,
                // VF is set to 1 if any screen pixels are flipped
                // from set to unset when the sprite is drawn,
                // and to 0 if that does not happen.
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let n = (opcode & 0x000F) as usize;
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;
                let mut vf = 0;
                for yline in 0..n {
                    let pixel = self.memory[self.i as usize + yline];
                    for xline in 0..8 {
                        if pixel & (0x80 >> xline) != 0 {
                            let index_x = (vx + xline) % GFX_SIZE_COL;
                            let index_y = (vy + yline) % GFX_SIZE_ROW;
                            let index = index_y * GFX_SIZE_COL + index_x;
                            if self.gfx[index] == 1 {
                                vf = 1;
                            }
                            self.gfx[index] ^= 1;
                        }
                    }
                }
                self.v[0xf] = vf;
                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x009E => {
                        // 0xEX9E: Skips the next instruction
                        // if the key stored in VX is pressed
                        if kb.key[self.v[x] as usize] != 0 {
                            self.pc += 4; // skip
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // 0xEXA1: Skips the next instruction
                        // if the key stored in VX is not pressed
                        if kb.key[self.v[x] as usize] == 0 {
                            self.pc += 4; // skip
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => return Err(format!("unknown opcode(0xE000): 0x{:x}", opcode)),
                }
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x0007 => {
                        // 0xFX07: Sets VX to the value of the delay timer
                        self.v[x] = self.delay_timer;
                        self.pc += 2;
                    }
                    0x000A => {
                        // 0xFX0A: A key press is awaited, and then stored in VX
                        for k in 0..KEY_NUM {
                            if kb.key[k] != 0 {
                                self.v[x] = k as u8;
                                self.pc += 2;
                                break;
                            }
                        }
                    }
                    0x0015 => {
                        // 0xFX15: Set delay timer to VX
                        self.delay_timer = self.v[x];
                        self.pc += 2;
                    }
                    0x0018 => {
                        // 0xFX18: Set sound timer to VX
                        self.sound_timer = self.v[x];
                        self.pc += 2;
                    }
                    0x001E => {
                        // 0xFX1E: Adds VX to I
                        self.i += self.v[x] as u16;
                        self.pc += 2;
                    }
                    0x0029 => {
                        // 0xFX29: Sets I to the location of the sprite for the character in VX
                        let c = self.v[x];
                        if c > 0xf {
                            return Err(format!("0xFX29 invalid character: {:x}", c));
                        }
                        self.i = (c as u16) * 5;
                        self.pc += 2;
                    }
                    0x0033 => {
                        // 0xFX33:
                        // Stores the binary-coded decimal representation of VX,
                        // with the hundreds digit in memory at location in I,
                        // the tens digit at location I+1, and the ones digit at location I+2.
                        let vx = self.v[x];
                        let i = self.i as usize;
                        self.memory[i] = vx / 100;
                        self.memory[i + 1] = (vx / 10) % 10;
                        self.memory[i + 2] = vx % 10;
                        self.pc += 2;
                    }
                    0x0055 => {
                        // 0xFX55:
                        // Stores from V0 to VX (including VX) in memory, starting at address I.
                        // The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                        for j in 0..=x {
                            self.memory[self.i as usize + j] = self.v[j];
                        }
                        self.pc += 2;
                    }
                    0x0065 => {
                        // 0xFX65:
                        // Fills from V0 to VX (including VX) with values from memory, starting at address I.
                        // The offset from I is increased by 1 for each value read, but I itself is left unmodified.
                        for j in 0..=x {
                            self.v[j] = self.memory[self.i as usize + j];
                        }
                        self.pc += 2;
                    }
                    _ => return Err(format!("unknown opcode(0xF000): 0x{:x}", opcode)),
                }
            }
            _ => return Err(format!("unknown opcode: 0x{:x}", opcode)),
        }
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
        for x in 0..GFX_SIZE_ROW {
            for y in 0..GFX_SIZE_COL {
                let idx = x * GFX_SIZE_COL + y;
                if self.gfx[idx] == 1 {
                    print!("\x1b[42m"); // green
                    print!("*");
                    print!("\x1b[0m"); // reset
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }

    pub fn dump(&self) {
        println!("memory:");
        let begin = 0x200;
        let end = 0x250;
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

impl KeyBoard {
    pub fn new() -> KeyBoard {
        KeyBoard {
            fin_flag: false,
            key: [0; KEY_NUM],
        }
    }
}

pub fn set_keys(kb: Arc<Mutex<KeyBoard>>) {
    let g = Getch::new();
    loop {
        let key = g.getch();
        {
            let mut key_board = kb.lock().unwrap();
            key_board.key = [0; KEY_NUM];
            match key {
                Ok(Key::Char('x')) => key_board.key[0x0] = 1,
                Ok(Key::Char('1')) => key_board.key[0x1] = 1,
                Ok(Key::Char('2')) => key_board.key[0x2] = 1,
                Ok(Key::Char('3')) => key_board.key[0x3] = 1,
                Ok(Key::Char('q')) => key_board.key[0x4] = 1,
                Ok(Key::Char('w')) => key_board.key[0x5] = 1,
                Ok(Key::Char('e')) => key_board.key[0x6] = 1,
                Ok(Key::Char('a')) => key_board.key[0x7] = 1,
                Ok(Key::Char('s')) => key_board.key[0x8] = 1,
                Ok(Key::Char('d')) => key_board.key[0x9] = 1,
                Ok(Key::Char('z')) => key_board.key[0xa] = 1,
                Ok(Key::Char('c')) => key_board.key[0xb] = 1,
                Ok(Key::Char('4')) => key_board.key[0xc] = 1,
                Ok(Key::Char('r')) => key_board.key[0xd] = 1,
                Ok(Key::Char('f')) => key_board.key[0xe] = 1,
                Ok(Key::Char('v')) => key_board.key[0xf] = 1,
                Ok(Key::Char(' ')) => {
                    key_board.fin_flag = true;
                    break;
                }
                _ => (),
            }
        }
    }
}

pub fn setup_graphics() {
    print!("\x1b[2J");
    print!("\x1b[2;1H");
    for _ in 0..GFX_SIZE_ROW {
        for _ in 0..GFX_SIZE_COL {
            print!(".");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_execute_00e0() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x00E0;
        chip8.gfx = [1; GFX_SIZE];

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!([0; GFX_SIZE], chip8.gfx);
        assert_eq!(true, chip8.draw_flag);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_00ee() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x00ee;
        chip8.sp = 16;
        chip8.stack[15] = 0x300;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(15, chip8.sp);
        assert_eq!(0x302, chip8.pc);
    }

    #[test]
    fn decode_execute_1nnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x1300;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x300, chip8.pc);
    }

    #[test]
    fn decode_execute_2nnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x2300;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(1, chip8.sp);
        assert_eq!(0x200, chip8.stack[0]);
        assert_eq!(0x300, chip8.pc);
    }

    #[test]
    fn decode_execute_3xnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x3123;

        chip8.v[1] = 0x00;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0x23;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_4xnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x4234;

        chip8.v[2] = 0x00;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x204, chip8.pc);

        chip8.v[2] = 0x34;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_5xy0() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x5670;

        chip8.v[6] = 0x12;
        chip8.v[7] = 0x34;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x202, chip8.pc);

        chip8.v[7] = 0x12;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_6xnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x6789;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x89, chip8.v[7]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_7xnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x7123;

        chip8.v[1] = 0x45;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x45 + 0x23, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0xff;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x22, chip8.v[1]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy0() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8120;
        chip8.v[1] = 0x12;
        chip8.v[2] = 0x34;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x34, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy1() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8121;
        chip8.v[1] = 0xf8;
        chip8.v[2] = 0x1f;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xff, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy2() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8122;
        chip8.v[1] = 0xf8;
        chip8.v[2] = 0x1f;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x18, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy3() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8123;
        chip8.v[1] = 0xac;
        chip8.v[2] = 0xca;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x66, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy4() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8124;

        chip8.v[1] = 0x80;
        chip8.v[2] = 0x7f;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xff, chip8.v[1]);
        assert_eq!(0, chip8.v[0xf]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0x80;
        chip8.v[2] = 0x80;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x00, chip8.v[1]);
        assert_eq!(1, chip8.v[0xf]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy5() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8125;

        chip8.v[1] = 0x01;
        chip8.v[2] = 0x01;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x00, chip8.v[1]);
        assert_eq!(0, chip8.v[0xf]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0x01;
        chip8.v[2] = 0x02;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xff, chip8.v[1]);
        assert_eq!(1, chip8.v[0xf]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy6() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8126;

        chip8.v[1] = 0xfe;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x7f, chip8.v[1]);
        assert_eq!(0, chip8.v[0xf]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0xff;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x7f, chip8.v[1]);
        assert_eq!(1, chip8.v[0xf]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_8xy7() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x8127;

        chip8.v[1] = 0x01;
        chip8.v[2] = 0x02;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x01, chip8.v[1]);
        assert_eq!(0, chip8.v[0xf]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0x02;
        chip8.v[2] = 0x01;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xff, chip8.v[1]);
        assert_eq!(1, chip8.v[0xf]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_8xye() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x812e;

        chip8.v[1] = 0x7f;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xfe, chip8.v[1]);
        assert_eq!(0, chip8.v[0xf]);
        assert_eq!(0x202, chip8.pc);

        chip8.v[1] = 0xff;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xfe, chip8.v[1]);
        assert_eq!(1, chip8.v[0xf]);
        assert_eq!(0x204, chip8.pc);
    }

    #[test]
    fn decode_execute_9xy0() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0x9670;

        chip8.v[6] = 0x12;
        chip8.v[7] = 0x34;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x204, chip8.pc);

        chip8.v[7] = 0x12;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_annn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xa123;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x123, chip8.i);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_bnnn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xb123;
        chip8.v[0] = 0x45;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x0045 + 0x0123, chip8.pc);
    }

    #[test]
    fn decode_execute_cxnn() {
        let k = KeyBoard::new();

        let opcode = 0xcd00;
        for _ in 0..10 {
            let mut chip8 = Chip8::new();
            chip8.decode_execute(opcode, &k).unwrap();
            println!("{}", chip8.v[0xd]);
            assert_eq!(0x00, chip8.v[0xd]);
            assert_eq!(0x202, chip8.pc);
        }

        let opcode = 0xcd0f;
        for _ in 0..10 {
            let mut chip8 = Chip8::new();
            chip8.decode_execute(opcode, &k).unwrap();
            println!("{}", chip8.v[0xd]);
            assert!(chip8.v[0xd] <= 0x0f);
            assert_eq!(0x202, chip8.pc);
        }

        let opcode = 0xcdff;
        for _ in 0..10 {
            let mut chip8 = Chip8::new();
            chip8.decode_execute(opcode, &k).unwrap();
            println!("{}", chip8.v[0xd]);
            assert_eq!(0x202, chip8.pc);
        }
    }

    #[test]
    fn decode_execute_dxyn() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xd125;
        chip8.v[1] = 0x01;
        chip8.v[2] = 0x02;
        chip8.i = 0x8 * 5;
        for y in 0..10 {
            for x in 3..10 {
                chip8.gfx[y * GFX_SIZE_COL + x] = 1;
            }
        }
        let mut des = chip8.gfx.clone();
        des[2 * GFX_SIZE_COL + 1] = 1;
        des[2 * GFX_SIZE_COL + 2] = 1;
        des[2 * GFX_SIZE_COL + 3] = 0;
        des[2 * GFX_SIZE_COL + 4] = 0;
        //
        des[3 * GFX_SIZE_COL + 1] = 1;
        des[3 * GFX_SIZE_COL + 4] = 0;
        //
        des[4 * GFX_SIZE_COL + 1] = 1;
        des[4 * GFX_SIZE_COL + 2] = 1;
        des[4 * GFX_SIZE_COL + 3] = 0;
        des[4 * GFX_SIZE_COL + 4] = 0;
        //
        des[5 * GFX_SIZE_COL + 1] = 1;
        des[5 * GFX_SIZE_COL + 4] = 0;
        //
        des[6 * GFX_SIZE_COL + 1] = 1;
        des[6 * GFX_SIZE_COL + 2] = 1;
        des[6 * GFX_SIZE_COL + 3] = 0;
        des[6 * GFX_SIZE_COL + 4] = 0;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(des, chip8.gfx);
        assert_eq!(0x202, chip8.pc);
        for y in 0..10 {
            let begin = y * GFX_SIZE_COL;
            println!("{:?}", &chip8.gfx[begin..(begin + 10)]);
        }
    }

    #[test]
    fn decode_execute_ex9e() {
        let mut chip8 = Chip8::new();
        let mut k = KeyBoard::new();
        let opcode = 0xe19e;
        chip8.v[1] = 0x0f;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x202, chip8.pc);

        k.key[0xf] = 1;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_exa1() {
        let mut chip8 = Chip8::new();
        let mut k = KeyBoard::new();
        let opcode = 0xe1a1;
        chip8.v[1] = 0x0f;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x204, chip8.pc);

        k.key[0xf] = 1;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x206, chip8.pc);
    }

    #[test]
    fn decode_execute_fx07() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf107;
        chip8.delay_timer = 0x12;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x12, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx0a() {
        let mut chip8 = Chip8::new();
        let mut k = KeyBoard::new();
        let opcode = 0xf10a;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x00, chip8.v[1]);
        assert_eq!(0x200, chip8.pc);

        k.key[0x8] = 1;
        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x08, chip8.v[1]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx15() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf115;
        chip8.v[1] = 0x12;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x12, chip8.delay_timer);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx18() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf218;
        chip8.v[2] = 0x34;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x34, chip8.sound_timer);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx1e() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf31e;
        chip8.i = 0x12;
        chip8.v[3] = 0x34;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0x0012 + 0x0034, chip8.i);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx29() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf429;
        chip8.v[4] = 0x0f;

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(0xf * 5, chip8.i);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx33() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf533;
        chip8.i = 0x300;
        chip8.v[5] = 0x80; // 128

        chip8.decode_execute(opcode, &k).unwrap();
        assert_eq!(1, chip8.memory[0x300]);
        assert_eq!(2, chip8.memory[0x301]);
        assert_eq!(8, chip8.memory[0x302]);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx55() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf655;
        chip8.i = 0x300;
        for i in 0..=6 {
            chip8.v[i] = i as u8;
        }

        chip8.decode_execute(opcode, &k).unwrap();
        for i in 0..=6 {
            assert_eq!(i as u8, chip8.memory[0x300 + i]);
        }
        assert_eq!(0x300, chip8.i);
        assert_eq!(0x202, chip8.pc);
    }

    #[test]
    fn decode_execute_fx65() {
        let mut chip8 = Chip8::new();
        let k = KeyBoard::new();
        let opcode = 0xf665;
        chip8.i = 0x300;
        for i in 0..=6 {
            chip8.memory[0x300 + i] = i as u8;
        }

        chip8.decode_execute(opcode, &k).unwrap();
        for i in 0..=6 {
            assert_eq!(i as u8, chip8.v[i]);
        }
        assert_eq!(0x300, chip8.i);
        assert_eq!(0x202, chip8.pc);
    }
}
