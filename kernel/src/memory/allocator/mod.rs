//! Provides the heap allocator for the kernel.

mod linked_list_allocator;

use self::linked_list_allocator::LinkedListAllocator;
use alloc::allocator::{GlobalAlloc, Layout};
use arch::{self, Architecture};
use memory::{Address, VirtualAddress};
use sync::mutex::Mutex;

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    // TODO: Read more on this trait and possibly make it more efficient.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.lock().allocate_first_fit(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.lock().free(ptr, layout.size(), layout.align());
    }
}

lazy_static! {
    /// The kernel heap allocator.
    static ref ALLOCATOR: Mutex<LinkedListAllocator> =
        Mutex::new(LinkedListAllocator::new(arch::Current::HEAP_AREA));
}

/// Aligns the given address to the given alignment.
///
/// The alignment must be a power of two.
fn align(address: VirtualAddress, alignment: usize) -> VirtualAddress {
    debug_assert!(alignment.is_power_of_two());

    if address.as_usize() % alignment == 0 {
        address
    } else {
        let alignment_bitmask = !(alignment - 1);
        VirtualAddress::from_usize((address.as_usize() & alignment_bitmask) + alignment)
    }
}
