// state.rs provides a model for the Sirius cpu
use organum::core::{Address, Interruptable};
use organum::premade::bus::BusPort;

use super::debugger::Debugger;
use super::decode::Decoder;
use crate::components::{TaleaCpuType, Uptr, Word};
use crate::components::cpu::mmu::Mmu;
use modular_bitfield_msb::specifiers::*;

const RGCOUNT: usize = 32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Status {
    Init,
    Running,
    Stopped,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Exceptions {
    Reset = 0,
    BusError = 2,
    AddressError = 3,
    IllegalInstruction = 4,
    ZeroDivide = 5,
    PrivilegeViolation = 6,
    PageFault = 7,
    AccessViolation = 8,
    //ChkInstruction = 6, this could be interesting, hardware bounds checking
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
    T6 = 31,
}

impl From<usize> for Register {
    fn from(value: usize) -> Self {
        let value = value & 0x1f;
        match value {
            0 => Register::Zero,
            1 => Register::Ra,
            2 => Register::Sp,
            3 => Register::Gp,
            4 => Register::Tp,
            5 => Register::T0,
            6 => Register::T1,
            7 => Register::T2,
            8 => Register::Fp,
            9 => Register::S1,
            10 => Register::A0,
            11 => Register::A1,
            12 => Register::A2,
            13 => Register::A3,
            14 => Register::A4,
            15 => Register::A5,
            16 => Register::A6,
            17 => Register::A7,
            18 => Register::S2,
            19 => Register::S3,
            20 => Register::S4,
            21 => Register::S5,
            22 => Register::S6,
            23 => Register::S7,
            24 => Register::S8,
            25 => Register::S9,
            26 => Register::S10,
            27 => Register::S11,
            28 => Register::T3,
            29 => Register::T4,
            30 => Register::T5,
            31 => Register::T6,
            _ => panic!("Invalid register value: {}", value),
        }
    }
}

impl From<Register> for usize {
    fn from(value: Register) -> Self {
        match value {
            Register::Zero => 0 ,
            Register::Ra => 1 ,
            Register::Sp => 2 ,
            Register::Gp => 3 ,
            Register::Tp => 4 ,
            Register::T0 => 5 ,
            Register::T1 => 6 ,
            Register::T2 => 7 ,
            Register::Fp => 8 ,
            Register::S1 => 9 ,
            Register::A0 => 10 ,
            Register::A1 => 11 ,
            Register::A2 => 12 ,
            Register::A3 => 13 ,
            Register::A4 => 14 ,
            Register::A5 => 15 ,
            Register::A6 => 16 ,
            Register::A7 => 17 ,
            Register::S2 => 18 ,
            Register::S3 => 19 ,
            Register::S4 => 20 ,
            Register::S5 => 21 ,
            Register::S6 => 22 ,
            Register::S7 => 23 ,
            Register::S8 => 24 ,
            Register::S9 => 25 ,
            Register::S10 => 26 ,
            Register::S11 => 27 ,
            Register::T3 => 28 ,
            Register::T4 => 29 ,
            Register::T5 => 30 ,
            Register::T6 => 31 ,
        }
    }
}

#[modular_bitfield_msb::bitfield]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StatusReg {
    pub supervisor: bool,
    pub interrupt_enabled: bool,
    pub mmu_enabled: bool,
    pub priority: B3,
    pub ivt: B6,
    pub pdt: u8,
    #[skip] __: B12
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
    pub virtual_pc : Word,
}

pub struct Sirius {
    pub cputype: TaleaCpuType,
    pub frequency: u32,
    pub state: State,
    pub mmu: Mmu,
    pub decoder: Decoder,
    pub debugger: Debugger,
    pub port: BusPort,
    pub port_d: BusPort,
    pub cycles: u128
}

impl State {
    pub fn new() -> Self {
        Self {
            status: Status::Init,
            current_ipl: InterruptPriority::NoInterrupt,
            pending_ipl: InterruptPriority::NoInterrupt,

            pc: 0,
            psr: StatusReg::new()
                .with_supervisor(true)
                .with_interrupt_enabled(false)
                .with_priority(7)
                .with_ivt(7),
            reg: [0; RGCOUNT],
            ssp: 0,
            usp: 0,
            virtual_pc: 0

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
            mmu: Mmu::new(),
            debugger: Debugger::new(),
            port: port,
            port_d: port_d,
            cycles: 0
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.state = State::new();
        self.decoder = Decoder::new();
        self.debugger = Debugger::new();
        self.mmu = Mmu::new();
    }

    pub fn dump_state(&mut self) {
        println!("PC: {:#08x}   PSR: {:?}", self.state.pc, self.state.psr);
        println!("SSP: {:#08x}  USP: {:#08x}", self.state.ssp, self.state.usp);
        println!("Status: {:?}", self.state.status);

        let mut iter = self.state.reg.iter();

        for r in (0..self.state.reg.len()).filter(|e| e % 4 == 0) {
            print!(
                "r{:02}: {:08x}   r{:02}: {:08x}\t",
                r,
                iter.next().unwrap(),
                r + 1,
                iter.next().unwrap()
            );
            print!(
                "r{:02}: {:08x}   r{:02}: {:08x}\n",
                r + 2,
                iter.next().unwrap(),
                r + 3,
                iter.next().unwrap()
            );
        }

        println!(
            "Current Instruction: {} {:?}",
            self.decoder.format_instruction_bytes(&mut self.port),
            self.decoder.instruction
        );
        println!("Stack:");
        self.port
            .dump_memory(self.state.reg[Register::Sp as usize] as Address, 0x40);
        println!("");
    }

    pub fn dump_state_str(&mut self) -> String {
        return format!("Pc: {:08x}", self.state.pc);
    }
}

impl Interruptable for Sirius {}

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
