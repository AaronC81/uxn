use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Sub};

use num_traits::{ops::overflowing::OverflowingAdd, Num, One};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMode {
    Working,
    Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemSize {
    Byte,
    Short,
}

pub trait Memory {
    type AddressSpace: Copy + Num + OverflowingAdd;

    fn read_byte(&self, addr: Self::AddressSpace) -> u8;
    fn write_byte(&mut self, addr: Self::AddressSpace, byte: u8);

    fn read_memory(&self, addr: Self::AddressSpace, item_size: ItemSize) -> Item {
        match item_size {
            ItemSize::Byte => Item::Byte(self.read_byte(addr) as i8),
            ItemSize::Short => Item::Short(
                i16::from_be_bytes([
                    self.read_byte(addr),
                    self.read_byte(addr.overflowing_add(&One::one()).0),
                ])
            ),
        }
    }

    fn write_memory(&mut self, addr: Self::AddressSpace, item: Item) {
        match item {
            Item::Byte(byte) => {
                self.write_byte(addr, byte as u8);
            },
            Item::Short(short) => {
                let [hi, lo] = short.to_be_bytes();
                self.write_byte(addr, hi);
                self.write_byte(addr.overflowing_add(&One::one()).0, lo);
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Byte(i8),
    Short(i16),
}

macro_rules! all_sizes {
    ($self:ident in $i:ident => $e:expr) => {
        match $self {
            Item::Byte($i) => Item::Byte($e),
            Item::Short($i) => Item::Short($e),
        }
    };
}

macro_rules! all_sizes_binop {
    ($self:ident, $other:ident in $a:ident, $b:ident => $e:expr) => {
        match ($self, $other) {
            (Item::Byte($a), Item::Byte($b)) => Item::Byte($e),
            (Item::Short($a), Item::Short($b)) => Item::Short($e),
            _ => unreachable!("mismatched item types")
        }
    };
}

impl Item {
    pub fn increment(self) -> Item {
        all_sizes!(self in n => n + 1)
    }

    pub fn shift(self, left: u8, right: u8) -> Item {
        all_sizes!(self in n => {
            n.unbounded_shr(right as u32).unbounded_shl(left as u32)
        })
    }
}

impl Add for Item {
    type Output = Item;
    fn add(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a.overflowing_add(b).0)
    }
}

impl Sub for Item {
    type Output = Item;
    fn sub(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a.overflowing_sub(b).0)
    }
}

impl Mul for Item {
    type Output = Item;
    fn mul(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a.overflowing_mul(b).0)
    }
}

impl Div for Item {
    type Output = Item;
    fn div(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => {
            if b == 0 { 0 } else { a.overflowing_div(b).0 }
        })
    }
}

impl BitAnd for Item {
    type Output = Item;
    fn bitand(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a & b)
    }
}

impl BitOr for Item {
    type Output = Item;
    fn bitor(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a | b)
    }
}

impl BitXor for Item {
    type Output = Item;
    fn bitxor(self, rhs: Self) -> Self::Output {
        all_sizes_binop!(self, rhs in a, b => a ^ b)
    }
}