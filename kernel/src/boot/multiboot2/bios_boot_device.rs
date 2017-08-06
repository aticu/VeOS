//!Handles the bios boot device multiboot2 tag.

///Represents the bios boot device tag.
#[repr(C)]
struct BiosBootDevice { //type = 5
    tag_type: u32,
    size: u32,
    biosdev: u32,
    partition: u32,
    sub_partition: u32
}

