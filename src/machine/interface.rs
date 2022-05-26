use super::chip8;
use super::chip8::CHIP8;

use ncurses::*;

pub fn draw_scr(chip: &mut CHIP8) {
    let height = chip8::DISP_HEIGHT as usize;
    let width = chip8::DISP_WIDTH as usize;

    clear();
    
    for line in 0..height {
        for col in 0..width {
            if chip.get_pixel(line, col) == 0 {
                addstr(" ");
            } else {
                addch(ACS_BLOCK());
            }
        }
        addstr("\n");
    }
}

pub fn run(mut chip: CHIP8) {
    initscr();
    noecho();
    
    while true {
        chip.cycle();
        draw_scr(&mut chip);
        refresh();
    }
    endwin();
}