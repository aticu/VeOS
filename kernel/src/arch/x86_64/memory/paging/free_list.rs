//! Handles the list of free physical memory areas.

use super::current_page_table::CURRENT_PAGE_TABLE;
use super::PAGE_SIZE;
use boot;
use memory::{MemoryArea, PhysicalAddress};
use sync::mutex::MutexGuard;
use sync::Mutex;

/// The list of free page frames.
///
/// # Safety
/// - The lock for this should never be acquired by an instance which currently
/// holds the page table lock.
pub static FREE_LIST: Mutex<FreeList> = Mutex::new(FreeList { first_entry: None });

/// An entry in the linked list of free memory locations.
#[derive(Clone, Copy)]
struct FreeListEntry {
    /// The length of this entry.
    length: usize,
    /// The start address of the next entry.
    next_entry: Option<PhysicalAddress>
}

impl FreeListEntry {
    /// Creates a new free list entry.
    fn new(length: usize, next_entry: Option<PhysicalAddress>) -> FreeListEntry {
        FreeListEntry { length, next_entry }
    }
}

/// Represents the list of free page frames.
pub struct FreeList {
    /// The first entry in the linked list.
    first_entry: Option<PhysicalAddress>
}

impl FreeList {
    /// Inserts a given entry into the list.
    ///
    /// # Safety
    /// - The memory that is being inserted into the free list should not be
    /// mapped anywhere
    /// (except for maybe the temporary map).
    pub unsafe fn insert(&mut self, mem_area: MemoryArea<PhysicalAddress>) {
        // REWRITEME: This whole method is too big and not readable.
        let mut current_page_table = CURRENT_PAGE_TABLE.lock();

        // Page align the length of this entry.
        let length = (mem_area.length() / PAGE_SIZE) * PAGE_SIZE;

        if self.first_entry.is_none() {
            // Adding the only entry in the list.
            let new_entry = FreeListEntry::new(length, None);
            current_page_table.write_at_physical(mem_area.start_address(), new_entry);

            self.first_entry = Some(mem_area.start_address());
        } else if self.first_entry.unwrap() > mem_area.start_address() {
            // Adding at the beginning of the list.

            let new_entry = if self.first_entry.unwrap() == mem_area.end_address() {
                let entry: FreeListEntry =
                    current_page_table.read_from_physical(self.first_entry.unwrap());
                FreeListEntry::new(entry.length + length, entry.next_entry)
            } else {
                FreeListEntry::new(length, self.first_entry)
            };
            current_page_table.write_at_physical(mem_area.start_address(), new_entry);

            self.first_entry = Some(mem_area.start_address());
        } else {
            // Adding at any position other than the beginning.

            let mut entry_address = self.first_entry.unwrap();
            let mut entry: FreeListEntry = current_page_table.read_from_physical(entry_address);

            while !entry.next_entry.is_none()
                && entry.next_entry.unwrap() < mem_area.start_address()
            {
                entry_address = entry.next_entry.unwrap();
                entry = current_page_table.read_from_physical(entry_address);
            }
            // "entry" is now the entry after which the new entry should be inserted.

            let next_entry = entry.next_entry;
            // Assert there is no overlap with previous and next entry.
            assert!(entry_address + entry.length <= mem_area.start_address());
            assert!(next_entry.is_none() || next_entry.unwrap() > mem_area.end_address() - 1);

            // The previous and the new entry need to be merged.
            let front_merge = entry_address + entry.length == mem_area.start_address();
            // The new and the following entry need to be merged.
            let back_merge = !next_entry.is_none() && mem_area.end_address() == next_entry.unwrap();

            if front_merge && back_merge {
                let next_entry =
                    current_page_table.read_from_physical::<FreeListEntry>(next_entry.unwrap());

                entry.next_entry = next_entry.next_entry;
                entry.length += mem_area.length() + next_entry.length;
            } else if front_merge {
                entry.length += mem_area.length();
            } else if back_merge {
                let next_entry =
                    current_page_table.read_from_physical::<FreeListEntry>(next_entry.unwrap());
                let new_entry =
                    FreeListEntry::new(length + next_entry.length, next_entry.next_entry);

                entry.next_entry = Some(mem_area.start_address());

                current_page_table.write_at_physical(mem_area.start_address(), new_entry);
            } else {
                let new_entry = FreeListEntry::new(length, next_entry);
                // Set the correct next pointer for the previous entry.
                entry.next_entry = Some(mem_area.start_address());

                current_page_table.write_at_physical(mem_area.start_address(), new_entry);
            }

            current_page_table.write_at_physical(entry_address, entry);
        }
    }

