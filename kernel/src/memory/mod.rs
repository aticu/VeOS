//! Handles all memory related things.

pub mod allocator;
pub mod address_space;

pub use arch::KERNEL_STACK_AREA_BASE;
pub use arch::KERNEL_STACK_MAX_SIZE;
pub use arch::KERNEL_STACK_OFFSET;
pub use arch::USER_STACK_AREA_BASE;
pub use arch::USER_STACK_OFFSET;
pub use arch::USER_STACK_MAX_SIZE;
pub use arch::PAGE_SIZE;
pub use arch::get_kernel_end_address;
pub use arch::get_kernel_start_address;
pub use arch::get_page_flags;
pub use arch::map_page;
pub use arch::map_page_at;
pub use arch::unmap_page;

use core::fmt;

/// Represents a physical address.
pub type PhysicalAddress = usize;

/// Represents a virtual address.
pub type VirtualAddress = usize;

/// Represents an unused chunk of memory in the physical address space.
#[derive(Clone, Copy)]
pub struct FreeMemoryArea {
    /// The address at which the chunk starts.
    pub start_address: PhysicalAddress,
    /// The length of the chunk.
    pub length: usize
}

bitflags! {
    /// The flags a page could possibly have.
    pub flags PageFlags: u8 {
        /// Set if the page can be read from.
        const READABLE = 1 << 0,
        /// Set if the page can be written to.
        const WRITABLE = 1 << 1,
        /// Set if code on the page can be executed.
        const EXECUTABLE = 1 << 2,
        /// Set if the page should not be cached.
        const NO_CACHE = 1 << 3,
        /// Set if the page should be accessible from user mode.
        const USER_ACCESSIBLE = 1 << 4,
        /// Set if the page is currently present.
        const PRESENT = 1 << 5
    }
}

impl FreeMemoryArea {
    /// Creates a new FreeMemoryArea.
    pub fn new(start_address: PhysicalAddress, length: usize) -> FreeMemoryArea {
        FreeMemoryArea {
            start_address,
            length
        }
    }

    /// Returns the start address of this free memory area.
    pub fn start_address(&self) -> PhysicalAddress {
        self.start_address
    }

    /// Returns the end address of this free memory area.
    pub fn end_address(&self) -> PhysicalAddress {
        self.start_address + self.length
    }

    /// Returns the same area except for the first frame.
    pub fn without_first_frame(&self) -> FreeMemoryArea {
        FreeMemoryArea {
            start_address: self.start_address + PAGE_SIZE,
            length: self.length - PAGE_SIZE
        }
    }
}

impl fmt::Debug for FreeMemoryArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Start: {:x}, Length: {:x}",
               self.start_address,
               self.length)
    }
}

/// Initializes the memory managing part of the kernel.
#[cfg(not(test))]
pub fn init() {
    assert_has_not_been_called!("Memory state should only be initialized once.");

    ::arch::memory_init();
}

/// This function gets called when the system is out of memory.
pub fn oom() -> ! {
    panic!("Out of memory!");
}
