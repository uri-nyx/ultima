pub mod font;
pub mod gpu;
pub mod screen;
pub mod kbd;

use font::Font;

use organum::core::*;
use organum::error::Error;
use organum::premade::{memory};
use organum::sys::System;

use winit::event::{WindowEvent, Event};
use winit::window::Window;
use codepage_437::CP437_WINGDINGS as cp437;
use pixels::Pixels;



pub const W_WIDTH:  usize = 640;
pub const W_HEIGHT: usize = 480;

pub const PIXELS: usize = W_WIDTH * W_HEIGHT;
pub const MCHARS: (usize, usize) = (80, 25);
pub const RCHARS: (usize, usize) = (160, 50);

pub const KBD_MODE_CHAR: u8 = 0x10;
pub const KBD_MODE_KCODE: u8 = 0x20;

pub const MCOLOR: u8 = 0xf0; 
pub const DEFAULT_PALETTE: [u8; 16] = [
    0x00, 0x03, 0x1c, 0x0f, 0xe0, 0xee, 0xcc, 0xb6, 
    0x49, 0x07, 0x5d, 0x1f, 0xe1, 0xe2, 0xf8, 0xff,
];

pub const REGISTER_COUNT: usize = 20;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    MText,
    RText,
    Graphic,
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        match value {
            0 => Mode::MText,
            1 => Mode::RText,
            2 => Mode::Graphic,
            _ => Mode::MText
        }
    }
}

#[repr(usize)]
#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    // KEYBOARD
    CHARACTER,
    CODE,
    MODIFIERS,
    KBDMODE,

    // SCREEN
    COMMAND,
    DATAH,
    DATAM,
    DATAL,

    //GPU
    GPU0,
    GPU1,
    GPU2,
    GPU3,

    GPU4,
    GPU5,
    GPU6,
    GPU7,

    // STATUS
    STATUS0,
    STATUS1,
    STATUS2,
    STATUS3
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Flag {
    ACK  = 1<<0,
    DONE = 1<<1,
}


pub struct Video {
    pub mem: TransmutableBox,

    //pub gpu: gpu::gpu,
    pub kbd: kbd::Keyboard,
    pub screen: screen::Screen,
}

impl Video {
    pub fn new(system: &mut System, base: Address, w: usize, h: usize, pixels: Pixels, fonts: Vec<Font>) -> Result<Self, Error> {
        let dev = wrap_transmutable(memory::MemoryBlock::new(vec![0u8; REGISTER_COUNT]));
        system.add_addressable_device_data(base, dev)?;


        let kbd = kbd::Keyboard::new();
        let screen = screen::Screen::new(w, h, pixels, fonts);
        //let gpu = gpu::gpu::new();

        Ok(Self {
            mem: system.get_data().get_device_at(base, REGISTER_COUNT).unwrap().0,
            kbd,
            screen,
            //gpu,
        })
    }

    pub fn update(&mut self, event: &Event<()>, system: &System, window: &Window) -> Result<(), Error> {
        self.process(event, system, window)?;
        self.expose(system)?;
        Ok(())
    }

    fn process(&mut self, event: &Event<()>, system: &System, window: &Window) -> Result<(), Error> {
        self.poll(event);
        let mut command = [0u8; 4];
        self.read(Register::COMMAND as Address, &mut command)?;
        self.screen.execute(system, window, screen::Command::from(command[0]), (command[1], command[2], command[3]))?;
        Ok(())
    }

    fn expose(&mut self, system: &System) -> Result<(), Error> {
        let data = [
            self.kbd.character,
            self.kbd.code,
            self.kbd.modifiers,
            self.kbd.mode,
        ];

        self.write(Register::CHARACTER as Address, &data)?;
        if self.kbd.mode & KBD_MODE_CHAR != 0 {
            system.get_interrupt_controller().set(true, 4, 0/*fire interrupt for char*/)?;
        }
        if self.kbd.mode & KBD_MODE_KCODE != 0 {
            system.get_interrupt_controller().set(true, 4, 0+1/*fire interrupt for code*/)?;
        }

        // Clear Input Registers
        let data = [0,0,0,0];
        self.write(Register::COMMAND as Address, &data)?;
        self.write(Register::GPU0 as Address, &data)?;
        self.write(Register::GPU4 as Address, &data)?;

        Ok(())
    }

    fn poll(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input:winit::event::KeyboardInput {
                        virtual_keycode:Some(keycode),
                        scancode,
                        ..
                    },
                    ..
                },
                ..
            } => {
                if self.kbd.mode % 2 == 0 {
                    self.kbd.code = *keycode as u8;
                } else {
                    self.kbd.code = *scancode as u8;
                }
            },
            Event::WindowEvent {
                event:WindowEvent::ReceivedCharacter(ch),
                ..
            } => { 
                self.kbd.character = encode_lossy(*ch);
            }, //TODO: Use custom encodings
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(modifiers),
                ..
            } => {
                let (shift, control, alt, logo) = (
                    modifiers.shift(),
                    modifiers.ctrl(),
                    modifiers.alt(),
                    modifiers.logo()
                );
                let modifiers = (shift as u8)   << 4| 
                                    (control as u8) << 3| 
                                    (alt as u8)     << 2| 
                                    (logo as u8);

                self.kbd.modifiers = modifiers;
            },
            _ => ()
        }
    }

}

impl Addressable for Video {

    fn len(&self) -> usize {
        self.mem.borrow_mut().as_addressable().unwrap().len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        self.mem.borrow_mut().as_addressable().unwrap()
        .read(addr, data)?;
        Ok(())
    }

    fn write(&mut self, addr:  Address, data: &[u8]) -> Result<(), Error> {
        self.mem.borrow_mut().as_addressable().unwrap()
        .write(addr, data)?;
        Ok(())
    }
}

// This is not steppable per se, as it runs in the main loop alone
impl Transmutable for Video {
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}

#[inline(always)]
fn encode_lossy(c: char) -> u8 {
    let encoded = cp437.encode(c);

    match encoded {
        Some(ch) => ch,
        None => c as u8
    }
}