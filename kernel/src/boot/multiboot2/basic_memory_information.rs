//!Handles the basic memory information multiboot2 tag.

///Represents the basic memory information multiboo2 tag.
#[repr(C)]
struct BasicMemoryInformation { //type = 4
    tag_type: u32,
    size: u32,
    mem_lower: u32,
    mem_upper: u32
}

