use std::io::Write;

use organum::core::{Steppable, Transmutable, ClockElapsed, Addressable, Address};
use organum::error::Error;
use organum::premade::bus::BusPort;
use organum::server::Server;
use organum::sys::System;

#[derive(Clone)]
pub struct Serial {
    pub rx: Address,
    pub rx_remaining: Address,
    pub tx: Address,
    pub tx_remaining: Address,

    pub transmitting: bool,
    pub port: BusPort
}

impl Serial {
    pub fn new(base: Address, port: BusPort) -> Self {
        Self { 
            rx: base, 
            rx_remaining: base + 1, 
            tx: base + 2, 
            tx_remaining: base + 3,

            transmitting: false,
            port: port
        }
    }

    #[inline(always)]
    pub fn transmit(&mut self, byte: u8, remaining: u8) -> Result<(), Error> {
        self.port.write_u8(self.tx, byte)?;
        self.port.write_u8(self.tx_remaining, remaining)?;

        Ok(())
    }

    #[inline(always)]
    pub fn receive(&mut self) -> Result<(u8, u8), Error> {
        let mut received = [0u8; 2];

        self.port.read(self.rx, &mut received)?;

        Ok((received[0], received[1]))
    }
}


pub struct Tty {
    pub server: Server,
    pub rx_buf: Vec<u8>,
    pub tx_buf: Vec<u8>,
    pub trigger: char,
    pub serial: Serial,
}

impl Tty {
    pub fn new(ip: std::net::IpAddr, port: u16, serial: &Serial) -> Self {
        Self {
            server: Server::new(ip, port).unwrap(),
            rx_buf: Vec::<u8>::new(),
            tx_buf: Vec::<u8>::new(),
            trigger: '\n',
            serial: serial.clone(),
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

                self.tx_buf.push(ch);
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
        println!("tx_buf: {:?}, transmitting: {}", self.tx_buf, self.serial.transmitting);

        if self.tx_buf.len() == 0 {
            self.serial.transmitting = false;
        }

        if self.serial.transmitting {
            self.serial.transmit(self.tx_buf.pop().unwrap_or('\0' as u8), self.tx_buf.len() as u8)?;
            system.get_interrupt_controller().set(true, 6, 12)?;
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<(), Error>{
        let (ch, remaining) = self.serial.receive()?;
        self.rx_buf.push(ch);
        println!("rx_buf: {:?}, remaining: {}", self.rx_buf, remaining);

        // send interrupt to acknowledge receipt

        if remaining == 0 {
            match self.server.to_client.write(&self.rx_buf) {
                Ok(_) => self.rx_buf.clear(),
                Err(e) => panic!("Error sending data to client: {}", e)
            }
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

        Ok(1_000_000_000 / 1_000_000)
    }
}
