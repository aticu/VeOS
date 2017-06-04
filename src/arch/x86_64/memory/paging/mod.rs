//! Deals with the page tables.
mod page_table;
mod page_table_entry;
mod current_page_table;
mod inactive_page_table;
mod free_list;
mod frame_allocator;
mod page_table_manager;

use core::fmt;
use memory::{PhysicalAddress, VirtualAddress};
use self::current_page_table::CURRENT_PAGE_TABLE;
use self::page_table_entry::*;
use self::page_table_manager::PageTableManager;
use self::frame_allocator::FRAME_ALLOCATOR;
use super::*;

/// The size of a single page.
pub const PAGE_SIZE: usize = 0x1000;

/// Initializes the paging.
pub fn init() {
    free_list::init();

    unsafe { remap_kernel() };
}

/// Maps the kernel properly for the first time.
unsafe fn remap_kernel() {
    let mut new_page_table = inactive_page_table::InactivePageTable::new();

    // Map the text section.
    let text_length = RODATA_START - TEXT_START;
    for i in 0..text_length / PAGE_SIZE {
        let address = TEXT_START + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(to_virtual!(address)), PageFrame::from_address(address), GLOBAL);
    }

    // Map the rodata section.
    let rodata_length = DATA_START - RODATA_START;
    for i in 0..rodata_length / PAGE_SIZE {
        let address = RODATA_START + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(to_virtual!(address)), PageFrame::from_address(address), GLOBAL | NO_EXECUTE);
    }

    // Map the data section.
    let data_length = BSS_START - DATA_START;
    for i in 0..data_length / PAGE_SIZE {
        let address = DATA_START + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(to_virtual!(address)), PageFrame::from_address(address), WRITABLE | GLOBAL | NO_EXECUTE);
    }
    
    // Map the bss section
    let bss_length = BSS_END - BSS_START;
    for i in 0..bss_length / PAGE_SIZE {
        let address = BSS_START + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(to_virtual!(address)), PageFrame::from_address(address), WRITABLE | GLOBAL | NO_EXECUTE);
    }

    // Map the VGA buffer.
    new_page_table.map_page_at(Page::from_address(to_virtual!(0xb8000)), PageFrame::from_address(0xb8000), WRITABLE | GLOBAL | NO_EXECUTE);

    // Map the stack pages.
    let stack_size = STACK_TOP - STACK_BOTTOM;
    for i in 0..stack_size / PAGE_SIZE {
        let physical_address = STACK_BOTTOM + i * PAGE_SIZE;
        let virtual_address = FINAL_STACK_TOP - stack_size + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(virtual_address), PageFrame::from_address(physical_address), WRITABLE | GLOBAL | NO_EXECUTE);
    }

    CURRENT_PAGE_TABLE.lock().switch(new_page_table).unmap();

    // Deallocate the inital, now unused, page tables.
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L4_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L3_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L2_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(STACK_L2_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(STACK_L1_TABLE));
}

/// Represents a page.
#[derive(Clone, Copy)]
pub struct Page(usize);

impl Page {
    /// Returns the page that contains the given virtual address.
    pub fn from_address(address: VirtualAddress) -> Page {
        Page(address & !(PAGE_SIZE - 1))
    }

    /// Returns the virtual address of this page.
    pub fn get_address(&self) -> VirtualAddress {
        self.0
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Page: {:x}", self.0)
    }
}

/// Represents a page frame.
#[derive(Clone, Copy)]
pub struct PageFrame(usize);

impl PageFrame {
    /// Returns the page frame that contains the given physical address.
    pub fn from_address(address: PhysicalAddress) -> PageFrame {
        PageFrame(address & !(PAGE_SIZE - 1))
    }

    /// Returns the physical address of this page frame.
    pub fn get_address(&self) -> PhysicalAddress {
        self.0
    }
}

impl fmt::Debug for PageFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageFrame: {:x}", self.0)
    }
}

