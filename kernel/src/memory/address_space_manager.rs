//! This module defines what an address space manager can do.

use super::{MemoryArea, PAGE_SIZE, PageFlags, PhysicalAddress, VirtualAddress};

/// This trait should be implemented by any architecture specific address space
/// manager.
pub trait AddressSpaceManager: Send {
    /// Writes the data in `buffer` to the `address` in the target address
    /// space setting the given flags.
    fn write_to(&mut self, buffer: &[u8], address: VirtualAddress, flags: PageFlags);

    /// Returns the address of the page table.
    ///
    /// # Safety
    /// - Should only be used by architecture specific code.
    unsafe fn get_page_table_address(&self) -> PhysicalAddress; // TODO: Find something better than exposing this publicly.

    /// Maps the given page in the managed address space.
    fn map_page(&mut self, page_address: VirtualAddress, flags: PageFlags);

    /// Unmaps the given page in the managed address space.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    unsafe fn unmap_page(&mut self, start_address: VirtualAddress);

    /// Unmaps the given page in the managed address space not checking if it
    /// was mapped.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    unsafe fn unmap_page_unchecked(&mut self, start_address: VirtualAddress); // TODO: Check if this is necessary.

    /// Creates a new kernel stack.
    ///
    /// This assumes that the given thread id is unused.
    //fn create_kernel_stack(id: ThreadId);

    /// Creates a new user mode stack.
    ///
    /// This assumes that the given thread id is unused.
    //fn create_user_stack(id: ThreadId);

    /// Zeroes the given area in the managed address space.
    fn zero(&mut self, area: MemoryArea<VirtualAddress>, flags: PageFlags) {
        let start = area.start_address();
        let length = area.length();
        let zero: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        let num_of_pages = (length - 1) / PAGE_SIZE + 1;

        for i in 0..num_of_pages {
            let buffer = {
                if (i + 1) * PAGE_SIZE > length {
                    &zero[0..length % PAGE_SIZE]
                } else {
                    &zero[..]
                }
            };
            self.write_to(buffer, start + i * PAGE_SIZE, flags);
        }
    }
}