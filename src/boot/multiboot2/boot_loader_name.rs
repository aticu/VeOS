#[repr(C)]
struct BootLoaderName { //type = 2
    tag_type: u32,
    size: u32,
    string: [u8]
}
