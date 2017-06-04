//! Handles page table entries.

use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use memory::PhysicalAddress;
use super::PageFrame;
use super::frame_allocator::FRAME_ALLOCATOR;
use sync::{PreemptionState, disable_preemption, restore_preemption_state, cpu_relax};

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
        /// Ensures mutual exclusion for pages this entry points to.
        const ENTRY_LOCK = 1 << 9,
        /// No code on this page can be executed.
        const NO_EXECUTE = 1 << 63,

        /// The flags used for page tables.
        const PAGE_TABLE_FLAGS = PRESENT.bits | WRITABLE.bits
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
    pub fn set_address(&mut self, address: PhysicalAddress) -> &mut PageTableEntry {
        assert_eq!(address & !PHYSICAL_ADDRESS_MASK, 0);
        self.0 &= !PHYSICAL_ADDRESS_MASK as u64; // Clear address field first.
        self.0 |= address as u64 & PHYSICAL_ADDRESS_MASK as u64;
        self
    }

    /// Sets the given flags in the entry.
    pub fn set_flags(&mut self, flags: PageTableEntryFlags) -> &mut PageTableEntry {
        if self.is_locked() {
            self.0 = (self.0 & PHYSICAL_ADDRESS_MASK as u64) | flags.bits() | ENTRY_LOCK.bits();
        } else {
            self.0 = (self.0 & PHYSICAL_ADDRESS_MASK as u64) | flags.bits();
        }
        self
    }

    /// Adds the given flags to the entry.
    pub fn add_flags(&mut self, flags: PageTableEntryFlags) -> &mut PageTableEntry {
        let mut current_flags = self.flags();
        current_flags.insert(flags);
        self.set_flags(current_flags)
    }

    /// Removes the given flags from the entry.
    pub fn remove_flags(&mut self, flags: PageTableEntryFlags) -> &mut PageTableEntry {
        let mut current_flags = self.flags();
        current_flags.remove(flags);
        self.set_flags(current_flags)
    }

    /// Unmaps and deallocates the frame this entry points to.
    pub fn unmap(&mut self) {
        let address = self.points_to().expect("Trying to unmap and unmapped page.");
        unsafe { FRAME_ALLOCATOR.deallocate(PageFrame::from_address(address)) };
        self.0 = 0;
    }

    /// Locks the pages this entry points to.
    ///
    /// They can't be accessed by other processors/threads after being locked.
    /// # Note
    /// The preemtion state must be restored when unlocking.
    pub fn lock(&mut self) -> PreemptionState {
        let mut preemption_state;
        let atomic_lock: &AtomicU64 = unsafe { &*((&mut self.0) as *mut u64 as *mut AtomicU64) };
        loop {
            unsafe {
                preemption_state = disable_preemption();
            }
            let lock_switch = atomic_lock.fetch_or(ENTRY_LOCK.bits(), Ordering::Acquire) & ENTRY_LOCK.bits() == 0;
            if lock_switch {
                break;
            } else {
                unsafe {
                    restore_preemption_state(&preemption_state);
                }
            }

            // Wait until the lock looks unlocked before retrying
            while atomic_lock.load(Ordering::Relaxed) & ENTRY_LOCK.bits() > 0 {
                cpu_relax();
            }
        }

        preemption_state
    }

    /// Unlocks the pages this entry points to and restores the preemption state.
    pub fn unlock(&mut self, preemption_state: &PreemptionState) {
        self.0 = self.0 & !ENTRY_LOCK.bits();
        unsafe {
            restore_preemption_state(preemption_state);
        }
    }

    /// Checks if this entry is locked.
    pub fn is_locked(&self) -> bool {
        self.flags().contains(ENTRY_LOCK)
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

    /// Tests that changing the flags doesn't change the address.
    #[test]
    fn test_flag_change() {
        let mut entry = PageTableEntry::new();
        let flags = PRESENT | DIRTY | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE;
        entry.set_address(0xcafeb000);
        entry.set_flags(flags);
        assert_eq!(entry.points_to(), Some(0xcafeb000));
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
