use std::io::{Result, Read};
use std::fs::File;

const MEM_SIZE: usize = 4096;

pub struct CHIP8 {
    mem: [u8; MEM_SIZE],
}

impl CHIP8 {
    pub fn new() -> Self{
        Self {
            mem: [0; MEM_SIZE]
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();

        let size = file.read_to_end(&mut data)?;

        for i in 0..size {
            self.mem[0x200 + i] = data[i];
        }

        Ok(())
    }
}