#[cfg(target_arch = "x86_64")]
use arch::x86_64::vga_buffer;
use super::get_tag;

#[repr(C)]
pub struct FramebufferInfo { //type = 8
    tag_type: u32,
    size: u32,
    pub framebuffer_addr: u64,
    framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    reserved: u8,
    color_info: u32 //this is just a placeholder, this depends on the framebuffer_type
}

#[cfg(target_arch = "x86_64")]
pub fn get_vga_info() -> vga_buffer::Info {
    let framebuffer_tag = unsafe { &*(get_tag(8).expect("Framebuffer tag required.") as *const FramebufferInfo) };
    vga_buffer::Info {
        height: framebuffer_tag.framebuffer_height as usize,
        width: framebuffer_tag.framebuffer_width as usize,
        address: framebuffer_tag.framebuffer_addr as usize
    }
}
