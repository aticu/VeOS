//! Handles all memory related things.

use core::fmt;

pub use arch::memory::*;

/// Represents a physical address.
pub type PhysicalAddress = usize;

/// Represents a virtual address.
pub type VirtualAddress = usize;

/// Represents an unused chunk of memory in the physical address space.
pub struct FreeMemoryArea {
    /// The address at which the chunk starts.
    pub start_address: PhysicalAddress,
    /// The length of the chunk.
    pub length: usize
}

impl FreeMemoryArea {
    /// Creates a new FreeMemoryArea.
    pub fn new(start_address: PhysicalAddress, length: usize) -> FreeMemoryArea {
        FreeMemoryArea {
            start_address,
            length
        }
    }

    /// Returns the end address of this free memory area.
    pub fn end_address(&self) -> PhysicalAddress {
        self.start_address + self.length
    }

    /// Returns the same area except for the first frame.
    pub fn without_first_frame(&self) -> FreeMemoryArea {
        FreeMemoryArea {
            start_address: self.start_address + paging::PAGE_SIZE,
            length: self.length - paging::PAGE_SIZE
        }
    }
}

impl fmt::Debug for FreeMemoryArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Start: {:x}, Length: {:x}", self.start_address, self.length)
    }
}

/// Initializes the memory managing part of the kernel.
pub fn init() {
    ::arch::memory::init();
}
