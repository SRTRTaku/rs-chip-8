use chip8::{set_keys, setup_graphics, Chip8, KeyBoard};
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod chip8;

fn main() {
    // check arg
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("invalid argumnts");
        return;
    }

    // Set up render system and resiger input callbacks
    //
    // setupGraphics
    // setupInput
    setup_graphics();
    let kb = Arc::new(Mutex::new(KeyBoard::new()));

    let kb1 = Arc::clone(&kb);
    thread::spawn(move || {
        // Store kye press state (Press and Release)
        set_keys(kb1);
    });

    // Initialize the Chip8 system and load the game into the memory
    let mut my_chip8 = Chip8::new();
    if let Err(e) = my_chip8.load_game(&args[1]) {
        println!("erro {}", e);
        return;
    }
    // my_chip8.dump();

    let mut count = 0;
    loop {
        count += 1;
        // Emulate one cycle
        {
            let key_board = kb.lock().unwrap();
            if key_board.fin_flag {
                break;
            }
            my_chip8.emulate_cycle(&key_board);
        }

        // If the draw flag is set, update the screen
        if my_chip8.draw_flag() {
            my_chip8.draw_graphics();
        }

        print!("\x1b[1;1H");
        print!("\x1b[2K");
        println!("{}", count);

        // print!("\x1b[34;1H");
        // print!("\x1b[0J");
        // my_chip8.dump();

        thread::sleep(Duration::from_millis(17)); // 60 Hz
    }

    // Cleanup
    print!("\x1b[1;1H");
    print!("\x1b[2J");
}
