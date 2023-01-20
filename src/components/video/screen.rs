// The screen module provides functions to render the different video modes for Sirius
use pixels::{Pixels, Error, wgpu::Color};
use image::{Rgba, Pixel};
use winit::{dpi::LogicalSize, window::Window};
use organum::sys::System;

use crate::components::video::*;
use crate::components::video::font::Font;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Nop,
    Clear,
    SetMode,
    //SetChar,
    SetFont,
    //SetPixel,
    Blit,
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0 => Command::Nop,
            1 => Command::Clear,
            2 => Command::SetMode,
            //3 => Command::SetChar,
            4 => Command::SetFont,
            //5 => Command::SetPixel,
            6 => Command::Blit,
            _ => Command::Nop,
        }
    }
}


#[derive(Debug)]
pub struct Screen {
    mode: Mode,
    pub width: usize,
    pub height: usize,
    pub framebuffer: Pixels,

    pub mtextbuf: [u8; MCHARS.0 * MCHARS.1],
    pub rtextbuf: [u8; RCHARS.0 * RCHARS.1 * 2],
    pub graphics: [u8; PIXELS],

    pub fonts: Vec<Font>,
    monochrome_font: usize,
    color_font: usize,


    blinking: bool,
    palette: [u8; 16],
    mtextcolor: u8,
    //TODO: add mor buffers to allow hardware scrolling?
}

#[allow(unused)]
impl Screen {
   
    pub fn new(width: usize, height: usize, pixels: Pixels, fonts: Vec<Font>) -> Self {
        Self {
            mode: Mode::MText,
            width,
            height,
            framebuffer: pixels,

            mtextbuf: [0u8; MCHARS.0 * MCHARS.1],
            rtextbuf: [0u8; RCHARS.0 * RCHARS.1 * 2],
            graphics: [0u8; PIXELS],

            fonts,
            monochrome_font: 0,
            color_font: 1,

            blinking: false,
            palette: DEFAULT_PALETTE,
            mtextcolor: MCOLOR,
        }
    }

