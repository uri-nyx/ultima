use organum::core::{Steppable, Transmutable, ClockElapsed, Clock, Addressable, Address, wrap_transmutable};
use organum::error::Error;
use organum::premade::{bus::Block, memory::MemoryBlock};
use organum::sys::System;

pub const INTERRUPT_TIMEOUT: u8 = 0x0f;
pub const INTERRUPT_INTERVAL: u8 = 0x10;


// TODO: figure out real timing of cycles (hopefully manage 10Mhz stable)
pub struct Timer {
    mem: Block,
    now: Clock,
    frequency: u64,
    timeout_enable: bool,
    interval_enable: bool,
}

pub enum Register {
    TIMEOUT = 0x0,
    INTERVAL = 0x2
}

pub const REGISTER_COUNT: usize = 4;

impl Timer {
    pub fn new(base: Address, frequency: u64) -> Self {
        let dev = MemoryBlock::new(vec![0u8;  REGISTER_COUNT]);
        Self {
            now: 0,
            timeout_enable: false,
            interval_enable: false,
            frequency,
            mem: Block {
                base,
                length: dev.len(),
                dev: wrap_transmutable(dev)
            },
        }
    }

    fn check_timeout(&mut self) -> Result<bool, Error> {
        let timeout = self.read_beu16(Register::TIMEOUT as Address)?.wrapping_sub(1);
        Ok(timeout > 0)
    }

    fn check_interval(&mut self) -> Result<bool, Error> {
        let interval = self.read_beu16(Register::INTERVAL as Address)?;
        if interval != 0 {
            Ok(self.now % interval as Clock == 0)
        } else {
            Ok(false)
        }
    }
}

impl Transmutable for Timer {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }

    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}

impl Steppable for Timer {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        self.now = self.now.wrapping_add(1);

        if self.timeout_enable && self.check_timeout()? {
            system.get_interrupt_controller().set(true,  6, INTERRUPT_TIMEOUT)?;
            self.timeout_enable = false;
        }

        if self.interval_enable && self.check_interval()? {
            system.get_interrupt_controller().set(true,  6, INTERRUPT_INTERVAL)?;
        }

        Ok(1_000_000_000 / self.frequency)
    }
}

impl Addressable for Timer {
    fn len(&self) -> usize {
        self.mem.length
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        self.mem.dev.borrow_mut().as_addressable().unwrap().read(addr, data)?;
        Ok(())
    }

 fn write(&mut self, addr:  Address, data: &[u8]) -> Result<(), Error> {
        match addr {
        0 | 1 => self.timeout_enable = true,
        2 | 3 => self.interval_enable = !self.interval_enable,
        _ => (),
        }

        self.mem.dev.borrow_mut().as_addressable().unwrap().write(addr, data)?;
        Ok(())
    }

}
