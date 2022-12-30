// this module contains the hardware configuration for the Taleä system
use pixels::{Pixels, SurfaceTexture};
use regex::Regex;
use std::{fs, io, net::IpAddr, path::Path};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use organum::{
    core::{wrap_transmutable, Address, Addressable, Debuggable},
    error::Error,
    premade::{bus::BusPort, memory::MemoryBlock, serial::Serial, serial},
    sys::System,
};
use winit_input_helper::WinitInputHelper;

use crate::components::{
    cpu::state::Sirius,
    storage::{drive, tps},
    tty::Tty,
    video::{font::Font, Video, W_HEIGHT, W_WIDTH},
};

pub type Word = u32;
pub type Uptr = u32;

const BIT: usize = 1;

pub const DATA_BUS_SIZE: usize = 32 * BIT;
pub const ADDR_BUS_MAIN_SIZE: usize = 24 * BIT;
pub const ADDR_BUS_DATA_SIZE: usize = 16 * BIT;

pub const MEMSIZE: usize = 1 << ADDR_BUS_MAIN_SIZE;
pub const DATSIZE: usize = 1 << ADDR_BUS_DATA_SIZE;

pub const IVT_SIZE: usize = 4 * 256;

pub const CPU_FREQUENCY: u32 = 10_000_000;
pub const TTY_FREQUENCY: u64 = 3_000_000;

pub const TTY_BASE: Address     = 0;
pub const VIDEO_BASE: Address   = TTY_BASE   + serial::REGISTER_COUNT as Address;
pub const TPS_BASE: Address     = VIDEO_BASE + video::REGISTER_COUNT as Address;
pub const DRIVE_BASE: Address   = TPS_BASE   + tps::REGISTER_COUNT as Address;
pub const END_IO: Address       = DRIVE_BASE + drive::REGISTER_COUNT as Address;

pub const DATA_MEMORY_REST: usize = DATSIZE - END_IO as usize;

pub const TITLE: &'static str = "Taleä Computing System";
pub const FONT_PATH: &'static str = "assets/fonts";
pub const TPS_PATH: &'static str = "dev/tps/tps";
pub const DISK_PATH: &'static str = "dev/drive/disk";

pub enum TaleaCpuType {
    SiriusType,
}

pub mod cpu;
pub mod storage;
pub mod tty;
pub mod video;

pub struct Talea {
    pub system: System,
    pub tty: Tty,
    pub video: Video,
    pub window: Window,
    pub event_loop: EventLoop<()>,
    pub input: WinitInputHelper,
}

impl Talea {
    pub fn server_run(&mut self) {
        self.tty.server.run()
    }
}

pub fn build_talea(rom_file: &Path, ip: IpAddr, port: u16, debug: bool) -> Result<Talea, Error> {
    let mut system = System::new();
    let main_port = BusPort::new(
        0,
        ADDR_BUS_MAIN_SIZE as u8,
        DATA_BUS_SIZE as u8,
        system.bus.clone(),
    );
    let data_port = BusPort::new(
        0,
        ADDR_BUS_DATA_SIZE as u8,
        DATA_BUS_SIZE as u8,
        system.bus_data.clone(),
    );

    let mut rom = MemoryBlock::load(rom_file.to_str().unwrap())?;
    let ram = MemoryBlock::new(vec![0; MEMSIZE - rom.len()]);
    let data = MemoryBlock::new(vec![0; DATA_MEMORY_REST]);

    rom.read_only();
    let rom_len = rom.len() as Address;
    system.add_addressable_device(0, wrap_transmutable(rom))?;
    system.add_addressable_device(rom_len, wrap_transmutable(ram))?;

    build_cpu(&mut system, CPU_FREQUENCY, main_port, data_port, debug)?;
    build_storage(&mut system, DRIVE_BASE, TPS_BASE)?;
    let tty = build_tty(&mut system, TTY_BASE, TTY_FREQUENCY, ip, port)?;
    let event_loop = EventLoop::new();
    let input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(W_WIDTH as f64, W_HEIGHT as f64);
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
        Pixels::new(W_WIDTH as u32, W_HEIGHT as u32, surface_texture)
            .or_else(|e| Err(organum::error::Error::new(&format!("{}", e))))?
    };
    system.add_addressable_device_data(END_IO, wrap_transmutable(data))?;
    let video = Video::new(
        &mut system,
        VIDEO_BASE,
        W_WIDTH,
        W_HEIGHT,
        pixels,
        collect_fonts(&Path::new(FONT_PATH)).unwrap(),
    )?;

    println!("Video len: {}", video.len());

    Ok(Talea {
        system,
        tty,
        video,
        window,
        event_loop,
        input,
    })
}

fn build_tty(
    system: &mut System,
    addr: Address,
    frequency: u64,
    ip: IpAddr,
    port: u16,
) -> Result<Tty, Error> {
    let serial = Serial::new(addr, frequency);
    system.add_addressable_device_data(addr, wrap_transmutable(serial.clone()))?;
    println!("Created Tty:  {}", serial.len());
    let tty = Tty::new(ip, port, &serial);
    Ok(tty)
}

pub fn add_tty(system: &mut System, tty: Tty) -> Result<(), Error> {
    system.add_device("Tty-0", wrap_transmutable(tty))?;
    Ok(())
}

fn build_storage(system: &mut System, drive_addr: Address, tps_addr: Address) -> Result<(), Error> {
    let drive_ports = MemoryBlock::new(vec![0; drive::REGISTER_COUNT]);
    let tps_ports = MemoryBlock::new(vec![0; tps::REGISTER_COUNT]);
    let drive = drive::Drive::new(DISK_PATH);
    let tps = tps::Drive::new(TPS_PATH);
    let drive_controller =
        drive::Controller::new(drive, wrap_transmutable(drive_ports.clone()), 1_000_000);
    let tps_controller = tps::Controller::new(tps, wrap_transmutable(tps_ports.clone()), 100_000);

    println!("Created storage: disk {}, tps {}", drive_ports.len(), tps_ports.len());

    system.add_peripheral_data(
        "Disk-Controller",
        drive_addr,
        wrap_transmutable(drive_controller),
    )?;
    system.add_peripheral_data(
        "Tps-Controller",
        tps_addr,
        wrap_transmutable(tps_controller),
    )?;
    Ok(())
}

fn build_cpu(
    system: &mut System,
    frequency: u32,
    port: BusPort,
    port_d: BusPort,
    debug: bool,
) -> Result<(), Error> {
    let mut cpu = Sirius::new(TaleaCpuType::SiriusType, frequency, port, port_d);
    if debug {
        cpu.add_breakpoint(0);
    }
    system.add_interruptable_device("Sirius-cpu", wrap_transmutable(cpu))?;
    Ok(())
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
