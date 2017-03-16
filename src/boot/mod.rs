mod multiboot2;
mod freestanding;

use super::vga_buffer;

enum BootMethod {
    Unknown,
    Multiboot2 //currently only multiboot2 is supported
}

//This will only be set once very early. After that it can be assumed to be static.
static mut BOOT_METHOD: BootMethod = BootMethod::Unknown;

///Initializes the boot module and all the data it provides.
pub fn init(magic_number: u32, information_structure_address: usize) {
    set_boot_method(magic_number);

    match *get_boot_method() {
        BootMethod::Unknown => freestanding::init(),
        BootMethod::Multiboot2 => multiboot2::init(information_structure_address)
    };
}

fn set_boot_method(magic_number: u32) {
    unsafe {
        BOOT_METHOD = match magic_number {
            0x36d76289 => BootMethod::Multiboot2,
            _ => BootMethod::Unknown
        }
    }
}

fn get_boot_method() -> &'static BootMethod {
    unsafe { &BOOT_METHOD }
}

pub fn get_vga_info() -> vga_buffer::Info {
    match *get_boot_method() {
        BootMethod::Unknown => freestanding::get_vga_info(),
        BootMethod::Multiboot2 => multiboot2::get_vga_info()
    }
}

pub fn get_bootloader_name() -> &'static str {
    match *get_boot_method() {
        BootMethod::Unknown => "None",
        BootMethod::Multiboot2 => multiboot2::get_bootloader_name()
    }
}
