use crate::{common::{Item, ItemSize}, Memory};

use super::{Core, ROM_BASE};

impl Core {
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
}

impl Memory for Core {
    type AddressSpace = u16;

    fn read_memory(&self, addr: Self::AddressSpace, item_size: ItemSize) -> Item {
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

    fn write_memory(&mut self, addr: Self::AddressSpace, item: Item) {
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
}
