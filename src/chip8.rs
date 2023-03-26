const MEMORY_SIZE: isize = 4096;
const V_SIZE: isize = 16;
const GFX_SIZE_COL: isize = 64;
const GFX_SIZE_ROW: isize = 32;
const GFX_SIZE: isize = GFX_SIZE_COL * GFX_SIZE_ROW;
const STACK_SIZE: isize = 16;
const KEY_NUM: isize = 16;

struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    V: [u8; V_SIZE],
    I: u16,
    pc: u16,
    gfx: [u8; GFX_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
    sp: u16,
    key: [u8; KEY_NUM],
}
