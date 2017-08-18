//! Handles the managment of an inactive page table.

use super::PageFrame;
use super::current_page_table::CURRENT_PAGE_TABLE;
use super::frame_allocator::FRAME_ALLOCATOR;
use super::page_table::{Level4, PageTable};
use super::page_table_entry::*;
use super::page_table_manager::PageTableManager;
use super::super::TEMPORARY_MAP_TABLE;
use core::ptr::Unique;
use sync::PreemptionState;
use x86_64::registers::control_regs::cr3;

/// The reference to the place where the level 4 table will be mapped.
const L4_TABLE: *mut PageTable<Level4> = 0xffffffffffffd000 as *mut PageTable<Level4>;

/// Represents a currently inactive page table that needs to be modified.
pub struct InactivePageTable {
    /// A reference to the level 4 table.
    l4_table: Unique<PageTable<Level4>>,
    /// The page frame of the level 4 table.
    l4_frame: PageFrame,
    /// Optionally contains the preemption state of the mapped entry in the
    /// current page table.
    preemption_state: Option<PreemptionState>
}

impl PageTableManager for InactivePageTable {
    fn get_l4(&mut self) -> &mut PageTable<Level4> {
        unsafe {
            self.map();
            self.l4_table.as_mut()
        }
    }
}

impl Drop for InactivePageTable {
    fn drop(&mut self) {
        match self.preemption_state {
            Some(_) => self.unmap(),
            None => (),
        }
    }
}

impl InactivePageTable {
    /// Creates a new inactive page table.
    ///
    /// # Safety
    /// - Should only be called during kernel setup.
    pub unsafe fn new() -> InactivePageTable {
        let frame = FRAME_ALLOCATOR.allocate();
        let preemption_state = CURRENT_PAGE_TABLE.lock().map_inactive(&frame);

        // Zero the page.
        let mut table = &mut *L4_TABLE;
        table.zero();

        // Set up some invariants.
        table[510]
            .set_address(TEMPORARY_MAP_TABLE)
            .set_flags(PRESENT | WRITABLE | NO_EXECUTE);
        table[511]
            .set_address(frame.get_address())
            .set_flags(PRESENT | WRITABLE | NO_EXECUTE);

        InactivePageTable {
            l4_table: Unique::new_unchecked(L4_TABLE),
            l4_frame: frame,
            preemption_state: Some(preemption_state)
        }
    }

    /// Creates a copy of the current page table kernel part as an inactive
    /// page table.
    pub fn copy_from_current() -> InactivePageTable {
        let frame = FRAME_ALLOCATOR.allocate();
        let preemption_state = unsafe { CURRENT_PAGE_TABLE.lock().map_inactive(&frame) };

        let mut table = unsafe { &mut *L4_TABLE };
        table.zero();

        table[256] = CURRENT_PAGE_TABLE.lock().get_l4()[256].clone();
        table[257] = CURRENT_PAGE_TABLE.lock().get_l4()[257].clone();
        table[506] = CURRENT_PAGE_TABLE.lock().get_l4()[506].clone();
        table[507] = CURRENT_PAGE_TABLE.lock().get_l4()[507].clone();

        unsafe {
            table[510]
                .set_address(TEMPORARY_MAP_TABLE)
                .set_flags(PRESENT | WRITABLE | NO_EXECUTE);
        }
        table[511]
            .set_address(frame.get_address())
            .set_flags(PRESENT | WRITABLE | NO_EXECUTE);

        CURRENT_PAGE_TABLE
            .lock()
            .unmap_inactive(&preemption_state);

        InactivePageTable {
            l4_table: unsafe { Unique::new_unchecked(L4_TABLE) },
            l4_frame: frame,
            preemption_state: None
        }
    }

    /// Creates an inactive page table at the given address.
    ///
    /// The old_table parameter points to a table containing the preemption
    /// state.
    pub fn from_frame(frame: PageFrame, old_table: &InactivePageTable) -> InactivePageTable {
        InactivePageTable {
            l4_table: unsafe { Unique::new_unchecked(L4_TABLE) },
            l4_frame: frame,
            preemption_state: Some(unsafe {
                                       old_table
                                           .preemption_state
                                           .as_ref()
                                           .expect("The old table was not mapped.")
                                           .copy()
                                   })
        }
    }

    /// Creates an inactive page table that points to the current page table.
    pub fn from_current_table() -> InactivePageTable {
        InactivePageTable {
            l4_table: unsafe { Unique::new_unchecked(L4_TABLE) },
            l4_frame: PageFrame::from_address(cr3().0 as usize),
            preemption_state: None
        }
    }

    /// Maps the inactive page table to the current one.
    ///
    /// # Safety
    /// - Ensure that it is properly unmapped every time it's mapped.
    unsafe fn map(&mut self) {
        if self.preemption_state.is_none() {
            let preemption_state = CURRENT_PAGE_TABLE.lock().map_inactive(&self.l4_frame);
            self.preemption_state = Some(preemption_state);
        }
    }

    /// Returns the page frame of this page table.
    pub fn get_frame(&self) -> PageFrame {
        unsafe { self.l4_frame.copy() }
    }

    /// Unmaps the currently loaded inactive page table.
    pub fn unmap(&mut self) {
        if !self.preemption_state.is_none() {
            CURRENT_PAGE_TABLE
                .lock()
                .unmap_inactive(self.preemption_state.as_ref().unwrap());
            self.preemption_state = None;
        }
    }
}
