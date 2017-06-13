//! Handles interactions with the current page table.

use super::{Page, PageFrame};
use super::inactive_page_table::InactivePageTable;
use super::page_table::{Level1, Level4, PageTable};
use super::page_table_entry::*;
use super::page_table_manager::PageTableManager;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::Unique;
use memory::PhysicalAddress;
use sync::{Mutex, PreemptionState};
use x86_64::instructions::tlb;
use x86_64::registers::control_regs;

/// The address of the current Level 4 table.
///
/// Note that this is only valid if the level 4 table is mapped recursively on
/// the last entry.
const L4_TABLE: *mut PageTable<Level4> = 0xfffffffffffff000 as *mut PageTable<Level4>;

/// The base address for all temporary addresses.
const TEMPORARY_ADDRESS_BASE: usize = 0xffffffffffc00000;

/// The method to access the current page table.
pub static CURRENT_PAGE_TABLE: CurrentPageTableLock =
    unsafe { CurrentPageTableLock::new(CurrentPageTable::new()) };

/// Protects the current page table from being accessed directly.
///
/// This serves to stop the page table from being switched while being accessed.
pub struct CurrentPageTableLock {
    current_page_table: UnsafeCell<CurrentPageTable>,
    reference_count: Mutex<usize>
}

// This is safe because the page table will manage it's own exclusion
// internally.
unsafe impl Sync for CurrentPageTableLock {}

impl CurrentPageTableLock {
    /// Creates a new current page table lock.
    ///
    /// # Safety
    /// This should only ever get called once at compile time.
    const unsafe fn new(table: CurrentPageTable) -> CurrentPageTableLock {
        CurrentPageTableLock {
            current_page_table: UnsafeCell::new(table),
            reference_count: Mutex::new(0)
        }
    }

    /// Locks the current page table.
    pub fn lock(&self) -> CurrentPageTableReference {
        let mut rc: &mut usize = &mut self.reference_count.lock();
        *rc += 1;
        CurrentPageTableReference {
            current_page_table: unsafe { &mut *self.current_page_table.get() },
            reference_count: &self.reference_count
        }
    }
}

/// Serves as a reference to a locked current page table.
pub struct CurrentPageTableReference<'a> {
    current_page_table: &'a mut CurrentPageTable,
    reference_count: &'a Mutex<usize>
}

impl<'a> Drop for CurrentPageTableReference<'a> {
    fn drop(&mut self) {
        let mut rc: &mut usize = &mut self.reference_count.lock();
        *rc -= 1;
    }
}

impl<'a> Deref for CurrentPageTableReference<'a> {
    type Target = CurrentPageTable;

    fn deref(&self) -> &CurrentPageTable {
        self.current_page_table
    }
}

impl<'a> DerefMut for CurrentPageTableReference<'a> {
    fn deref_mut(&mut self) -> &mut CurrentPageTable {
        self.current_page_table
    }
}

/// Owns the page table currently in use.
pub struct CurrentPageTable {
    l4_table: Unique<PageTable<Level4>>
}

impl PageTableManager for CurrentPageTable {
    fn get_l4(&self) -> &PageTable<Level4> {
        unsafe { self.l4_table.as_ref() }
    }

    fn get_l4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.l4_table.as_mut() }
    }
}

impl CurrentPageTable {
    /// Returns the current page table.
    ///
    /// # Safety
    /// - At any point in time there should only be exactly one current page
    /// table struct.
    const unsafe fn new() -> CurrentPageTable {
        CurrentPageTable { l4_table: Unique::new(L4_TABLE) }
    }

    /// Tries to map an inactive page table.
    ///
    /// Returns true if the mapping was successful.
    ///
    /// # Safety
    /// - Should not be called while another inactive table is mapped.
    pub unsafe fn map_inactive(&mut self, frame: &PageFrame) -> PreemptionState {
        let mut l4 = self.get_l4_mut();
        let mut entry = &mut l4[509];
        let preemption_state = entry.lock();
        if !entry.flags().contains(PRESENT) {
            entry
                .set_flags(PRESENT | WRITABLE | NO_EXECUTE)
                .set_address(frame.get_address());
        }

        preemption_state
    }

