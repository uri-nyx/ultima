use std::io::Write;

use organum::core::{Steppable, Transmutable, ClockElapsed, Addressable, Address};
use organum::error::Error;
use organum::premade::server::Server;
use organum::sys::System;
use organum::premade::serial::Serial;

pub const INTERRUPT_TRANSMIT: u8 = 0x0a;

pub struct Tty {
    pub server: Server,
    pub trigger: char,
    pub serial: Serial,
    pub received: bool,
    pub stop: bool,
}

impl Tty {
    pub fn new(ip: std::net::IpAddr, port: u16, serial: Serial) -> Self {
        Self {
            server: Server::new(ip, port).unwrap(),
            trigger: '\n',
            serial: serial,
            received: false,
            stop: false
        }
    }

    pub fn get_chars(&mut self, system: &System) -> Result<(), Error> {

        match self.server.receiver.try_recv() {
            Ok(msg) => {
                let ch = msg[0];

                if ch as char == self.trigger {
                    system.get_interrupt_controller().set(true, 4, INTERRUPT_TRANSMIT)?;
                    return Ok(());
                }

                self.serial.tx_buffer.push(ch);
                Ok(())
            }

            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(()),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                println!("Client disconnected");
                Err(Error::new(&format!(
                    "Client disconnected: {}",
                    std::sync::mpsc::TryRecvError::Disconnected
                )))
            }
        }
    }


    fn receive(&mut self) -> Result<(), Error>{
        

        if self.serial.rx()? {
            match self.server.to_client.write(&self.serial.rx_buffer) {
                Ok(_) => self.serial.rx_buffer.clear(),
                Err(e) => {
                    self.stop = true;
                    return Err(Error::breakpoint(&format!("Error sending data to client: {}", e)))
                }
            }
        }

        Ok(())
    }

            
}

impl Transmutable for Tty {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }

    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}

impl Steppable for Tty {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        if !self.stop {
            self.get_chars(system)?;
            self.receive()?;
        }

        Ok(1_000_000_000 / self.serial.frequency)
    }
}

impl Addressable for Tty {
    fn len(&self) -> usize {
        self.serial.len()
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        self.serial.read(addr, data)?;
        Ok(())
    }

    fn write(&mut self, addr:  Address, data: &[u8]) -> Result<(), Error> {
        self.serial.write(addr, data)?;
        Ok(())
    }
}
