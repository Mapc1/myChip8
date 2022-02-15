use std::io::{Result, Read};
use std::fs::File;
use std::collections::HashMap;
use rand::random;

const MEM_SIZE: usize = 4096;
const DISP_SIZE: usize = 2048; // 64x32 pixels
const DISP_WIDTH: u8 = 64;
const DISP_HEIGHT: u8 = 32;

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

type INSTRUCTION = fn(&mut CPU);

pub struct CPU {
    mem: [u8; MEM_SIZE],
    display_mem: [u32; DISP_SIZE],
    v_regs: [u8; 16],
    index_ptr: u16,
    prg_counter: u16,
    stack: [u16; 16],
    stack_ptr: u8,
    delay_timer: u8,
    sound_timer: u8,
    opcode: u16,
    keypad: [u8; 16],

    // Lookup tables where we associate an opcode with the respective function
    lookup_table: HashMap<u16, INSTRUCTION>,
    lookup_table_0: HashMap<u16, INSTRUCTION>,
    lookup_table_8: HashMap<u16, INSTRUCTION>,
    lookup_table_e: HashMap<u16, INSTRUCTION>,
    lookup_table_f: HashMap<u16, INSTRUCTION>
}

impl CPU {
    fn load_chars(&mut self) {
        for i in 0..FONT_SET.len() {
            self.mem[i+FONT_SET_START_ADDR] = FONT_SET[i];
        }
    }

    fn search_table_0(&mut self) {
        let code = self.opcode & 0x000F;

        self.lookup_table_0.get(&code).unwrap() (self);
    }

    fn search_table_8(&mut self) {
        let code = self.opcode & 0x000F;
        
        self.lookup_table_8.get(&code).unwrap() (self);
    }

    fn search_table_e(&mut self) {
        let code = self.opcode & 0x000F;
        
        self.lookup_table_e.get(&code).unwrap() (self);
    }

    fn search_table_f(&mut self) {
        let code = self.opcode & 0x000F;
        
        self.lookup_table_f.get(&code).unwrap() (self);
    }

    /** 
     * Adds all opcodes and the respective function to the lookup tables.
     * There, most likely, is a much better and cleaner solution, but this
     * is what we have.
     */
    fn add_ops(&mut self) {
        self.lookup_table.insert(0x0, CPU::search_table_0);
        self.lookup_table.insert(0x1, CPU::op_1nnn);
        self.lookup_table.insert(0x2, CPU::op_2nnn);
        self.lookup_table.insert(0x3, CPU::op_3xkk);
        self.lookup_table.insert(0x4, CPU::op_4xkk);
        self.lookup_table.insert(0x5, CPU::op_5xy0);
        self.lookup_table.insert(0x6, CPU::op_6xkk);
        self.lookup_table.insert(0x7, CPU::op_7xkk);
        self.lookup_table.insert(0x8, CPU::search_table_8);
        self.lookup_table.insert(0x9, CPU::op_9xy0);
        self.lookup_table.insert(0xA, CPU::op_annn);
        self.lookup_table.insert(0xB, CPU::op_bnnn);
        self.lookup_table.insert(0xC, CPU::op_cxkk);
        self.lookup_table.insert(0xD, CPU::op_dxyn);
        self.lookup_table.insert(0xE, CPU::search_table_e);
        self.lookup_table.insert(0xF, CPU::search_table_f);

        self.lookup_table_0.insert(0x0, CPU::op_00e0);
        self.lookup_table_0.insert(0xE, CPU::op_00ee);

        self.lookup_table_8.insert(0x0, CPU::op_8xy0);
        self.lookup_table_8.insert(0x1, CPU::op_8xy1);
        self.lookup_table_8.insert(0x4, CPU::op_8xy2);
        self.lookup_table_8.insert(0x3, CPU::op_8xy3);
        self.lookup_table_8.insert(0x4, CPU::op_8xy4);
        self.lookup_table_8.insert(0x5, CPU::op_8xy5);
        self.lookup_table_8.insert(0x6, CPU::op_8xy6);
        self.lookup_table_8.insert(0x7, CPU::op_8xy7);
        self.lookup_table_8.insert(0xE, CPU::op_8xye);

        self.lookup_table_e.insert(0x1, CPU::op_exa1);
        self.lookup_table_e.insert(0xE, CPU::op_ex9e);

        self.lookup_table_f.insert(0x07, CPU::op_fx07);
        self.lookup_table_f.insert(0x0A, CPU::op_fx0a);
        self.lookup_table_f.insert(0x15, CPU::op_fx15);
        self.lookup_table_f.insert(0x18, CPU::op_fx18);
        self.lookup_table_f.insert(0x1E, CPU::op_fx1e);
        self.lookup_table_f.insert(0x29, CPU::op_fx29);
        self.lookup_table_f.insert(0x33, CPU::op_fx33);
        self.lookup_table_f.insert(0x55, CPU::op_fx55);
        self.lookup_table_f.insert(0x65, CPU::op_fx65);
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
            opcode: 0,
            keypad: [0; 16],
            lookup_table: HashMap::new(),
            lookup_table_0: HashMap::new(),
            lookup_table_8: HashMap::new(),
            lookup_table_e: HashMap::new(),
            lookup_table_f: HashMap::new()
        };
        chip.load_chars();
        chip.add_ops();

