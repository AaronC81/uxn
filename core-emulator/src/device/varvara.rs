use std::process::exit;

use minifb::{Window, WindowOptions};

use crate::Memory;

use super::{Device, DeviceEvent};

pub struct VarvaraDevice {
    screen: Screen,
}

impl VarvaraDevice {
    pub fn new() -> Self {
        Self {
            screen: Screen::new(),
        }
    }
}

impl Device for VarvaraDevice {
    fn wait_for_event(&mut self) -> DeviceEvent {
        if !self.screen.window.is_open() {
            return DeviceEvent::Exit
        }

        if let Some(vector) = self.screen.vector {
            // TODO: currently, this means whatever we draw is one frame behind
            // This is *probably* fine but does need to be sorted at some point
            self.screen.update();
            DeviceEvent::Vector(vector)
        } else {
            DeviceEvent::Exit
        }
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
            },

            // .Screen/vector
            0x20 => {
                self.screen.vector = Some(replace_high_byte(self.screen.vector.unwrap_or(0), byte));
            },
            0x21 => {
                self.screen.vector = Some(replace_low_byte(self.screen.vector.unwrap_or(0), byte));
            },

            // .Screen/width
            0x22 => self.screen.map_size(|w, h| (replace_high_byte(w, byte), h)),
            0x23 => self.screen.map_size(|w, h| (replace_low_byte(w, byte), h)),

            // .Screen/height
            0x24 => self.screen.map_size(|w, h| (w, replace_high_byte(h, byte))),
            0x25 => self.screen.map_size(|w, h| (w, replace_low_byte(h, byte))),

            _ => panic!("unsupported device port {addr}")
        }
    }
}

struct Screen {
    vector: Option<u16>,
    window: Window,
    colors: [u32; 4], // 0RGB format
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            vector: None,
            window: Self::create_window(200, 200),
            colors: [0; 4],
        }
    }

    pub fn get_size(&self) -> (u16, u16) {
        let (w, h) = self.window.get_size();
        (w as u16, h as u16)
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        // You can't resize the window in minifb - just create a new one instead
        self.window = Self::create_window(width, height);
    }

    pub fn map_size(&mut self, func: impl FnOnce(u16, u16) -> (u16, u16)) {
        let (w, h) = self.get_size();
        let (w, h) = func(w, h);
        self.set_size(w, h);
    }

    fn create_window(width: u16, height: u16) -> Window {
        let mut window = Window::new(
            "uxn",
            width as usize, height as usize, // Correct-feeling default size
            WindowOptions { resize: false, ..WindowOptions::default() },
        ).expect("could not create window");
        window.set_target_fps(60);
        window
    }

    pub fn update(&mut self) {
        self.window.update(); // TODO: with a framebuffer
    }
}

fn replace_high_byte(short: u16, new: u8) -> u16 {
    (short & 0x00FF) | ((new as u16) << 8)
}

fn replace_low_byte(short: u16, new: u8) -> u16 {
    (short & 0xFF00) | (new as u16)
}