//!Handles the memory map multiboot2 tag.
use super::get_tag;

///Represents the memory map tag.
#[repr(C)]
struct MemoryMap { //type = 6
    tag_type: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    entries: usize
}

///Represents an entry of version 0 in the memory map.
#[repr(C)]
struct MemoryMapEntryVersion0 {
    base_addr: u64,
    length: u64,
    memory_type: u32,
    reserved: u32
}

///Represents an iterator for the memory map tags.
struct MemoryMapEntryVersion0Iterator {
    memory_map: &'static MemoryMap,
    current_address: usize
}

impl MemoryMapEntryVersion0Iterator {
    ///Creates a new iterator for the memory map tags.
    fn new(address: usize) -> MemoryMapEntryVersion0Iterator {
        let memory_map = unsafe { &*(address as *const MemoryMap) };
        MemoryMapEntryVersion0Iterator {
            memory_map: memory_map,
            current_address: address + 16
        }
    }
}

impl Iterator for MemoryMapEntryVersion0Iterator {
    type Item = &'static MemoryMapEntryVersion0;

    fn next(&mut self) -> Option<&'static MemoryMapEntryVersion0> {
        if self.current_address < (self.memory_map as *const MemoryMap as usize) + self.memory_map.size as usize {
            let entry = unsafe { &*(self.current_address as *const MemoryMapEntryVersion0) };
            self.current_address += self.memory_map.entry_size as usize;
            Some(entry)
        }
        else {
            None
        }
    }
}
