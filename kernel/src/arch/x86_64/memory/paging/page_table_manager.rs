//! Uses a trait that has general page table managing functions.

use super::{Page, PageFrame};
use super::frame_allocator::FRAME_ALLOCATOR;
use super::page_table::{Level1, Level2, Level4, PageTable};
use super::page_table_entry::{PRESENT, PageTableEntry, PageTableEntryFlags};
use core::ops::{Deref, DerefMut};
use memory::{PhysicalAddress, VirtualAddress};
use sync::PreemptionState;
use x86_64::instructions::tlb;

/// A reference to a locked level 1 page table.
pub struct Level1TableReference<'a> {
    /// The reference to the level 2 table that contains the level 1 table.
    table: &'a mut PageTable<Level2>,
    /// The address of the level 1 table.
    address: VirtualAddress,
    /// The preemption state of the lock.
    preemption_state: PreemptionState
}

impl<'a> Deref for Level1TableReference<'a> {
    type Target = PageTable<Level1>;

    fn deref(&self) -> &PageTable<Level1> {
        self.table.get_next_level(self.address).unwrap()
    }
}

impl<'a> DerefMut for Level1TableReference<'a> {
    fn deref_mut(&mut self) -> &mut PageTable<Level1> {
        self.table.get_next_level_mut(self.address).unwrap()
    }
}

impl<'a> Drop for Level1TableReference<'a> {
    fn drop(&mut self) {
        let table_index = PageTable::<Level2>::table_index(self.address);
        let mut l2_entry = &mut self.table[table_index];

        l2_entry.unlock(&self.preemption_state);
    }
}

/// A reference to a page table entry in a locked level 1 page table.
pub struct PageTableEntryReference<'a> {
    table_reference: Level1TableReference<'a>
}

impl<'a> Deref for PageTableEntryReference<'a> {
    type Target = PageTableEntry;

    fn deref(&self) -> &PageTableEntry {
        let index = PageTable::<Level1>::table_index(self.table_reference.address);
        &self.table_reference[index]
    }
}

impl<'a> DerefMut for PageTableEntryReference<'a> {
    fn deref_mut(&mut self) -> &mut PageTableEntry {
        let index = PageTable::<Level1>::table_index(self.table_reference.address);
        &mut self.table_reference[index]
    }
}

/// Structs managing a level 4 page table and it's decendants can implement
/// this to manage paging.
pub trait PageTableManager {
    /// Returns a mutable reference to the level 4 page table.
    fn get_l4(&mut self) -> &mut PageTable<Level4>;

    /// Returns the corresponding physical address to a virtual address.
    fn translate_address(&mut self, address: VirtualAddress) -> Option<PhysicalAddress> {
        self.get_l1(address)
            .and_then(|l1| l1[PageTable::<Level1>::table_index(address)].points_to())
            .map(|page_address| page_address + (address & 0xfff))
    }

    /// Returns a mutable reference to the level 1 table corresponding to the
    /// given address.
    fn get_l1(&mut self, address: VirtualAddress) -> Option<Level1TableReference> {
        assert!(valid_address!(address));

        let table_index = PageTable::<Level2>::table_index(address);
        let preemption_state = {
            let l4 = self.get_l4();

            let l2 = l4.get_next_level_mut(address)
                .and_then(|l3| l3.get_next_level_mut(address));

            match l2 {
                Some(table) => {
                    let l2_entry = &mut table[table_index];
                    if l2_entry.points_to().is_some() {
                        Some(l2_entry.lock())
                    } else {
                        None
                    }
                },
                None => None,
            }
        };

        match preemption_state {
            Some(preemption_state) => {
                Some(Level1TableReference {
                         table: self.get_l4()
                             .get_next_level_mut(address)
                             .and_then(|l3| l3.get_next_level_mut(address))
                             .unwrap(),
                         address,
                         preemption_state
                     })
            },
            None => None,
        }
    }

    /// Returns a reference to the level 1 table at the given address, possibly
    /// creating it.
    ///
    /// This creates new page tables if the parent tables for the wanted table
    /// are not already mapped.
    fn get_l1_and_map(&mut self, address: VirtualAddress) -> Level1TableReference {
        assert!(valid_address!(address));


        let table_index = PageTable::<Level2>::table_index(address);
        let preemption_state = {
            let l2 = self.get_l4()
                .next_level_and_map(address)
                .next_level_and_map(address);
            let l2_entry = &mut l2[table_index];
            l2_entry.lock()
        };

        // Make sure the next level is mapped.
        self.get_l4()
            .next_level_and_map(address)
            .next_level_and_map(address)
            .next_level_and_map(address);

        Level1TableReference {
            table: self.get_l4()
                .next_level_and_map(address)
                .next_level_and_map(address),
            address,
            preemption_state
        }
    }

    /// Returns a reference to the page table entry corresponding to the given
    /// address.
    ///
    /// This creates new page tables if the parent tables for the wanted table
    /// are not already mapped.
    fn get_entry_and_map(&mut self, address: VirtualAddress) -> PageTableEntryReference {
        let l1 = self.get_l1_and_map(address);
        PageTableEntryReference { table_reference: l1 }
    }

    /// Returns a mutable reference to the level 1 page table entry
    /// corresponding to the given address.
    fn get_entry(&mut self, address: VirtualAddress) -> Option<PageTableEntryReference> {
        let l1 = self.get_l1(address);
        l1.map(|l1| PageTableEntryReference { table_reference: l1 })
    }

    /// Maps the given page to the given frame with the given flags.
    fn map_page_at(&mut self, page: Page, frame: PageFrame, flags: PageTableEntryFlags) {
        if let Some(entry) = self.get_entry(page.get_address()) {
            debug_assert!(!entry.flags().contains(PRESENT),
                          "Trying to double map page {:x}",
                          page.get_address());
        }

        let target_address = page.get_address();
        let mut entry = self.get_entry_and_map(target_address);

        entry
            .set_address(frame.get_address())
            .set_flags(flags | PRESENT);
    }

    /// Maps the given page to an allocated frame with the given flags.
    fn map_page(&mut self, page: Page, flags: PageTableEntryFlags) {
        if let Some(entry) = self.get_entry(page.get_address()) {
            debug_assert!(!entry.flags().contains(PRESENT),
                          "Trying to double map page {:x}",
                          page.get_address());
        }

        let frame = FRAME_ALLOCATOR.allocate();

        self.map_page_at(page, frame, flags);
    }

    /// Changes the permissions of the page or map it, if it wasn't mapped.
    fn change_permissions_or_map(&mut self, page: Page, flags: PageTableEntryFlags) {
        let is_mapped = {
            if let Some(mut entry) = self.get_entry(page.get_address()) {
                entry.flags().contains(PRESENT)
            } else {
                false
            }
        };

        if is_mapped {
            self.get_entry(page.get_address())
                .unwrap()
                .set_flags(PRESENT | flags);
        } else {
            self.map_page(page, flags);
        }
    }

    /// Unmaps the given page.
    ///
    /// # Safety
    /// - Make sure the page isn't referenced anywhere anymore.
    unsafe fn unmap_page(&mut self, page: Page) {
        // TODO: Consider multiple CPUs.
        // TODO: Consider that the page may still be in use elsewhere (don't free the
        // frame then).
        let entry = self.get_entry(page.get_address());

        entry
            .expect("Trying to unmap a page that isn't mapped.")
            .unmap();
        tlb::flush(::x86_64::VirtualAddress(page.get_address()));
    }
}
