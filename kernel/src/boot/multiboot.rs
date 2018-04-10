//! Handles the multiboot information structure.

use core::mem::size_of;
use memory::{Address, MemoryArea, PhysicalAddress, VirtualAddress};

/// Represents the multiboot information structure.
#[repr(C)]
struct MultibootInformation {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    elf_num: u32, // Only elf tags are supported, because this kernel is an ELF file.
    elf_size: u32,
    elf_addr: u32,
    elf_shndx: u32,
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16
}

bitflags! {
    ///The possible flags in the flags field.
    flags MultibootFlags: u32 {
        ///Basic memory information is available.
        const BASIC_MEMORY = 1 << 0,
        ///Boot device information is available.
        const BOOT_DEVICE = 1 << 1,
        ///A command line is available.
        const CMDLINE = 1 << 2,
        ///Module information is available.
        const MODULES = 1 << 3,
        ///a.out information is available.
        const A_OUT = 1 << 4,
        ///Elf information is available.
        const ELF = 1 << 5,
        ///A memory map is available.
        const MMAP = 1 << 6,
        ///Information about drives  is available.
        const DRIVES = 1 << 7,
        ///A config table is available.
        const CONFIG_TABLE = 1 << 8,
        ///The boot loader name is available.
        const BOOT_LOADER_NAME = 1 << 9,
        ///An APM table is available.
        const APM_TABLE = 1 << 10,
        ///VBE information is available.
        const VBE = 1 << 11
    }
}

/// Represents an entry in the given memory map.
#[repr(C, packed)]
struct MmapEntry {
    /// The size of the entry.
    size: u32,
    /// The base address of the memory area.
    base_addr: PhysicalAddress,
    /// The length of the memory area.
    length: usize,
    /// The type of memory contained in the area.
    ///
    /// 1 means usable memory.
    mem_type: u32
}

/// Represents a module loaded by the boot loader.
#[repr(C, packed)]
struct ModuleEntry {
    /// The start address of the module.
    mod_start: u32,
    /// The end address of the module.
    mod_end: u32,
    /// The string associated with the module.
    string: u32,
    /// Reserved, don't use.
    reserved: u32
}

/// The base address for the information strucuture.
// This is only valid after init was called.
static mut STRUCT_BASE_ADDRESS: *const MultibootInformation = 0 as *const MultibootInformation;

/// Initializes the multiboot module.
pub fn init(information_structure_address: usize) {
    assert_has_not_been_called!("The multiboot module should only be initialized once.");

    unsafe {
        STRUCT_BASE_ADDRESS =
            to_virtual!(information_structure_address) as *const MultibootInformation
    };

    assert!(!get_flags().contains(A_OUT | ELF));
}

/// Returns the memory area of the initramfs.
pub fn get_initramfs_area() -> MemoryArea<PhysicalAddress> {
    let module_entry = get_initramfs_module_entry();

    MemoryArea::from_start_and_end(
        PhysicalAddress::from_usize(module_entry.mod_start as usize),
        PhysicalAddress::from_usize(module_entry.mod_end as usize)
    )
}

/// Returns the module entry for the initramfs.
fn get_initramfs_module_entry() -> &'static ModuleEntry {
    let info = get_info();
    let mod_count = info.mods_count as usize;
    let mod_addr = to_virtual!(info.mods_addr) as usize;

    for i in 0..mod_count {
        let mod_entry =
            unsafe { &*((mod_addr + i * size_of::<ModuleEntry>()) as *const ModuleEntry) };
        let mod_string = from_c_str!(VirtualAddress::from_usize(to_virtual!(
            mod_entry.string as usize
        ))).unwrap();
        if mod_string == "initramfs" {
            return mod_entry;
        }
    }

    panic!("No initramfs found.");
}

/// Returns the name of the boot loader.
pub fn get_bootloader_name() -> &'static str {
    if get_flags().contains(BOOT_LOADER_NAME) {
        from_c_str!(VirtualAddress::from_usize(to_virtual!(
            get_info().boot_loader_name
        ))).unwrap()
    } else {
        // When no specific name was given by the boot loader.
        "a multiboot compliant bootloader"
    }
}

/// Returns the flags of the multiboot structure.
fn get_flags() -> MultibootFlags {
    MultibootFlags::from_bits_truncate(get_info().flags)
}

/// Returns the multiboot structure.
fn get_info() -> &'static MultibootInformation {
    unsafe { &*STRUCT_BASE_ADDRESS }
}

/// Provides an iterator for the memory map.
pub struct MemoryMapIterator {
    /// The address of the current entry in the memory map.
    address: usize,
    /// The address after the last entry in the memory map.
    max_address: usize
}

impl MemoryMapIterator {
    /// Creates a new iterator through the memory map.
    fn new() -> MemoryMapIterator {
        if get_flags().contains(MMAP) {
            MemoryMapIterator {
                address: to_virtual!(get_info().mmap_addr),
                max_address: to_virtual!(get_info().mmap_addr + get_info().mmap_length)
            }
        } else {
            MemoryMapIterator {
                address: 0,
                max_address: 0
            }
        }
    }
}

impl Iterator for MemoryMapIterator {
    type Item = MemoryArea<PhysicalAddress>;

    fn next(&mut self) -> Option<MemoryArea<PhysicalAddress>> {
        while self.address < self.max_address {
            let current_entry = unsafe { &*(self.address as *const MmapEntry) };

            self.address += size_of::<u32>() + current_entry.size as usize;

            if current_entry.mem_type == 1 {
                // only a type of 1 is usable memory
                return Some(MemoryArea::new(
                    current_entry.base_addr,
                    current_entry.length
                ));
            }
        }
        None
    }
}

/// Returns the memory map given by the boot loader.
pub fn get_memory_map() -> MemoryMapIterator {
    MemoryMapIterator::new()
}
