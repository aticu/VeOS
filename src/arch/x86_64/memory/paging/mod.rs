//! Deals with the page tables.
mod page_table;
mod page_table_entry;
mod current_page_table;
mod inactive_page_table;
mod free_list;
mod frame_allocator;
mod page_table_manager;

use self::current_page_table::CURRENT_PAGE_TABLE;
use self::frame_allocator::FRAME_ALLOCATOR;
use self::page_table_entry::*;
use self::page_table_manager::PageTableManager;
use super::*;
use core::fmt;
use memory;
use memory::{PageFlags, PhysicalAddress, VirtualAddress};
use x86_64::instructions::tables::{DescriptorTablePointer, lgdt};

/// Initializes the paging.
pub fn init() {
    assert_has_not_been_called!("The x86_64 paging module should only be initialized once.");

    free_list::init();

    unsafe { remap_kernel() };
}

/// Maps the given page using the given flags.
pub fn map_page(start_address: VirtualAddress, flags: PageFlags) {
    let mut real_flags = PRESENT;

    if flags.contains(memory::WRITABLE) {
        real_flags |= WRITABLE;
    }

    if !flags.contains(memory::EXECUTABLE) {
        real_flags |= NO_EXECUTE;
    }

    CURRENT_PAGE_TABLE
        .lock()
        .map_page(Page::from_address(start_address), real_flags);
}

/// Unmaps the given page.
///
/// # Safety
/// - Make sure this page isn't referenced anymore when unmapping it.
pub unsafe fn unmap_page(start_address: VirtualAddress) {
    CURRENT_PAGE_TABLE
        .lock()
        .unmap_page(Page::from_address(start_address));
}

/// Maps the kernel properly for the first time.
///
/// # Safety
/// - This should only be called once.
unsafe fn remap_kernel() {
    assert_has_not_been_called!("The kernel should only be remapped once.");

    let mut new_page_table = inactive_page_table::InactivePageTable::new();

    {
        // Map a section.
        let mut map_section = |size: usize, start: usize, flags: PageTableEntryFlags| for i in
            0..size / PAGE_SIZE {
            let address = start + i * PAGE_SIZE;
            new_page_table.map_page_at(Page::from_address(to_virtual!(address)),
                                       PageFrame::from_address(address),
                                       flags);
        };

        // Map the text section.
        // TODO: This doesn't seem to be read only. Check that some time.
        map_section(RODATA_START - TEXT_START, TEXT_START, GLOBAL);

        // Map the rodata section.
        map_section(DATA_START - RODATA_START, RODATA_START, GLOBAL | NO_EXECUTE);

        // Map the data section.
        map_section(BSS_START - DATA_START,
                    DATA_START,
                    WRITABLE | GLOBAL | NO_EXECUTE);

        // Map the bss section
        map_section(BSS_END - BSS_START,
                    BSS_START,
                    WRITABLE | GLOBAL | NO_EXECUTE);
    }

    // Map the GDT.
    new_page_table.map_page_at(Page::from_address(to_virtual!(GDT)),
                               PageFrame::from_address(GDT),
                               GLOBAL | NO_EXECUTE);

    // Map the VGA buffer.
    new_page_table.map_page_at(Page::from_address(to_virtual!(0xb8000)),
                               PageFrame::from_address(0xb8000),
                               WRITABLE | GLOBAL | NO_EXECUTE);

    // Map the stack pages.
    let stack_size = STACK_TOP - STACK_BOTTOM;
    for i in 0..stack_size / PAGE_SIZE {
        let physical_address = STACK_BOTTOM + i * PAGE_SIZE;
        let virtual_address = FINAL_STACK_TOP - stack_size + i * PAGE_SIZE;
        new_page_table.map_page_at(Page::from_address(virtual_address),
                                   PageFrame::from_address(physical_address),
                                   WRITABLE | GLOBAL | NO_EXECUTE);
    }

    CURRENT_PAGE_TABLE.lock().switch(new_page_table).unmap();

    // Deallocate the inital, now unused, page tables.
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L4_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L3_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(L2_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(STACK_L2_TABLE));
    FRAME_ALLOCATOR.deallocate(PageFrame::from_address(STACK_L1_TABLE));

    // Reload the now invalid GDT.
    lgdt(&*(GDT_PTR as *const DescriptorTablePointer));
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

    pub fn copy(&self) -> PageFrame {
        PageFrame(self.0)
    }
}

impl fmt::Debug for PageFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageFrame: {:x}", self.0)
    }
}
