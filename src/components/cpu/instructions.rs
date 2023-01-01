use crate::components::cpu::state::Register as Reg;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Undefined(u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstructionType {
    Undefined,
    U,
    J,
    B,
    I,
    R,
    S,
    M,
    T,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum U {
    Lui(Reg, u32),
    Auipc(Reg, u32),
}
pub const LUI: u8 = 0x21;
pub const AUIPC: u8 = 0x22;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum J {
    Jal(Reg, i32),
}
pub const JAL: u8 = 0x20;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum B {
    Beq(Reg, Reg, i32),
    Bne(Reg, Reg, i32),
    Blt(Reg, Reg, i32),
    Bge(Reg, Reg, i32),
    Bltu(Reg, Reg, i32),
    Bgeu(Reg, Reg, i32),
}
pub const BRANCH: u8 = 0x30;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum I {
    Jalr(Reg, Reg, i32),

    Lb(Reg, Reg, i32),
    Lbu(Reg, Reg, i32),
    Lbd(Reg, Reg, i32),
    Lbud(Reg, Reg, i32),
    Lh(Reg, Reg, i32),
    Lhu(Reg, Reg, i32),
    Lhd(Reg, Reg, i32),
    Lhud(Reg, Reg, i32),
    Lw(Reg, Reg, i32),
    Lwd(Reg, Reg, i32),

    Muli(Reg, Reg, i32),
    Mulih(Reg, Reg, i32),
    Idivi(Reg, Reg, i32),
    Addi(Reg, Reg, i32),
    Subi(Reg, Reg, i32),

    Ori(Reg, Reg, i32),
    Andi(Reg, Reg, i32),
    Xori(Reg, Reg, i32),
    ShiRa(Reg, Reg, i32),
    ShiRl(Reg, Reg, i32),
    ShiLl(Reg, Reg, i32),
}
pub const JALR: u8 = 0x41;
pub const LOAD: u8 = 0x40;
pub const ALUI: u8 = 0x50;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum R {
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Idiv(Reg, Reg, Reg, Reg),
    Mul(Reg, Reg, Reg, Reg),

    Or(Reg, Reg, Reg),
    And(Reg, Reg, Reg),
    Xor(Reg, Reg, Reg),

    Not(Reg, Reg),
    Ctz(Reg, Reg),
    Clz(Reg, Reg),
    Popcount(Reg, Reg),

    ShRa(Reg, Reg, Reg),
    ShRl(Reg, Reg, Reg),
    ShLl(Reg, Reg, Reg),
    Ror(Reg, Reg, Reg),
    Rol(Reg, Reg, Reg),
}
pub const ALUR: u8 = 0x60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum S {
    Sb(Reg, Reg, i32),
    Sbd(Reg, Reg, i32),
    Sh(Reg, Reg, i32),
    Shd(Reg, Reg, i32),
    Sw(Reg, Reg, i32),
    Swd(Reg, Reg, i32),
}
pub const STORE: u8 = 0x70;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum M {
    Copy(Reg, Reg, Reg),
    Swap(Reg, Reg, Reg),
    Fill(Reg, Reg, Reg),
    Through(Reg, Reg),
    From(Reg, Reg),

    Popb(Reg, Reg),
    Poph(Reg, Reg),
    Pop(Reg, Reg),
    Pushb(Reg, Reg),
    Pushh(Reg, Reg),
    Push(Reg, Reg),

    Save(Reg, Reg, Reg),
    Restore(Reg, Reg, Reg),
    Exch(Reg, Reg),
}
pub const MEM: u8 = 0x10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum T {
    Syscall(Reg, u8),
    GsReg(Reg),
    SsReg(Reg),
    Sysret,
}
pub const SYS: u8 = 0x00;

// Decoding
// GGG_OOOO_RRRRR_RRRRR_RRRRR_0000_0000
// GGG_OOOO_RRRRR_IIII_IIII_IIII_IIII_IIII
// GGG_OOOO_RRRRR_RRRRR_III_IIII_IIII_IIII
pub const GROUP_MASK: u32 = 0xE000_0000;
pub const OPCODE_MASK: u32 = 0x1E00_0000;
pub const RD_MASK: u32 = 0x01F0_0000;
pub const RS1_MASK: u32 = 0x000F_8000;
pub const RS2_MASK: u32 = 0x0000_7C00;
pub const RS3_MASK: u32 = 0x0000_03E0;
pub const IMM15_MASK: u32 = 0x0000_3FFF;
pub const IMM20_MASK: u32 = 0x000F_FFFF;
pub const TRAP_MASK: u32 = 0x0000_FFFF;

pub const GROUP_SHIFT: u32 = 25;
pub const OPCODE_SHIFT: u32 = 25;
pub const RD_SHIFT: u32 = 20;
pub const RS1_SHIFT: u32 = 15;
pub const RS2_SHIFT: u32 = 10;
pub const RS3_SHIFT: u32 = 5;


#[inline(always)]
fn get_group(value: u32) -> u8 {
    ((value & GROUP_MASK) >> GROUP_SHIFT) as u8
}

#[inline(always)]
fn get_opcode(value: u32) -> u8 {
    ((value & OPCODE_MASK) >> OPCODE_SHIFT) as u8
}

#[inline(always)]
fn get_rd(value: u32) -> u8 {
    ((value & RD_MASK) >> RD_SHIFT) as u8
}

#[inline(always)]
fn get_rs1(value: u32) -> u8 {
    ((value & RS1_MASK) >> RS1_SHIFT) as u8
}


#[inline(always)]
fn get_rs2(value: u32) -> u8 {
    ((value & RS2_MASK) >> RS2_SHIFT) as u8
}

#[inline(always)]
fn get_rs3(value: u32) -> u8 {
    ((value & RS3_MASK) >> RS3_SHIFT) as u8
}

#[inline(always)]
fn get_imm15(value: u32) -> u32 {
    value & IMM15_MASK
}

#[inline(always)]
fn get_imm20(value: u32) -> u32 {
    value & IMM20_MASK
}

#[inline(always)]
fn get_trap(value: u32) -> u16 {
    (value & TRAP_MASK) as u16
}


impl From<u32> for InstructionType {
    fn from(value: u32) -> Self {
        let group = get_group(value);
        let opcode = get_opcode(value);

        match group | opcode {
            LUI | AUIPC => InstructionType::U,
            JAL => InstructionType::J,
            _ => match group {
                BRANCH => InstructionType::B,
                JALR | LOAD | ALUI => InstructionType::I,
                ALUR => InstructionType::R,
                STORE => InstructionType::S,
                MEM => InstructionType::M,
                SYS => InstructionType::T,
                _ => InstructionType::Undefined,
            },
        }
    }
}

impl TryFrom<u32> for U {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);
        let rd = Reg::from(get_rd(value) as usize);
        let imm = get_imm20(value) << 12;
//TODO: opcodes to constants, why only 4 bits instead of 12
        match opcode {
            0x1 => Ok(U::Lui(rd, imm)),
            0x2 => Ok(U::Auipc(rd, imm)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for J {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);
        let rd = Reg::from(get_rd(value) as usize);
        let imm = sign_extend( get_imm20(value) << 2, 22);

        match opcode {
            0x0 => Ok(J::Jal(rd, imm)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for B {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);
        let rs1 = Reg::from(get_rd(value) as usize);
        let rs2 = Reg::from(get_rs1(value) as usize);
        let pcrel_17 = sign_extend(get_imm15(value) << 2, 17);

        match opcode {
            0x0 => Ok(B::Beq(rs1, rs2, pcrel_17)),
            0x1 => Ok(B::Bne(rs1, rs2, pcrel_17)),
            0x2 => Ok(B::Blt(rs1, rs2, pcrel_17)),
            0x3 => Ok(B::Bge(rs1, rs2, pcrel_17)),
            0x4 => Ok(B::Bltu(rs1, rs2, pcrel_17)),
            0x5 => Ok(B::Bgeu(rs1, rs2, pcrel_17)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for I {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let group = get_group(value);
        let opcode = get_opcode(value);
        let rd = Reg::from(get_rd(value) as usize);
        let rs1 = Reg::from(get_rs1(value) as usize);
        let imm = sign_extend(get_imm15(value), 15);

        match (group, opcode) {
            (0x40, 0x1) => Ok(I::Jalr(rd, rs1, imm)),

            (LOAD, 0x2) => Ok(I::Lb(rd, rs1, imm)),
            (LOAD, 0x3) => Ok(I::Lbu(rd, rs1, imm)),
            (LOAD, 0x4) => Ok(I::Lbd(rd, rs1, imm)),
            (LOAD, 0x5) => Ok(I::Lbud(rd, rs1, imm)),
            (LOAD, 0x6) => Ok(I::Lh(rd, rs1, imm)),
            (LOAD, 0x7) => Ok(I::Lhu(rd, rs1, imm)),
            (LOAD, 0x8) => Ok(I::Lhd(rd, rs1, imm)),
            (LOAD, 0x9) => Ok(I::Lhud(rd, rs1, imm)),
            (LOAD, 0xa) => Ok(I::Lw(rd, rs1, imm)),
            (LOAD, 0xb) => Ok(I::Lwd(rd, rs1, imm)),
            (LOAD, _) => Err(Undefined(value)),

            (ALUI, 0x0) => Ok(I::Muli(rd, rs1, imm)),
            (ALUI, 0x1) => Ok(I::Mulih(rd, rs1, imm)),
            (ALUI, 0x2) => Ok(I::Idivi(rd, rs1, imm)),
            (ALUI, 0x3) => Ok(I::Addi(rd, rs1, imm)),
            (ALUI, 0x4) => Ok(I::Subi(rd, rs1, imm)),

            (ALUI, 0x5) => Ok(I::Ori(rd, rs1, imm)),
            (ALUI, 0x6) => Ok(I::Andi(rd, rs1, imm)),
            (ALUI, 0x7) => Ok(I::Xori(rd, rs1, imm)),
            (ALUI, 0x8) => Ok(I::ShiRa(rd, rs1, imm)),
            (ALUI, 0x9) => Ok(I::ShiRl(rd, rs1, imm)),
            (ALUI, 0xa) => Ok(I::ShiLl(rd, rs1, imm)),
            (ALUI, _) => Err(Undefined(value)),

            (_, _) => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for S {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);
        let rd = Reg::from(get_rd(value) as usize);
        let rs1 = Reg::from(get_rs1(value) as usize);
        let imm = sign_extend(get_imm15(value), 15);

        match opcode {
            0x0 => Ok(S::Sb(rd, rs1, imm)),
            0x1 => Ok(S::Sbd(rd, rs1, imm)),
            0x2 => Ok(S::Sh(rd, rs1, imm)),
            0x3 => Ok(S::Shd(rd, rs1, imm)),
            0x4 => Ok(S::Sw(rd, rs1, imm)),
            0x5 => Ok(S::Swd(rd, rs1, imm)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for R {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);
        let rd = Reg::from(get_rd(value) as usize);
        let rs1 = Reg::from(get_rs1(value) as usize);
        let rs2 = Reg::from(get_rs2(value) as usize);
        let rs3 = Reg::from(get_rs3(value) as usize);

        match opcode {
            0x0 => Ok(R::Add(rd, rs1, rs2)),
            0x1 => Ok(R::Sub(rd, rs1, rs2)),
            0x2 => Ok(R::Idiv(rd, rs1, rs2, rs3)),
            0x3 => Ok(R::Mul(rd, rs1, rs2, rs3)),

            0x4 => Ok(R::Or(rd, rs1, rs2)),
            0x5 => Ok(R::And(rd, rs1, rs2)),
            0x6 => Ok(R::Xor(rd, rs1, rs2)),

            0x7 => Ok(R::Not(rd, rs1)),
            0x8 => Ok(R::Ctz(rd, rs1)),
            0x9 => Ok(R::Clz(rd, rs1)),
            0xa => Ok(R::Popcount(rd, rs1)),

            0xb => Ok(R::ShRa(rd, rs1, rs2)),
            0xc => Ok(R::ShRl(rd, rs1, rs2)),
            0xd => Ok(R::ShLl(rd, rs1, rs2)),
            0xe => Ok(R::Ror(rd, rs1, rs2)),
            0xf => Ok(R::Rol(rd, rs1, rs2)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for M {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);;
        let rd = Reg::from(get_rd(value) as usize);
        let rs1 = Reg::from(get_rs1(value) as usize);
        let rs2 = Reg::from(get_rs2(value) as usize);

        match opcode {
            0x0 => Ok(M::Copy(rd, rs1, rs2)),
            //0x0 => Ok(M::Copydd(rd, rs1, rs2)), //TODO: copies from data to data and from main to data and viceversa
            //0x0 => Ok(M::Copymd(rd, rs1, rs2)),
            0x1 => Ok(M::Swap(rd, rs1, rs2)),
            0x2 => Ok(M::Fill(rd, rs1, rs2)),
            0x3 => Ok(M::Through(rd, rs1)),
            0x4 => Ok(M::From(rd, rs1)),

            0x5 => Ok(M::Popb(rd, rs1)),
            0x6 => Ok(M::Poph(rd, rs1)),
            0x7 => Ok(M::Pop(rd, rs1)),
            0x8 => Ok(M::Pushb(rd, rs1)),
            0x9 => Ok(M::Pushh(rd, rs1)),
            0xa => Ok(M::Push(rd, rs1)),

            0xb => Ok(M::Save(rd, rs1, rs2)),
            0xc => Ok(M::Restore(rd, rs1, rs2)),
            0xd => Ok(M::Exch(rd, rs1)),
            _ => Err(Undefined(value)),
        }
    }
}

impl TryFrom<u32> for T {
    type Error = Undefined;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = get_opcode(value);;
        let rd = Reg::from(get_rd(value) as usize);
        let vector = get_trap(value);

        match opcode {
            0x2 => Ok(T::Syscall(rd, vector as u8)),
            0x3 => Ok(T::GsReg(rd)),
            0x4 => Ok(T::SsReg(rd)),
            0x6 => Ok(T::Sysret),
            _ => Err(Undefined(value)),
        }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Undefined(u32),
    U(U),
    J(J),
    B(B),
    I(I),
    R(R),
    S(S),
    M(M),
    T(T),
}
