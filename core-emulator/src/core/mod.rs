use uxn_utils::assemble_uxntal;

use crate::stack::Stack;

#[derive(Clone)]
pub struct Core {
    pub program_counter: u16,
    pub memory: [u8; 2usize.pow(16)],
    pub working_stack: Stack,
    pub return_stack: Stack,
}

const ROM_BASE: u16 = 0x0100;

impl Core {
    pub fn new() -> Self {
        Self {
            program_counter: ROM_BASE,
            memory: [0; 2usize.pow(16)],
            working_stack: Stack::new(),
            return_stack: Stack::new(),
        }
    }

    pub fn new_with_rom(rom: &[u8]) -> Self {
        let mut this = Self::new();
        this.load_rom(rom);
        this
    }

    pub fn new_with_uxntal(code: &str) -> Self {
        let rom = assemble_uxntal(code).unwrap();
        Self::new_with_rom(&rom)
    }
}

mod exec;
pub use exec::*;

mod mem;
pub use mem::*;

#[cfg(test)]
mod tests;
