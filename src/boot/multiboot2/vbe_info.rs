#[repr(C)]
struct VBEInfo { //type = 7
    tag_type: u32,
    size: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,
    vbe_control_info: [u8; 512],
    vbe_mode_info: [u8; 256]
}

