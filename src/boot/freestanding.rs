//! Aims to provide info without a boot loader.

#[cfg(target_arch = "x86_64")]
use arch::vga_buffer;

/// Initialize the system without help of a boot loader.
pub fn init() {
    // TODO: This gets called when the OS is booted using an unknown bootloader.
    // It should try to figure out all the necessary details using other methods
    // here.
}

/// Return the vga information.
#[cfg(target_arch = "x86_64")]
pub fn get_vga_info() -> vga_buffer::Info {
    // Currently this is just a best guess.
    vga_buffer::Info {
        height: 25,
        width: 80,
        address: 0xffff8000000b8000
    }
}
