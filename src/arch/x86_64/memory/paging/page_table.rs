//! Contains code for dealing with page tables.

use super::page_table_entry::*;
use memory::{PhysicalAddress, VirtualAddress};
use core::marker::PhantomData;
use core::ops::Index;
use core::ops::IndexMut;

/// The number of entries in a page table.
const ENTRY_NUMBER: usize = 512;

/// Represents a page table level.
pub trait PageTableLevel {
    fn get_level() -> usize;
}

/// Represents a page table level that is not one.
pub trait ReducablePageTableLevel: PageTableLevel {
    /// The page table level below this one.
    type NextLevel: PageTableLevel;
}

/// Page table level 4.
pub struct Level4;
impl PageTableLevel for Level4 {
    fn get_level() -> usize {
        4
    }
}
impl ReducablePageTableLevel for Level4 {
    type NextLevel = Level3;
}

/// Page table level 3.
pub struct Level3;
impl PageTableLevel for Level3 {
    fn get_level() -> usize {
        3
    }
}
impl ReducablePageTableLevel for Level3 {
    type NextLevel = Level2;
}

/// Page table level 2.
pub struct Level2;
impl PageTableLevel for Level2 {
    fn get_level() -> usize {
        2
    }
}
impl ReducablePageTableLevel for Level2 {
    type NextLevel = Level1;
}

/// Page table level 1.
pub struct Level1;
impl PageTableLevel for Level1 {
    fn get_level() -> usize {
        1
    }
}

/// Represents a page table.
#[repr(C)]
pub struct PageTable<T> {
    entries: [PageTableEntry; ENTRY_NUMBER],
    level: PhantomData<T>
}

impl<T: ReducablePageTableLevel> PageTable<T> {
    /// Returns the address of the next page table if there is one.
    fn get_next_level_address(&self, index: usize) -> Option<PhysicalAddress> {
        assert!(index < ENTRY_NUMBER);
        let flags = self[index].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
            Some((self as *const _ as usize | index << 3) << 9)
        } else {
            None
        }
    }

    /// Returns a reference to next page table of there is one.
    pub fn get_next_level(&self, index: usize) -> Option<&PageTable<T::NextLevel>> {
        self.get_next_level_address(index)
            .map(|address| unsafe { &*(address as *const PageTable<T::NextLevel>) })
    }

    /// Returns a mutable reference to next page table of there is one.
    pub fn get_next_level_mut(&mut self, index: usize) -> Option<&mut PageTable<T::NextLevel>> {
        self.get_next_level_address(index)
            .map(|address| unsafe { &mut *(address as *mut PageTable<T::NextLevel>) })
    }
}

//impl PageTable<Level1> {
    ///// Returns the index into the current level of the page table.
    //pub fn table_index(address: VirtualAddress) -> usize {
        //// address & (0o777 << (12 + 9 * 0))
        //(address >> (12 + 9 * 0)) & 0o777
    //}
//}
//
//impl PageTable<Level2> {
    ///// Returns the index into the current level of the page table.
    //pub fn table_index(address: VirtualAddress) -> usize {
        //// address & (0o777 << (12 + 9 * 1))
        //(address >> (12 + 9 * 1)) & 0o777
    //}
//}
//
//impl PageTable<Level3> {
    ///// Returns the index into the current level of the page table.
    //pub fn table_index(address: VirtualAddress) -> usize {
        //// address & (0o777 << (12 + 9 * 2))
        //(address >> (12 + 9 * 2)) & 0o777
    //}
//}
//
//impl PageTable<Level4> {
    ///// Returns the index into the current level of the page table.
    //pub fn table_index(address: VirtualAddress) -> usize {
        //// address & (0o777 << (12 + 9 * 3))
        //(address >> (12 + 9 * 3)) & 0o777
    //}
//}

impl<T: PageTableLevel> PageTable<T> {
    pub fn table_index(address: VirtualAddress) -> usize {
        (address >> (12 + 9 * (T::get_level() - 1))) & 0o777
    }
}

impl<T: PageTableLevel> Index<usize> for PageTable<T> {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

impl<T: PageTableLevel> IndexMut<usize> for PageTable<T> {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}
