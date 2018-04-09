//! Provides the heap allocator for the kernel.

mod linked_list_allocator;

use self::linked_list_allocator::LinkedListAllocator;
use alloc::allocator::{Alloc, AllocErr, Layout};
use arch::{HEAP_MAX_SIZE, HEAP_START};
use memory::{Address, VirtualAddress};
use sync::mutex::Mutex;

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    // TODO: Read more on this trait and possibly make it more efficient.
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        Ok(ALLOCATOR
               .lock()
               .allocate_first_fit(layout.size(), layout.align()))
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.lock().free(ptr, layout.size(), layout.align());
    }
}

lazy_static! {
    /// The kernel heap allocator.
    static ref ALLOCATOR: Mutex<LinkedListAllocator> =
        Mutex::new(LinkedListAllocator::new(HEAP_START, HEAP_START + HEAP_MAX_SIZE));
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
