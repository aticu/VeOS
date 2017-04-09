//!Handles the module multiboot2 tag.

///Represents the module tag.
#[repr(C)]
struct Module { //type = 3
    tag_type: u32,
    size: u32,
    mod_start: usize, //verify this is really 64 bit
    mod_end: usize,
    string: [u8]
}

