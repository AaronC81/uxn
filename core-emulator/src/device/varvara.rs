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
        // TODO: reading mostly unimplemented
        match addr {
            // .Screen/width
            0x22 => ((self.screen.get_size().0 & 0xFF00) >> 8) as u8,
            0x23 => ((self.screen.get_size().0 & 0x00FF)     ) as u8,

            // .Screen/height
            0x24 => ((self.screen.get_size().1 & 0xFF00) >> 8) as u8,
            0x25 => ((self.screen.get_size().1 & 0x00FF)     ) as u8,

            // .Screen/x
            0x28 => ((self.screen.x & 0xFF00) >> 8) as u8,
            0x29 => ((self.screen.x & 0x00FF)     ) as u8,

            // .Screen/y
            0x2a => ((self.screen.y & 0xFF00) >> 8) as u8,
            0x2b => ((self.screen.y & 0x00FF)     ) as u8,            

            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: Self::AddressSpace, byte: u8) {
        // See: https://wiki.xxiivv.com/site/varvara.html
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
                self.screen.vector = Some(with_high_byte(self.screen.vector.unwrap_or(0), byte));
            },
            0x21 => {
                self.screen.vector = Some(with_low_byte(self.screen.vector.unwrap_or(0), byte));
            },

            // .Screen/width
            0x22 => self.screen.map_size(|w, h| (with_high_byte(w, byte), h)),
            0x23 => self.screen.map_size(|w, h| (with_low_byte(w, byte), h)),

            // .Screen/height
            0x24 => self.screen.map_size(|w, h| (w, with_high_byte(h, byte))),
            0x25 => self.screen.map_size(|w, h| (w, with_low_byte(h, byte))),

            // .Screen/x
            0x28 => set_high_byte(&mut self.screen.x, byte),
            0x29 => set_low_byte( &mut self.screen.x, byte),

            // .Screen/y
            0x2a => set_high_byte(&mut self.screen.y, byte),
            0x2b => set_low_byte( &mut self.screen.y, byte),

            // .Screen/addr
            0x2c => set_high_byte(&mut self.screen.sprite_addr, byte),
            0x2d => set_low_byte( &mut self.screen.sprite_addr, byte),

            // .Screen/pixel
            0x2e => {
                let (fill, layer, flip_y, flip_x, _, _, c1, c0) = explode_byte(byte);

                // 2-bit number is a colour index
                let colour = self.screen.colours[((c1 as usize) << 1) | (c0 as usize)];
                let layer = if layer { Layer::Foreground } else { Layer::Background };

                if fill {
                    let x_dir = if flip_x { FillDirection::Negative } else { FillDirection::Positive };
                    let y_dir = if flip_y { FillDirection::Negative } else { FillDirection::Positive };

                    self.screen.fill_pixels(self.screen.x, self.screen.y, x_dir, y_dir, colour, layer);
                } else {
                    self.screen.draw_pixel(self.screen.x, self.screen.y, colour, layer);
                }
            },

            // .Screen/sprite
            0x2f => {
                // TODO
                println!("Warning: Tried to draw a sprite, not supported yet")
            }

            _ => panic!("unsupported device port {addr}")
        }
    }
}

struct Screen {
    vector: Option<u16>,
    window: Window,
    colours: [Colour; 4],

    framebuffer_background: Vec<u32>,
    framebuffer_foreground: Vec<u32>,

    x: u16,
    y: u16,
    sprite_addr: u16,
}

impl Screen {
    pub fn new() -> Self {
        let mut screen = Screen {
            vector: None,
            window: Self::create_window(800, 600),
            colours: [Colour::new(); 4],

            framebuffer_background: vec![],
            framebuffer_foreground: vec![],

            x: 0,
            y: 0,
            sprite_addr: 0,
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

    fn create_window(mut width: u16, mut height: u16) -> Window {
        if width == 0 { width = 1 }
        if height == 0 { height = 1 }

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

        let fb = self.overlay_framebuffers();
        self.window
            .update_with_buffer(&fb, width as usize, height as usize)
            .expect("could not update framebuffer");
    }

    fn reset_framebuffer(&mut self) {
        // Each frame starts off filled with colour 0
        let colour = self.colours[0].to_0rgb();
        let (width, height) = self.get_size();

        let size = (width as usize) * (height as usize);

        self.framebuffer_background = vec![colour; size];
        self.framebuffer_foreground = vec![colour; size];
    }

    fn overlay_framebuffers(&mut self) -> Vec<u32> {
        let transparency = self.colours[0].to_0rgb();

        self.framebuffer_background.iter().zip(&self.framebuffer_foreground)
            .map(|(bg, fg)| {
                // colour 0 is transparent on the foreground
                if *fg == transparency {
                    *bg
                } else {
                    *fg
                }
            })
            .collect()
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, c: Colour, layer: Layer) {
        // Ignore off-screen painting
        let (width, height) = self.get_size();
        if x >= width || y >= height {
            return;
        }

        let index = y as usize * width as usize + x as usize;
        self.get_framebuffer(layer)[index] = c.to_0rgb();
    }

    pub fn fill_pixels(&mut self, x_start: u16, y_start: u16, x_dir: FillDirection, y_dir: FillDirection, c: Colour, layer: Layer) {
        // Ignore fill if it starts off-screen
        let (width, height) = self.get_size();
        if x_start >= width || y_start >= height {
            return;
        }

        let x_range = match x_dir {
            FillDirection::Positive => x_start..width,
            FillDirection::Negative => 0..x_start,
        };
        let y_range = match y_dir {
            FillDirection::Positive => y_start..height,
            FillDirection::Negative => 0..y_start,
        };

        // TODO: can do memset or something
        for x in x_range {
            for y in y_range.clone() {
                self.draw_pixel(x, y, c, layer);
            }
        }
    }

    fn get_framebuffer(&mut self, layer: Layer) -> &mut Vec<u32> {
        match layer {
            Layer::Foreground => &mut self.framebuffer_foreground,
            Layer::Background => &mut self.framebuffer_background,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Foreground,
    Background,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillDirection {
    Positive,
    Negative,
}

fn with_high_byte(short: u16, new: u8) -> u16 {
    (short & 0x00FF) | ((new as u16) << 8)
}

fn with_low_byte(short: u16, new: u8) -> u16 {
    (short & 0xFF00) | (new as u16)
}

fn set_high_byte(short: &mut u16, new: u8) {
    *short = with_high_byte(*short, new);
}

fn set_low_byte(short: &mut u16, new: u8) {
    *short = with_low_byte(*short, new);
}

// MSB first
fn explode_byte(byte: u8) -> (bool, bool, bool, bool, bool, bool, bool, bool) {
    (
        byte & 0b1000_0000 != 0,
        byte & 0b0100_0000 != 0,
        byte & 0b0010_0000 != 0,
        byte & 0b0001_0000 != 0,
        byte & 0b0000_1000 != 0,
        byte & 0b0000_0100 != 0,
        byte & 0b0000_0010 != 0,
        byte & 0b0000_0001 != 0,
    )
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
