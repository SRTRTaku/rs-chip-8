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

#[derive(Debug)]
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
        }
    }
}