    /// Unmaps the currently mapped inactive page table.
    pub fn unmap_inactive(&mut self, preemption_state: &PreemptionState) {
        let mut l4 = self.get_l4_mut();
        let mut entry = &mut l4[509];
        debug_assert!(entry.flags().contains(PRESENT));
        entry.remove_flags(PRESENT);
        entry.unlock(&preemption_state);
    }

    /// Returns a mutable reference to the temporary mapping page table.
    fn get_temporary_map_table(&mut self) -> &mut PageTable<Level1> {
        let l4 = self.get_l4_mut();

        l4.get_next_level_mut(TEMPORARY_ADDRESS_BASE)
            .and_then(|l3| l3.get_next_level_mut(TEMPORARY_ADDRESS_BASE))
            .and_then(|l2| l2.get_next_level_mut(TEMPORARY_ADDRESS_BASE))
            .expect("Temporary page table not mapped.")
    }

    /// Performs the given action with the mapped page.
    pub fn with_temporary_page<F, T>(&mut self, frame: &PageFrame, action: F) -> T
        where F: Fn(&mut Page) -> T
    {
        // Map the page.
        let index = page_frame_hash(frame);
        let mut temporary_map_table = self.get_temporary_map_table();
        let mut entry = &mut temporary_map_table[index];
        let preemption_state = entry.lock();

        let virtual_address = TEMPORARY_ADDRESS_BASE + (index << 12);

        if entry.points_to() != Some(frame.get_address()) {
            tlb::flush(::x86_64::VirtualAddress(virtual_address));
            entry.set_address(frame.get_address());
            entry.set_flags(PRESENT | WRITABLE | DISABLE_CACHE | NO_EXECUTE);
        }

        // Perform the action.
        let result: T = action(&mut Page::from_address(virtual_address));

        // Unlock this entry.
        entry.unlock(&preemption_state);

        result
    }

    /// Writes the given value to the given physical address.
    pub fn write_at_physical<T: Sized + Copy>(&mut self,
                                              physical_address: PhysicalAddress,
                                              data: T) {
        self.with_temporary_page(&PageFrame::from_address(physical_address), |page| {
            let virtual_address = page.get_address() | (physical_address & 0xfff);

            unsafe {
                ptr::write(virtual_address as *mut T, data);
            }
        });
    }

    /// Reads from the given physical address.
    pub fn read_from_physical<T: Sized + Copy>(&mut self, physical_address: PhysicalAddress) -> T {
        self.with_temporary_page(&PageFrame::from_address(physical_address), |page| {
            let virtual_address = page.get_address() | (physical_address & 0xfff);

            unsafe { ptr::read(virtual_address as *mut T) }
        })
    }

    /// Switches to the new page table returning the current one.
    ///
    /// The old page table will not be mapped into the new one. This should be
    /// done manually.
    pub unsafe fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_frame = PageFrame::from_address(control_regs::cr3().0 as PhysicalAddress);
        let old_table = InactivePageTable::from_frame(old_frame.copy(), &new_table);

        let new_frame = new_table.get_frame();

        drop(new_table);

        // Make the switch.
        control_regs::cr3_write(::x86_64::PhysicalAddress(new_frame.get_address() as u64));

        // Map the now inactive old table.
        self.map_inactive(&old_frame);

        old_table
    }
}

/// Hashes page frames to values from 0 to 511.
///
/// This serves to speed up temporary mapping of page frames,
/// by better utilizing the available space.
fn page_frame_hash(frame: &PageFrame) -> usize {
    let mut address = frame.get_address() >> 12;
    address *= 101489;
    address % 512
}
