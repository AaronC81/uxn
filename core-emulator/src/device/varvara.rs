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
            // TODO: reduce duplication in colour channel code

            // .System/red
            0x08 => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[0].set_red_from_nibble(hi);
                self.screen.colours[1].set_red_from_nibble(lo);
            },
            0x09 => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[2].set_red_from_nibble(hi);
                self.screen.colours[3].set_red_from_nibble(lo);
            },

            // .System/blue
            0x0a => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[0].set_blue_from_nibble(hi);
                self.screen.colours[1].set_blue_from_nibble(lo);
            },
            0x0b => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[2].set_blue_from_nibble(hi);
                self.screen.colours[3].set_blue_from_nibble(lo);
            },

            // .System/green
            0x0c => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[0].set_green_from_nibble(hi);
                self.screen.colours[1].set_green_from_nibble(lo);
            },
            0x0d => {
                let (hi, lo) = split_nibbles(byte);
                self.screen.colours[2].set_green_from_nibble(hi);
                self.screen.colours[3].set_green_from_nibble(lo);
            },

            // .System/state
            0x0f => {
                if byte != 0 {
                    let exit_code = (byte as u8) & 0x7f;
                    exit(exit_code as i32);
                }
            },

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
    colours: [Colour; 4],
    framebuffer: Vec<u32>,
}

impl Screen {
    pub fn new() -> Self {
        let mut screen = Screen {
            vector: None,
            window: Self::create_window(200, 200),
            colours: [Colour::new(); 4],
            framebuffer: vec![],
        };
        screen.reset_framebuffer();
        screen
    }

    pub fn get_size(&self) -> (u16, u16) {
        let (w, h) = self.window.get_size();
        (w as u16, h as u16)
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        // You can't resize the window in minifb - just create a new one instead
        self.window = Self::create_window(width, height);

        // Ensure there's no stale framebuffer
        self.reset_framebuffer();
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
        let (width, height) = self.get_size();
        self.window
            .update_with_buffer(&self.framebuffer, width as usize, height as usize)
            .expect("could not update framebuffer");

        // Get ready for our next frame
        self.reset_framebuffer(); 
    }

    fn reset_framebuffer(&mut self) {
        // Each frame starts off filled with colour 0
        let colour = self.colours[0].to_0rgb();
        let (width, height) = self.get_size();

        self.framebuffer = vec![colour; (width as usize) * (height as usize)];
    }
}

fn replace_high_byte(short: u16, new: u8) -> u16 {
    (short & 0x00FF) | ((new as u16) << 8)
}

fn replace_low_byte(short: u16, new: u8) -> u16 {
    (short & 0xFF00) | (new as u16)
}

/// A Varvara-compatible colour.
/// 
/// This holds a `minifb`-compatible 0RGB representation with 8-bits per channel, but it is in fact
/// limited to only showing Varvara's colour space, with 4 bits per channel instead of 8.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Colour(u32);

impl Colour {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set_red_from_nibble(&mut self, value: u8) {
        let scaled = (value << 4) | value;
        let [z, _, b, g] = self.0.to_be_bytes();
        self.0 = u32::from_be_bytes([z, scaled, b, g]);
    }

    pub fn set_blue_from_nibble(&mut self, value: u8) {
        let scaled = (value << 4) | value;
        let [z, r, _, g] = self.0.to_be_bytes();
        self.0 = u32::from_be_bytes([z, r, scaled, g]);
    }

    pub fn set_green_from_nibble(&mut self, value: u8) {
        let scaled = (value << 4) | value;
        let [z, r, b, _] = self.0.to_be_bytes();
        self.0 = u32::from_be_bytes([z, r, b, scaled]);
    }

    pub fn to_0rgb(self) -> u32 {
        self.0
    }
}

fn split_nibbles(byte: u8) -> (u8, u8) {
    ((byte & 0xF0) >> 4, byte & 0x0F)
}
