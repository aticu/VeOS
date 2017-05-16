//! Handles all x86_64 memory related issues.

use memory::{VirtualAddress, PhysicalAddress};

pub mod paging;

/// The maximum address of the lower part of the virtual address space.
const VIRTUAL_LOW_MAX_ADDRESS: VirtualAddress = 0x00000fffffffffff;

/// The minimum address of the higher part of the virtual address space.
const VIRTUAL_HIGH_MIN_ADDRESS: VirtualAddress = 0xffff800000000000;

extern "C" {
    /// The start of the .text segment.
    static TEXT_START: PhysicalAddress;
    /// The start of the .rodata segment.
    static RODATA_START: PhysicalAddress;
    /// The start of the .data segment.
    static DATA_START: PhysicalAddress;
    /// The start of the .bss segment.
    static BSS_START: PhysicalAddress;
    /// The end of the .bss segment.
    static BSS_END: PhysicalAddress;
}

/// The physical address at which the kernel starts.
pub fn get_kernel_start_address() -> PhysicalAddress {
    unsafe { TEXT_START }
}

/// The physical address at which the kernel ends.
pub fn get_kernel_end_address() -> PhysicalAddress {
    unsafe { BSS_END }
}

/// Initializes the memory manager.
pub fn init() {
    paging::init();
}
