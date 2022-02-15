mod machine;

use machine::cpu::CPU;

fn main() {
    let mut chip:CPU = CPU::new();

    chip.load_rom("games/PONG").expect("Error loading Rom!");
}
