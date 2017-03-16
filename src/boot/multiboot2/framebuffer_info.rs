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

