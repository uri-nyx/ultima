
use organum::core::{Address, Debuggable};
use organum::error::Error;
use organum::sys::System;

use crate::components::Uptr;
use crate::cpu::state::Sirius;


pub struct Debugger {
    pub enabled: bool,
    pub breakpoints: Vec<Uptr>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            enabled: false,
            breakpoints: vec!(),
        }
    }
}

impl Debuggable for Sirius {
    fn debugging_enabled(&mut self) -> bool {
        self.debugger.enabled
    }

    fn set_debugging(&mut self, enable: bool) {
        self.debugger.enabled = enable;
    }

    fn add_breakpoint(&mut self, addr: Address) {
        self.debugger.breakpoints.push(Uptr::new(addr as u32));
        self.debugger.enabled = true;
    }

    fn remove_breakpoint(&mut self, addr: Address) {
        if let Some(index) = self.debugger.breakpoints.iter().position(|a| *a == Uptr::new(addr as u32)) {
            self.debugger.breakpoints.remove(index);
            self.debugger.enabled = !self.debugger.breakpoints.is_empty();
        }
    }

    fn print_current_step(&mut self, _system: &System) -> Result<(), Error> {
        self.decoder.decode_at(&mut self.port, self.state.pc)?;
        self.decoder.dump_decoded(&mut self.port);
        self.dump_state();
        Ok(())
    }

    fn print_disassembly(&mut self, _addr: Address, _count: usize) {
        /*let mut decoder = Decoder::new();
        decoder.dump_disassembly(&mut self.port, addr as u16, count as u16);*/
    } /*No dissasembler for now*/ 

    fn execute_command(&mut self, _system: &System, _args: &[&str]) -> Result<bool, Error> {
        /* no commands to execute */
        Ok(false)
    }
}

impl Sirius {
    pub fn check_breakpoints(&mut self, system: &System) {
        for breakpoint in &self.debugger.breakpoints {
            if *breakpoint == self.state.pc {
                println!("Breakpoint reached: {:08x}", *breakpoint);
                system.enable_debugging();
                break;
            }
        }
    }
}

