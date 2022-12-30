use std::fs::{File, OpenOptions};
use std::io::{Seek, Write};
use std::path::Path;
use std::io::SeekFrom;
use std::io::Read;

use organum::core::{TransmutableBox, Transmutable, Steppable, Addressable, Address, ClockElapsed};
use organum::sys::System;
use organum::error::Error;

pub const INTERRUPT_LOADED: u8 = 0xd;

pub struct Tps {
    filename: String,
    descriptor: File,
    sectors: u8
}

struct Sector {
    data: [u8; 512]
}

pub struct Drive {
    tps: Vec<Tps>,
    current: usize,
}

impl Drive {
    pub fn new(path: &str) -> Self {
        let mut tps = Vec::new();
        for i in 0..2 {
            tps.push(Tps::new(format!("{}_{}", path, i)));
        }

        Self {
            tps,
            current: 0
        }
    }
}

pub enum Command {
    Nop,
    IsBootable,
    IsPresent,
    Open,
    Close,
    StoreSector,
    LoadSector
}

impl From<u8> for Command {
    fn from(value: u8) -> Command {
        let value = value & !0x80;
        match value {
            0 => Command::Nop,
            1 => Command::IsBootable,
            2 => Command::IsPresent,
            3 => Command::Open,
            4 => Command::Close,
            5 => Command::StoreSector,
            6 => Command::LoadSector,
            _ => Command::Nop,
        }
    }
}

pub enum Register {
    COMMAND,
    DATA,

    POINTH,
    POINTL,

    STATUSH,
    STATUSL,
}
pub const REGISTER_COUNT: usize = 6;

impl Tps {
    pub fn new(filename: String) -> Self {
        let descriptor = OpenOptions::new()
                .create(false)
                .read(true)
                .write(true)
                .open(Path::new(&filename)).unwrap();
        descriptor.set_len(u8::MAX as u64 * 512);
        Self {
            filename,
            descriptor,
            sectors: u8::MAX
        }
    }

    pub fn open(filename: &str) -> Self {
        let descriptor = OpenOptions::new()
                .read(true)
                .write(true)
                .open(Path::new(&filename)).unwrap();

        Self {
            filename: filename.to_owned(),
            descriptor,
            sectors: u8::MAX
        }
    }

    pub fn store_sector(&mut self, sector: u8, data: &Sector) -> Result<(), std::io::Error> {
        self.descriptor.seek(SeekFrom::Start(sector as u64 * data.data.len() as u64))?;
        self.descriptor.write_all(&data.data)?;
        Ok(())
    }

    pub fn load_sector(&mut self, sector: u8, data: &mut Sector) -> Result<(), std::io::Error> {
        self.descriptor.seek(SeekFrom::Start(sector as u64 * data.data.len() as u64))?;
        self.descriptor.read(&mut data.data);
        Ok(())
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

    pub fn execute(&mut self, system: &System, command: u8, (data, point) : (u8, u16)) -> Result<(), Error> {
        let b = command & 0x80 == 0;
        self.drive.current = if b { 0 } else { 1 };
        let command = Command::from(command);

        match command {
            Command::Nop => Ok(()),
            Command::StoreSector => {
                system.get_bus().read(point as Address * 512, &mut self.incoming.data)?;

                self.drive.tps[self.drive.current].store_sector(data, &mut self.incoming)
                    .or_else(|e| {Err(Error::new(&format!("{}", e)))})?;
                Ok(())
            },
            Command::LoadSector => {
                self.drive.tps[self.drive.current].load_sector(data, &mut self.outcoming)
                    .or_else(|e| {Err(Error::new(&format!("{}", e)))})?;
                system.get_bus().write(point as Address * 512, &self.outcoming.data)?;
                system.get_interrupt_controller().set(true, 5, INTERRUPT_LOADED)?;
                Ok(())
            },
            Command::IsBootable => {
                self.drive.tps[self.drive.current].load_sector(data, &mut self.outcoming)
                    .or_else(|e| {Err(Error::new(&format!("{}", e)))})?;
                
                if self.outcoming.data[510..511] == [0xA1, 0xEA] {
                    self.write_u8(Register::STATUSL as Address, 1)?;
                } else {
                    self.write_u8(Register::STATUSL as Address, 0)?;
                }

                Ok(())
            },
            Command::IsPresent => {
                let exists = Path::new(&self.drive.tps[self.drive.current].filename).exists();

                if exists {
                    self.write_u8(Register::STATUSL as Address, 1)?;
                } else {
                    self.write_u8(Register::STATUSL as Address, 0)?;
                }

                Ok(())
            },
            Command::Open => {
                let filename = &self.drive.tps[self.drive.current].filename;
                self.drive.tps[self.drive.current] = Tps::open(filename);
                Ok(())
            },
            Command::Close => {
                Ok(())
            }
        }
    }

    pub fn expose(&self) {
        // TODO: Do something with status
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
        let mut command = [0u8; 4];
        self.read(Register::COMMAND as Address, &mut command);
        let (data, point) = (
            command[1],
            (command[2] as u16) << 8 | command[3] as u16,
        );

        self.execute(system, command[0], (data, point))?;
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