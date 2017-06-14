//! Contains code for dealing with page tables.

use super::frame_allocator::FRAME_ALLOCATOR;
use super::page_table_entry::*;
use core::marker::PhantomData;
use core::ops::Index;
use core::ops::IndexMut;
use memory::VirtualAddress;

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
    /// Represents the entries in a page table.
    entries: [PageTableEntry; ENTRY_NUMBER],
    level: PhantomData<T>
}

impl<T: ReducablePageTableLevel> PageTable<T> {
    /// Returns the address of the next page table if there is one.
    fn get_next_level_address(&self, index: usize) -> Option<VirtualAddress> {
        assert!(index < ENTRY_NUMBER);
        let flags = self[index].flags();
        debug_assert!(!flags.contains(HUGE_PAGE));
        if flags.contains(PRESENT) {
            Some((self as *const _ as usize | index << 3) << 9)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the next table.
    ///
    /// If no next table exists yet, then it is allocated and zeroed.
    pub fn next_level_and_map(&mut self, address: VirtualAddress) -> &mut PageTable<T::NextLevel> {
        let index = PageTable::<T>::table_index(address);
        let flags = self[index].flags();
        debug_assert!(!flags.contains(HUGE_PAGE));
        // TODO: here would be the place to check whether the page is swapped out

        let new_table = if !flags.contains(PRESENT) {
            // create a new table
            let frame = FRAME_ALLOCATOR.allocate();
            self[index].set_flags(PAGE_TABLE_FLAGS);
            self[index].set_address(frame.get_address());
            true
        } else {
            false
        };
        let table = unsafe {
            &mut *(((self as *const _ as usize | index << 3) << 9) as *mut PageTable<T::NextLevel>)
        };
        if new_table {
            // zero the table out
            table.zero();
        }
        table
    }

    /// Returns a reference to next page table of there is one.
    pub fn get_next_level(&self, address: VirtualAddress) -> Option<&PageTable<T::NextLevel>> {
        let index = PageTable::<T>::table_index(address);
        self.get_next_level_address(index)
            .map(|address| unsafe { &*(address as *const PageTable<T::NextLevel>) })
    }

    /// Returns a mutable reference to next page table of there is one.
    pub fn get_next_level_mut(&mut self,
                              address: VirtualAddress)
                              -> Option<&mut PageTable<T::NextLevel>> {
        let index = PageTable::<T>::table_index(address);
        self.get_next_level_address(index)
            .map(|address| unsafe { &mut *(address as *mut PageTable<T::NextLevel>) })
    }
}

impl<T: PageTableLevel> PageTable<T> {
    /// Returns the index of the given page table level in the given address.
    pub fn table_index(address: VirtualAddress) -> usize {
        (address >> (12 + 9 * (T::get_level() - 1))) & 0o777
    }

    /// Zeros the given table out.
    pub fn zero(&mut self) {
        for i in 0..512 {
            self[i] = PageTableEntry::new();
        }
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
