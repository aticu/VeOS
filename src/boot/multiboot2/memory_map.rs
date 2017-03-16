#[repr(C)]
struct MemoryMap { //type = 6
    tag_type: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    entries: [MemoryMapEntryVersion0]
}

#[repr(C)]
struct MemoryMapEntryVersion0 {
    base_addr: u64,
    length: u64,
    memory_type: u32,
    reserved: u32
}
