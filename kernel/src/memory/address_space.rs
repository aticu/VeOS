//! This module defines address spaces.

use super::address_space_manager::AddressSpaceManager;
use super::{PageFlags, PhysicalAddress, VirtualAddress};
use alloc::Vec;
use arch::{self, Architecture};
use core::mem::size_of_val;
use core::slice;
use memory::{MemoryArea, PAGE_SIZE, USER_ACCESSIBLE};
use multitasking::{Stack, ThreadID};

/// Represents an address space
pub struct AddressSpace {
    /// The segments that are part of the address space.
    segments: Vec<Segment>,
    /// The address space manager.
    manager: <arch::Current as Architecture>::AddressSpaceManager
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
            manager:
                <<arch::Current as Architecture>::AddressSpaceManager as AddressSpaceManager>::new()
        }
    }

    /// Creates a new address space for the idle threads.
    pub fn idle_address_space() -> AddressSpace {
        AddressSpace {
            segments: Vec::new(),
            manager:
                <<arch::Current as Architecture>::AddressSpaceManager as AddressSpaceManager>::idle(
                )
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

        if segment_to_add.flags.contains(USER_ACCESSIBLE)
            && !(arch::Current::is_userspace_address(segment_to_add.start_address())
                && arch::Current::is_userspace_address(segment_to_add.end_address()))
        {
            false
        } else {
            self.segments.push(segment_to_add);
            true
        }
    }

    /// Writes to the given address in the address space.
    pub fn write_to(&mut self, buffer: &[u8], address: VirtualAddress) {
        let area = MemoryArea::new(address, buffer.len());
        let segment_flags = { self.get_segment(area).map(|segment| segment.flags) };

        if let Some(segment_flags) = segment_flags {
            self.manager.write_to(buffer, address, segment_flags);
        } else {
            self.handle_out_of_segment(area);
        }
    }

    /// Zeros an already mapped area.
    pub fn zero_mapped_area(&mut self, area: MemoryArea<VirtualAddress>) {
        let segment_flags = { self.get_segment(area).map(|segment| segment.flags) };

        if let Some(segment_flags) = segment_flags {
            self.manager.zero(area, segment_flags);
        } else {
            self.handle_out_of_segment(area);
        }
    }

    /// Writes the given value to the given address in this address space.
    pub unsafe fn write_val<T>(&mut self, value: T, address: VirtualAddress) {
        let value_ptr = &value as *const T;
        let buffer = slice::from_raw_parts(value_ptr as *const u8, size_of_val(&value));
        self.write_to(buffer, address)
    }

    /// Returns the segment that contains the address with length bytes space
    /// after, if it exists.
    fn get_segment(&self, area: MemoryArea<VirtualAddress>) -> Option<&Segment> {
        for segment in &self.segments {
            if segment.contains_area(area) {
                return Some(segment);
            }
        }
        None
    }

    /// Handles the case of accesses outside of a segment.
    fn handle_out_of_segment(&self, area: MemoryArea<VirtualAddress>) {
        panic!("Out of segment access (area: {:?})", area);
    }

    /// Returns true if the given memory area is contained within a single
    /// segment.
    ///
    /// The range starts at `start` and is `length` bytes long.
    pub fn contains_area(&self, area: MemoryArea<VirtualAddress>) -> bool {
        let segment = self.get_segment(area);

        segment.is_some()
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
            self.get_segment(MemoryArea::new(page_address, 0))
                .map(|segment| segment.flags)
        };

        if let Some(segment_flags) = segment_flags {
            self.manager.map_page(page_address, segment_flags);
        } else {
            self.handle_out_of_segment(MemoryArea::new(page_address, 0));
        }
    }

    /// Unmaps the given page in the address space.
    ///
    /// # Safety
    /// - Nothing should reference the unmapped pages.
    pub unsafe fn unmap_page(&mut self, start_address: VirtualAddress) {
        self.manager.unmap_page(start_address);
    }

    /// Creates a new kernel stack.
    pub fn create_kernel_stack(&mut self, id: ThreadID) -> Stack {
        <<arch::Current as Architecture>::AddressSpaceManager as AddressSpaceManager>::create_kernel_stack(id, self)
    }

    /// Creates a new user stack.
    pub fn create_user_stack(&mut self, id: ThreadID) -> Stack {
        <<arch::Current as Architecture>::AddressSpaceManager as AddressSpaceManager>::create_user_stack(id, self)
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
#[derive(Debug)]
pub struct Segment {
    /// The memory area of the segment.
    memory_area: MemoryArea<VirtualAddress>,
    /// The flags this segment is mapped with.
    flags: PageFlags,
    /// The type of the segment.
    segment_type: SegmentType
}

impl Segment {
    /// Creates a new segment with the given parameters.
    pub fn new(
        memory_area: MemoryArea<VirtualAddress>,
        flags: PageFlags,
        segment_type: SegmentType
    ) -> Segment {
        Segment {
            memory_area,
            flags,
            segment_type
        }
    }

    /// Returns true if the intersection of the segments is not empty.
    fn overlaps(&self, other: &Segment) -> bool {
        self.memory_area.overlaps_with(other.memory_area)
    }

    /// Checks whether this segment contains the given memory area.
    fn contains_area(&self, area: MemoryArea<VirtualAddress>) -> bool {
        area.is_contained_in(self.memory_area)
    }

    /// Returns the start address of this segment.
    fn start_address(&self) -> VirtualAddress {
        self.memory_area.start_address()
    }

    /// Returns the end address of this segment.
    fn end_address(&self) -> VirtualAddress {
        self.memory_area.end_address()
    }

    /// Unmaps this segment.
    fn unmap(&self, manager: &mut <arch::Current as Architecture>::AddressSpaceManager) {
        let pages_in_segment = (self.memory_area.length() - 1) / PAGE_SIZE + 1;
        for page_num in 0..pages_in_segment {
            unsafe {
                match self.segment_type {
                    SegmentType::FromFile => {
                        manager.unmap_page(self.start_address() + page_num * PAGE_SIZE)
                    },
                    SegmentType::MemoryOnly => {
                        manager.unmap_page_unchecked(self.start_address() + page_num * PAGE_SIZE)
                    },
                }
            }
        }
    }
}
