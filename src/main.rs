use chip8::{Chip8, KeyBoard};
use io::IO;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

mod chip8;
mod io;

fn main() {
    // check arg
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("invalid argumnts");
        return;
    }

    // Set up render system and resiger input callbacks
    let mut io = IO::setup();
    let mut key_board = KeyBoard::new();

    // Initialize the Chip8 system and load the game into the memory
    let mut my_chip8 = Chip8::new();
    if let Err(e) = my_chip8.load_game(&args[1]) {
        println!("error {}", e);
        return;
    }
    // my_chip8.dump();
    let d = Duration::from_nanos(1_000_000_000 / 60);
    loop {
        let s = Instant::now();

        // Emulate one cycle
        my_chip8.emulate_cycle(&key_board);

        // If the draw flag is set, update the screen
        if my_chip8.draw_flag() {
            io.draw_graphics(&my_chip8);
        }

        io.set_key(&mut key_board);
        if key_board.fin_flag {
            break;
        }

        let prog = Instant::now() - s;
        if prog < d {
            thread::sleep(d - prog); // 60 Hz
        } else {
            println!("{:?}", prog);
        }
    }
}
