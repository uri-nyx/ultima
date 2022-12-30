use std::io::Write;

use organum::core::{Steppable, Transmutable, ClockElapsed};
use organum::error::Error;
use organum::server::Server;
use organum::sys::System;
use organum::premade::serial::{Serial, Flag};

pub const INTERRUPT_TRANSMIT: u8 = 0x0a;

pub struct Tty {
    pub server: Server,
    pub trigger: char,
    pub serial: Serial,
    pub received: bool,
}

impl Tty {
    pub fn new(ip: std::net::IpAddr, port: u16, serial: &Serial) -> Self {
        Self {
            server: Server::new(ip, port).unwrap(),
            trigger: '\n',
            serial: serial.clone(),
            received: false,
        }
    }

    pub fn get_chars(&mut self) -> Result<(), Error> {

        match self.server.receiver.try_recv() {
            Ok(msg) => {
                let ch = msg[0];

                if ch as char == self.trigger {
                    self.serial.transmitting = true;
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

    pub fn transmit(&mut self, system: &System) -> Result<(), Error> {
        self.get_chars()?;

        if self.serial.transmitting {
            self.serial.transmit()?;
            system.get_interrupt_controller().set(true, 4, INTERRUPT_TRANSMIT)?;
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<(), Error>{
        
        self.serial.receive()?;

        if self.serial.done()? {
            match self.server.to_client.write(&self.serial.rx_buffer) {
                Ok(_) => self.serial.rx_buffer.clear(),
                Err(e) => panic!("Error sending data to client: {}", e)
            }
            self.serial.clear_status(Flag::DONE as u8)?;
        }

        Ok(())
    }
}

impl Transmutable for Tty {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}

impl Steppable for Tty {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        self.transmit(system)?;
        self.receive()?;

        Ok(1_000_000_000 / self.serial.frequency)
    }
}
