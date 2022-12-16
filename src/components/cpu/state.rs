// state.rs provides a model for the Sirius cpu

use organum::core::{Address, Interruptable, Transmutable};
use organum::premade::bus::BusPort;
use organum::timers::CpuTimer;

use crate::components::{Uptr, Word, TaleaCpuType};
use modular_bitfield_msb::specifiers::*;
use super::decode::Decoder;
use super::debugger::Debugger;

const RGCOUNT: usize = 32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Status {
    Init,
    Running,
    Stopped
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Exceptions {
    BusError            = 2,
    AddressError        = 3,
    IllegalInstruction  = 4,
    ZeroDivide          = 5,
    ChkInstruction      = 6,
    TrapvInstruction    = 7,
    PrivilegeViolation  = 8,
    Trace               = 9,
    LineAEmulator       = 10,
    LineFEmulator       = 11,
}


#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterruptPriority {
    NoInterrupt = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Register {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    S1 = 9,
    Fp = 8,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31
}

#[modular_bitfield_msb::bitfield]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StatusReg {
    pub supervisor: bool,
    pub interrupt_enabled: bool,
    pub priority: B3,
    reserved: B27,
}


#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub status: Status,
    pub current_ipl: InterruptPriority,
    pub pending_ipl: InterruptPriority,

    pub pc: Uptr,
    pub psr: StatusReg,
    pub reg: [Word; RGCOUNT],
    pub ssp: Word,
    pub usp: Word,
}

pub struct Sirius {
    pub cputype: TaleaCpuType,
    pub frequency: u32,
    pub state: State,
    pub decoder: Decoder,
    pub debugger: Debugger,
    pub port: BusPort,
    pub port_d: BusPort,
    pub timer: CpuTimer
}

impl State {
    pub fn new() -> Self {
        Self {
            status: Status::Init,
            current_ipl: InterruptPriority::NoInterrupt,
            pending_ipl: InterruptPriority::NoInterrupt,

            pc: Uptr::new(0),
            psr: StatusReg::new()
                .with_supervisor(true)
                .with_interrupt_enabled(false)
                .with_priority(7)
                .with_reserved(0),
            reg: [0; RGCOUNT],
            ssp: 0,
            usp: 0
        }
    }

    pub fn get_register(&mut self, reg: Register) -> Word {
        self.reg[reg as usize]
    }

    pub fn set_register(&mut self, reg: Register, value: Word) {
        if reg == Register::Sp {
            if self.psr.supervisor() {
                self.ssp = value;
            } else {
                self.usp = value;
            }
        }
    }
}



impl Sirius {
    pub fn new(cputype: TaleaCpuType, frequency: u32, port: BusPort, port_d: BusPort) -> Self {
        Self {
            cputype,
            frequency,
            state: State::new(),
            decoder: Decoder::new(),
            debugger: Debugger::new(),
            port: port,
            port_d: port_d,
            timer: CpuTimer::new()
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.state = State::new();
        self.decoder = Decoder::new();
        self.debugger = Debugger::new();
    }

    pub fn dump_state(&mut self) {
        println!("Status: {:?}", self.state.status);
        println!("PC: {:#08x}", self.state.pc);
        println!("SSP: {:#08x}", self.state.ssp);
        println!("USP: {:#08x}", self.state.usp);
        println!("PSR: {:?}", self.state.psr);

        let mut i = 0;
        for reg in self.state.reg {
            println!("r{}: {:08x}", i, reg);
            i += 1;
        }

        println!("Current Instruction: {} {:?}", self.decoder.format_instruction_bytes(&mut self.port), self.decoder.instruction);
        println!("");
        self.port.dump_memory(self.state.reg[Register::Sp as usize] as Address, 0x40);
        println!("");
    }

    pub fn dump_state_str(&mut self) -> String {
       return format!("Pc: {:08x}", self.state.pc)
    }
}

impl Interruptable for Sirius {
    
}

impl InterruptPriority {
    pub fn from_u8(priority: u8) -> InterruptPriority {
        match priority {
            0 => InterruptPriority::NoInterrupt,
            1 => InterruptPriority::Level1,
            2 => InterruptPriority::Level2,
            3 => InterruptPriority::Level3,
            4 => InterruptPriority::Level4,
            5 => InterruptPriority::Level5,
            6 => InterruptPriority::Level6,
            _ => InterruptPriority::Level7,
        }
    }
}