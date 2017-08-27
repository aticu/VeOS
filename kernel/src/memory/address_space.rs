//! This module defines address spaces.

use super::{PageFlags, PhysicalAddress, VirtualAddress};
use alloc::Vec;
use alloc::boxed::Box;
use arch::{idle_address_space_manager, new_address_space_manager};
use core::mem::size_of_val;
use core::slice;
use memory::{PAGE_SIZE, is_userspace_address, USER_ACCESSIBLE};

/// Represents an address space
pub struct AddressSpace {
    /// The segments that are part of the address space.
    segments: Vec<Segment>,
    /// The address space manager.
    manager: Box<AddressSpaceManager>
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        for segment in &mut self.segments {
            segment.unmap(&mut self.manager);
        }
    }
}

impl AddressSpace {
    /// Creates a new address space.
    pub fn new() -> AddressSpace {
        AddressSpace {
            segments: Vec::new(),
            manager: new_address_space_manager()
        }
    }

    /// Creates a new address space for the idle threads.
    pub fn idle_address_space() -> AddressSpace {
        AddressSpace {
            segments: Vec::new(),
            manager: idle_address_space_manager()
        }
    }

    /// Adds the segment to the address space.
    ///
    /// Returns true if the segment was successfully added.
    pub fn add_segment(&mut self, segment_to_add: Segment) -> bool {
        for segment in &self.segments {
            if segment_to_add.overlaps(segment) {
                return false;
            }
        }

        if segment_to_add.flags.contains(USER_ACCESSIBLE) && !(is_userspace_address(segment_to_add.start) && is_userspace_address(segment_to_add.end())) {
            false
        } else {
            self.segments.push(segment_to_add);
            true
        }
    }

    /// Writes to the given address in the address space.
    pub fn write_to(&mut self, buffer: &[u8], address: VirtualAddress) {
        let segment_flags = {
            let segment = self.get_segment_from_address(address);

            if let Some(segment) = segment {
                assert!(segment.contains(address + buffer.len() - 1));

                segment.flags
            } else {
                panic!("Write outside of segment. TODO: Handle this better.");
            }
        };

        self.manager.write_to(buffer, address, segment_flags);
    }

    /// Writes the given value to the given address in this address space.
    pub unsafe fn write_val<T>(&mut self, value: T, address: VirtualAddress) {
        let value_ptr = &value as *const T;
        let buffer = slice::from_raw_parts(value_ptr as *const u8, size_of_val(&value));
        self.write_to(buffer, address)
    }

    /// Returns the segment that contains the address, if it exists.
    fn get_segment_from_address(&self, address: VirtualAddress) -> Option<&Segment> {
        for segment in &self.segments {
            if segment.contains(address) {
                return Some(segment);
            }
        }
        None
    }

    /// Returns true if the given memory area is contained within a single segment.
    ///
    /// The range starts at `start` and is `length` bytes long.
    pub fn contains_range(&self, start: VirtualAddress, length: usize) -> bool {
        let segment = self.get_segment_from_address(start);

        if let Some(segment) = segment {
            segment.contains(start) && segment.contains(start + length)
        } else {
            false
        }
    }

    /// Returns the address of the page table.
    ///
    /// # Safety
    /// - Should only be called by architecture specific code.
    pub unsafe fn get_page_table_address(&self) -> PhysicalAddress {
        self.manager.get_page_table_address()
    }

    /// Maps the given page in the address space.
    pub fn map_page(&mut self, page_address: VirtualAddress) {
        let segment_flags = {
            let segment = self.get_segment_from_address(page_address);

            if let Some(segment) = segment {
                segment.flags
            } else {
                panic!("Write outside of segment. TODO: Handle this better.");
            }
        };
        self.manager.map_page(page_address, segment_flags);
    }

    /// Unmaps the given page in the address space.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    pub unsafe fn unmap_page(&mut self, start_address: VirtualAddress) {
        self.manager.unmap_page(start_address);
    }
}

/// All types of segments that are possible.
#[derive(Debug)]
pub enum SegmentType {
    /// The content of the segment was read from a file.
    FromFile,
    /// The content of the segment is only in memory.
    MemoryOnly
}

/// Represents a segment of memory in the address space.
pub struct Segment {
    /// The start address of the segment.
    start: VirtualAddress,
    /// The length of the segment.
    length: usize,
    /// The flags this segment is mapped with.
    flags: PageFlags,
    /// The type of the segment.
    segment_type: SegmentType
}

impl Segment {
    /// Creates a new segment with the given parameters.
    pub fn new(start: VirtualAddress, length: usize, flags: PageFlags, segment_type: SegmentType) -> Segment {
        Segment {
            start,
            length,
            flags,
            segment_type
        }
    }

    /// Checks if the address is contained within the segment.
    fn contains(&self, address: VirtualAddress) -> bool {
        self.start <= address && address < self.end()
    }

    /// Returns true if the intersection of the segments is not empty.
    fn overlaps(&self, other: &Segment) -> bool {
        self.contains(other.start) || other.contains(self.start)
    }
    
    /// Returns the end address (exclusive) of the segment.
    fn end(&self) -> VirtualAddress {
        self.start.saturating_add(self.length)
    }

    /// Unmaps this segment.
    fn unmap(&self, manager: &mut Box<AddressSpaceManager>) {
        let pages_in_segment = (self.length - 1) / PAGE_SIZE + 1;
        for page_num in 0..pages_in_segment {
            unsafe {
                match self.segment_type {
                    SegmentType::FromFile => manager.unmap_page(self.start + page_num * PAGE_SIZE),
                    SegmentType::MemoryOnly => manager.unmap_page_unchecked(self.start + page_num * PAGE_SIZE)
                }
            }
        }
    }
}

/// This trait should be implemented by any architecture specific address space
/// manager.
pub trait AddressSpaceManager: Send {
    /// Writes the data in `buffer` to the `address` in the target address
    /// space setting the given flags.
    fn write_to(&mut self, buffer: &[u8], address: VirtualAddress, flags: PageFlags);

    /// Returns the address of the page table.
    ///
    /// # Safety
    /// - Should only be used by architecture specific code.
    unsafe fn get_page_table_address(&self) -> VirtualAddress;

    /// Maps the given page in the managed address space.
    fn map_page(&mut self, page_address: VirtualAddress, flags: PageFlags);

    /// Unmaps the given page in the managed address space.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    unsafe fn unmap_page(&mut self, start_address: VirtualAddress);

    /// Unmaps the given page in the managed address space not checking if it was mapped.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    unsafe fn unmap_page_unchecked(&mut self, start_address: VirtualAddress);
}
