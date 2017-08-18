//! Deals with the page tables.
mod page_table;
pub mod page_table_entry;
mod current_page_table;
pub mod inactive_page_table;
mod free_list;
mod frame_allocator;
pub mod page_table_manager;

pub use self::current_page_table::CURRENT_PAGE_TABLE;
use self::frame_allocator::FRAME_ALLOCATOR;
use self::page_table_entry::*;
use self::page_table_manager::PageTableManager;
use super::*;
use core::fmt;
use memory;
use memory::{PageFlags, PhysicalAddress, VirtualAddress};

/// Initializes the paging.
pub fn init(initramfs_start: PhysicalAddress, initramfs_length: usize) {
    assert_has_not_been_called!("The x86_64 paging module should only be initialized once.");

    free_list::init();

    unsafe { remap_kernel() };

    unsafe { map_initramfs(initramfs_start, initramfs_length) };
}

/// Converts the general `PageFlags` to x86_64-specific flags.
pub fn convert_flags(flags: PageFlags) -> PageTableEntryFlags {
    let mut entry_flags = PRESENT;

    if flags.contains(memory::WRITABLE) {
        entry_flags |= WRITABLE;
    }

    if !flags.contains(memory::EXECUTABLE) {
        entry_flags |= NO_EXECUTE;
    }

    if flags.contains(memory::NO_CACHE) {
        entry_flags |= DISABLE_CACHE;
    }

    if flags.contains(memory::USER_ACCESSIBLE) {
        entry_flags |= USER_ACCESSIBLE;
    }

    entry_flags
}

/// Returns the flags for the given page, if the page is mapped.
pub fn get_page_flags(page_address: VirtualAddress) -> PageFlags {
    let mut flags = PageFlags::empty();
    let mut table = CURRENT_PAGE_TABLE.lock();

    if let Some(entry) = table.get_entry(Page::from_address(page_address).get_address()) {
        let entry_flags = entry.flags();

        if entry_flags.contains(PRESENT) {
            flags |= ::memory::PRESENT;
        }

        if entry_flags.contains(WRITABLE) {
            flags |= memory::WRITABLE;
        }

        if !entry_flags.contains(NO_EXECUTE) {
            flags |= memory::EXECUTABLE;
        }

        if entry_flags.contains(DISABLE_CACHE) {
            flags |= memory::NO_CACHE;
        }

        if entry_flags.contains(USER_ACCESSIBLE) {
            flags |= memory::USER_ACCESSIBLE;
        }
    }

    flags
}

/// Returns the size of unused physical memory.
pub fn get_free_memory_size() -> usize {
    FRAME_ALLOCATOR.get_free_frame_num() * PAGE_SIZE
}

/// Maps the given page to the given frame using the given flags.
pub fn map_page_at(page_address: VirtualAddress, frame_address: VirtualAddress, flags: PageFlags) {
    CURRENT_PAGE_TABLE
        .lock()
        .map_page_at(Page::from_address(page_address),
                     PageFrame::from_address(frame_address),
                     convert_flags(flags));
}

/// Maps the given page using the given flags.
pub fn map_page(page_address: VirtualAddress, flags: PageFlags) {
    CURRENT_PAGE_TABLE
        .lock()
        .map_page(Page::from_address(page_address), convert_flags(flags));
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

/// Maps the initramfs into the kernel.
///
/// # Safety
/// - This should only be called once.
unsafe fn map_initramfs(initramfs_start: PhysicalAddress, initramfs_length: usize) {
    assert_has_not_been_called!("Trying to map the initramfs twice");

    if initramfs_length > 0 {
        let initramfs_page_amount = (initramfs_length - 1) / PAGE_SIZE + 1;

        // Map the initramfs.
        for i in 0..initramfs_page_amount {
            let physical_address = initramfs_start + i * PAGE_SIZE;
            let virtual_address = INITRAMFS_MAP_AREA_START + i * PAGE_SIZE;
            map_page_at(virtual_address, physical_address, memory::READABLE);
        }
    }
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

    /// Creates a copy of the page frame.
    ///
    /// # Safety
    /// - Make sure that each frame is still only managed once.
    pub unsafe fn copy(&self) -> PageFrame {
        PageFrame(self.0)
    }
}

impl fmt::Debug for PageFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageFrame: {:x}", self.0)
    }
}
