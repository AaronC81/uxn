#![feature(type_changing_struct_update)]
#![feature(unbounded_shifts)]

mod common;
mod stack;

use common::{Item, ItemSize};
use stack::{AccessMode, Stack};
use uxn_utils::assemble_uxntal;

#[derive(Clone)]
pub struct Core {
    program_counter: u16,
    memory: [u8; 2usize.pow(16)],
    working_stack: Stack,
    return_stack: Stack,
}

const ROM_BASE: u16 = 0x0100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMode {
    Working,
    Return,
}

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

        let stack = if use_return_stack > 0 { StackMode::Return } else { StackMode::Working };
        let item_size = if use_short > 0 { ItemSize::Short } else { ItemSize::Byte };
        let mode = if keep > 0 { AccessMode::Keep } else { AccessMode::Pop };

        // Create an operand accessor, ready for instructions which need one
        let op = self.target_stack(stack).take_operands(mode, item_size);

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

                self.target_stack(stack).push_item(item);
            }

            // INC
            0x01 => {
                let (item,) = op.item().done();
                self.target_stack(stack).push_item(item.increment());
            },

            // POP
            0x02 => {
                op.item().done();
            },

            // NIP
            0x03 => {
                let (_, item) = op.item().then_item().done();
                self.target_stack(stack).push_item(item);
            },

            // SWP
            0x04 => {
                let (first, second) = op.item().then_item().done();

                let stack = self.target_stack(stack);
                stack.push_item(first);
                stack.push_item(second);
            },

            // ROT
            0x05 => {
                let (a, b, c) = op.item().then_item().then_item().done();

                let stack = self.target_stack(stack);
                stack.push_item(b);
                stack.push_item(c);
                stack.push_item(a);
            },

            // DUP
            0x06 => {
                let (item,) = op.item().done();

                let stack = self.target_stack(stack);
                stack.push_item(item);
                stack.push_item(item);
            },

            // OVR
            0x07 => {
                let (a, b) = op.item().then_item().done();

                let stack = self.target_stack(stack);
                stack.push_item(a);
                stack.push_item(b);
                stack.push_item(a);
            },

            // EQU
            0x08 => {
                let (a, b) = op.item().then_item().done();
                self.target_stack(stack).push_byte(if a == b { 1 } else { 0 });
            },

            // NEQ
            0x09 => {
                let (a, b) = op.item().then_item().done();
                self.target_stack(stack).push_byte(if a != b { 1 } else { 0 });
            },

            // GTH
            0x0A => {
                let (a, b) = op.item().then_item().done();
                self.target_stack(stack).push_byte(if b > a { 1 } else { 0 });
            },

            // LTH
            0x0B => {
                let (a, b) = op.item().then_item().done();
                self.target_stack(stack).push_byte(if b < a { 1 } else { 0 });
            },

            // JMP
            0x0C => {
                let (dest,) = op.item().done();
                self.jump_to_dynamic_address(dest);
            },

            // JCN
            0x0D => {
                let (dest, cond) = op.item().then_byte().done();
                if cond != 0 {
                    self.jump_to_dynamic_address(dest);
                }
            },

            // JSR
            0x0E => {
                let (dest,) = op.item().done();
                self.return_stack.push_short(self.program_counter as i16);
                self.jump_to_dynamic_address(dest);
            },

            // STH
            0x0F => {
                let (item,) = op.item().done();
                self.other_stack(stack).push_item(item);
            },

            // LDZ
            0x10 => {
                let (addr,) = op.byte().done(); 
                let item = self.read_memory(addr as u16, item_size);
                self.target_stack(stack).push_item(item);
            },

            // STZ
            0x11 => {
                let (addr, item) = op.byte().then_item().done();
                self.write_memory(addr as u16, item);
            },

            // LDR
            0x12 => {
                let (addr,) = op.byte().done();
                let abs_addr = (self.program_counter as i32).overflowing_add(addr as i32).0 as i16; // TODO: what is right here? same with STR
                let item = self.read_memory(abs_addr as u16, item_size);
                self.target_stack(stack).push_item(item);
            },

            // STR
            0x13 => {
                let (addr, item) = op.byte().then_item().done();
                let abs_addr = (self.program_counter as i32).overflowing_add(addr as i32).0 as i16;
                self.write_memory(abs_addr as u16, item);
            },

            // LDA
            0x14 => {
                let (addr,) = op.short().done();
                let item = self.read_memory(addr as u16, item_size);
                self.target_stack(stack).push_item(item);
            },

            // STA
            0x15 => {
                let (addr, item) = op.short().then_item().done();
                self.write_memory(addr as u16, item);
            },

            // DEI
            0x16 => todo!(),

            // DEO
            0x17 => todo!(),

            // ADD
            0x18 => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a + b);
            },

            // SUB
            0x19 => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a - b);
            },

            // MUL
            0x1A => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a * b);
            },

            // DIV
            0x1B => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a / b);
            },

            // AND
            0x1C => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a & b);
            },

            // ORA
            0x1D => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a | b);
            },

            // EOR
            0x1E => {
                let (b, a) = op.item().then_item().done();
                self.target_stack(stack).push_item(a ^ b);
            },

            // SFT
            0x1F => {
                let (shift, a) = op.byte().then_item().done();

                let shift_left = (0xF0 & (shift as u8)) >> 4;
                let shift_right = 0x0F & (shift as u8);

                println!("Shifting:  left({shift_left}) right({shift_right})");

                let item = a.shift(shift_left, shift_right);
                self.target_stack(stack).push_item(item);
            },

            _ => unreachable!(),
        }

        ExecutionResult::Continue
    }

    fn jump_to_dynamic_address(&mut self, dest: Item) {
        match dest {
            Item::Byte(rel) => {
                // Relative
                self.program_counter = self.program_counter
                    .overflowing_add(rel as u16).0;
            },
            Item::Short(abs) => {
                // Absolute
                self.program_counter = abs as u16;
            },
        }
    }

    fn target_stack(&mut self, stack: StackMode) -> &mut Stack {
        match stack {
            StackMode::Working => &mut self.working_stack,
            StackMode::Return => &mut self.return_stack,
        }
    }

    fn other_stack(&mut self, stack: StackMode) -> &mut Stack {
        match stack {
            StackMode::Working => &mut self.return_stack,
            StackMode::Return => &mut self.working_stack,
        }
    }

    fn read_memory(&self, addr: u16, item_size: ItemSize) -> Item {
        match item_size {
            ItemSize::Byte => Item::Byte(self.memory[addr as usize] as i8),
            ItemSize::Short => Item::Short(
                i16::from_be_bytes([
                    self.memory[addr as usize],
                    self.memory[addr.overflowing_add(1).0 as usize],
                ])
            ),
        }
    }

    fn write_memory(&mut self, addr: u16, item: Item) {
        match item {
            Item::Byte(byte) => {
                self.memory[addr as usize] = byte as u8;
            },
            Item::Short(short) => {
                let [hi, lo] = short.to_be_bytes();
                self.memory[addr as usize] = hi;
                self.memory[addr.overflowing_add(1).0 as usize] = lo;
            },
        }
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
    // A number of these test cases are taken from the examples on the uxntal reference:
    //   https://wiki.xxiivv.com/site/uxntal_reference.html

    use std::str;

    use crate::Core;

    #[test]
    fn test_inc() {
        assert_eq!(execute("#01 INC BRK"), [2]); // Byte mode
        assert_eq!(execute("#00ff INC2 BRK"), [01, 00]); // Short mode
        assert_eq!(execute("#00ff INC2k BRK"), [00, 0xff, 01, 00]); // Keep mode
    }

    #[test]
    fn test_jmp() {
        assert_eq!(execute("#01 #02 ,&skip-rel JMP BRK BRK BRK &skip-rel #03"), [1, 2, 3]); // Relative mode
        assert_eq!(execute("#01 #02 ;&skip-abs JMP2 BRK BRK BRK &skip-abs #03"), [1, 2, 3]); // Absolute mode
    }

    #[test]
    fn test_jcn() {
        assert_eq!(execute("#01 ,&true JCN ,&false JMP  &true #42 BRK  &false #ff BRK"), [0x42]); // True
        assert_eq!(execute("#00 ,&true JCN ,&false JMP  &true #42 BRK  &false #ff BRK"), [0xff]); // False
    }

    #[test]
    fn test_ldr() {
        assert_eq!(execute(",cell LDR BRK @cell 12"), [0x12]); // Byte
        assert_eq!(execute(",cell LDR2 BRK @cell abcd"), [0xab, 0xcd]); // Short
    }

    #[test]
    fn test_sft() {
        assert_eq!(execute("#34 #10 SFT BRK"), [0x68]);
        assert_eq!(execute("#34 #01 SFT BRK"), [0x1a]);
        assert_eq!(execute("#1248 #34 SFTk2 BRK"), [0x12, 0x48, 0x34, 0x09, 0x20]);
    }

    fn execute(code: &str) -> Vec<u8> {
        let mut core = Core::new_with_uxntal(code);
        core.execute_until_break();
        core.working_stack.bytes().to_vec()
    }
}
