//! Overcomplicated circular stack implementation, with type-safe APIs for working with the stack
//! as either a byte or short stack, and supporting the "keep" mode.

use crate::common::{Item, ItemSize};

/// Models uxn's circular stack.
#[derive(Clone)]
pub struct Stack {
    pub pointer: u8,
    pub data: [u8; 256], // Easier to store and shorts and cast on the way out, imo
}

impl Stack {
    pub fn new() -> Self {
        Self {
            pointer: 0,
            data: [0; 256],
        }
    }

    pub fn first_bytes<const N: usize>(&self) -> [u8; N] {
        self.data[..N].try_into().unwrap()
    }

    pub fn new_with_data(data: &[i8]) -> Self {
        let mut stack = Self::new();

        for (i, datum) in data.iter().enumerate() {
            stack.push_byte(*datum);
        }

        stack
    }

    pub fn push_byte(&mut self, byte: i8) {
        self.data[self.pointer as usize] = byte as u8;
        self.pointer = self.pointer.overflowing_add(1).0;
    }

    pub fn push_short(&mut self, short: i16) {
        let [hi, lo] = short.to_be_bytes();
        self.push_byte(hi as i8);
        self.push_byte(lo as i8);
    }

    pub fn push_item(&mut self, item: Item) {
        match item {
            Item::Byte(byte) => self.push_byte(byte),
            Item::Short(short) => self.push_short(short),
        }
    }

    pub fn take_operands(&mut self, mode: AccessMode, item_size: ItemSize) -> StackOperandAccessor<()> {
        StackOperandAccessor::new(self, mode, item_size)
    }
}

pub struct StackOperandAccessor<'s, T> {
    stack: &'s mut Stack,
    pointer: u8,
    mode: AccessMode,
    item_size: ItemSize,
    data: T,
}

impl<'s> StackOperandAccessor<'s, ()> {
    fn new(stack: &'s mut Stack, mode: AccessMode, item_size: ItemSize) -> Self {
        StackOperandAccessor {
            pointer: stack.pointer,
            stack,
            mode,
            item_size,
            data: (),
        }
    }
}

impl<'s, T> StackOperandAccessor<'s, T> {
    fn this_byte(&self) -> (i8, u8) {
        let (pointer, _) = self.pointer.overflowing_sub(1);
        let byte = self.stack.data[pointer as usize] as i8;
        (byte, pointer)
    }

    fn this_short(&self) -> (i16, u8) {
        let (pointer, _) = self.pointer.overflowing_sub(2);
        let bytes = [
            self.stack.data[pointer as usize],
            self.stack.data[(pointer.overflowing_add(1).0) as usize],
        ];
        let short = i16::from_be_bytes(bytes);
        (short, pointer)
    }

    fn this_item(&self) -> (Item, u8) {
        match self.item_size {
            ItemSize::Byte => {
                let (byte, pointer) = self.this_byte();
                (Item::Byte(byte), pointer)
            },
            ItemSize::Short => {
                let (short, pointer) = self.this_short();
                (Item::Short(short), pointer)
            },
        }
    }

    pub fn done(self) -> T {
        if self.mode == AccessMode::Pop {
            self.stack.pointer = self.pointer;
        }

        self.data
    }
}

impl<'s> StackOperandAccessor<'s, ()> {
    pub fn byte(self) -> StackOperandAccessor<'s, (i8,)> {
        let (byte, pointer) = self.this_byte();
        StackOperandAccessor { pointer, data: (byte,), ..self }
    }

    pub fn short(self) -> StackOperandAccessor<'s, (i16,)> {
        let (short, pointer) = self.this_short();
        StackOperandAccessor { pointer, data: (short,), ..self }
    }

    pub fn item(self) -> StackOperandAccessor<'s, (Item,)> {
        let (item, pointer) = self.this_item();
        StackOperandAccessor { pointer, data: (item,), ..self }
    }
}

impl<'s, T1> StackOperandAccessor<'s, (T1,)> {
    pub fn then_byte(self) -> StackOperandAccessor<'s, (T1, i8)> {
        let (byte, pointer) = self.this_byte();
        let (d1,) = self.data;
        StackOperandAccessor { pointer, data: (d1, byte), ..self }
    }

    pub fn then_short(self) -> StackOperandAccessor<'s, (T1, i16)> {
        let (short, pointer) = self.this_short();
        let (d1,) = self.data;
        StackOperandAccessor { pointer, data: (d1, short), ..self }
    }

    pub fn then_item(self) -> StackOperandAccessor<'s, (T1, Item)> {
        let (item, pointer) = self.this_item();
        let (d1,) = self.data;
        StackOperandAccessor { pointer, data: (d1, item), ..self }
    }
}

impl<'s, T1, T2> StackOperandAccessor<'s, (T1, T2)> {
    pub fn then_byte(self) -> StackOperandAccessor<'s, (T1, T2, i8)> {
        let (byte, pointer) = self.this_byte();
        let (d1, d2) = self.data;
        StackOperandAccessor { pointer, data: (d1, d2, byte), ..self }
    }

    pub fn then_short(self) -> StackOperandAccessor<'s, (T1, T2, i16)> {
        let (short, pointer) = self.this_short();
        let (d1, d2) = self.data;
        StackOperandAccessor { pointer, data: (d1, d2, short), ..self }
    }

    pub fn then_item(self) -> StackOperandAccessor<'s, (T1, T2, Item)> {
        let (item, pointer) = self.this_item();
        let (d1, d2) = self.data;
        StackOperandAccessor { pointer, data: (d1, d2, item), ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    /// Remove accessed items from the stack.
    Pop,

    /// Keep accessed items on the stack.
    Keep,
}


#[cfg(test)]
mod test {
    use crate::{common::Item, stack::ItemSize};
    use super::{AccessMode, Stack};

    #[test]
    fn test_stack_pop() {
        let mut stack = Stack::new_with_data(&[1, 2, 3, 4]);

        let (byte1, short, byte2) = stack
            .take_operands(AccessMode::Keep, ItemSize::Byte)
            .byte().then_short().then_byte()
            .done();

        assert_eq!(byte1, 4);
        assert_eq!(short, 0x0203);
        assert_eq!(byte2, 1);
    }

    #[test]
    fn test_stack_pop_byte_item() {
        let mut stack = Stack::new_with_data(&[1, 2, 3, 4]);

        let (byte1, short, byte2) = stack
            .take_operands(AccessMode::Keep, ItemSize::Byte)
            .item().then_short().then_item()
            .done();

        assert_eq!(byte1, Item::Byte(4));
        assert_eq!(short, 0x0203);
        assert_eq!(byte2, Item::Byte(1));
    }

    #[test]
    fn test_stack_pop_word_item() {
        let mut stack = Stack::new_with_data(&[1, 2, 3, 4]);

        let (short1, short2) = stack
            .take_operands(AccessMode::Keep, ItemSize::Short)
            .item().then_short()
            .done();

        assert_eq!(short1, Item::Short(0x0304));
        assert_eq!(short2, 0x0102);
    }
}