    pub fn execute(&mut self, system: &System, window: &Window, command: Command, (dh, dm, dl) : (u8, u8, u8)) -> Result<(), organum::error::Error> {
        match command {
            Command::Nop => Ok(()),
            Command::Clear => {
                self.clear();
                Ok(())
            },
            Command::SetMode => {
                println!("Mode set");
                set_mode_and_resize(self, Mode::from(dh), &window).or_else(|e| {Err(organum::error::Error::new(&format!("{}", e)))})
            },
            //Command::SetChar => self.set_char(), //TODO: Maybe it is not necessary fi we provide a pointer to the buffer
            Command::SetFont => {
                self.set_font(dh);
                Ok(())
            }
            //Command::SetPixel => self.set_pixel(),
            Command::Blit => {
                println!("blit");
                self.blit(system, dh, dm, dl)?;
                Ok(())
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode, color: Color) -> Result<(usize, usize), Error> {
        self.framebuffer.set_clear_color(color);
        // Set the mode and resize if necessary
        match mode {
            Mode::MText => {
                let font = &self.fonts[self.monochrome_font];
                let new_w = font.char_w * MCHARS.0;
                let new_h = font.char_h * MCHARS.1;
                if new_w != self.width || new_h != self.height {
                    self.framebuffer.resize_buffer(new_w as u32, new_h as u32)?;
                    self.width = new_w;
                    self.height = new_h;
                }
                self.framebuffer.render()?;
                self.mode = mode;
                Ok((new_w, new_h))
            },
            Mode::RText => {
                let font = &self.fonts[self.color_font];
                let new_w = font.char_w * RCHARS.0;
                let new_h = font.char_h * RCHARS.1;
                if new_w != self.width || new_h != self.height {
                    self.framebuffer.resize_buffer(new_w as u32, new_h as u32)?;
                    self.width = new_w;
                    self.height = new_h;
                }
                self.framebuffer.render()?;
                self.mode = mode;
                Ok((new_w, new_h))

            }
            Mode::Graphic => {
                if W_WIDTH != self.width || W_HEIGHT != self.height {
                    self.framebuffer.resize_buffer(W_WIDTH as u32, W_HEIGHT as u32)?;
                    self.width = W_WIDTH;
                    self.height = W_HEIGHT;
                }
                self.framebuffer.render()?;
                self.mode = mode;
                Ok((W_WIDTH, W_HEIGHT))
            }
        }
    }


    pub fn render(&mut self) -> Result<(), Error>{
        self.framebuffer.get_frame_mut().fill(0);
        match self.mode {
            Mode::MText => {
                let font = self.fonts[self.monochrome_font].to_owned();
                self.render_mtext(&font);
                self.framebuffer.render()?;
                Ok(())
            },
            Mode::RText => {
                let font = self.fonts[self.color_font].to_owned();
                self.blinking = !self.blinking;
                self.render_rtext(&font);
                self.framebuffer.render()?;
                Ok(())
            }
            Mode::Graphic => {
                self.render_graphic();
                self.framebuffer.render()?;
                Ok(())
            }
        }
    }

    // consider rendering text in a shader -> It involves far too much boilerplate
    pub fn render_mtext(&mut self, font: &Font) {
        let color = self.mtextcolor.clone();
        let iter = self.mtextbuf.clone();
        let iter = iter.chunks_exact(MCHARS.0).enumerate().clone();
        for (row, line) in iter {
            let y = row * font.char_h.clone();
            self.render_slice(color, line, 0, y, font);
        }
    }

    pub fn render_rtext(&mut self, font: &Font) {
        let iter = self.rtextbuf.clone();
        let iter = iter.chunks_exact((RCHARS.0) * 2).enumerate().clone();
        for (row, line) in iter {
            let y = row * font.char_h.clone();
            self.render_slice_u16(line, 0, y, font);
        }
    }

    pub fn render_graphic(&mut self) {
        // map 3RGB to RGBA and copy into framebuffer
        let fb = self.framebuffer.get_frame_mut();

        for (i, pixel) in fb.chunks_exact_mut(4).enumerate() {

            let (r, g, b, a) = rgb332_rgba(self.graphics[i]);
            pixel.copy_from_slice(&[r, g, b, a]);
        }
    }


    pub fn render_string(&mut self, color: u8, txt: &str, x: usize, y: usize, font: &Font) {
        let fb = self.framebuffer.get_frame_mut();
        let mut cursor = x;
        for ch in txt.chars() {
            let (r,g,b,a) = rgb332_rgba(color);
            render_char(ch, Rgba([r,g,b,a]), cursor, y, font, fb, self.width);
            cursor += font.char_w;
        }
    }

    pub fn render_slice(&mut self, color: u8, txt: &[u8], x: usize, y: usize, font: &Font) {
        let fb = self.framebuffer.get_frame_mut();
        let mut cursor = x;
        for ch in txt {
            let (r,g,b,a) = rgb332_rgba(color);
            render_char(*ch as char, Rgba([r,g,b,a]), cursor, y, font, fb, self.width);
            cursor += font.char_w;
        }
    }

    pub fn render_slice_u16(&mut self, txt: &[u8], x: usize, y: usize, font: &Font) {
        let fb = self.framebuffer.get_frame_mut();
        let mut cursor = x;
        for word in txt.chunks_exact(2) {
            let ch = word[0];
            let properties = word[1];
            let blink = properties & 0x80 != 0;
            let bg = (properties & 0x70) >> 4;
            let fg = properties & 0x0f;

            if blink && !self.blinking {
                cursor += font.char_w;
                continue
            } else {
                let (r,g,b,a) = rgb332_rgba(self.palette[bg as usize]);
                render_box(Rgba([r,g,b,a]), cursor, y, font.char_w, font.char_h, fb, self.width);
                let (r,g,b,a) = rgb332_rgba(self.palette[fg as usize]);
                render_char(ch as char, Rgba([r,g,b,a]), cursor, y, font, fb, self.width);
                cursor += font.char_w;
            }
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        let fb = self.framebuffer.get_frame_mut();
        let (r,g,b,a) = rgb332_rgba(color);
        let index = (y * self.width + x) * 4;
        fb[index + 0] = r;
        fb[index + 1] = g;
        fb[index + 2] = b;
        fb[index + 2] = a;
    }

    pub fn clear(&mut self) {
        let fb = self.framebuffer.get_frame_mut();
        for i in 0..self.width * self.height * 4 {
            fb[i] = 0;
        }
    }

    pub fn set_font(&mut self, font: u8) {
        let font = if font as usize > self.fonts.len() {
            font as usize
        } else {
            0
        };

        match self.mode {
            Mode::MText => self.monochrome_font = font,
            Mode::RText => self.color_font = font,
            _ => ()
        }
    }

    pub fn blit(&mut self, system: &System,h: u8, m: u8, l: u8) -> Result<(), organum::error::Error> {
        let addr = (h as Address) << 16 | (m as Address) << 8 | l as Address;
        match self.mode {
            Mode::MText => {
                let mut data = [0u8; MCHARS.0 * MCHARS.1];
                system.get_bus().read(addr, &mut data)?;
                self.mtextbuf.copy_from_slice(&mut data);
            },
            Mode::RText => {
                let mut data = [0u8; RCHARS.0 * RCHARS.1];
                system.get_bus().read(addr, &mut data)?;
                self.rtextbuf.copy_from_slice(&mut data);
            },
            Mode::Graphic => {
                let mut data = [0u8; PIXELS];
                system.get_bus().read(addr, &mut data)?;
                self.graphics.copy_from_slice(&mut data);
            }
        };

        Ok(())
    }
}

#[inline(always)]
fn rgb332_rgba(col: u8) -> (u8, u8, u8, u8) {
    // Convert 3-bit RGB to RGBA
    let col = col as u32;
    let r = (((col & 0b1110_0000) >> 5) * 255) / 7;
    let g = (((col & 0b0001_1100) >> 2) * 255) / 7;
    let b =  ((col & 0b0000_0011) * 255) / 3;
    let a = col;

    (r as u8, g as u8, b as u8, a as u8)
}

#[allow(unused)]
#[inline(always)]
fn rgb888_to_rgb332(r: u8, g: u8, b: u8) -> u8 {
    let r = ((r as u32 * 8) / 256) as u8;
    let g = ((g as u32 * 8) / 256) as u8;
    let b = ((b as u32 * 4) / 256) as u8;
    (r << 5) + (g << 2) + b
}

#[inline(always)]
fn render_char(ch: char, color: Rgba<u8>, x: usize, y: usize, font: &Font, frame: &mut [u8], width: usize) {
    let char_bmp = &font.data[ch as usize];
    for i in y..y+font.char_h {
        for j in x..x+font.char_w {
            let fb_index = j*4 + i*4 * width;
            let color_index = (j - x) + (i - y) * char_bmp.w as usize;
            let pixel = char_bmp.data[color_index];
            let pixel = pixel.channels();

            if pixel != &[255, 255, 255, 255] {
                continue
            }
            //println!("[{}x{}] j: {}*4 + (y:{}+i:{})*4 * w:{} -> {}", x, y, j, i, y, width, fb_index);
            frame[fb_index + 0] = color[0];
            frame[fb_index + 1] = color[1];
            frame[fb_index + 2] = color[2];
            frame[fb_index + 3] = color[3];
        }
    }

} 

#[inline(always)]
fn render_box(color: Rgba<u8>, x: usize, y: usize, w: usize, h: usize, frame: &mut [u8], width: usize) {
    
    for i in y..y+h {
        for j in x..x+w {
            let fb_index = j*4 + (y+i)*4 * width;

            if fb_index < frame.len() {
                frame[fb_index + 0] = color[0];
                frame[fb_index + 1] = color[1];
                frame[fb_index + 2] = color[2];
                frame[fb_index + 3] = color[3];
            }
        }
    }

}

fn set_mode_and_resize(screen: &mut Screen, mode: Mode, window: &winit::window::Window) -> Result<(), Error> {
    let (w, h) = screen.set_mode(mode, Color::TRANSPARENT)?;
    screen.framebuffer.resize_surface(w as u32, h as u32)?;
    if !window.is_maximized() {
        window.set_inner_size(LogicalSize::new(w as u32, h as u32));
    }

    Ok(())
}