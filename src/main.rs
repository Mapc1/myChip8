mod machine;

use machine::chip8::CHIP8;

fn main() {
    let mut chip:CHIP8 = CHIP8::new();

    chip.load_rom("games/PONG").expect("Error loading Rom!");
}
