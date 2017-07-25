//! Provides information about the initial status of the system.
mod multiboot2;
mod multiboot;
mod freestanding;

#[cfg(target_arch = "x86_64")]
use arch::vga_buffer;
use memory::{FreeMemoryArea, PAGE_SIZE, PhysicalAddress, get_kernel_end_address,
             get_kernel_start_address};

/// Lists possiblities for boot sources.
enum BootMethod {
    /// No known bootloader could be found.
    Unknown,
    /// The system was booted using multiboot.
    Multiboot,
    /// The system was booted using multiboot2.
    Multiboot2
}

/// Represents an area to be excluded from the available memory map.
struct MemoryMapExcludeArea {
    /// The start address of the area to be excluded.
    start: PhysicalAddress,
    /// The length of the area to be excluded.
    length: usize
}

impl MemoryMapExcludeArea {
    /// The memory area containing the kernel.
    fn kernel() -> MemoryMapExcludeArea {
        MemoryMapExcludeArea {
            start: get_kernel_start_address(),
            length: get_kernel_end_address() - get_kernel_start_address()
        }
    }

    /// The memory area containing the initramfs.
    fn initramfs() -> MemoryMapExcludeArea {
        let initramfs_start = get_initramfs_start() / PAGE_SIZE * PAGE_SIZE;
        let initramfs_length = (get_initramfs_length() - 1) / PAGE_SIZE * PAGE_SIZE + PAGE_SIZE;
        MemoryMapExcludeArea {
            start: initramfs_start,
            length: initramfs_length
        }
    }

    /// Checks if the area is contained within another area.
    fn is_contained_in(&self, area: FreeMemoryArea) -> bool {
        area.start_address() <= self.start && self.start + self.length <= area.end_address()
    }

    /// Returns the first address not contained in the area.
    fn end_address(&self) -> PhysicalAddress {
        self.start + self.length
    }
}

/// Provides an iterator for a memory map.
pub struct MemoryMapIterator {
    multiboot_iterator: multiboot::MemoryMapIterator,
    to_exclude: [MemoryMapExcludeArea; 2],
    current_entry: Option<FreeMemoryArea>,
    exclude_index: usize
}

impl MemoryMapIterator {
    /// Creates a new memory map iterator.
    fn new() -> MemoryMapIterator {
        let kernel_area = MemoryMapExcludeArea::kernel();
        let initramfs_area = MemoryMapExcludeArea::initramfs();

        let to_exclude = if kernel_area.start <= initramfs_area.start {
            [kernel_area, initramfs_area]
        } else {
            [initramfs_area, kernel_area]
        };

        let mut multiboot_iterator = multiboot::get_memory_map();

        let current_entry = multiboot_iterator.next();

        let exclude_index = 0;

        MemoryMapIterator {
            multiboot_iterator,
            to_exclude,
            current_entry,
            exclude_index
        }
    }
}

impl Iterator for MemoryMapIterator {
    type Item = FreeMemoryArea;

    fn next(&mut self) -> Option<FreeMemoryArea> {
        // NOTE: This assumes function makes a few assumptions to work properly:
        // - The to_exclude list must be ordered by the start addresses.
        // - The to_exclude entries must not overlap.
        // - The memory areas must not overlap.
        // - A to_exclude entry must lie completely within a memory area.

        let get_next_entry = |iterator: &mut MemoryMapIterator| match *get_boot_method() {
            BootMethod::Multiboot => iterator.multiboot_iterator.next(),
            _ => unimplemented!(),
        };

        loop {
            return if let Some(current_entry) = self.current_entry {
                       if self.exclude_index >= self.to_exclude.len() {
                           // If all the exclude areas were handled.

                           self.current_entry = get_next_entry(self);

                           Some(current_entry)
                       } else {
                           // Handle the exclude areas.


                           if self.to_exclude[self.exclude_index].is_contained_in(current_entry) {
                               // The area to exclude is contained in the current free entry.
                               let (entry_before, entry_after) = {
                                   let exclude_area = &self.to_exclude[self.exclude_index];

                                   (FreeMemoryArea::new(current_entry.start_address(),
                                                        exclude_area.start -
                                                        current_entry.start_address()),
                                    FreeMemoryArea::new(exclude_area.end_address(),
                                                        current_entry.end_address() -
                                                        exclude_area.end_address()))
                               };

                               self.exclude_index += 1;

                               if entry_after.end_address() == entry_after.start_address() {
                                   self.current_entry = get_next_entry(self);
                               } else {
                                   self.current_entry = Some(entry_after);
                               }

                               if entry_before.end_address() == entry_before.start_address() {
                                   continue;
                               } else {
                                   Some(entry_before)
                               }
                           } else {
                               self.current_entry = get_next_entry(self);

                               Some(current_entry)
                           }
                       }
                   } else {
                       None
                   };
        }
    }
}

/// The method that the system was booted with.
// This will only be set once very early. After that it can be assumed to be
// static.
static mut BOOT_METHOD: BootMethod = BootMethod::Unknown;

/// Initializes the boot module and all the data it provides.
pub fn init(magic_number: u32, information_structure_address: usize) {
    assert_has_not_been_called!("Boot information should only be initialized once.");

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

/// Returns the start address of the initramfs.
pub fn get_initramfs_start() -> PhysicalAddress {
    match *get_boot_method() {
        BootMethod::Multiboot => multiboot::get_initramfs_start(),
        _ => unimplemented!(),
    }
}

/// Returns the length of the initramfs.
pub fn get_initramfs_length() -> usize {
    match *get_boot_method() {
        BootMethod::Multiboot => multiboot::get_initramfs_length(),
        _ => unimplemented!(),
    }
}

/// Returns an iterator for the map of usable memory.
pub fn get_memory_map() -> MemoryMapIterator {
    MemoryMapIterator::new()
}