    /// Removes the given entry from the free list, if it exists.
    pub fn remove(&mut self, mem_area: MemoryArea<PhysicalAddress>) {
        let mut current_page_table = CURRENT_PAGE_TABLE.lock();

        let mut entry_address = self.first_entry.unwrap();
        let mut entry: FreeListEntry = current_page_table.read_from_physical(entry_address);

        if entry_address == mem_area.start_address() {
            // If the first entry is to be removed.
            self.first_entry = entry.next_entry;
        } else {
            while !entry.next_entry.is_none()
                && entry.next_entry.unwrap() < mem_area.start_address()
            {
                entry_address = entry.next_entry.unwrap();
                entry = current_page_table.read_from_physical(entry_address);
            }
            // Entry is now the entry after which the entry should be deleted.

            let next_entry = entry.next_entry;
            if !next_entry.is_none() && next_entry.unwrap() == mem_area.start_address() {
                let current_entry: FreeListEntry =
                    current_page_table.read_from_physical(mem_area.start_address());

                if current_entry.length == mem_area.length() {
                    // At this point we are sure that we found the correct entry.

                    entry.next_entry = current_entry.next_entry;
                    current_page_table.write_at_physical(entry_address, entry);
                } else {
                    panic!("Trying to remove a non-existing entry from the free list");
                }
            } else {
                panic!("Trying to remove a non-existing entry from the free list");
            }
        }
    }
}

/// An iterator for the chunks in the free list.
pub struct FreeListIterator<'a> {
    // The free list is kept here, to protect it from being changed while being read.
    /// The free list.
    #[allow(dead_code)]
    list: MutexGuard<'a, FreeList>,
    /// The next address in the free list.
    next_address: Option<PhysicalAddress>
}

impl<'a> FreeListIterator<'a> {
    /// Creates a new iterator over the free list.
    pub fn new() -> FreeListIterator<'a> {
        let list = FREE_LIST.lock();
        let next_address = list.first_entry;
        FreeListIterator { list, next_address }
    }

    /// Creates a new iterator over the already locked free list.
    pub fn from_guard(list: MutexGuard<'a, FreeList>) -> FreeListIterator<'a> {
        let next_address = list.first_entry;
        FreeListIterator { list, next_address }
    }

    pub fn finish(self) -> MutexGuard<'a, FreeList> {
        let FreeListIterator { list, .. } = self;
        list
    }
}

impl<'a> Iterator for FreeListIterator<'a> {
    type Item = MemoryArea<PhysicalAddress>;

    fn next(&mut self) -> Option<MemoryArea<PhysicalAddress>> {
        if !self.next_address.is_none() {
            let mut current_page_table = CURRENT_PAGE_TABLE.lock();
            let address = self.next_address.unwrap();
            let entry: FreeListEntry = current_page_table.read_from_physical(address);
            self.next_address = entry.next_entry;
            Some(MemoryArea::new(address, entry.length))
        } else {
            None
        }
    }
}

/// Initializes the list of free page frames.
pub fn init() {
    assert_has_not_been_called!("The free list should only be initialized once.");

    let mut free_list = FREE_LIST.lock();
    for entry in boot::get_memory_map() {
        unsafe { free_list.insert(entry) };
    }
}
