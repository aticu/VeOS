//! Handles the allocation of physical page frames.

use super::{PAGE_SIZE, PageFrame};
use super::free_list::{FREE_LIST, FreeListIterator};
use core::cell::Cell;
use memory::FreeMemoryArea;

/// Used to allocate page frames.
pub struct FrameAllocator {
    free_frames: Cell<usize>
}

// It is save to implement sync, because access is restricted by the lock on
// the free list. Should this change, this needs to be removed.
unsafe impl Sync for FrameAllocator {}

/// The page frame allocator of the kernel.
lazy_static! {
    pub static ref FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {
        free_frames: {
            let mut number = 0;

            for entry in FreeListIterator::new() {
                number += entry.length / PAGE_SIZE;
            }

            Cell::new(number)
        }
    };
}


impl FrameAllocator {
    /// Allocates a page frame.
    pub fn allocate(&self) -> PageFrame {
        // NOTE: The lock on the list also locks the allocator, should the inner
        // workings of the allocator be changed, then there will also need to be a
        // locking mechanism.
        let list = FREE_LIST.lock();
        let mut iterator = FreeListIterator::from_guard(list);

        let free_area = iterator.next();
        let mut list = iterator.finish();

        if !free_area.is_none() {
            let free_area = free_area.unwrap();
            let page_frame = PageFrame::from_address(free_area.start_address);

            let new_free_area = free_area.without_first_frame();

            list.remove(free_area);
            unsafe {
                if new_free_area.length > 0 {
                    list.insert(new_free_area);
                }
            }
            self.free_frames.set(self.free_frames.get() + 1);

            page_frame
        } else {
            panic!("Out of memory!");
            // TODO: Free up some memory here.
        }
    }

    /// Deallocates the page frame.
    ///
    /// # Safety
    /// - Must not be called on used page frames.
    pub unsafe fn deallocate(&self, frame: PageFrame) {
        // NOTE: The lock on the list also locks the allocator, should the inner
        // workings of the allocator be changed, then there will also need to be a
        // locking mechanism.
        let mut list = FREE_LIST.lock();
        self.free_frames.set(self.free_frames.get() + 1);
        list.insert(FreeMemoryArea::new(frame.get_address(), PAGE_SIZE));
    }
}
