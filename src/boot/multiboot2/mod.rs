mod framebuffer_info;

use super::super::vga_buffer;

#[repr(C)]
struct BasicTag {
    tag_type: u32,
    size: u32
}

struct BasicTagIterator {
    current_address: usize
}

impl BasicTagIterator {
    fn new() -> BasicTagIterator {
        unsafe { BasicTagIterator { current_address: STRUCT_BASE_ADDRESS + 8 } }
    }
}

static mut STRUCT_BASE_ADDRESS: usize = 0; //this will only be valid after init was called and will never be changed afterwards

impl Iterator for BasicTagIterator {
    type Item = *const BasicTag;

    fn next(&mut self) -> Option<*const BasicTag> {
        let current_tag = unsafe { &*(self.current_address as *const BasicTag) };
        if current_tag.tag_type == 0 && current_tag.size == 8 {
            None
        }
        else {
            let last_address = self.current_address;
            self.current_address += current_tag.size as usize;
            self.current_address += if self.current_address % 8 == 0 {0} else {8 - (self.current_address % 8)};
            Some(last_address as *const BasicTag)
        }
    }
}

pub fn init(information_structure_address: usize) {
    assert!(check_validity(information_structure_address));
    unsafe { STRUCT_BASE_ADDRESS = information_structure_address };
}

fn check_validity(information_structure_address: usize) -> bool {
    let total_size: u32 = unsafe { *(information_structure_address as *const u32) };
    let end_tag_type: u32 = unsafe { *((information_structure_address + total_size as usize - 8) as *const u32) };
    let end_tag_size: u32 = unsafe { *((information_structure_address + total_size as usize - 4) as *const u32) };
    end_tag_type == 0 && end_tag_size == 8
}

fn get_tag(tag_type: u32) -> *const BasicTag {
    unsafe { BasicTagIterator::new().find(|tag| (**tag).tag_type == tag_type).expect("Tag type not found") }
}

pub fn get_vga_info() -> vga_buffer::Info {
    let framebuffer_tag = unsafe { &*(get_tag(8) as *const framebuffer_info::FramebufferInfo) };
    vga_buffer::Info {
        height: framebuffer_tag.framebuffer_height as usize,
        width: framebuffer_tag.framebuffer_width as usize,
        address: framebuffer_tag.framebuffer_addr as usize
    }
}
