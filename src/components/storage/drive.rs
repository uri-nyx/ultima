use std::fs::{File, OpenOptions};
use std::io::{Seek, Write};
use std::path::Path;
use std::io::SeekFrom;
use std::io::Read;

use organum::core::{TransmutableBox, Transmutable, Steppable, Addressable, Address, ClockElapsed};
use organum::sys::System;
use organum::error::Error;

pub const INTERRUPT_LOADED: u8 = 0xe;

pub enum Command {
    Nop,
    StoreSector,
    LoadSector,
    //Query
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0 => Command::Nop,
            1 => Command::StoreSector,
            2 => Command::LoadSector,
            _ => Command::Nop,
        }
    }
}

#[repr(usize)]
pub enum Register {
    COMMAND,
    DATA,   // Argument

    // Sector to load/store (16bit)
    SECTORH,
    SECTORL,

    // Loading point (15bit)
    POINTH,
    POINTL,

    STATUS0,
    STATUS1
}

pub const REGISTER_COUNT: usize = 8;

struct Sector {
    data: [u8; 512]
}

pub struct  Disk {
    filename: String,
    descriptor: File,
    sectors: u16
}

impl Disk {
    pub fn new(filename: String) -> Option<Self> {
        println!("PATH: {filename}");
        let descriptor = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(Path::new(&filename));
        if descriptor.is_err() { return None }
        let descriptor = descriptor.unwrap();
        descriptor.set_len(u16::MAX as u64 * 512);
        Some(Self {
            filename,
            descriptor,
            sectors: u16::MAX
        })
    }

    pub fn open(filename: String) -> Option<Self> {
        let descriptor = OpenOptions::new()
                .read(true)
                .write(true)
                .open(Path::new(&filename));
        if descriptor.is_err() { return None }
        let descriptor = descriptor.unwrap();

       Some(Self {
            filename,
            descriptor,
            sectors: u16::MAX
        })
    }


    pub fn store_sector(&mut self, sector: u16, data: &Sector) -> Result<(), std::io::Error> {
        self.descriptor.seek(SeekFrom::Start(sector as u64 * data.data.len() as u64))?;
        self.descriptor.write_all(&data.data)?;
        Ok(())
    }

    pub fn load_sector(&mut self, sector: u16, data: &mut Sector) -> Result<(), std::io::Error> {
        self.descriptor.seek(SeekFrom::Start(sector as u64 * data.data.len() as u64))?;
        self.descriptor.read(&mut data.data);
        Ok(())
    }
}

pub struct Drive {
    disk: Vec<Disk>,
    current: usize,
}


impl Drive {
    pub fn new(path: &str) -> Self {
    	let name = "disk";
        let mut disk = Vec::new();
        for i in 0..4 {
            let mut d = Disk::open(format!("{path}/{name}_{i}"));
            if d.is_none() {
            	d = Disk::new(format!("{path}/{name}_{i}"));
            	if d.is_none() {
            		use std::fs;
            		fs::create_dir_all(path).expect("Unable to create directory for disk devices");
            		d = Disk::new(format!("{path}/{name}_{i}"));	
            	}
            }
            let d = d.unwrap();
            disk.push(d);
        }

        Self {
            disk,
            current: 0
        }
    }
}

pub struct Controller {
    frequency: ClockElapsed,
    dev: TransmutableBox,
    drive: Drive,

    incoming: Sector,
    outcoming: Sector,
}

impl Controller {
    pub fn new(drive: Drive, dev: TransmutableBox, frequency: ClockElapsed) -> Self {
        Self {
            frequency,
            dev,
            drive,
            incoming: Sector {
                data: [0; 512]
            },
            outcoming: Sector {
                data: [0; 512]
            }
        }
    }

    pub fn execute(&mut self, system: &System, command: Command, (data, sector, point) : (u8, u16, u16)) -> Result<(), Error> {
        match command {
            Command::Nop => Ok(()),
            Command::StoreSector => {
                self.drive.current = self.u8_to_drive(data);

                system.get_bus().read(point as Address * 512, &mut self.incoming.data)?;

                self.drive.disk[self.drive.current].store_sector(sector, &mut self.incoming)
                    .or_else(|e| {Err(Error::new(&format!("{}", e)))})?;
                Ok(())
            },
            Command::LoadSector => {
                self.drive.current = self.u8_to_drive(data);

                self.drive.disk[self.drive.current].load_sector(sector, &mut self.outcoming)
                    .or_else(|e| {Err(Error::new(&format!("{}", e)))})?;
                system.get_bus().write(point as Address * 512, &self.outcoming.data)?;
                system.get_interrupt_controller().set(true, 5, INTERRUPT_LOADED)?;
                Ok(())
            },
        }
    }

    pub fn expose(&self) {
        // TODO: Do something with status
    }

    fn u8_to_drive(&mut self, d: u8) -> usize {
        match d {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 0,
        }
    }
}


impl Addressable for Controller {
    fn len(&self) -> usize {
        self.dev.borrow_mut().as_addressable().unwrap().len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        self.dev.borrow_mut().as_addressable().unwrap().read(addr, data)
    }

    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Error> {
        self.dev.borrow_mut().as_addressable().unwrap().write(addr, data)
    }
}

impl Steppable for Controller {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        let mut command = [0u8; 6];
        self.read(Register::COMMAND as Address, &mut command)?;
        let (data, sector, point) = (
            command[1],
            (command[2] as u16) << 8 | command[3] as u16,
            (command[4] as u16) << 8 | command[5] as u16,
        );

        self.execute(system, Command::from(command[0]), (data, sector, point))?;
        self.expose();

        Ok(1_000_000_000 / self.frequency)
    }
}

impl Transmutable for Controller {
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }

    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}
