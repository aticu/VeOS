//! Deals with the page tables.
mod page_table;
mod page_table_entry;
mod current_page_table;
mod free_list;
mod frame_allocator;

use core::fmt;
use memory::{PhysicalAddress, VirtualAddress};

/// The size of a single page.
pub const PAGE_SIZE: usize = 0x1000;

/// Represents a page.
pub struct Page(usize);

/// Initializes the paging.
pub fn init() {
    free_list::init();
}

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
}

impl fmt::Debug for PageFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageFrame: {:x}", self.0)
    }
}

/// Tests for methods used in the paging module.
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the points_to method of a page table entry.
    #[test]
    fn test_points_to() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xdeadb000);
        assert_eq!(entry.points_to(), None);
        entry.set_flags(PRESENT);
        assert_eq!(entry.points_to(), Some(0xdeadb000));
    }

    /// Tests that unaligned addresses panic.
    #[test]
    #[should_panic]
    fn test_unaligned_address() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xdeadbeef);
    }

    /// Tests that overflowing addresses panic.
    #[test]
    #[should_panic]
    fn test_address_overflow() {
        let mut entry = PageTableEntry::new();
        entry.set_address(0xcafebabedeadb000);
    }

    /// Tests that the flags field works as expected.
    #[test]
    fn test_flags() {
        let mut entry = PageTableEntry::new();
        let flags = PRESENT | DIRTY | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE;
        entry.set_flags(flags);
        assert_eq!(entry.flags(), flags);
    }

    /// Tests that the binary representation is as expected.
    #[test]
    fn test_representation() {
        let mut entry = PageTableEntry::new();
        let flags = PRESENT | DIRTY | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE;
        entry.set_flags(flags);
        entry.set_address(0xdeadb000);
        assert_eq!(entry.0,
                   0xdeadb000 | (1 << 0) | (1 << 6) | (1 << 2) | (1 << 1) | (1 << 63));
    }
}
