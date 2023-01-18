use core::panic;

use organum::core::{Address, Addressable, ClockElapsed, Debuggable, Steppable, Transmutable};
use organum::{
    error::{Error, ErrorType},
    sys::System,
};

use crate::components::cpu::instructions::{Instruction, U,
J,
B,
I,
R,
S,
M,
T,};
use crate::components::cpu::state::{
    Exceptions, InterruptPriority, Register, Sirius, Status, StatusReg,
};
use crate::components::{Word, IVT_SIZE};

const DEV_NAME: &'static str = "Sirius-cpu";

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
                    Err(Error {
                        err: ErrorType::Processor,
                        native,
                        ..
                    }) => {
                        // TODO match arm conditional is temporary: illegal instructions generate a top level error in order to debug and fix issues with decode
                        if native != Exceptions::IllegalInstruction as u32 {
                        self.exception(native as u8, false)?;
                        } else {
                            println!("{:?} @ {:x}", self.decoder.instruction, self.state.pc);
                            panic!("Illegal Instruction",)
                        }
                        Ok(1)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub fn init(&mut self) -> Result<ClockElapsed, Error> {
        // TODO maybe change some flags
        let ivt = self.state.psr.ivt() as u16;
        self.state.ssp = self.port_d.read_beu32((ivt * 256) as Address + 0)?; // reset vector
        self.state.pc = self.port_d.read_beu32((ivt * 256) as Address + 4)?;
        self.state.status = Status::Running;
        Ok(1)
    }

    pub fn cycle_one(&mut self, system: &System) -> Result<ClockElapsed, Error> {
        self.decode_next()?;
        self.execute_current()?;

        self.check_pending_interrupts(system)?;
        self.check_breakpoints(system);
        Ok((1_000_000_000 / self.frequency as u64) * 5 as ClockElapsed )
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
                println!(
                    "{} interrupt: {} @ {} ns",
                    DEV_NAME, pending_ipl, system.clock
                );
                self.state.current_ipl = self.state.pending_ipl;
                let ack_num = system
                    .get_interrupt_controller()
                    .acknowledge(self.state.current_ipl as u8)?;
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

        // IMPORTANT: La diferencia entre excepción y fault es que la última intenta corregir el problema y VUELVE A EJECUTAR la instrucción que la causó

        if number == Exceptions::BusError as u8 || number == Exceptions::AddressError as u8 {
            let result = self.setup_fault(number);
            // Double Fault
            if let Err(err) = result {
                self.state.status = Status::Stopped;
                return Err(err);
            }
        } else if number == Exceptions::PrivilegeViolation as u8 {
            panic!();
        } else {
            self.setup_normal_exception(number, is_interrupt)?;
        }

        Ok(())
    }

    pub fn setup_fault(&mut self, number: u8) -> Result<(), Error> {
        let ins_word = self.decoder.instruction_word;

        self.push_long(Word::from(self.state.pc))?; // Retrocede antes de la excepcion
        self.push_long(Word::from_le_bytes(self.state.psr.into_bytes()))?;
        self.push_long(ins_word)?; // Guarda la instrucción en el stack

        self.state.psr.set_supervisor(true);

        let offset = (number as u16) << 2;
        let ivt =  self.state.psr.ivt() as u32 * IVT_SIZE as u32;
        let vector = ivt + offset as u32;
        let addr = self.port_d.read_beu32(vector as Address)?;
        self.set_pc(addr)?;

        Ok(())
    }

    pub fn setup_normal_exception(&mut self, number: u8, is_interrupt: bool) -> Result<(), Error> {
        //self.state.request.i_n_bit = true; // no entiendo esto
        self.push_long(self.get_pc())?;
        self.push_long(Word::from_le_bytes(self.state.psr.into_bytes()))?;

        self.state.psr.set_supervisor(true);

        if is_interrupt && self.state.psr.interrupt_enabled() {
            self.state
                .psr
                .set_priority(self.state.current_ipl as u8);
        }


        let offset = (number as u16) << 2;
        let ivt =  self.state.psr.ivt() as u32 * IVT_SIZE as u32;
        let vector = ivt + offset as u32;
        let addr = self.port_d.read_beu32(vector as Address)?;
        self.set_pc(addr)?; //TODO: this address should be real?

        Ok(())
    }

    fn syscall(&mut self, id: u8) -> Result<(), Error> {
        self.setup_normal_exception(id, false)
    }

    fn sysret(&mut self) -> Result<(), Error> {
        let psr = self.pop_long()?;
        let pc = self.pop_long()?;

        self.state.psr = StatusReg::from_bytes(Word::to_le_bytes(psr));
        self.set_pc(pc)?;

        Ok(())
    }

    pub fn decode_next(&mut self) -> Result<(), Error> {
        let pc = self.get_pc();
        self.decoder.decode_at(&mut self.port, pc)?;
        self.set_pc(self.decoder.at)?;//TODO: I don't know how this behaves with paging

        Ok(())
    }

    pub fn execute_current(&mut self) -> Result<(), Error> {
        match self.decoder.instruction {
            Instruction::Undefined(_) => {
                return Err(Error::processor(Exceptions::IllegalInstruction as u32));
            },
            Instruction::U(instruction) => {
                match instruction {
                    U::Lui(rd, imm) => {
                        *self.get_reg_mut(rd) = imm;
                    },
                    U::Auipc(rd, imm) => {
                        *self.get_reg_mut(rd) = self.get_pc().wrapping_sub(4).wrapping_add(imm);
                    },
                }
            },
            Instruction::J(instruction) => {
                match instruction {
                    J::Jal(rd, imm) => {
                        *self.get_reg_mut(rd) = self.get_pc();
                        self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                    },
                }
            },
            Instruction::B(instruction) => {
                match instruction {
                    B::Beq(rs1, rs2, imm) => {
                        let equal = self.get_reg(rs1) == self.get_reg(rs2);

                        if equal {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }

                    },
                    B::Bne(rs1, rs2, imm) => {
                        let not_equal  = self.get_reg(rs1) != self.get_reg(rs2);

                        if not_equal {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }
                    },
                    B::Blt(rs1, rs2, imm) => {
                        let lt  = (self.get_reg(rs1) as i32) < (self.get_reg(rs2) as i32);

                        if lt {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }
                    },
                    B::Bge(rs1, rs2, imm) => {
                        let ge  = (self.get_reg(rs1) as i32) >= (self.get_reg(rs2) as i32);

                        if ge {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }
                    },
                    B::Bltu(rs1, rs2, imm) => {
                        let lt  = self.get_reg(rs1) < self.get_reg(rs2);

                        if lt {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }
                    },
                    B::Bgeu(rs1, rs2, imm) => {
                        let ge  = self.get_reg(rs1) >= self.get_reg(rs2);

                        if ge {
                            self.set_pc(self.get_pc().wrapping_sub(4).wrapping_add(imm as u32))?;
                        }
                    },
                }
            },
            Instruction::S(instruction) => {
                match instruction {
                    S::Sb(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd) as u8;
                        self.write_u8(addr as Address, value)?;
                    },
                    S::Sbd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd) as u8;
                        self.port_d.write_u8(addr as Address, value)?;
                    },
                    S::Sh(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd) as u16;
                        self.write_beu16(addr as Address, value)?;
                    },
                    S::Shd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd) as u16;
                        self.port_d.write_beu16(addr as Address, value)?;
                    },
                    S::Sw(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd);
                        self.write_beu32(addr as Address, value)?;
                    },
                    S::Swd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.get_reg(rd);
                        self.port_d.write_beu32(addr as Address, value)?;
                    },
                }
            },
            Instruction::I(instruction) => {
                match instruction {
                    I::Jalr(rd, rs1, imm) => {
                        *self.get_reg_mut(rd) = self.get_pc();
                        let jump = self.get_reg(rs1).wrapping_add(imm as u32);
                        self.set_pc(jump)?;
                    },

                    I::Lb(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = sign_extend(self.read_u8(addr as Address)? as u32, 8);
                        *self.get_reg_mut(rd) = value as u32;

                    },
                    I::Lbu(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.read_u8(addr as Address)? as u32;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lbd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = sign_extend(self.port_d.read_u8(addr as Address)? as u32, 8);
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lbud(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.port_d.read_u8(addr as Address)? as u32;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lh(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = sign_extend(self.read_beu16(addr as Address)? as u32, 16);
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lhu(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.read_beu16(addr as Address)? as u32;
                        *self.get_reg_mut(rd) = value as u32;

                    },
                    I::Lhd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = sign_extend(self.port_d.read_beu16(addr as Address)? as u32, 16);
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lhud(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.port_d.read_beu16(addr as Address)? as u32;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lw(rd, rs1, imm) => {
                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.read_beu32(addr as Address)?;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Lwd(rd, rs1, imm) => {
                        self.require_supervisor()?;

                        let addr = self.get_reg(rs1).wrapping_add(imm as u32);
                        let value = self.port_d.read_beu32(addr as Address)?;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                
                    I::Muli(rd, rs1, imm) => {
                        let value = (self.get_reg(rs1) as u64).wrapping_mul(imm as u64);
                        *self.get_reg_mut(rd) = (value & 0xff_ff_ff_ff) as u32;
                    },
                    I::Mulih(rd, rs1, imm) => {
                        let value = (self.get_reg(rs1) as u64).wrapping_mul(imm as u64);
                        *self.get_reg_mut(rd) = (value >> 32) as u32;
                    },
                    I::Idivi(rd, rs1, imm) => {
                        if imm == 0 {
                            return Err(Error::processor(Exceptions::ZeroDivide as u32)); //TODO: consider raising exceptions to a function
                        }
                        let value = (self.get_reg(rs1) as i32).wrapping_div(imm);
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::Addi(rd, rs1, imm) => {
                        let value = self.get_reg(rs1).wrapping_add(imm as u32);
                        *self.get_reg_mut(rd) = value;    
                    },
                    I::Subi(rd, rs1, imm) => {
                        let value = self.get_reg(rs1).wrapping_sub(imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                
                    I::Ori(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) | (imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                    I::Andi(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) & (imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                    I::Xori(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) ^ (imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                    I::ShiRa(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) as i32 >> (imm);
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    I::ShiRl(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) >> (imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                    I::ShiLl(rd, rs1, imm) => {
                        let value = self.get_reg(rs1) << (imm as u32);
                        *self.get_reg_mut(rd) = value;
                    },
                    I::Slti(rd, rs1, imm) => {
                        let value = if (self.get_reg(rs1) as i32) < (imm) {1} else {0};
                        *self.get_reg_mut(rd) = value;
                    },
                    I::Sltiu(rd, rs1, imm) => {
                        let value = if self.get_reg(rs1) < (imm as u32) {1} else {0};
                        *self.get_reg_mut(rd) = value;
                    }
                }
            },
            Instruction::R(instruction) => {
                match instruction {
                    R::Add(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1).wrapping_add(self.get_reg(rs2));
                        *self.get_reg_mut(rd) = value;
                    }, 
                    R::Sub(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1).wrapping_sub(self.get_reg(rs2));
                        *self.get_reg_mut(rd) = value;
                    }, 
                    R::Idiv(rd, rd2, rs1, rs2) => {
                        let dividend = self.get_reg(rs1) as i32;
                        let divisor = self.get_reg(rs2) as i32;
                        if divisor == 0 {
                            return Err(Error::processor(Exceptions::ZeroDivide as u32)); //TODO: consider raising exceptions to a function
                        }
                        let quotient = (dividend).wrapping_div(divisor);
                        let remainder = (dividend).wrapping_rem(divisor);
                        *self.get_reg_mut(rd) = quotient as u32;
                        *self.get_reg_mut(rd2) = remainder as u32;
                    },
                    R::Mul(rd, rd2, rs1, rs2) => {
                        let value = (self.get_reg(rs1) as u64).wrapping_mul(self.get_reg(rs2) as u64);
                        *self.get_reg_mut(rd) = (value >> 32) as u32;
                        *self.get_reg_mut(rd2) = (value & 0xff_ff_ff_ff) as u32;
                    }, 
                
                    R::Or(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) | self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    }, 
                    R::And(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) & self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    }, 
                    R::Xor(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) ^ self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    },
                
                    R::Not(rd, rs2) => {
                        let value = !self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    },
                    R::Ctz(rd, rs2) => {
                        let value = self.get_reg(rs2).trailing_zeros();
                        *self.get_reg_mut(rd) = value;
                    },
                    R::Clz(rd, rs2) => {
                        let value = self.get_reg(rs2).leading_zeros();
                        *self.get_reg_mut(rd) = value;
                    },
                    R::Popcount(rd, rs2) => {
                        let value = self.get_reg(rs2).count_ones();
                        *self.get_reg_mut(rd) = value;
                    },
                
                
                    R::ShRa(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) as i32 >> self.get_reg(rs2) as i32;
                        *self.get_reg_mut(rd) = value as u32;
                    },
                    R::ShRl(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) >> self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    },
                    R::ShLl(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1) << self.get_reg(rs2);
                        *self.get_reg_mut(rd) = value;
                    },
                    R::Ror(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1).rotate_right(self.get_reg(rs2));
                        *self.get_reg_mut(rd) = value;
                    },
                    R::Rol(rd, rs1, rs2) => {
                        let value = self.get_reg(rs1).rotate_left(self.get_reg(rs2));
                        *self.get_reg_mut(rd) = value;
                    },
                }          

            },
            Instruction::M(instruction) => {
                match instruction {
                    M::Copy(src, dest, len) => {
                        // copies region of len bytes starting at address start to address rs1
                        let src = self.get_reg(src) as Address;
                        let dest = self.get_reg(dest) as Address;
                        let len = self.get_reg(len) as usize;

                        let mut data = vec![0u8; len];
                        self.read(src, &mut data)?;
                        self.write(dest, &mut data)?;

                    },
                    M::Swap(rs1, rs2, len) => {
                        // swaps region of len bytes starting at address start with address rs1
                        let rs1 = self.get_reg(rs1) as Address;
                        let rs2 = self.get_reg(rs2) as Address;
                        let len = self.get_reg(len) as usize;

                        let mut data_a = vec![0u8; len];
                        let mut data_b = vec![0u8; len];
                        
                        self.read(rs1, &mut data_a)?;
                        self.read(rs2, &mut data_b)?;
                        self.write(rs1, &mut data_b)?;
                        self.write(rs2, &mut data_a)?;
                    },
                    M::Fill(rd, len, fill) => {
                        let rd = self.get_reg(rd) as Address;
                        let fill = self.get_reg(fill) as u8;
                        let len = self.get_reg(len) as usize;

                        let mut data_a = vec![fill; len];
                        
                        self.write(rd, &mut data_a)?;
                    },
                    M::Through(data, pointer) => {
                        // [pointer] = data -> performs two levels of indirection
                        let pointer = self.get_reg(pointer) as Address;
                        let effective_address = self.read_beu32(pointer)? as Address; 
                        let data = self.get_reg(data);

                        self.write_beu32(effective_address, data)?;

                    },
                    M::From(rd, pointer) => {
                        // data = [pointer] -> performs two levels of indirection
                        let pointer = self.get_reg(pointer) as Address;
                        let effective_address = self.read_beu32(pointer)? as Address; 
                        *self.get_reg_mut(rd) = self.read_beu32(effective_address)?;
                    },
                
                    M::Popb(rd, sp) => {
                        let addr = self.get_reg(sp);
                        let new_sp = self.get_reg(sp).wrapping_add(1);
                        *self.get_reg_mut(rd) = self.read_u8(addr as Address)? as u32;
                        *self.get_reg_mut(sp) = new_sp;
                    },
                    M::Poph(rd, sp) => {
                        let addr = *self.get_reg_mut(sp);
                        *self.get_reg_mut(rd) = self.read_beu16(addr as Address)? as u32;
                        *self.get_reg_mut(sp) = self.get_reg(sp).wrapping_add(2);
                    },
                    M::Pop(rd, sp) => {
                        let addr = *self.get_reg_mut(sp);
                        *self.get_reg_mut(rd) = self.read_beu32(addr as Address)?;
                        *self.get_reg_mut(sp) = self.get_reg(sp).wrapping_add(4);
                    },
                    M::Pushb(rd, sp) => {
                        let new_sp = self.get_reg(sp).wrapping_sub(1);
                        *self.get_reg_mut(sp) = new_sp;
                        let addr = self.get_reg(sp);
                        let value = self.get_reg(rd);
                        self.write_u8(addr as Address, value as u8)?;
                    },
                    M::Pushh(rd, sp) => {
                        *self.get_reg_mut(sp) = self.get_reg(sp).wrapping_sub(2);
                        let addr = self.get_reg(sp);
                        let value = self.get_reg(rd);
                        self.write_beu16(addr as Address, value as u16)?;
                    },
                    M::Push(rd, sp) => {
                        *self.get_reg_mut(sp) = self.get_reg(sp).wrapping_sub(4);
                        let addr = self.get_reg(sp);
                        let value = self.get_reg(rd);
                        self.write_beu32(addr as Address, value)?;
                    },
                
                    M::Save(start, end, rs1) => {
                        let addr = self.get_reg(rs1) as Address;

                        for reg in usize::from(start)..usize::from(end) {
                            self.write_beu32(addr as Address, self.get_reg(Register::from(reg)))?;
                        }
                    },
                    M::Restore(start, end, rs1) => {
                        let addr = self.get_reg(rs1) as Address;

                        for (i, reg) in (usize::from(start)..usize::from(end) + 1).enumerate() {
                            *self.get_reg_mut(Register::from(reg)) = self.read_beu32(addr + (i*4) as Address)?;
                        }
                    },
                    M::Exch(rs1, rs2) => {
                        let a = self.get_reg(rs1);
                        let b = self.get_reg(rs2);
                        *self.get_reg_mut(rs1) = b;
                        *self.get_reg_mut(rs2) = a;
                    },
                    M::Slt(rd, rs1, rs2) => {
                        let value = if (self.get_reg(rs1) as i32) < (self.get_reg(rs2) as i32) {1} else {0};
                        *self.get_reg_mut(rd) = value;
                    },
                    M::Sltu(rd, rs1, rs2) => {
                        let value = if self.get_reg(rs1) < self.get_reg(rs2) {1} else {0};
                        *self.get_reg_mut(rd) = value;
                    }
                }
            },
            Instruction::T(instruction) => {
                match instruction {
                    T::Syscall(rd, id) => {
                        if rd == Register::Zero {
                            self.syscall(id)?;
                        } else {
                            self.syscall(self.get_reg(rd) as u8)?;
                        }
                    },
                    T::GsReg(rd) => {
                        *self.get_reg_mut(rd) = self.get_psr();
                    },
                    T::SsReg(rs1) => {
                        self.require_supervisor()?;
                        self.set_psr(self.get_reg(rs1));
                    },
                    T::Sysret => {
                        self.require_supervisor()?;
                        self.sysret()?;
                    },
                }
            },
        }
        Ok(())
    }

    fn push_long(&mut self, value: u32) -> Result<(), Error> {
        *self.get_stack_pointer_mut() = self.get_stack_pointer_mut().wrapping_sub(4);
        let addr = *self.get_stack_pointer_mut();
        self.write_beu32(addr as Address, value)
    }

    fn pop_long(&mut self) -> Result<u32, Error> {
        let addr = *self.get_stack_pointer_mut();
        let value = self.read_beu32(addr as Address)?;
        *self.get_stack_pointer_mut() = self.get_stack_pointer_mut().wrapping_add(4);
        Ok(value)
    }

    fn get_pc(&self) -> Word {
        if self.state.psr.mmu_enabled() {
            self.state.virtual_pc
        } else {
            self.state.pc
        }
    }

    fn set_pc(&mut self, value: Word) -> Result<(), Error> {
        if self.state.psr.mmu_enabled() {
            let (real, _, x) = self.translate(value as Address)?;
            if x {
                self.state.pc = real as Word;
                self.state.virtual_pc = value;
                Ok(())
            } else {
                Err(Error::processor(Exceptions::AccessViolation as u32))
            }
        } else {
            self.state.pc = value; 
            Ok(())
        }
    }

    #[inline(always)]
    fn get_stack_pointer_mut(&mut self) -> &mut u32 {
        if self.is_supervisor() {
            &mut self.state.ssp
        } else {
            &mut self.state.usp
        }
    }

    #[inline(always)]
    fn get_reg(&self, reg: Register) -> Word {

        match reg {
            Register::Zero => 0,
            Register::Sp => {
                if self.is_supervisor() {
                    return self.state.ssp
                } else {
                    return self.state.usp
                }
            },
            _ => self.state.reg[reg as usize]
        }
    }

    #[inline(always)]
    fn get_reg_mut(&mut self, reg: Register) -> &mut Word {
        match reg {
            Register::Zero => {
                &mut self.state.blackhole
            },
            Register::Sp => {
                self.get_stack_pointer_mut()
            },
            _ => &mut self.state.reg[reg as usize]
        }
    }

    #[inline(always)]
    fn translate(&mut self, addr: Address) -> Result<(Address, bool, bool), Error> {
        if self.state.psr.mmu_enabled() {
            let pdt = self.state.psr.pdt() as Address * 256;
            self.mmu.translate(addr as u32, &mut self.port_d, &mut self.port, pdt)
        } else {
            Ok((addr, true, true))
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

    fn get_psr(&self) -> u32 {
        let bytes = self.state.psr.into_bytes(); //TODO figure out the endianness of the modular bitfield crate
        (bytes[0] as u32) << 24 | (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | bytes[3] as u32
    }

}

// I don't think I need to validate the address, it's done at decoding
fn validate_address(addr: u32) -> Result<u32, Error> {
    if addr & 0x3 == 0 {
        Ok(addr)
    } else {
        Err(Error::processor(Exceptions::AddressError as u32))
    }
}

#[inline(always)]
fn sign_extend(x: u32, bits: u32) -> i32 {
    let mask = (1u32 << bits) - 1;
    let sign_bit = 1u32 << (bits - 1);
    if x & sign_bit == sign_bit {
        (!mask | x) as i32
    } else {
        x as i32
    }
}

impl Addressable for Sirius {
    fn len(&self) -> usize {
        0
    }

    fn read(&mut self, addr: Address, data: &mut [u8]) -> Result<(), Error> {
        let (real, _, _) = self.translate(addr)?;
        self.port.read(real, data)
    }
    
    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Error> {
        let (real, w, _) = self.translate(addr)?;

        if w {
            self.port.write(real, data)
        } else {
            Err(Error::processor(Exceptions::AccessViolation as u32))
        }
    }
}