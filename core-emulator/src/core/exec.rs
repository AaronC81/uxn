use std::process::exit;

use crate::{common::{Item, ItemSize, StackMode}, stack::{AccessMode, Stack}, Memory};

use super::Core;

pub enum ExecutionResult {
    Continue,
    Break,
}

impl Core {
    pub fn execute_until_break(&mut self) {
        loop {
            let ins = self.memory[self.program_counter as usize];
            self.program_counter = self.program_counter.overflowing_add(1).0;

            match self.execute_one_instruction(ins) {
                ExecutionResult::Continue => {},
                ExecutionResult::Break => return,
            }
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
            0x17 => {
                // TODO: handle non-byte writes.
                //       this should just be treated as normal memory, and if stuff needs to be
                //       triggered, it gets triggered for both bytes

                // TODO: absolute minimal Varvara implementation for printing
                // See: https://wiki.xxiivv.com/site/varvara.html#console
                let (device, value) = op.byte().then_item().done();
                let Item::Byte(byte) = value else { panic!() };

                match device {
                    // .System/state
                    0x0f => {
                        if byte != 0 {
                            let exit_code = (byte as u8) & 0x7f;
                            exit(exit_code as i32);
                        }
                    }

                    // .Console/write
                    0x18 => {
                        print!("{}", byte as u8 as char);
                    }

                    _ => panic!("unsupported device port {device}")
                }
            },

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
}
