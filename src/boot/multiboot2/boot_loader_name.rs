//!Handles the boot loader name tag in multiboot2.
use super::get_tag;

///Represents the tag of the boot loader name.
#[repr(C)]
struct BootLoaderName {
    //type = 2
    tag_type: u32,
    size: u32,
    string: usize,
}

///Returns the name of the boot loader.
pub fn get_bootloader_name() -> &'static str {
    let tag_address: *const BootLoaderName = get_tag(2).expect("Boot loader name required.") as
                                             *const BootLoaderName;
    let tag: &BootLoaderName = unsafe { &*tag_address };
    let string_address: usize = tag_address as usize + 8;
    from_c_str!(string_address, tag.size as usize - 9).expect("Bootloader name illegally formatted")
}
