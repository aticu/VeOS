//! This is the x86_64 implementation of the `AddressSpaceManager` trait.

use super::PAGE_SIZE;
use super::paging::{CURRENT_PAGE_TABLE, Page, PageFrame, convert_flags};
use super::paging::inactive_page_table::InactivePageTable;
use super::paging::page_table_entry::*;
use super::paging::page_table_manager::PageTableManager;
use alloc::boxed::Box;
use core::ptr;
use memory::{PageFlags, PhysicalAddress, VirtualAddress};
use memory::address_space;

struct AddressSpaceManager {
    table: InactivePageTable
}

pub fn new_address_space_manager() -> Box<address_space::AddressSpaceManager> {
    Box::new(AddressSpaceManager { table: InactivePageTable::copy_from_current() })
}

pub fn idle_address_space_manager() -> Box<address_space::AddressSpaceManager> {
    Box::new(AddressSpaceManager { table: InactivePageTable::from_current_table() })
}

impl address_space::AddressSpaceManager for AddressSpaceManager {
    fn write_to(&mut self, buffer: &[u8], address: VirtualAddress, flags: PageFlags) {
        let flags = convert_flags(flags);

        let start_page_num = address / PAGE_SIZE;
        let end_page_num = (address + buffer.len() - 1) / PAGE_SIZE + 1;

        let mut current_offset = address % PAGE_SIZE;
        let mut current_buffer_position = 0;

        // For all pages.
        for page_num in start_page_num..end_page_num {
            let page_address = page_num * PAGE_SIZE;

            // First map with write permissions.
            self.table
                .change_permissions_or_map(Page::from_address(page_address), WRITABLE);

            // Get the physical address.
            let mut entry = self.table.get_entry_and_map(page_address);
            let physical_address = entry
                .points_to()
                .expect("The just mapped page isn't mapped.");

            // Write to the physical address.
            let (new_current_buffer_position, new_current_offset) = CURRENT_PAGE_TABLE
                .lock()
                .with_temporary_page(&PageFrame::from_address(physical_address), |page| {
                    let start_address = page.get_address() + current_offset;

                    let write_length = if (PAGE_SIZE - current_offset) >=
                                          buffer.len() - current_buffer_position {
                        // If the rest fits within the page.
                        buffer.len() - current_buffer_position
                    } else {
                        // There is still more to fill.
                        PAGE_SIZE - current_offset
                    };

                    unsafe {
                        ptr::copy_nonoverlapping(buffer.as_ptr(),
                                                 start_address as *mut u8,
                                                 write_length);
                    }

                    (current_buffer_position + write_length,
                     (current_offset + write_length) % PAGE_SIZE)
                });

            current_offset = new_current_offset;
            current_buffer_position = new_current_buffer_position;

            // Change to the desired flags.
            entry.set_flags(flags);
        }

        self.table.unmap();
    }

    unsafe fn get_page_table_address(&self) -> PhysicalAddress {
        self.table.get_frame().get_address()
    }

    fn map_page(&mut self, page_address: VirtualAddress, flags: PageFlags) {
        let flags = convert_flags(flags);

        self.table
            .map_page(Page::from_address(page_address), flags);

        self.table.unmap();
    }

    unsafe fn unmap_page(&mut self, start_address: VirtualAddress) {
        self.table.unmap_page(Page::from_address(start_address));

        self.table.unmap();
    }
}
