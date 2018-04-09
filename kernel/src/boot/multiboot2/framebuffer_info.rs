//! Handles the framebuffer information tag.

use super::get_tag;
#[cfg(target_arch = "x86_64")]
use arch::vga_buffer;
use memory::{Address, VirtualAddress};

/// Represents the framebuffer information tag.
#[repr(C)]
pub struct FramebufferInfo {
    // type = 8
    tag_type: u32,
    size: u32,
    pub framebuffer_addr: u64,
    framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    reserved: u8,
    color_info: u32 // This is just a placeholder, this depends on the framebuffer_type.
}

/// Returns the VGA buffer information requested.
#[cfg(target_arch = "x86_64")]
pub fn get_vga_info() -> vga_buffer::Info {
    let framebuffer_tag = get_tag(8);
    match framebuffer_tag {
        Some(framebuffer_tag_address) => {
            let framebuffer_tag = unsafe { &*(framebuffer_tag_address as *const FramebufferInfo) };
            vga_buffer::Info {
                height: framebuffer_tag.framebuffer_height as usize,
                width: framebuffer_tag.framebuffer_width as usize,
                address: VirtualAddress::from_usize(to_virtual!(framebuffer_tag.framebuffer_addr))
            }
        },
        None => {
            vga_buffer::Info {
                height: 25,
                width: 80,
                address: VirtualAddress::from_usize(to_virtual!(0xb8000))
            }
        },
    }
}
