// core.rs is a collection of generics to build the interfaces for a system
use core::cell::RefCell;
use std::rc::Rc;
use crate::error::Error;
use crate::sys::System;

pub type Address = u64;
pub type Clock = u64;
pub type ClockElapsed = u64;
pub type TransmutableBox = Rc<RefCell<Box<dyn Transmutable>>>;

/// A device that can be addressed to read data from or write data to the device.
pub trait Addressable {
    fn len(&self) -> usize;
    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error>;
    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Error>;

    fn read_u8(&mut self, addr: Address) -> Result<u8, Error> {
        let mut data = [0; 1];
        self.read(addr, &mut data)?;
        Ok(data[0])
    }

    fn read_beu16(&mut self, addr: Address) -> Result<u16, Error> {
        let mut data = [0; 2];
        self.read(addr, &mut data)?;
        Ok(read_beu16(&data))
    }

    fn read_leu16(&mut self, addr: Address) -> Result<u16, Error> {
        let mut data = [0; 2];
        self.read(addr, &mut data)?;
        Ok(read_leu16(&data))
    }

    fn read_beu32(&mut self, addr: Address) -> Result<u32, Error> {
        let mut data = [0; 4];
        self.read(addr, &mut data)?;
        Ok(read_beu32(&data))
    }

    fn read_leu32(&mut self, addr: Address) -> Result<u32, Error> {
        let mut data = [0; 4];
        self.read(addr, &mut data)?;
        Ok(read_leu32(&data))
    }

    fn write_u8(&mut self, addr: Address, value: u8) -> Result<(), Error> {
        let data = [value];
        self.write(addr, &data)
    }

    fn write_beu16(&mut self, addr: Address, value: u16) -> Result<(), Error> {
        let mut data = [0; 2];
        write_beu16(&mut data, value);
        self.write(addr, &data)
    }

    fn write_leu16(&mut self, addr: Address, value: u16) -> Result<(), Error> {
        let mut data = [0; 2];
        write_leu16(&mut data, value);
        self.write(addr, &data)
    }

    fn write_beu32(&mut self, addr: Address, value: u32) -> Result<(), Error> {
        let mut data = [0; 4];
        write_beu32(&mut data, value);
        self.write(addr, &data)
    }

    fn write_leu32(&mut self, addr: Address, value: u32) -> Result<(), Error> {
        let mut data = [0; 4];
        write_leu32(&mut data, value);
        self.write(addr, &data)
    }
}

pub trait Steppable {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error>;
    fn on_error(&mut self, _system: &System) { }
}
/// A device (cpu) that can debugged using the built-in debugger
pub trait Debuggable {
    fn debugging_enabled(&mut self) -> bool;
    fn set_debugging(&mut self, enable: bool);
    fn add_breakpoint(&mut self, addr: Address);
    fn remove_breakpoint(&mut self, addr: Address);

    fn print_current_step(&mut self, system: &System) -> Result<(), Error>;
    fn print_disassembly(&mut self, addr: Address, count: usize);
    fn execute_command(&mut self, system: &System, args: &[&str]) -> Result<bool, Error>;
}

/// A device (peripheral) that can inspected using the built-in debugger
pub trait Inspectable {
    fn inspect(&mut self, system: &System, args: &[&str]) -> Result<(), Error>;
}

pub trait Interruptable {
    
}
pub trait Transmutable {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        None
    }

    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        None
    }

    fn as_interruptable(&mut self) -> Option<&mut dyn Interruptable> {
        None
    }

    fn as_debuggable(&mut self) -> Option<&mut dyn Debuggable> {
        None
    }

    fn as_inspectable(&mut self) -> Option<&mut dyn Inspectable> {
        None
    }
}

pub fn wrap_transmutable<T: Transmutable + 'static>(value: T) -> TransmutableBox {
    Rc::new(RefCell::new(Box::new(value)))
}

#[inline(always)]
pub fn read_beu16(data: &[u8]) -> u16 {
    (data[0] as u16) << 8 |
    (data[1] as u16)
}

#[inline(always)]
pub fn read_leu16(data: &[u8]) -> u16 {
    (data[1] as u16) << 8 |
    (data[0] as u16)
}

#[inline(always)]
pub fn read_beu32(data: &[u8]) -> u32 {
    (data[0] as u32) << 24 |
    (data[1] as u32) << 16 |
    (data[2] as u32) << 8 |
    (data[3] as u32)
}

#[inline(always)]
pub fn read_leu32(data: &[u8]) -> u32 {
    (data[3] as u32) << 24 |
    (data[2] as u32) << 16 |
    (data[1] as u32) << 8 |
    (data[0] as u32)
}



#[inline(always)]
pub fn write_beu16(data: &mut [u8], value: u16) -> &mut [u8] {
    data[0] = (value >> 8) as u8;
    data[1] = value as u8;
    data
}

#[inline(always)]
pub fn write_leu16(data: &mut [u8], value: u16) -> &mut [u8] {
    data[0] = value as u8;
    data[1] = (value >> 8) as u8;
    data
}

#[inline(always)]
pub fn write_beu32(data: &mut [u8], value: u32) -> &mut [u8] {
    data[0] = (value >> 24) as u8;
    data[1] = (value >> 16) as u8;
    data[2] = (value >> 8) as u8;
    data[3] = value as u8;
    data
}

#[inline(always)]
pub fn write_leu32(data: &mut [u8], value: u32) -> &mut [u8] {
    data[0] = value as u8;
    data[1] = (value >> 8) as u8;
    data[2] = (value >> 16) as u8;
    data[3] = (value >> 24) as u8;
    data
}