//! Handles the multiboot2 information structure.
mod framebuffer_info;
mod boot_loader_name;

pub use self::boot_loader_name::get_bootloader_name;
pub use self::framebuffer_info::get_vga_info;

/// Represents a tag in the information structure.
#[repr(C)]
struct BasicTag {
    tag_type: u32,
    size: u32
}

/// Represents an iterator for the tags.
struct BasicTagIterator {
    current_address: usize
}

impl BasicTagIterator {
    /// Returns a new iterator for the tags.
    fn new() -> BasicTagIterator {
        unsafe { BasicTagIterator { current_address: STRUCT_BASE_ADDRESS + 8 } }
    }
}

/// The base address for the information structure.
// this will only be valid after init was called and will never be changed
// afterwards
static mut STRUCT_BASE_ADDRESS: usize = 0;

impl Iterator for BasicTagIterator {
    type Item = *const BasicTag;

    fn next(&mut self) -> Option<*const BasicTag> {
        let current_tag = unsafe { &*(self.current_address as *const BasicTag) };
        if current_tag.tag_type == 0 && current_tag.size == 8 {
            None
        } else {
            let last_address = self.current_address;
            self.current_address += current_tag.size as usize;
            self.current_address += if self.current_address % 8 == 0 {
                0
            } else {
                8 - (self.current_address % 8)
            };
            Some(last_address as *const BasicTag)
        }
    }
}

/// Initializes the multiboot2 module.
pub fn init(information_structure_address: usize) {
    assert_has_not_been_called!("The multiboot2 module should only be initialized once.");

    assert!(check_validity(information_structure_address));
    unsafe { STRUCT_BASE_ADDRESS = information_structure_address };
}

/// Checks if the passed information structure is valid.
fn check_validity(information_structure_address: usize) -> bool {
    let total_size: u32 = unsafe { *(information_structure_address as *const u32) };
    let end_tag_type: u32 =
        unsafe { *((information_structure_address + total_size as usize - 8) as *const u32) };
    let end_tag_size: u32 =
        unsafe { *((information_structure_address + total_size as usize - 4) as *const u32) };
    end_tag_type == 0 && end_tag_size == 8
}

/// Returns the tag that corresponds to the given number.
fn get_tag(tag_type: u32) -> Option<*const BasicTag> {
    unsafe { BasicTagIterator::new().find(|tag| (**tag).tag_type == tag_type) }
}
