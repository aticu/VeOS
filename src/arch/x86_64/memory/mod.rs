//! Handles all x86_64 memory related issues.

use memory::{PhysicalAddress, VirtualAddress};

pub mod paging;

/// The maximum address of the lower part of the virtual address space.
const VIRTUAL_LOW_MAX_ADDRESS: VirtualAddress = 0x00000fffffffffff;

/// The minimum address of the higher part of the virtual address space.
const VIRTUAL_HIGH_MIN_ADDRESS: VirtualAddress = 0xffff800000000000;

/// The top of the stack after the kernel has been remapped.
const FINAL_STACK_TOP: VirtualAddress = 0xfffffe8000000000;

extern "C" {
    /// The end of the kernel in its initial mapping.
    static KERNEL_END: PhysicalAddress;
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
    /// The address of the temporary map table.
    static TEMPORARY_MAP_TABLE: PhysicalAddress;
    /// The address of the initial level 4 page table.
    static L4_TABLE: PhysicalAddress;
    /// The address of the initial level 3 page table.
    static L3_TABLE: PhysicalAddress;
    /// The address of the initial level 2 page table.
    static L2_TABLE: PhysicalAddress;
    /// The address of the initial stack level 2 page table.
    static STACK_L2_TABLE: PhysicalAddress;
    /// The address of the initial stack level 1 page table.
    static STACK_L1_TABLE: PhysicalAddress;
    /// The bottom of the initial kernel stack.
    static STACK_BOTTOM: PhysicalAddress;
    /// The top of the initial kernel stack.
    static STACK_TOP: PhysicalAddress;
}

/// The physical address at which the kernel starts.
pub fn get_kernel_start_address() -> PhysicalAddress {
    unsafe { TEXT_START }
}

/// The physical address at which the kernel ends.
pub fn get_kernel_end_address() -> PhysicalAddress {
    unsafe { KERNEL_END }
}

/// Initializes the memory manager.
pub fn init() {
    paging::init();
}
