//! Provides information about the initial status of the system.
mod multiboot2;
mod multiboot;
mod freestanding;

#[cfg(target_arch = "x86_64")]
use arch::vga_buffer;
use memory::FreeMemoryArea;

/// Lists possiblities for boot sources.
enum BootMethod {
    /// No known bootloader could be found.
    Unknown,
    /// The system was booted using multiboot.
    Multiboot,
    /// The system was booted using multiboot2.
    Multiboot2
}

/// Provides an iterator for a memory map.
pub struct MemoryMapIterator {
    multiboot_iterator: multiboot::MemoryMapIterator
}

impl MemoryMapIterator {
    /// Creates a new memory map iterator.
    fn new() -> MemoryMapIterator {
        MemoryMapIterator { multiboot_iterator: multiboot::get_memory_map() }
    }
}

impl Iterator for MemoryMapIterator {
    type Item = FreeMemoryArea;

    fn next(&mut self) -> Option<FreeMemoryArea> {
        match *get_boot_method() {
            BootMethod::Multiboot => self.multiboot_iterator.next(),
            _ => None,
        }
    }
}

/// The method that the system was booted with.
// This will only be set once very early. After that it can be assumed to be
// static.
static mut BOOT_METHOD: BootMethod = BootMethod::Unknown;

/// Initializes the boot module and all the data it provides.
pub fn init(magic_number: u32, information_structure_address: usize) {
    set_boot_method(magic_number);

    match *get_boot_method() {
        BootMethod::Multiboot2 => multiboot2::init(information_structure_address),
        BootMethod::Multiboot => multiboot::init(information_structure_address),
        _ => freestanding::init(),
    };
}

/// Identifies the boot method.
fn set_boot_method(magic_number: u32) {
    unsafe {
        BOOT_METHOD = match magic_number {
            0x36d76289 => BootMethod::Multiboot2,
            0x2badb002 => BootMethod::Multiboot,
            _ => BootMethod::Unknown,
        }
    }
}

/// Returns the method the system was booted with.
fn get_boot_method() -> &'static BootMethod {
    unsafe { &BOOT_METHOD }
}

/// Returns information about the VGA buffer.
#[cfg(target_arch = "x86_64")]
pub fn get_vga_info() -> vga_buffer::Info {
    match *get_boot_method() {
        BootMethod::Multiboot2 => multiboot2::get_vga_info(),
        _ => freestanding::get_vga_info(),
    }
}

/// Returns the name of the boot loader.
pub fn get_bootloader_name() -> &'static str {
    match *get_boot_method() {
        BootMethod::Multiboot2 => multiboot2::get_bootloader_name(),
        BootMethod::Multiboot => multiboot::get_bootloader_name(),
        _ => "no boot loader",
    }
}

/// Returns an iterator for the map of usable memory.
pub fn get_memory_map() -> MemoryMapIterator {
    MemoryMapIterator::new()
}
