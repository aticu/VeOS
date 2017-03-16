#[repr(C)]
struct BootCommandLine { //type = 1
    tag_type: u32,
    size: u32,
    string: [u8]
}
