use std::collections::HashMap;

/*    Paging in Taleä is a simple single level scheme to reduce complexity.
    Swapping to disk (if desired), shold be implemented in software.
    It is recommended that the OS sets a maximum process number to help with not eating up Data Memory (maybe 10 or 20)
    ╭──────────────┬──────────┬────────┬──────────╮
    │Linear Address│ device: 8│page: 12│offset: 12│
    ╰──────────────┴──────────┴────────┴──────────╯
    ╭────────────────┬─────────────────────────────────────────────╮
    │Physical Address│PageTable[page * sizeof(Entry)] * 4K + offset│
    ╰────────────────┴─────────────────────────────────────────────╯

    Entries in the page table must be 2 bytes long, (a halfword) and contain this fields:
    ╭─────┬────────────────────────┬───┬───┬───────┬─────────╮
    │Entry│Physical page address:12│w:1│x:1│dirty:1│present:1│
    ╰─────┴────────────────────────┴───┴───┴───────┴─────────╯
    Pages are allways readable in user mode, and no restrictions aplly in supervisor mode.


    Device field is used for memory mapping: (UNUSED //TODO: Think if this is interesting)
    ╭──────┬────╮
    │Device│ ID │
    ├──────┼────┤
    │RAM   │0x00│
    ├──────┼────┤
    │ROM   │0x01│
    ├──────┼────┤
    │Video │0x02│
    ├──────┼────┤
    │Output│0x03│
    ╰──────┴────╯
*/

use crate::components::MEMSIZE;
use crate::components::cpu::state::Exceptions;
use modular_bitfield_msb::prelude::*;
use organum::{core::{Addressable, Address}, error::Error};

pub const ENTRY_SIZE: usize = 2;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 1024;
pub const PAGE_TABLE_SIZE: usize = PAGE_TABLE_ENTRIES * ENTRY_SIZE;
pub const PAGE_DIRECTORY_ENTRIES: usize = (MEMSIZE/PAGE_SIZE) / PAGE_TABLE_ENTRIES;

pub const PT_SHIFT: u32 = PAGE_TABLE_ENTRIES.count_ones();
pub const PT_MASK: u32 = (PAGE_DIRECTORY_ENTRIES as u32) << PT_SHIFT;
pub const PD_SHIFT: u32 = PAGE_DIRECTORY_ENTRIES.count_ones() + PT_SHIFT;
pub const PD_MASK: u32 = PAGE_DIRECTORY_ENTRIES as u32 - 1;
pub const OFFSET_MASK: u32 = PAGE_SIZE as u32 - 1;

#[modular_bitfield_msb::bitfield]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PageDirectoryEntry {
     // Page directory sits in Data Memory
     pub physical_addr: B12,
     #[skip] __: B4, //TODO: which flags should I include here
}

#[modular_bitfield_msb::bitfield]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PageTableEntry {
     // Page Tables sit in RAM
     pub physical_addr: B12,
     pub w: bool,
     pub x: bool,
     pub dirty: bool,
     pub present: bool
}

pub struct Tlb {
     pub tlb: HashMap<u16, (u16, bool, bool)>,
}

impl Tlb{
     //due to the nature of the hash map, the tlb need not be invalidated even in context switches
     // this is not real, but it's handy (//TODO: check cost of insertion between the other)
     pub fn new() -> Self {
          Self {
               tlb: HashMap::new(),
          }
     }

     pub fn record(&mut self, linear: u16, (physical, w, x): (u16, bool, bool)) {
          self.tlb.insert(linear, (physical, w, x));
     }

     pub fn invalidate(&mut self, entry: u16) {
          self.tlb.remove(&entry);
     }

     pub fn clear(&mut self) {
          self.tlb.clear();
     }

     pub fn get(&self, linear: u16) -> Option<(u16, bool, bool)> {
          self.tlb.get(&linear).cloned()
     }
}

pub struct Mmu {
     pub tlb: Tlb,
}

impl Mmu {
     pub fn new() -> Self {
          Self {
               tlb: Tlb::new()
          }
     }

     pub fn translate(&mut self, linear: u32, directory: &mut dyn Addressable, table: &mut dyn Addressable, directory_pointer: Address) -> Result<(Address, bool, bool), Error> {
          match self.tlb.get((&linear >> PT_SHIFT) as u16) {
               Some(entry) => Ok((((entry.0 as Address) << PT_SHIFT | (linear as Address) & OFFSET_MASK as Address), entry.1, entry.2)),
               None => {
                    let table_entry = (linear as Address >> PD_SHIFT) & PD_MASK as Address;

                    let table_entry = PageDirectoryEntry::from_bytes(u16::to_be_bytes(directory.read_beu16(directory_pointer + table_entry)?));
                    let table_entry = (table_entry.physical_addr() as Address) << PT_SHIFT; 
          
                    let page_offset = (linear as Address >> PT_SHIFT) & PT_MASK as Address;
                    let page = PageTableEntry::from_bytes(u16::to_be_bytes(table.read_beu16(table_entry | page_offset)?));
                    let w = page.w();
                    let x = page.x();
                    if !page.present() {
                         println!("{:?}, {:?}", page, page.into_bytes());
                         return Err(Error::processor(Exceptions::PageFault as u32));
                    }
                    let page = (page.physical_addr() as Address) << PT_SHIFT as Address;
                    let physical = page | (linear as Address & OFFSET_MASK as Address);
                    self.tlb.record((linear >> PT_MASK as Address) as u16, (((page >> PT_MASK as Address) as u16), w, x));
                    Ok((physical, w, x))
               }
          }
          // TODO: account for the flags and those things Implement dirty bit
     }
}