use crate::Memory;

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

    fn read_byte(&self, addr: Self::AddressSpace) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, addr: Self::AddressSpace, byte: u8) {
        self.memory[addr as usize] = byte;
    }
}
