// this module contains the hardware configuration for the Taleä system
pub type Word = u32;
pub type Uptr = ux::u24;
pub type Byte = u8;

const BIT: usize = 1;

pub const DATA_BUS_SIZE: usize = 32*BIT;
pub const ADDR_BUS_SIZE: usize = 24*BIT;

pub const MEMSIZE: usize = 1 << ADDR_BUS_SIZE;
pub const DATSIZE: usize = 0xffff; // *64K* o 1 Mb?

pub enum TaleaCpuType {
    SiriusType
}

pub mod cpu;
pub mod gpu;
pub mod kbd;
pub mod tty;
pub mod storage;