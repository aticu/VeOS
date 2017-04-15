//!Contains code for dealing with page tables.
use core::marker::PhantomData;
use core::ptr::Unique;
use core::ops::Index;
use core::ops::IndexMut;
use super::page_table_entry::*;
use super::super::{VirtualAddress, PhysicalAddress};

///The address of the current Level 4 table.
///
///Note that this is only valid if the level 4 table is mapped recursively on the last entry.
const L4_TABLE: *mut PageTable<Level4> = 0xfffffffffffff000 as *mut PageTable<Level4>;

///The number of entries in a page table.
const ENTRY_NUMBER: usize = 512;

///Represents a page table level.
trait PageTableLevel {}

///Represents a page table level that is not one.
trait ReducablePageTableLevel: PageTableLevel {
    ///The page table level below this one.
    type NextLevel: PageTableLevel;
}

///Page table level 4.
struct Level4;
impl PageTableLevel for Level4 {}
impl ReducablePageTableLevel for Level4 {
    type NextLevel = Level3;
}

///Page table level 3.
struct Level3;
impl PageTableLevel for Level3 {}
impl ReducablePageTableLevel for Level3 {
    type NextLevel = Level2;
}

///Page table level 2.
struct Level2;
impl PageTableLevel for Level2 {}
impl ReducablePageTableLevel for Level2 {
    type NextLevel = Level1;
}

///Page table level 1.
struct Level1;
impl PageTableLevel for Level1 {}

///Represents a page table.
#[repr(C)]
struct PageTable<T> {
    entries: [PageTableEntry; ENTRY_NUMBER],
    level: PhantomData<T>,
}

impl<T: ReducablePageTableLevel> PageTable<T> {
    ///Returns the address of the next page table if there is one.
    fn get_next_level_address(&self, index: usize) -> Option<PhysicalAddress> {
        assert!(index < ENTRY_NUMBER);
        let flags = self[index].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
            Some((self as *const _ as usize | index << 3) << 9)
        } else {
            None
        }
    }

    ///Returns a reference to next page table of there is one.
    pub fn get_next_level(&self, index: usize) -> Option<&PageTable<T::NextLevel>> {
        self.get_next_level_address(index).map(|address| unsafe { &*(address as *const PageTable<T::NextLevel>) })
    }

    ///Returns a mutable reference to next page table of there is one.
    pub fn get_next_level_mut(&mut self, index: usize) -> Option<&mut PageTable<T::NextLevel>> {
        self.get_next_level_address(index).map(|address| unsafe { &mut*(address as *mut PageTable<T::NextLevel>) })
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

///Owns the page table currently in use.
pub struct CurrentPageTable {
    l4_table: Unique<PageTable<Level4>>
}

impl CurrentPageTable {
    ///Returns the current page table.
    ///
    ///#Safety
    ///At any point in time there should only be exactly one current page table struct.
    pub unsafe fn new() -> CurrentPageTable {
        CurrentPageTable {
            l4_table: Unique::new(L4_TABLE),
        }
    }

    ///Returns a reference to the level 4 page table.
    fn get_l4(&self) -> &PageTable<Level4> {
        unsafe { self.l4_table.get() }
    }

    ///Returns a mutable reference to the level 4 page table.
    fn get_l4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.l4_table.get_mut() }
    }

    ///Returns the corresponding physical address to a virtual address.
    pub fn translate_address(&self, address: VirtualAddress) -> Option<PhysicalAddress> {
        assert!(address < 0x0000800000000000 || address >= 0xffff800000000000);
        let l4 = self.get_l4();
        let mut address = address;
        let page_index = address & 0xfff;
        address = address >> 12;
        let l1_index = address & 0o777;
        address = address >> 9;
        let l2_index = address & 0o777;
        address = address >> 9;
        let l3_index = address & 0o777;
        address = address >> 9;
        let l4_index = address & 0o777;

        l4.get_next_level(l4_index)
          .and_then(|l3| l3.get_next_level(l3_index))
          .and_then(|l2| l2.get_next_level(l2_index))
          .and_then(|l1| l1[l1_index].points_to())
          .map(|page_address| page_address + page_index)
    }
}
