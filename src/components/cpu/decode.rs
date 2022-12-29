use organum::core::{Address, Addressable};
use organum::error::Error;

use crate::components::cpu::instructions::{Instruction, InstructionType, B, I, J, M, R, S, T, U};
use crate::components::cpu::mmu::Mmu;
use crate::components::{Uptr, Word};

pub struct Decoder {
    pub at: Uptr,
    pub instruction_word: Word,
    pub instruction: Instruction,
    pub execution_time: u16,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            at: 0,
            instruction_word: 0,
            instruction: Instruction::Undefined(0),
            execution_time: 0,
        }
    }
}

impl Decoder {
    pub fn decode_at(&mut self, memory: &mut dyn Addressable, at: Word) -> Result<(), Error> {
        self.at = at;
        self.instruction = self.decode_one(memory)?;
        Ok(())
    }

    pub fn decode_one(&mut self, memory: &mut dyn Addressable) -> Result<Instruction, Error> {
        let ins = self.read_instruction(memory)?;
        let instype = InstructionType::from(ins);

        match instype {
            InstructionType::Undefined => Ok(Instruction::Undefined(ins)),
            InstructionType::U => {
                let i = U::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::U(i.unwrap()))
            }
            InstructionType::J => {
                let i = J::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::J(i.unwrap()))
            }
            InstructionType::B => {
                let i = B::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::B(i.unwrap()))
            }
            InstructionType::I => {
                let i = I::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::I(i.unwrap()))
            }
            InstructionType::R => {
                let i = R::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::R(i.unwrap()))
            }
            InstructionType::S => {
                let i = S::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::S(i.unwrap()))
            }
            InstructionType::M => {
                let i = M::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::M(i.unwrap()))
            }
            InstructionType::T => {
                let i = T::try_from(ins);
                if i.is_err() {
                    return Ok(Instruction::Undefined(ins));
                }

                Ok(Instruction::T(i.unwrap()))
            }
        }
    }

    fn read_instruction(&mut self, device: &mut dyn Addressable) -> Result<Word, Error> {
        let word = device.read_beu32(Address::from(self.at))?;
        self.at = self.at.wrapping_add(4);
        Ok(word)
    }

    pub fn format_instruction_bytes(&mut self, memory: &mut dyn Addressable) -> String {
        let ins_data: String = (0..4)
            .map(|offset| {
                format!(
                    "{:02x} ",
                    memory.read_u8(Address::from(self.at) + offset).unwrap()
                )
            })
            .collect();
        ins_data
    }

    pub fn dump_decoded(&mut self, memory: &mut dyn Addressable) {
        let ins_data = self.format_instruction_bytes(memory);
        println!("{:#06x}: {}\n\t{:?}\n", self.at, ins_data, self.instruction);
    }
}
