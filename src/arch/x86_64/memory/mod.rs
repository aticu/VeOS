//! Handles all x86_64 memory related issues.

mod paging;

/// Represents a physical address.
type PhysicalAddress = usize;

/// Represents a virtual address.
type VirtualAddress = usize;

/// The maximum address of the lower part of the virtual address space.
const VIRTUAL_LOW_MAX_ADDRESS: VirtualAddress = 0x00000fffffffffff;

/// The minimum address of the higher part of the virtual address space.
const VIRTUAL_HIGH_MIN_ADDRESS: VirtualAddress = 0xffff800000000000;

pub fn debug() {
    paging::debug();
}
