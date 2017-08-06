//!Handles the apm table multiboot2 tag.

///Represents the apm table tag.
#[repr(C)]
struct ApmTable { //type = 10
    tag_type: u32,
    size: u32,
    version: u16,
    cseg: u16,
    offset: u32,
    cseg_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16
}

