#[repr(C)]
struct BasicMemoryInformation { //type = 4
    tag_type: u32,
    size: u32,
    mem_lower: u32,
    mem_upper: u32
}

