// Serial provides a simple generic serial interface
 
use crate::core::{Transmutable, Address, Addressable, wrap_transmutable};
use crate::premade::{memory::MemoryBlock, bus::Block};
use crate::error::Error;

#[derive(Clone)]
pub struct Serial {
    pub mem: Block,
    pub tx_buffer: Vec<u8>,
    pub rx_buffer: Vec<u8>,
    pub transmitting: bool,
    pub receiving: bool,
    pub frequency: u64
}

#[repr(usize)]
#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    TX = 0x00,
    _TXL,
    RX = 0x02,
    _RXL,
    STATUS = 0x04,
    CTRL = 0x05,
}

pub const REGISTER_COUNT: usize = 6;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Flag {
    ACK  = 1<<0,
    DONE = 1<<1,
}


impl Serial {
    pub fn new(base: Address, frequency: u64) -> Self {
        let dev = MemoryBlock::new(vec![0u8;  REGISTER_COUNT]);
        Self { 
            mem: Block {
                base,
                length: dev.len(),
                dev: wrap_transmutable(dev)
            },
            tx_buffer: Vec::new(),
            rx_buffer: Vec::new(),
            transmitting: false,
            receiving: false,
            frequency
        }
    }

    pub fn transmit(&mut self) -> Result<(), Error> {
        // Steppable
        if self.transmitting && self.status()? & Flag::ACK as u8 != 0 {
            let byte = self.tx_buffer.pop();
            match byte {
                Some(byte) => self.tx(byte, self.tx_buffer.len() as u8)?,
                None => self.transmitting = false,
            }

            let stat = &[self.status()? & !(Flag::ACK as u8)];

            self.write(Register::STATUS as Address, stat)?;

            return Ok(())
        }

        Ok(())
    }

    pub fn receive(&mut self) -> Result<(), Error> {
        // Steppable
        let (byte, remaining) = self.rx()?;
        if remaining != 0 {
            self.set_status(Flag::DONE as u8)?; //TODO: temporal
            self.rx_buffer.push(byte);
        } else {
            self.set_status(Flag::DONE as u8)?;
        }

        Ok(())
    }

    pub fn done(&mut self) -> Result<bool, Error> {
        let stat = self.status()?;
        Ok(stat & Flag::DONE as u8 != 0)

    }

    #[inline(always)]
    fn tx(&mut self, byte: u8, remaining: u8) -> Result<(), Error> {
        self.write(Register::TX as Address, &[byte, remaining])?;
        Ok(())
    }

    #[inline(always)]
    fn rx(&mut self) -> Result<(u8, u8), Error> {
        let mut received = [0u8; 2];
        self.read(Register::RX as Address, &mut received)?;

        Ok((received[0], received[1]))
    }

    #[inline(always)]
    pub fn set_status(&mut self, flags: u8) -> Result<(), Error> {
        let stat = self.status()?;
        
        self.write(Register::STATUS as Address, &[stat | flags])?;
        Ok(())
    }

    #[inline(always)]
    pub fn clear_status(&mut self, flags: u8) -> Result<(), Error> {
        let stat = self.status()?;
        
        self.write(Register::STATUS as Address, &[stat & !flags])?;
        Ok(())
    }

    #[inline(always)]
    pub fn status(&mut self) -> Result<u8, Error> {
        let mut received = [0u8; 1];
        self.read(Register::STATUS as Address, &mut received)?;

        Ok(received[0])
    }

    #[inline(always)]
    pub fn ctrl(&mut self) -> Result<u8, Error> {
        let mut received = [0u8; 1];
        self.read(Register::CTRL as Address, &mut received)?;

        Ok(received[0])
    }
}

impl Addressable for Serial {
    fn len(&self) -> usize {
        self.mem.dev.borrow_mut().as_addressable().unwrap().len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        self.mem.dev.borrow_mut().as_addressable().unwrap()
        .read(addr, data)?;
        Ok(())
    }

    fn write(&mut self, addr:  Address, data: &[u8]) -> Result<(), Error> {
        self.mem.dev.borrow_mut().as_addressable().unwrap()
        .write(addr, data)?;
        Ok(())
    }
}

impl Transmutable for Serial {
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}