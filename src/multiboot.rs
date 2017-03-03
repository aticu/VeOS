
#[repr(C)]
#[derive(Debug)]
pub struct MultibootInformation {
    pub total_size : u32,
    reserved : u32,
    tags : u32
}
