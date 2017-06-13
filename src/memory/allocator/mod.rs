//! Provides the heap allocator for the kernel.

mod linked_list_allocator;

use self::linked_list_allocator::LinkedListAllocator;
use arch::memory::{HEAP_MAX_SIZE, HEAP_START};
use core::{cmp, ptr};
use memory::VirtualAddress;
use sync::mutex::Mutex;

lazy_static! {
    /// The kernel heap allocator.
    static ref ALLOCATOR: Mutex<LinkedListAllocator> =
        Mutex::new(LinkedListAllocator::new(HEAP_START, HEAP_START + HEAP_MAX_SIZE));
}

/// Aligns the given address to the given alignment.
///
/// The alignment must be a power of two.
fn align(address: VirtualAddress, alignment: usize) -> VirtualAddress {
    if address % alignment == 0 {
        address
    } else {
        let alignment_bitmask = !(alignment - 1);
        (address & alignment_bitmask) + alignment
    }
}

/// Allocates size bytes aligned with align, returning the start address.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    ALLOCATOR.lock().allocate_first_fit(size, align)
}

/// Allocates and zeroes size bytes aligned with align, returning the start
/// address.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_allocate_zeroed(size: usize, align: usize) -> *mut u8 {
    let ptr = __rust_allocate(size, align);
    unsafe { ptr::write_bytes(ptr, 0, size) };
    ptr
}

/// Returns the usable size of an allocation with the given size and alignment.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}

/// Deallocates the previously allocated memory ptr points to, using the same
/// parameters used for allocation.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {
    ALLOCATOR.lock().free(ptr, size, align);
}

/// Reallocates the memory pointed to by ptr, changing it's size.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8,
                                    size: usize,
                                    new_size: usize,
                                    align: usize)
                                    -> *mut u8 {
    let new_ptr = __rust_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    __rust_deallocate(ptr, size, align);
    new_ptr
}

/// Attempts to change the size of the given allocation in place. Returns the
/// new usable size.
#[allow(dead_code)]
#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(_: *mut u8,
                                            size: usize,
                                            _: usize,
                                            align: usize)
                                            -> usize {
    __rust_usable_size(size, align)
}
