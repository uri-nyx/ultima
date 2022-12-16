use organum::core::{ClockElapsed, Address, Steppable, Addressable, Debuggable, Transmutable};
use organum::{sys::System, error::{Error, ErrorType}};

use crate::components::{Word, Uptr};
use crate::cpu::state::{Sirius, Status, StatusReg, Exceptions, InterruptPriority, Register};
use crate::cpu::instructions::Instruction;


const DEV_NAME: &'static str = "Sirius-cpu";

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Used {
    Once,
    Twice,
}

impl Steppable for Sirius {
    fn step(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        self.step_internal(system)
    }

    fn on_error(&mut self, _system: &System) {
        self.dump_state();
    }
}

impl Transmutable for Sirius {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }

    fn as_debuggable(&mut self) -> Option<&mut dyn Debuggable> {
        Some(self)
    }

    fn as_interruptable(&mut self) -> Option<&mut dyn organum::core::Interruptable> {
        Some(self)
    }
}


impl Sirius {
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.state.status != Status::Stopped
    }

    pub fn step_internal(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        match self.state.status {
            Status::Init => self.init(),
            Status::Stopped => Err(Error::new("CPU stopped")),
            Status::Running => {
                match self.cycle_one(system) {
                    Ok(diff) => Ok(diff),
                    Err(Error { err: ErrorType::Processor, native, .. }) => {
                    // TODO match arm conditional is temporary: illegal instructions generate a top level error in order to debug and fix issues with decode
                    //Err(Error { err: ErrorType::Processor, native, .. }) if native != Exceptions::IllegalInstruction as u32 => {
                        self.exception(native as u8, false)?;
                        Ok(4)
                    },
                    Err(err) => Err(err),
                }
            },
        }
    }

    pub fn init(&mut self) -> Result<ClockElapsed, Error> {
        // TODO get the ivt location in Data memory from the psr plz
        self.state.ssp = self.port_d.read_beu32(0)?;
        self.state.pc = Uptr::new(self.port_d.read_beu32(4)?);
        self.state.status = Status::Running;
        Ok(16)
    }

    pub fn cycle_one(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        // TODO decide if I want timing of instructions
        self.decode_next()?;
        self.execute_current()?;

        self.check_pending_interrupts(system)?;
        self.check_breakpoints(system);
        Ok((1_000_000_000 / self.frequency as u64) * 10 as ClockElapsed)
    }

    pub fn check_pending_interrupts(&mut self, system: &System) -> Result<(), Error> {
        self.state.pending_ipl = match system.get_interrupt_controller().check() {
            (true, priority) => InterruptPriority::from_u8(priority),
            (false, _) => InterruptPriority::NoInterrupt,
        };

        let current_ipl = self.state.current_ipl as u8;
        let pending_ipl = self.state.pending_ipl as u8;

        if self.state.pending_ipl != InterruptPriority::NoInterrupt {
            let priority_mask = self.state.psr.priority();

            if (pending_ipl > priority_mask || pending_ipl == 7) && pending_ipl >= current_ipl {
                println!("{} interrupt: {} @ {} ns", DEV_NAME, pending_ipl, system.clock);
                self.state.current_ipl = self.state.pending_ipl;
                let ack_num = system.get_interrupt_controller().acknowledge(self.state.current_ipl as u8)?;
                self.exception(ack_num, true)?;
                return Ok(());
            }
        }

        if pending_ipl < current_ipl {
            self.state.current_ipl = self.state.pending_ipl;
        }

        Ok(())
    }

    pub fn exception(&mut self, number: u8, is_interrupt: bool) -> Result<(), Error> {
        println!("{}: raising exception {}", DEV_NAME, number);

        if number == Exceptions::BusError as u8 || number == Exceptions::AddressError as u8 {
            let result = self.setup_group0_exception(number);
            if let Err(err) = result {
                self.state.status = Status::Stopped;
                return Err(err);
            }
        } else {
            self.setup_normal_exception(number, is_interrupt)?;
        }

        Ok(())
    }

    pub fn setup_group0_exception(&mut self, number: u8) -> Result<(), Error> {
        let psr = self.state.psr;
        let ins_word = self.decoder.instruction_word;

        // Changes to the flags must happen after the previous value has been pushed to the stack
        // TODO check offsets and types for the exception vectosr
        self.state.psr.set_supervisor(true);

        let offset = (number as u16) << 2;

        self.push_long(Word::from(self.state.pc))?; // Retrocede antes de la excepcion
        self.push_long(Word::from_be_bytes(psr.into_bytes()))?;
        self.push_long(ins_word)?;

        //let vector = self.state.vbr + offset as u32; // the vector in the vector table?
        let addr = self.port_d.read_beu32(offset as Address)?;
        self.set_pc(addr)?;
        panic!("Exception raised, not yet implemented");

        Ok(())
    }

    pub fn setup_normal_exception(&mut self, number: u8, is_interrupt: bool) -> Result<(), Error> {
        let psr = self.state.psr;
        //self.state.request.i_n_bit = true; // no entiendo esto

        // Changes to the flags must happen after the previous value has been pushed to the stack
        self.state.psr.set_supervisor(true);

        if is_interrupt {
            self.state.psr.set_priority_checked(self.state.current_ipl as u8);
        }

        let offset = (number as u16) << 2;
        self.push_long(Word::from(self.state.pc))?; // pc is 24 bits but maybe its better to keep the stack aligned
        self.push_long(Word::from_be_bytes(psr.into_bytes()))?;

        //let vector = self.state.vbr + offset as u32;
        let addr = self.port.read_beu32(offset as Address)?;
        self.set_pc(addr)?;
        panic!("Exception raised, not yet implemented");

        Ok(())
    }

    pub fn decode_next(&mut self) -> Result<(), Error> {
        //self.timing.reset();

        self.timer.decode.start();
        //self.start_instruction_request(self.state.pc)?;
        self.decoder.decode_at(&mut self.port, self.state.pc)?;
        self.timer.decode.end();

        //self.timing.add_instruction(&self.decoder.instruction);

        /*if self.debugger.use_tracing {
            self.decoder.dump_decoded(&mut self.port);
        }*/

        self.state.pc = self.decoder.end;

        Ok(())
    }

    pub fn execute_current(&mut self) -> Result<(), Error> {
        self.timer.execute.start();
        match self.decoder.instruction {
            _ => ()
        }

        self.timer.execute.end();
        Ok(())
    }

    fn push_word(&mut self, value: u16) -> Result<(), Error> {
        *self.get_stack_pointer_mut() -= 2;
        let addr = *self.get_stack_pointer_mut();
        self.port.write_beu16(addr as Address, value)
    }

    fn pop_word(&mut self) -> Result<u16, Error> {
        let addr = *self.get_stack_pointer_mut();
        let value = self.port.read_beu16(addr as Address)?;
        *self.get_stack_pointer_mut() += 2;
        Ok(value)
    }

    fn push_long(&mut self, value: u32) -> Result<(), Error> {
        *self.get_stack_pointer_mut() -= 4;
        let addr = *self.get_stack_pointer_mut();
        self.port.write_beu32(addr as Address, value)
    }

    fn pop_long(&mut self) -> Result<u32, Error> {
        let addr = *self.get_stack_pointer_mut();
        let value = self.port.read_beu32(addr as Address)?;
        *self.get_stack_pointer_mut() += 4;
        Ok(value)
    }

    fn set_pc(&mut self, value: Word) -> Result<(), Error> {
        self.state.pc = Uptr::new(value); //TODO set value to Uptr
        Ok(())
    }

    #[inline(always)]
    fn get_stack_pointer_mut(&mut self) -> &mut u32 {
        if self.is_supervisor() { &mut self.state.ssp } else { &mut self.state.usp }
    }

    #[inline(always)]
    fn get_reg(&self, reg: Register) -> u32 {
        if reg == Register::Sp {
            if self.is_supervisor() { self.state.ssp } else { self.state.usp }
        } else {
            self.state.reg[reg as usize]
        }
    }

    #[inline(always)]
    fn get_reg_mut(&mut self, reg: Register) -> &mut u32 {
        if reg == Register::Sp {
            if self.is_supervisor() { &mut self.state.ssp } else { &mut self.state.usp }
        } else {
            &mut self.state.reg[reg as usize]
        }
    }

    #[inline(always)]
    fn is_supervisor(&self) -> bool {
        self.state.psr.supervisor()
    }

    #[inline(always)]
    fn require_supervisor(&self) -> Result<(), Error> {
        if self.is_supervisor() {
            Ok(())
        } else {
            Err(Error::processor(Exceptions::PrivilegeViolation as u32))
        }
    }

    fn set_psr(&mut self, value: Word) {
        self.state.psr = StatusReg::from_bytes(value.to_ne_bytes()); //TODO figure out the endianness of the modular bitfield crate
    }

    /*#[inline(always)]
    fn get_flag(&self, flag: Flags) -> bool {
        (self.state.psr & (flag as u16)) != 0
    }*/

    /*#[inline(always)]
    fn set_flag(&mut self, flag: Flags, value: bool) {
        self.state.sr = (self.state.sr & !(flag as u16)) | (if value { flag as u16 } else { 0 });
    }*

    fn set_compare_flags(&mut self, value: u32, size: Size, carry: bool, overflow: bool) {
        let value = sign_extend_to_long(value, size);

        let mut flags = 0x0000;
        if value < 0 {
            flags |= Flags::Negative as u16;
        }
        if value == 0 {
            flags |= Flags::Zero as u16;
        }
        if carry {
            flags |= Flags::Carry as u16;
        }
        if overflow {
            flags |= Flags::Overflow as u16;
        }
        self.state.sr = (self.state.sr & 0xFFF0) | flags;
    }

    fn set_logic_flags(&mut self, value: u32, size: Size) {
        let mut flags = 0x0000;
        if get_msb(value, size) {
            flags |= Flags::Negative as u16;
        }
        if value == 0 {
            flags |= Flags::Zero as u16;
        }
        self.state.sr = (self.state.sr & 0xFFF0) | flags;
    }

    fn set_bit_test_flags(&mut self, value: u32, bitnum: u32, size: Size) -> u32 {
        let mask = 0x1 << (bitnum % size.in_bits());
        self.set_flag(Flags::Zero, (value & mask) == 0);
        mask
    }

    fn set_bit_field_test_flags(&mut self, field: u32, msb_mask: u32) {
        let mut flags = 0x0000;
        if (field & msb_mask) != 0 {
            flags |= Flags::Negative as u16;
        }
        if field == 0 {
            flags |= Flags::Zero as u16;
        }
        self.state.sr = (self.state.sr & 0xFFF0) | flags;
    }*/
}

// This tests for alignment?
fn validate_address(addr: u32) -> Result<u32, Error> {
    if addr & 0x1 == 0 {
        Ok(addr)
    } else {
        Err(Error::processor(Exceptions::AddressError as u32))
    }
}