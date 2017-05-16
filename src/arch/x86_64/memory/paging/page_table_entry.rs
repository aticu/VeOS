//! Handles page table entries.

use memory::PhysicalAddress;
use core::fmt;

/// Serves as a mask for the physical address in a page table entry.
const PHYSICAL_ADDRESS_MASK: usize = 0xffffffffff << 12;

/// Represents a page table entry.
#[repr(C)]
pub struct PageTableEntry(u64);

bitflags! {
    /// The possible flags in a page table entry.
    pub flags PageTableEntryFlags: u64 {
        /// The page is present.
        const PRESENT = 1 << 0,
        /// The page is writable.
        const WRITABLE = 1 << 1,
        /// The page is accessible in user mode.
        const USER_ACCESSIBLE = 1 << 2,
        /// Writes will not be cached.
        const WRITE_TROUGH_CACHING = 1 << 3,
        /// Page accesses will not be cached.
        const DISABLE_CACHE = 1 << 4,
        /// The page was accessed.
        const ACCESSED = 1 << 5,
        /// The page was written to.
        const DIRTY = 1 << 6,
        /// The page is a huge page.
        const HUGE_PAGE = 1 << 7,
        /// The page is global.
        ///
        /// This means that it won't be flushed from the caches on an address space switch.
        const GLOBAL = 1 << 8,
        /// Ensures mutual exclusion for temporary pages.
        ///
        /// It also makes sure that the page entry can't be changed while in use.
        /// Only valid in the temporary mapping tables.
        const TEMPORARY_TABLE_LOCK = 1 << 9,
        /// No code on this page can be executed.
        const NO_EXECUTE = 1 << 63,
    }
}

impl PageTableEntry {
    /// Creates a new page table entry.
    pub fn new() -> PageTableEntry {
        PageTableEntry(0)
    }

    /// Gets the flags from a page table entry.
    pub fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags::from_bits_truncate(self.0)
    }

    /// Returns the address this entry points to.
    pub fn points_to(&self) -> Option<PhysicalAddress> {
        if self.flags().contains(PRESENT) {
            Some(self.0 as usize & PHYSICAL_ADDRESS_MASK)
        } else {
            None
        }
    }

    /// Sets the address of this entry.
    ///
    /// Returns the entry for convenience when chaining functions.
    pub fn set_address(&mut self, address: PhysicalAddress) {
        assert!(address & !PHYSICAL_ADDRESS_MASK == 0);
        self.0 &= !PHYSICAL_ADDRESS_MASK as u64; // clear address field first
        self.0 |= address as u64 & PHYSICAL_ADDRESS_MASK as u64;
    }

    /// Sets the given flags in the entry.
    pub fn set_flags(&mut self, flags: PageTableEntryFlags) {
        self.0 = (self.0 & PHYSICAL_ADDRESS_MASK as u64) | flags.bits();
    }

    /// Adds the given flags to the entry.
    pub fn add_flags(&mut self, flags: PageTableEntryFlags) {
        let mut current_flags = self.flags();
        current_flags.insert(flags);
        self.set_flags(current_flags);
    }

    /// Removes the given flags from the entry.
    pub fn remove_flags(&mut self, flags: PageTableEntryFlags) {
        let mut current_flags = self.flags();
        current_flags.remove(flags);
        self.set_flags(current_flags);
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.flags().contains(PRESENT) {
            write!(f,
                   "Entry(Address=0x{:x}, Flags={:?})",
                   self.points_to().unwrap(),
                   self.flags())
        } else {
            write!(f, "Entry(Address=invalid, Flags={:?})", self.flags())
        }
    }
}
