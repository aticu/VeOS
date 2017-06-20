//! Models the interrupt descriptor table.

use super::idt_entry::{IdtEntry, WithErrorCode, WithoutErrorCode};
use core::mem::size_of;
use x86_64::instructions::tables::{DescriptorTablePointer, lidt};

/// Represents the interrupt descriptor table.
pub struct Idt {
    /// The divide by zero exception entry.
    pub divide_by_zero: IdtEntry<WithoutErrorCode>,
    /// The debug exception entry.
    pub debug: IdtEntry<WithoutErrorCode>,
    /// The NMI extry.
    pub non_maskable_interrupt: IdtEntry<WithoutErrorCode>,
    /// The breakpoint exception entry.
    pub breakpoint: IdtEntry<WithoutErrorCode>,
    /// The overflow exception entry.
    pub overflow: IdtEntry<WithoutErrorCode>,
    /// The bound range exceeded exception entry.
    pub bound_range: IdtEntry<WithoutErrorCode>,
    /// The invalid opcode exception entry.
    pub invalid_opcode: IdtEntry<WithoutErrorCode>,
    /// The device not available exception entry.
    pub device_not_available: IdtEntry<WithoutErrorCode>,
    /// The double fault exception entry.
    pub double_fault: IdtEntry<WithErrorCode>,
    /// The coprocessor segment overrun exception entry.
    pub coprocessor_segment_overrun: IdtEntry<WithoutErrorCode>,
    /// The invalid TSS exception entry.
    pub invalid_tss: IdtEntry<WithErrorCode>,
    /// The segment not present exception entry.
    pub segment_not_present: IdtEntry<WithErrorCode>,
    /// The stack fault exception entry.
    pub stack_fault: IdtEntry<WithErrorCode>,
    /// The general protection fault entry.
    pub general_protection_fault: IdtEntry<WithErrorCode>,
    /// The page fault entry.
    pub page_fault: IdtEntry<WithErrorCode>,
    /// The first reserved entry.
    #[allow(dead_code)]
    reserved_0: IdtEntry<WithoutErrorCode>,
    /// The x87 FPU floating point error entry.
    pub x87_floating_point_error: IdtEntry<WithoutErrorCode>,
    /// The alignment check exception entry.
    pub alignment_check: IdtEntry<WithErrorCode>,
    /// The machine check exception entry.
    pub machine_check: IdtEntry<WithoutErrorCode>,
    /// The SIMD floating point exception entry.
    pub simd_floating_point: IdtEntry<WithoutErrorCode>,
    /// The virtualization exception entry.
    pub virtualization: IdtEntry<WithoutErrorCode>,
    /// The next nine reserved entries.
    #[allow(dead_code)]
    reserved_1: [IdtEntry<WithoutErrorCode>; 9],
    /// The security exception entry.
    pub security: IdtEntry<WithErrorCode>,
    /// The last reserved entry.
    #[allow(dead_code)]
    reserved_2: IdtEntry<WithoutErrorCode>,
    /// The entries that are freely available for interrupts.
    pub interrupts: [IdtEntry<WithoutErrorCode>; 224]
}

impl Idt {
    /// Creates a new IDT.
    pub fn new() -> Idt {
        Idt {
            divide_by_zero: IdtEntry::unused(),
            debug: IdtEntry::unused(),
            non_maskable_interrupt: IdtEntry::unused(),
            breakpoint: IdtEntry::unused(),
            overflow: IdtEntry::unused(),
            bound_range: IdtEntry::unused(),
            invalid_opcode: IdtEntry::unused(),
            device_not_available: IdtEntry::unused(),
            double_fault: IdtEntry::unused(),
            coprocessor_segment_overrun: IdtEntry::unused(),
            invalid_tss: IdtEntry::unused(),
            segment_not_present: IdtEntry::unused(),
            stack_fault: IdtEntry::unused(),
            general_protection_fault: IdtEntry::unused(),
            page_fault: IdtEntry::unused(),
            reserved_0: IdtEntry::unused(),
            x87_floating_point_error: IdtEntry::unused(),
            alignment_check: IdtEntry::unused(),
            machine_check: IdtEntry::unused(),
            simd_floating_point: IdtEntry::unused(),
            virtualization: IdtEntry::unused(),
            reserved_1: [IdtEntry::unused(); 9],
            security: IdtEntry::unused(),
            reserved_2: IdtEntry::unused(),
            interrupts: [IdtEntry::unused(); 224]
        }
    }

    /// Loads the IDT, so it can be used by the CPU.
    pub unsafe fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: self as *const Idt as u64,
            limit: (size_of::<Idt>() - 1) as u16
        };

        lidt(&ptr);
    }
}
