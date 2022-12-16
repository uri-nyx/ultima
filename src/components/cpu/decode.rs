
use organum::core::{Address, Addressable};
use organum::error::Error;

use crate::components::{Uptr, Word};
use crate::cpu::state::{Register};
use crate::cpu::instructions::Instruction;

pub struct Decoder {
    pub start: Uptr,
    pub end: Uptr,
    pub instruction_word: Word,
    pub instruction: Instruction,
    pub execution_time: u16,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            start: Uptr::new(0),
            end: Uptr::new(0),
            instruction_word: 0,
            instruction: Instruction::Nop,
            execution_time: 0,
        }
    }
}

impl Decoder {
    pub fn decode_at(&mut self, memory: &mut dyn Addressable, start: Uptr) -> Result<(), Error> {
        self.start = start;
        self.end = start;
        self.execution_time = 0;
        self.instruction = self.decode_one(memory)?;
        Ok(())
    }

    pub fn decode_one(&mut self, memory: &mut dyn Addressable) -> Result<Instruction, Error> {
        let ins = self.read_instruction(memory)?;

        match ins {
            _ => Ok(Instruction::Nop)
        }

    }

    fn read_instruction(&mut self, device: &mut dyn Addressable) -> Result<Word, Error> {
        let word = device.read_beu32(Address::from(self.end))?;
        self.end = self.end + Uptr::new(0);
        self.execution_time += 8;
        Ok(word)
    }

    pub fn format_instruction_bytes(&mut self, memory: &mut dyn Addressable) -> String {
        let ins_data: String =
            (0..u32::from(self.end - self.start)).map(|offset|
                format!("{:02x} ", memory.read_u8(Address::from(self.start + Uptr::new(offset))).unwrap())
            ).collect();
        ins_data
    }

    pub fn dump_decoded(&mut self, memory: &mut dyn Addressable) {
        let ins_data = self.format_instruction_bytes(memory);
        println!("{:#06x}: {}\n\t{:?}\n", self.start, ins_data, self.instruction);
    }

    pub fn dump_disassembly(&mut self, memory: &mut dyn Addressable, start: Uptr, length: Uptr) {
        let mut next = start;
        while next < (start + length) {
            match self.decode_at(memory, next) {
                Ok(()) => {
                    self.dump_decoded(memory);
                    next = self.end;
                },
                Err(err) => {
                    println!("{:?}", err);
                    return;
                },
            }
        }
    }
}