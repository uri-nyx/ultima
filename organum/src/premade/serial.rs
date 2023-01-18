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
    pub busy: bool,
    pub frequency: u64
}

#[repr(usize)]
#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    TX = 0x00,
    RX = 0x02,
    STATUS = 0x04,
    CTRL = 0x05,
}

pub const REGISTER_COUNT: usize = 6;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Flag {
    BUSY  = 1<<0,
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
            busy: false,
            frequency
        }
    }

    pub fn tx(&mut self) -> Result<u8, Error> {
            match self.tx_buffer.pop() {
                Some(byte) => {
                    Ok(byte)
                }
                None => {
                    Ok(0)
                }
        }
    }

    pub fn rx(&mut self) -> Result<bool, Error> {
        let byte = self.read_u8(Register::RX as Address)?;
        self.rx_buffer.push(byte);
        self.write_u8(Register::RX as Address, 0)?;

        Ok(byte == 0)
    }

    pub fn is_busy(&mut self) -> Result<bool, Error> {
        Ok(self.read_u8(Register::STATUS as Address)? & Flag::BUSY as u8 != 0)
    }

    pub fn set_done(&mut self) -> Result<(), Error> {
        let stat = self.read_u8(Register::STATUS as Address)?;
        self.write_u8(Register::STATUS as Address, stat | Flag::DONE as u8)?;
        Ok(())
    }

}

impl Addressable for Serial {
    fn len(&self) -> usize {
        self.mem.dev.borrow_mut().as_addressable().unwrap().len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {

        for i in 0..data.len() {
            let addr = addr + i as Address;
            match addr {
                0 => {
                    let b = self.tx()?;
                    data[i] = b;
                }
                1 => {
                    let b = self.tx_buffer.len() as u8;
                    data[i] = b;
                }
                _ => data[i] = self.mem.dev.borrow_mut().as_addressable().unwrap().read_u8(addr)?
            }
        }

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