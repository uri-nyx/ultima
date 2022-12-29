// A very simple TCP server that should allow to implement comunications 
// between the emulator and other devices such as tty's or debuggers

use std::net::{TcpListener, TcpStream, IpAddr, SocketAddr};
use std::sync::mpsc::{Sender, Receiver};
use std::{thread, sync::mpsc, io::prelude::*};

#[derive(Debug)]
pub struct Server {
    pub sender: Sender<[u8; 1]>,
    pub receiver: Receiver<[u8; 1]>,
    pub to_client:   TcpStream,
    pub client_addr: SocketAddr,
    pub ip: IpAddr,
    pub port: u16,

}


impl Server {
    pub fn new(ip: IpAddr, port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind((ip, port))?;
        let (stream, addr) = listener.accept()?;

        let (sender, receiver) = mpsc::channel();

        Ok(Self {
            sender: sender,
            receiver: receiver,
            to_client: stream,
            client_addr: addr,
            ip: ip,
            port: port
        })
    }

    pub fn run(&mut self) {
        let mut from_client = self.to_client.try_clone().unwrap();
        let sender = self.sender.clone();

        thread::spawn( move || {
            loop {
                let mut buf = [0u8; 1];
                match from_client.read(&mut buf) {
                    Ok(0) => {
                        drop(sender);
                        println!("[SERVER] CLIENT closed connection.");
                        break
                    },
    
                    Ok(_) => {
                        match sender.send(buf) {
                            Ok(_) => continue,
                            Err(e) => { 
                                panic!("Stream Error: {}", e);
                            }
                        }
                    },
    
                    Err(e) => { 
                        panic!("Stream Error: {}", e); 
                    }
                }
            }
            
        });
    }
}