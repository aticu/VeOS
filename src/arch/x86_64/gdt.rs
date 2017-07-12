//! Provides the global descriptor table used by the operating system.

use super::memory::FINAL_STACK_TOP;
use core::mem::size_of;
use x86_64::PrivilegeLevel;
use x86_64::VirtualAddress;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::{DescriptorTablePointer, lgdt, load_tss};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;

/// The amount of entries the GDT has.
const GDT_ENTRY_NUM: usize = 8;

/// The kernel code segment.
pub const KERNEL_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);

/// The kernel data segment.
pub const KERNEL_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);

/// The (unused) kernel 32-bit code selector.
pub const USER_32BIT_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring3);

/// The user data segment.
pub const USER_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring3);

/// The user code segment.
pub const USER_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring3);

/// The TSS selector in the GDT.
pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(6, PrivilegeLevel::Ring0);

/// Represents the GDT.
pub struct Gdt {
    /// The actual entries of the GDT.
    entries: [u64; GDT_ENTRY_NUM],
    /// The index where the next new entry will be created.
    next_entry: usize
}

cpu_local! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[0] = VirtualAddress(FINAL_STACK_TOP);
        tss
    };
}

cpu_local! {
    /// The global descriptor table of the system.
    pub static ref GDT: Gdt = {
        let mut gdt = Gdt::new();
        gdt.add_entry(Descriptor::code(DPL0));
        gdt.add_entry(Descriptor::data(DPL0));
        gdt.add_entry(Descriptor::unused());
        gdt.add_entry(Descriptor::data(DPL3));
        gdt.add_entry(Descriptor::code(DPL3));
        gdt.add_entry(Descriptor::tss(&*TSS));

        gdt
    };
}

impl Gdt {
    /// Creates a new zeroed global descriptor table.
    fn new() -> Gdt {
        let entries: [u64; GDT_ENTRY_NUM] = [0; GDT_ENTRY_NUM];
        Gdt {
            entries,
            next_entry: 1
        }
    }

    /// Adds an entry to the GDT.
    fn add_entry(&mut self, descriptor: Descriptor) {
        match descriptor {
            Descriptor::UserDescriptor(value) => {
                self.entries[self.next_entry] = value;
                self.next_entry += 1;
            },
            Descriptor::SystemDescriptor(values) => {
                self.entries[self.next_entry] = values[0];
                self.entries[self.next_entry + 1] = values[1];
                self.next_entry += 2;
            },
        }
    }

    /// Loads this descriptor table.
    pub unsafe fn load(&'static self) {
        let table_pointer = DescriptorTablePointer {
            limit: (GDT_ENTRY_NUM * size_of::<u64>() - 1) as u16,
            base: self as *const _ as u64
        };

        lgdt(&table_pointer);
        set_cs(KERNEL_CODE_SEGMENT);
        load_tss(TSS_SELECTOR);
    }
}

bitflags! {
    /// The possible flags for a 64-bit descriptor.
    flags DescriptorFlags: u64 {
        /// The segment is readable.
        const READABLE           = 1 << 9 + 32,
        /// The segment is executable.
        const EXECUTABLE         = 1 << 11 + 32,
        /// The descriptor is a user descriptor.
        const USER_SEGMENT       = 1 << 12 + 32,
        /// The privilege level of this descriptor is 0 (highest).
        const DPL0               = 0 << 13 + 32,
        /// The privilege level of this descriptor is 1.
        const DPL1               = 1 << 13 + 32,
        /// The privilege level of this descriptor is 2.
        const DPL2               = 2 << 13 + 32,
        /// The privilege level of this descriptor is 3 (lowest).
        const DPL3               = 3 << 13 + 32,
        /// This descriptor is present.
        const PRESENT            = 1 << 15 + 32,
        /// This is a long mode descriptor.
        const LONG_MODE          = 1 << 21 + 32
    }
}

/// Represents a descriptor in the GDT.
enum Descriptor {
    /// Represents either a code or a data descriptor.
    UserDescriptor(u64),
    /// Represents a system descriptor (e.g. TSS).
    SystemDescriptor([u64; 2])
}

impl Descriptor {
    /// Creates a new code descriptor.
    fn code(dpl: DescriptorFlags) -> Descriptor {
        let val = READABLE | EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE | dpl;
        Descriptor::UserDescriptor(val.bits())
    }

    /// Creates a new data descriptor.
    fn data(dpl: DescriptorFlags) -> Descriptor {
        let val = READABLE | USER_SEGMENT | PRESENT | LONG_MODE | dpl;
        Descriptor::UserDescriptor(val.bits())
    }

    /// Creates a new unused descriptor.
    fn unused() -> Descriptor {
        Descriptor::UserDescriptor(0)
    }

    /// Creates a new TSS descriptor.
    fn tss(segment: &'static TaskStateSegment) -> Descriptor {
        let limit = (size_of::<TaskStateSegment>() - 1) as u64;
        let base = segment as *const _ as u64;

        let mut low_val = limit; // The segment limit.
        low_val |= (base & 0xffff) << 16; // The lowest two bytes of the base address.
        low_val |= (base >> 16 & 0xff) << 32; // The third byte of the base address.
        low_val |= 0b1001 << 8 + 32; // The type of this descriptor.
        low_val |= PRESENT.bits(); // The present bit of the descriptor.
        low_val |= (base >> 24 & 0xff) << 24 + 32; // The fourth byte of the base address.

        let high_val = base >> 32; // The last four bytes of the base address.

        Descriptor::SystemDescriptor([low_val, high_val])
    }
}
