#[cfg(target_arch = "x86_64")]
use arch::x86_64::vga_buffer;

pub fn init() {
    //TODO this gets called when the OS is booted using an unknown bootloader
    //try to figure out all the necessary details using other methods here
}

pub fn get_vga_info() -> vga_buffer::Info {
    unimplemented!();
}
