use std::process::Command;

fn main() {
    println!("Hello, world!");
    Command::new("clear").status().unwrap();
    for _i in 0..5 {
        for _j in 0..63 {
            print!("*");
        }
        println!("");
    }
}
