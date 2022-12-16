// memory.rs provides simple abstractions for byte-addressable memory
use crate::core::*;
use crate::error::Error;
use std::fs;

pub struct MemoryBlock {
    read_only: bool,
    contents: Vec<u8>,
}

impl MemoryBlock {
    pub fn new(contents: Vec<u8>) -> MemoryBlock {
        MemoryBlock {
            read_only: false,
            contents
        }
    }

    pub fn load(filename: &str) -> Result<MemoryBlock, Error> {
        match fs::read(filename) {
            Ok(contents) => Ok(MemoryBlock::new(contents)),
            Err(_) => Err(Error::new(&format!("Error reading contents of {}", filename))),
        }
    }

    pub fn load_at(&mut self, addr: Address, filename: &str) -> Result<(), Error> {
        match fs::read(filename) {
            Ok(contents) => {
                for i in 0..contents.len() {
                    self.contents[(addr as usize) + i] = contents[i];
                }
                Ok(())
            },
            Err(_) => Err(Error::new(&format!("Error reading contents of {}", filename))),
        }
    }

    pub fn read_only(&mut self) {
        self.read_only = true;
    }

    pub fn resize(&mut self, new_size: usize) {
        self.contents.resize(new_size, 0);
    }
}

impl Addressable for MemoryBlock {
    fn len(&self) -> usize {
        self.contents.len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        for i in 0..data.len() {
            data[i] = self.contents[(addr as usize) + i];
        }
        Ok(())
    }

    fn write(&mut self, addr:  Address, data: &[u8]) -> Result<(), Error> {
        if self.read_only {
            return Err(Error::breakpoint(&format!("Attempt to write to read-only memory at {:x} with data {:?}", addr, data)));
        }

        for i in 0..data.len() {
            self.contents[(addr as usize) + i] = data[i];
        }
        Ok(())
    }
}

impl Transmutable for MemoryBlock {
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}