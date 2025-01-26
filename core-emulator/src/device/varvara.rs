use std::process::exit;

use crate::Memory;

pub struct VarvaraDevice;

impl VarvaraDevice {
    pub fn new() -> Self {
        Self
    }
}

impl Memory for VarvaraDevice {
    type AddressSpace = u8;

    fn read_byte(&self, addr: Self::AddressSpace) -> u8 {
        // TODO: no reading yet
        0
    }

    fn write_byte(&mut self, addr: Self::AddressSpace, byte: u8) {
        // TODO: absolute minimal Varvara implementation for printing
        // See: https://wiki.xxiivv.com/site/varvara.html#console
        match addr {
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

            _ => panic!("unsupported device port {addr}")
        }
    }
}
