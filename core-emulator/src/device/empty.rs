use crate::Memory;

/// A stub device which simply acts as a normal memory page.
pub struct EmptyDevice {
    memory: [u8; 256]
}

impl EmptyDevice {
    pub fn new() -> Self {
        Self {
            memory: [0; 256],
        }
    }
}

impl Memory for EmptyDevice {
    type AddressSpace = u8;

    fn read_byte(&self, addr: Self::AddressSpace) -> u8 {
        self.memory[addr as usize]
    }
    
    fn write_byte(&mut self, addr: Self::AddressSpace, byte: u8) {
        self.memory[addr as usize] = byte;
    }
}
