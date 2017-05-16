use super::{Page, PageFrame};
use super::page_table::{Level1, Level2, Level3, Level4, PageTable};
use super::page_table_entry::*;
use super::super::{VIRTUAL_HIGH_MIN_ADDRESS, VIRTUAL_LOW_MAX_ADDRESS};
use memory::{PhysicalAddress, VirtualAddress};
use core::ptr;
use core::ptr::Unique;
use sync::cpu_relax;
use sync::Mutex;
use x86_64::instructions::tlb;

/// The address of the current Level 4 table.
///
/// Note that this is only valid if the level 4 table is mapped recursively on
/// the last entry.
const L4_TABLE: *mut PageTable<Level4> = 0xfffffffffffff000 as *mut PageTable<Level4>;

/// The base address for all temporary addresses.
const TEMPORARY_ADDRESS_BASE: usize = 0xffffffffffc00000;

/// Returns true for a valid virtual address.
macro_rules! valid_address {
    ($address: expr) => {{
        (VIRTUAL_LOW_MAX_ADDRESS >= $address || $address >= VIRTUAL_HIGH_MIN_ADDRESS)
    }};
}

/// The method to access the current page table.
pub static CURRENT_PAGE_TABLE: Mutex<CurrentPageTable> =
    Mutex::new(unsafe { CurrentPageTable::new() });

/// Owns the page table currently in use.
pub struct CurrentPageTable {
    l4_table: Unique<PageTable<Level4>>
}

impl CurrentPageTable {
    /// Returns the current page table.
    ///
    /// #Safety
    /// At any point in time there should only be exactly one current page
    /// table struct.
    const unsafe fn new() -> CurrentPageTable {
        CurrentPageTable { l4_table: Unique::new(L4_TABLE) }
    }

    /// Returns a reference to the level 4 page table.
    fn get_l4(&self) -> &PageTable<Level4> {
        unsafe { self.l4_table.get() }
    }

    /// Returns a mutable reference to the level 4 page table.
    fn get_l4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.l4_table.get_mut() }
    }

    /// Returns the corresponding physical address to a virtual address.
    #[allow(dead_code)]
    pub fn translate_address(&self, address: VirtualAddress) -> Option<PhysicalAddress> {
        self.get_l1(address)
            .and_then(|l1| l1[PageTable::<Level1>::table_index(address)].points_to())
            .map(|page_address| page_address + (address & 0xfff))
    }

    /// Returns a mutable reference to the temporary mapping page table.
    fn get_temporary_map_table(&mut self) -> &mut PageTable<Level1> {
        self.get_l1_mut(TEMPORARY_ADDRESS_BASE).expect("Temporary page map not mapped.")
    }

    /// Returns a reference to the level 1 table corresponding to the given
    /// address.
    fn get_l1(&self, address: VirtualAddress) -> Option<&PageTable<Level1>> {
        assert!(valid_address!(address));

        let l4 = self.get_l4();

        l4.get_next_level(PageTable::<Level4>::table_index(address))
            .and_then(|l3| l3.get_next_level(PageTable::<Level3>::table_index(address)))
            .and_then(|l2| l2.get_next_level(PageTable::<Level2>::table_index(address)))
    }

    /// Returns a mutable reference to the level 1 table corresponding to the
    /// given address.
    fn get_l1_mut(&mut self, address: VirtualAddress) -> Option<&mut PageTable<Level1>> {
        assert!(valid_address!(address));

        let l4 = self.get_l4_mut();

        l4.get_next_level_mut(PageTable::<Level4>::table_index(address))
            .and_then(|l3| l3.get_next_level_mut(PageTable::<Level3>::table_index(address)))
            .and_then(|l2| l2.get_next_level_mut(PageTable::<Level2>::table_index(address)))
    }

    /// Tries to map the given page frame into the address space temporarily.
    fn try_temporary_map(&mut self, frame: &PageFrame) -> Option<Page> {
        let index = page_frame_hash(frame);
        let mut temporary_map_table = self.get_temporary_map_table();
        let mut entry = &mut temporary_map_table[index];

        if !entry.flags().contains(TEMPORARY_TABLE_LOCK) {
            let virtual_address = TEMPORARY_ADDRESS_BASE + (index << 12);

            if entry.points_to() != Some(frame.get_address()) {
                tlb::flush(::x86_64::VirtualAddress(virtual_address));
                entry.set_address(frame.get_address());
                entry.set_flags(TEMPORARY_TABLE_LOCK | PRESENT | WRITABLE | DISABLE_CACHE | NO_EXECUTE);
            } else {
                entry.add_flags(TEMPORARY_TABLE_LOCK);
            }
            Some(Page::from_address(virtual_address))
        } else {
            None
        }
    }

    /// Maps the given page frame to a page temporarily.
    fn temporary_map(&mut self, frame: &PageFrame) -> Page {
        let mut map = self.try_temporary_map(frame);
        while map.is_none() {
            cpu_relax();
            map = self.try_temporary_map(frame);
        }
        map.unwrap()
    }

    /// Signals that the mapped page isn't used anymore.
    fn unmap_temporary_map(&mut self, frame: &PageFrame) {
        let index = page_frame_hash(frame);
        let mut temporary_map_table = self.get_temporary_map_table();
        let mut entry = &mut temporary_map_table[index];

        assert!(entry.flags().contains(TEMPORARY_TABLE_LOCK));

        entry.remove_flags(TEMPORARY_TABLE_LOCK);
    }

    /// Performs the given action with the mapped page.
    pub fn with_temporary_page<F>(&mut self, frame: &PageFrame, action: F)
        where F: Fn(&mut Page)
    {
        let mut map = self.temporary_map(frame);

        action(&mut map);

        self.unmap_temporary_map(frame);
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
        let frame = PageFrame::from_address(physical_address);
        let page = self.temporary_map(&frame);
        
        let virtual_address = page.get_address() | (physical_address & 0xfff);
        let data: T = unsafe { ptr::read(virtual_address as *mut T) };

        self.unmap_temporary_map(&frame);

        data
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
