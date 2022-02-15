use std::io::{Result, Read};
use std::fs::File;
use rand::random;

const MEM_SIZE: usize = 4096;
const DISP_SIZE: usize = 2048; // 64x32 pixels

const PROG_START_ADDR: usize = 0x200;

const FONT_SET_START_ADDR: usize = 0x50;
const FONT_SET: [u8; 80] = [      // 16 Characters of 5 bytes each = 80 bytes
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct CPU {
    mem: [u8; MEM_SIZE],
    display_mem: [u8; DISP_SIZE],
    v_regs: [u8; 16],
    index_ptr: u16,
    prg_counter: u16,
    stack: [u16; 16],
    stack_ptr: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
}

impl CPU {
    fn load_chars(&mut self) {
        for i in 0..FONT_SET.len() {
            self.mem[i+FONT_SET_START_ADDR] = FONT_SET[i];
        }
    }

    pub fn new() -> Self{
        let mut chip = Self {
            mem: [0; MEM_SIZE],
            display_mem: [0; DISP_SIZE],
            v_regs: [0; 16],
            index_ptr: 0,
            prg_counter: PROG_START_ADDR as u16,
            stack: [0; 16],
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16]
        };
        chip.load_chars();

        return chip
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();

        let size = file.read_to_end(&mut data)?;

        for i in 0..size {
            self.mem[PROG_START_ADDR + i] = data[i];
        }

        Ok(())
    }

    fn rng() -> u8 {
        random()
    } 
}