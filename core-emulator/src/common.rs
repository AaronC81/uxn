#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemSize {
    Byte,
    Short,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Byte(i8),
    Short(i16),
}

macro_rules! all_sizes {
    ($self:ident, $i:ident, $e:expr) => {
        match $self {
            Item::Byte($i) => Item::Byte($e),
            Item::Short($i) => Item::Short($e),
        }
    };
}

impl Item {
    pub fn increment(self) -> Item {
        all_sizes!(self, n, n + 1)
    }
}
