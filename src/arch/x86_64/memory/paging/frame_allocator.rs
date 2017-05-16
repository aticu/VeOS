//! Handles the allocation of physical page frames.

use memory::FreeMemoryArea;
use super::free_list::{FREE_LIST, FreeListIterator};
use super::{PageFrame, PAGE_SIZE};

/// Used to allocate page frames.
pub struct FrameAllocator {}

/// The frame allocator of the kernel.
pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {};

impl FrameAllocator {
    /// Allocates a page frame.
    pub fn allocate(&self) -> Option<PageFrame> {
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

            Some(page_frame)
        } else {
            None
        }
    }

    /// Deallocates the page frame.
    ///
    /// # Safety
    /// - Must not be called on used page frames.
    pub unsafe fn deallocate(&self, frame: PageFrame) {
        FREE_LIST.lock().insert(FreeMemoryArea::new(frame.get_address(), PAGE_SIZE));
    }
}
