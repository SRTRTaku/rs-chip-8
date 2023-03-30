use chip8::Chip8;
use std::process::Command;

mod chip8;

fn main() {
    // Set up render system and resiger input callbacks
    // setupGraphics
    // setupInput

    // Initialize the Chip8 system and load the game into the memory
    let my_chip8 = Chip8::new();
    // my_chip8.load_game("pong");
    my_chip8.dump();

    loop {
        return;
        // Emulate one cycle
        // my_chip8.emulate_cycle();

        // If the draw flag is set, update the screen
        // if my_chip8.drawFlag() {
        //     my_chip8.drawGraphics();
        // }

        // Store kye press state (Press and Release)
        // my_chip8.set_keys();
    }

    /*
    Command::new("clear").status().unwrap();
    for _i in 0..5 {
        for _j in 0..63 {
            print!("*");
        }
        println!("");
    }
    */
}
