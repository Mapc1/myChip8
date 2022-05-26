mod machine;

use machine::chip8::CHIP8;
use machine::interface::run;

fn main() {
    let mut chip:CHIP8 = CHIP8::new();

    chip.load_rom("roms/PONG").expect("Error loading Rom!");

    run(chip);
}
