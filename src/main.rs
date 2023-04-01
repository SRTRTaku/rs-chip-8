use chip8::{Chip8, KeyBoard};

mod chip8;

fn main() {
    // Set up render system and resiger input callbacks
    //
    // setupGraphics
    // setupInput
    chip8::setup_graphics();
    let mut kb = KeyBoard::new();

    // Initialize the Chip8 system and load the game into the memory
    let mut my_chip8 = Chip8::new();
    if let Err(e) = my_chip8.load_game("MAZE") {
        println!("erro {}", e);
        return;
    }
    // my_chip8.dump();

    let mut count = 0;
    loop {
        count += 1;
        // Emulate one cycle
        my_chip8.emulate_cycle(&kb);

        // If the draw flag is set, update the screen
        if my_chip8.draw_flag() {
            my_chip8.draw_graphics();
        }

        // Store kye press state (Press and Release)
        let fin_flag = kb.set_keys();
        if fin_flag {
            break;
        }

        print!("\x1b[1;1H");
        print!("\x1b[2K");
        println!("{}", count);

        print!("\x1b[34;1H");
        print!("\x1b[0J");
        my_chip8.dump();
    }
    print!("\x1b[1;1H");
    print!("\x1b[2J");
}