        return chip
    }

    // Loads a ROM into memory address 0x200
    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();

        let size = file.read_to_end(&mut data)?;

        for i in 0..size {
            self.mem[PROG_START_ADDR + i] = data[i];
        }

        Ok(())
    }

    fn rng(&mut self) -> u8 {
        random()
    } 

    // CLS
    fn op_00e0(&mut self) {
        self.display_mem = [0; DISP_SIZE];
    }

    // RET
    fn op_00ee(&mut self) {
        self.stack_ptr -= 1;
        self.prg_counter = self.stack[self.stack_ptr as usize];
    }

    // JP addr
    fn op_1nnn(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.prg_counter = addr;
    }

    // Call addr
    fn op_2nnn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.stack[self.stack_ptr as usize] = self.prg_counter;
        self.stack_ptr += 1;
        self.prg_counter = addr;
    }

    // SE Vx, byte
    fn op_3xkk(&mut self) {
        let reg: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        if self.v_regs[reg as usize] == byte {
            self.prg_counter += 2;
        }
    }

    // SNE Vx, byte
    fn op_4xkk(&mut self) {
        let reg = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        if self.v_regs[reg] != byte {
            self.prg_counter += 2;
        }
    }

    // SE Vx, Vy
    fn op_5xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        
        if self.v_regs[vx] == self.v_regs[vy] {
            self.prg_counter += 2;
        }
    }

    // LD Vx kk
    fn op_6xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        self.v_regs[vx] = byte
    }

    // ADD Vx, byte
    fn op_7xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        self.v_regs[vx] = byte;
    }

    // LD Vx, Vy
    fn op_8xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.v_regs[vx] = self.v_regs[vy];
    }

    // OR Vx, Vy
    fn op_8xy1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.v_regs[vx] |= self.v_regs[vy];
    }

    // AND Vx, Vy
    fn op_8xy2(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.v_regs[vx] &= self.v_regs[vy];
    }

    // XOR Vx, Vy
    fn op_8xy3(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.v_regs[vx] ^= self.v_regs[vy];
    }

    // ADD Vx, Vy
    fn op_8xy4(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let sum: u16 = (self.v_regs[vx] + self.v_regs[vy]) as u16;
        
        if sum > 255 {
            self.v_regs[0xF] = 1;
        } else {
            self.v_regs[0xF] = 0;
        }

        self.v_regs[vx] = (sum & 0xFF) as u8;
    }

    // SUB Vx, Vy
    fn op_8xy5(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.v_regs[vx] > self.v_regs[vy] {
            self.v_regs[0xF] = 1;
        } else {
            self.v_regs[0xF] = 0;
        }

        self.v_regs[vx] -= self.v_regs[vy];
    }

    // SHR Vx
    fn op_8xy6(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.v_regs[0xF] = self.v_regs[vx] & 0x1;

        self.v_regs[vx] >>= 1;
    }

    // SUBN Vx, Vy
    fn op_8xy7(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.v_regs[vy] > self.v_regs[vx] {
            self.v_regs[0xF] = 1;
        } else {
            self.v_regs[0xF] = 0;
        }

        self.v_regs[vx] = self.v_regs[vy] - self.v_regs[vx];
    }

    // SHL Vx {, Vy}
    fn op_8xye(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.v_regs[0xF] = (self.v_regs[vx] & 0x80) >> 7;

        self.v_regs[vx] <<= 1;
    }

    // SNE Vx, Vy
    fn op_9xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.v_regs[vx] != self.v_regs[vy] {
            self.prg_counter +=2;
        }
    }

    // LD I, addr
    fn op_annn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.index_ptr = addr;
    }

    // JP V0, addr
    fn op_bnnn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.prg_counter = self.v_regs[0] as u16 + addr;
    }

    // RND Vx, byte
    fn op_cxkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        self.v_regs[vx] = self.rng() & byte;
    }

    // DRW Vx, byte
    fn op_dxyn(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let height = self.opcode & 0x000F;

        let x = (self.v_regs[vx] % DISP_WIDTH) as u16;
        let y = (self.v_regs[vy] % DISP_HEIGHT) as u16;

        self.v_regs[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.mem[(self.index_ptr + row) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                let screen_pixel = &mut self.display_mem[((y+row) * DISP_WIDTH as u16 + (x + col)) as usize];
                if sprite_pixel != 0 {
                    if *screen_pixel == 0xFFFFFFFF {
                        self.v_regs[0xF] = 1;
                    }

                    *screen_pixel ^= 0xFFFFFFFF;
                }
            }
        }
    }

    // SKP Vx
    fn op_ex9e(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.v_regs[vx] as usize;

        if self.keypad[key] != 0 {
            self.prg_counter += 2;
        }
    }

    // SKNP Vx
    fn op_exa1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.v_regs[vx] as usize;
    
        if self.keypad[key] == 0 {
            self.prg_counter += 2;
        }
    }

    // KD Vx, DT
    fn op_fx07(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        
        self.v_regs[vx] = self.delay_timer;
    }

    // LD Vx, K
    fn op_fx0a(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        if self.keypad[0] != 0 {
            self.v_regs[vx] = 0;
        } else if self.keypad[1] != 0 {
            self.v_regs[vx] = 1;
        } else if self.keypad[2] != 0 {
            self.v_regs[vx] = 2;
        } else if self.keypad[3] != 0 {
            self.v_regs[vx] = 3;
        } else if self.keypad[4] != 0 {
            self.v_regs[vx] = 4;
        } else if self.keypad[5] != 0 {
            self.v_regs[vx] = 5;
        } else if self.keypad[6] != 0 {
            self.v_regs[vx] = 6;
        } else if self.keypad[7] != 0 {
            self.v_regs[vx] = 7;
        } else if self.keypad[8] != 0 {
            self.v_regs[vx] = 8;
        } else if self.keypad[9] != 0 {
            self.v_regs[vx] = 9;
        } else if self.keypad[10] != 0 {
            self.v_regs[vx] = 10;
        } else if self.keypad[11] != 0 {
            self.v_regs[vx] = 11;
        } else if self.keypad[12] != 0 {
            self.v_regs[vx] = 12;
        } else if self.keypad[13] != 0 {
            self.v_regs[vx] = 13;
        } else if self.keypad[14] != 0 {
            self.v_regs[vx] = 14;
        } else if self.keypad[15] != 0 {
            self.v_regs[vx] = 15;
        } else {
            self.prg_counter -= 2; // Execute this instruction again (It's so we can wait until a key is pressed)
        }
    }

    // LD DT, Vx
    fn op_fx15(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.delay_timer = self.v_regs[vx];
    }

    // LD ST, Vx
    fn op_fx18(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.sound_timer = self.v_regs[vx];
    }

    // ADD I, Vx
    fn op_fx1e(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.index_ptr += self.v_regs[vx] as u16;
    }

    // LD F, Vx
    fn op_fx29(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let digit = self.v_regs[vx] as usize;

        self.index_ptr = (FONT_SET_START_ADDR + (5 * digit)) as u16;
    }

    // LD B, Vx
    fn op_fx33(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let value = self.v_regs[vx];

        // Ones digit
        self.mem[self.index_ptr as usize + 2] = value % 10;
        let value = value / 10;

        // Tens digit
        self.mem[self.index_ptr as usize + 1] = value % 10;
        let value = value / 10;

        // Hundreds digit
        self.mem[self.index_ptr as usize]  = value % 10;
    }

    // LD [I], Vx
    fn op_fx55(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        for i in 0..vx {
            self.mem[self.index_ptr as usize + i] = self.v_regs[i];
        }
    }

    // LD Vx, [I]
    fn op_fx65(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        
        for i in 0..vx {
            self.v_regs[i] = self.mem[self.index_ptr as usize + i];
        }
    }
}