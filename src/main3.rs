mod components;

use std::time;
use std::fs;
use std::path::PathBuf;
use std::net::SocketAddr;

use log::error;
use winit::{event::Event, event_loop::ControlFlow};
use clap::{arg, command, value_parser, ArgAction, Command};

use organum::error::Error;
use std::{
    net::IpAddr, 
    path::Path,
    io,
};
use regex::Regex;
use pixels::{Pixels, SurfaceTexture};
use winit::{

    dpi::LogicalSize,
    window::{WindowBuilder, Window},
    event_loop::EventLoop,
};

use organum::{
    sys::System,
    core::{wrap_transmutable, Address, Addressable, Debuggable},
    premade::{memory::MemoryBlock, serial::{self, Serial}, bus::BusPort}, 

};
use winit_input_helper::WinitInputHelper;

use crate::components::{
    TaleaCpuType,
    tty::Tty,
    cpu::state::Sirius,
    storage::{drive, tps},
    video::{W_WIDTH, W_HEIGHT, Video, font::Font},
};


pub const DATA_BUS_SIZE: usize = 32*1;
pub const ADDR_BUS_MAIN_SIZE: usize = 24*1;
pub const ADDR_BUS_DATA_SIZE: usize = 16*1;


pub const MEMSIZE: usize = 1 << ADDR_BUS_MAIN_SIZE;
pub const DATSIZE: usize = 1 << ADDR_BUS_DATA_SIZE;

pub const IVT_SIZE: usize = 4 * 256;

pub const CPU_FREQUENCY: u32 = 10_000_000;
pub const TTY_FREQUENCY: u64 = 3_000_000;
pub const DRIVE_BASE: Address = 0x0;
pub const TPS_BASE: Address = 0xa;
pub const TTY_BASE: Address = 0x12;
pub const VIDEO_BASE: Address = 0x1c;
pub const END_IO: Address = 0x30 as Address;
pub const DATA_MEMORY_REST: usize = DATSIZE - END_IO as usize;
pub const TITLE: &'static str = "Taleä Computing System";
pub const FONT_PATH: &'static str = "assets/fonts";
pub const TPS_PATH: &'static str = "dev/tps/tps";
pub const DISK_PATH: &'static str = "dev/drive/disk";




fn main() -> Result<(), Error> {    
    let mut system = System::new();
    let main_port = BusPort::new(0, ADDR_BUS_MAIN_SIZE as u8, DATA_BUS_SIZE as u8, system.bus.clone());
    let data_port = BusPort::new(0, ADDR_BUS_DATA_SIZE as u8, DATA_BUS_SIZE as u8, system.bus_data.clone());

    let mut rom = MemoryBlock::load("tests/bin/hello.bin")?;
    let ram = MemoryBlock::new(vec![0; MEMSIZE - rom.len()]);
    let data = MemoryBlock::new(vec![0; DATA_MEMORY_REST]);
    let drive_ports = MemoryBlock::new(vec![0; drive::REGISTER_COUNT]);
    let tps_ports = MemoryBlock::new(vec![0; tps::REGISTER_COUNT]);
    let drive = drive::Drive::new(DISK_PATH);
    let tps = tps::Drive::new(TPS_PATH);
    let drive_controller = drive::Controller::new(drive, wrap_transmutable(drive_ports.clone()), 1_000_000);
    let tps_controller = tps::Controller::new(tps, wrap_transmutable(tps_ports.clone()), 100_000);
    
    rom.read_only();
    let rom_len = rom.len() as Address;
    system.add_addressable_device(0, wrap_transmutable(rom))?;
    system.add_addressable_device(rom_len, wrap_transmutable(ram))?;

    let mut cpu = Sirius::new(
        TaleaCpuType::SiriusType,
        10_000_000,
        main_port,
        data_port,
    );
    cpu.add_breakpoint(0);
    system.enable_debugging();
    system.add_interruptable_device("Sirius-cpu", wrap_transmutable(cpu))?;


    system.add_peripheral_data("Drive-Controller", DRIVE_BASE, wrap_transmutable(drive_controller))?;
    system.add_peripheral_data("Tps-Controller", TPS_BASE, wrap_transmutable(tps_controller))?;
    

    let serial = Serial::new(TTY_BASE, TTY_FREQUENCY);
    system.add_addressable_device_data(TTY_BASE, wrap_transmutable(serial.clone()))?;
    let mut tty = Tty::new("127.0.0.1".parse().unwrap(), 65432, &serial);
    tty.server.run();
    system.add_device("Tty-0", wrap_transmutable(tty))?;



    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(W_WIDTH as f64 , W_HEIGHT as f64);
        WindowBuilder::new()
            .with_title(TITLE)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(W_WIDTH as u32, W_HEIGHT as u32, surface_texture).or_else(|e| {Err(organum::error::Error::new(&format!("{}", e)))})?
    };
    system.add_addressable_device_data(END_IO, wrap_transmutable(data))?;
    let mut video = Video::new(&mut system, VIDEO_BASE, W_WIDTH, W_HEIGHT, pixels, collect_fonts(&Path::new(FONT_PATH)).unwrap())?;

    event_loop.run(move |event, _, control_flow| {

        let now = time::Instant::now();
        if let Event::RedrawRequested(_) = event {

            if let Err(err) = video.screen.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = video.screen.framebuffer.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            // Update internal state and request a redraw

            window.request_redraw();
        }

        video.update(&event, &system, &window);
        let elapsed = now.elapsed().as_millis();
        println!("Frame took: {}ms to render", elapsed);

        let now = time::Instant::now();
        system.run_for(10_000_000);
        let elapsed = now.elapsed().as_millis();
        println!("Cpu took: {}ms to run 10k cycles", elapsed);
    });
}

fn collect_fonts(p: &Path) -> io::Result<Vec<Font>> {
    let re = Regex::new(r".*_(\d*)x(\d*)\..*").unwrap();
    let mut fonts = Vec::new();
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        if entry.path().is_file() {
            let fname = entry.file_name();
            let cap = re.captures(fname.to_str().unwrap());
            if cap.is_some() {
                let cap = cap.unwrap();
                let width: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                let height: usize = cap.get(2).unwrap().as_str().parse().unwrap();
                let path = entry.path();
                let path = path.as_path();
                let font = Font::new(width, height, path);
                fonts.push(font)
            }
        }
    }
    Ok(fonts)
}