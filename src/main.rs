use chip8::Chip8;

mod chip8;

fn main() {
    // Set up render system and resiger input callbacks
    //
    // setupGraphics
    // setupInput
    chip8::setup_graphics();

    // Initialize the Chip8 system and load the game into the memory
    let mut my_chip8 = Chip8::new();
    if let Err(e) = my_chip8.load_game("PONG") {
        println!("erro {}", e);
        return;
    }
    my_chip8.dump();

    let mut count = 0;
    loop {
        count += 1;
        // Emulate one cycle
        my_chip8.emulate_cycle();
        my_chip8.dump();

        // If the draw flag is set, update the screen
        if my_chip8.draw_flag() {
            my_chip8.draw_graphics();
        }

        // Store kye press state (Press and Release)
        // my_chip8.set_keys();

        break;
        print!("\x1b[1;1H");
        print!("\x1b[2K");
        println!("{}", count);
    }
}
