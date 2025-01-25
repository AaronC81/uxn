#![feature(type_changing_struct_update)]

mod common;
mod stack;

use common::{Item, ItemSize};
use stack::{AccessMode, Stack};
use uxn_utils::assemble_uxntal;

#[derive(Clone)]
struct Core {
    program_counter: u16,
    memory: [u8; 2usize.pow(16)],
    working_stack: Stack,
    return_stack: Stack,
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

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.clear_memory();

        for (i, byte) in rom.iter().enumerate() {
            self.memory[ROM_BASE as usize + i] = *byte;
        }
    }

    pub fn clear_memory(&mut self) {
        // NOTE: there is an uxn convention to keep <0x0100 on a "soft reboot"
        //       https://wiki.xxiivv.com/site/uxntal_memory.html
        for item in &mut self.memory {
            *item = 0;
        }
    }

    pub fn execute_one_instruction(&mut self, ins: u8) -> ExecutionResult {
        //
        //   .- Don't pop any operands
        //   |.- Operate on the return stack
        //   ||.- Operate on shorts instead of bytes
        //   |||.---. Opcode
        // 0b11111111
        //
        let keep = ins & 0x80;
        let use_return_stack = ins & 0x40;
        let use_short = ins & 0x20;
        let opcode = ins & 0x1F;

        let target_stack =
            if use_return_stack != 0 {
                &mut self.return_stack
            } else {
                &mut self.working_stack
            };

        let item_size = if use_short > 0 { ItemSize::Short } else { ItemSize::Byte };
        let mode = if keep > 0 { AccessMode::Keep } else { AccessMode::Pop };

        match opcode {
            // BRK
            0x00 => {
                // This instruction is drastically different depending on the modes.

                // If not `keep`, then it's BRK
                if mode != AccessMode::Keep {
                    return ExecutionResult::Break
                }

                // Otherwise it is LIT
                // Program counter has already been incremented, so we're already pointing at the
                // first byte.
                let item = match item_size {
                    ItemSize::Byte => {
                        let byte = self.memory[self.program_counter as usize];
                        self.program_counter = self.program_counter.overflowing_add(1).0;

                        Item::Byte(byte as i8)
                    },

                    ItemSize::Short => {
                        let bytes = [
                            self.memory[self.program_counter as usize],
                            self.memory[self.program_counter.overflowing_add(1).0 as usize],
                        ];
                        self.program_counter = self.program_counter.overflowing_add(2).0;

                        let short = i16::from_be_bytes(bytes);
                        Item::Short(short)
                    },
                };

                target_stack.push_item(item);
            }

            // INC
            0x01 => {
                let (item,) = target_stack
                    .take_operands(mode, item_size)
                    .item()
                    .done();
                target_stack.push_item(item.increment());
            },

            0x02 => todo!(),
            0x03 => todo!(),
            0x04 => todo!(),
            0x05 => todo!(),
            0x06 => todo!(),
            0x07 => todo!(),
            0x08 => todo!(),
            0x09 => todo!(),
            0x0A => todo!(),
            0x0B => todo!(),
            0x0C => todo!(),
            0x0D => todo!(),
            0x0E => todo!(),
            0x0F => todo!(),
            0x10 => todo!(),
            0x11 => todo!(),
            0x12 => todo!(),
            0x13 => todo!(),
            0x14 => todo!(),
            0x15 => todo!(),
            0x16 => todo!(),
            0x17 => todo!(),
            0x18 => todo!(),
            0x19 => todo!(),
            0x1A => todo!(),
            0x1B => todo!(),
            0x1C => todo!(),
            0x1D => todo!(),
            0x1E => todo!(),
            0x1F => todo!(),

            _ => unreachable!(),
        }

        ExecutionResult::Continue
    }

    pub fn execute_until_break(&mut self) {
        loop {
            let ins = self.memory[self.program_counter as usize];
            // TODO: maybe shouldn't always do this, depending on jump semantics
            self.program_counter = self.program_counter.overflowing_add(1).0;

            match self.execute_one_instruction(ins) {
                ExecutionResult::Continue => {},
                ExecutionResult::Break => return,
            }
        }
    }
}

pub enum ExecutionResult {
    Continue,
    Break,
}

#[cfg(test)]
mod test {
    use crate::Core;

    #[test]
    fn test_inc() {
        // Byte mode
        let mut core = Core::new_with_uxntal("|100 #01 INC BRK");
        core.execute_until_break();
        assert_eq!(core.working_stack.first_bytes(), [2]);

        // Short mode
        let mut core = Core::new_with_uxntal("|100 #00ff INC2 BRK");
        core.execute_until_break();
        assert_eq!(core.working_stack.first_bytes(), [01, 00]);

        // Keep mode
        let mut core = Core::new_with_uxntal("|100 #00ff INC2k BRK");
        core.execute_until_break();
        assert_eq!(core.working_stack.first_bytes(), [00, 0xff, 01, 00]);
    }    
}
