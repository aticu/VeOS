//! The x86_64 architecture.
//!
//! This module does all the architecture specific things for x86_64.

pub mod context;
mod gdt;
mod interrupts;
pub mod memory;
pub mod sync;
mod syscalls;
pub mod vga_buffer;

pub use self::context::Context;
use self::gdt::{GDT, TSS};
use self::interrupts::issue_self_interrupt;
use self::interrupts::SCHEDULE_INTERRUPT_NUM;
use super::Architecture;
use core::fmt;
use core::fmt::Write;
use memory::{Address, MemoryArea, PageFlags, PhysicalAddress, VirtualAddress};
use multitasking::{StackType, CURRENT_THREAD};
use raw_cpuid::CpuId;
use sync::time::Timestamp;
use x86_64::instructions::{rdmsr, wrmsr};
use x86_64::registers::*;

pub struct X86_64;

impl Architecture for X86_64 {
    type AddressSpaceManager = memory::address_space_manager::AddressSpaceManager;

    type Context = context::Context;

    const STACK_TYPE: StackType = StackType::FullDescending;

    fn early_init() {
        assert_has_not_been_called!(
            "Early x86_64 specific initialization should only be called once."
        );

        let cpuid = CpuId::new();
        let mut supported = true;

        if let Some(features) = cpuid.get_feature_info() {
            supported &= features.has_apic();
        } else {
            supported = false;
        }

        if let Some(function_info) = cpuid.get_extended_function_info() {
            supported &= function_info.has_syscall_sysret();
            supported &= function_info.has_execute_disable();
        } else {
            supported = false;
        }

        if !supported {
            panic!("Your hardware unfortunately does not supported VeOS.");
        }

        unsafe {
            // Enable syscall/sysret instructions and the NXE bit in the page table.
            wrmsr(msr::IA32_EFER, rdmsr(msr::IA32_EFER) | 1 << 11 | 1);

            // Enable global pages.
            let cr4_flags = control_regs::cr4() | control_regs::Cr4::ENABLE_GLOBAL_PAGES;
            control_regs::cr4_write(cr4_flags);

            // Enable read only pages.
            let cr0_flags = control_regs::cr0() | control_regs::Cr0::WRITE_PROTECT;
            control_regs::cr0_write(cr0_flags);
        }
    }

    fn memory_init() {
        memory::init();
    }

    fn init() {
        assert_has_not_been_called!(
            "x86_64 specific initialization code should only be called once."
        );

        unsafe {
            GDT.load();
        }

        syscalls::init();
        interrupts::init();
    }

    fn get_cpu_num() -> usize {
        CpuId::new()
            .get_feature_info()
            .unwrap()
            .max_logical_processor_ids() as usize
    }

    fn get_cpu_id() -> usize {
        CpuId::new()
            .get_feature_info()
            .unwrap()
            .initial_local_apic_id() as usize
    }

    fn schedule() {
        issue_self_interrupt(SCHEDULE_INTERRUPT_NUM);
    }

    unsafe fn enter_first_thread() {
        let stack_pointer = CURRENT_THREAD
            .without_locking()
            .context
            .kernel_stack_pointer;
        TSS.as_mut().privilege_stack_table[0] = ::x86_64::VirtualAddress(stack_pointer.as_usize());
        asm!("mov rsp, $0
            ret"
            : : "r"(stack_pointer) : : "intel", "volatile");
        unreachable!();
    }

    #[inline(always)]
    fn cpu_relax() {
        sync::cpu_relax()
    }

    #[inline(always)]
    unsafe fn cpu_halt() {
        sync::cpu_halt()
    }

    fn get_interrupt_state() -> bool {
        sync::interrupts_enabled()
    }

    #[inline(always)]
    unsafe fn disable_interrupts() {
        sync::disable_interrupts()
    }

    #[inline(always)]
    unsafe fn enable_interrupts() {
        sync::enable_interrupts()
    }

    fn get_current_timestamp() -> Timestamp {
        sync::get_current_timestamp()
    }

    unsafe fn switch_context(old_context: &mut Context, new_context: &Context) {
        context::switch_context(old_context, new_context)
    }

    fn map_page(page_address: VirtualAddress, flags: PageFlags) {
        memory::map_page(page_address, flags)
    }

    unsafe fn unmap_page(page_address: VirtualAddress) {
        memory::unmap_page(page_address)
    }

    fn get_kernel_area() -> MemoryArea<PhysicalAddress> {
        memory::get_kernel_area()
    }

    fn get_initramfs_area() -> MemoryArea<VirtualAddress> {
        memory::get_initramfs_area()
    }

    fn get_page_flags(page_address: VirtualAddress) -> PageFlags {
        memory::get_page_flags(page_address)
    }

    fn is_userspace_address(address: VirtualAddress) -> bool {
        memory::is_userspace_address(address)
    }

    const PAGE_SIZE: usize = memory::PAGE_SIZE;

    const HEAP_AREA: MemoryArea<VirtualAddress> =
        MemoryArea::new(memory::HEAP_START, memory::HEAP_MAX_SIZE);

    //TODO: user stacks
    //TODO: get memory information

    //pub use self::$name::memory::new_address_space_manager;
    //pub use self::$name::memory::idle_address_space_manager;

    fn write_fmt(args: fmt::Arguments) {
        vga_buffer::WRITER.lock().write_fmt(args).unwrap();
    }
}

/// The stack type used for the x86_64 architecture.
pub const STACK_TYPE: StackType = StackType::FullDescending;

/// Initializes the machine state for the x86_64 architecture to a bare minimum.
pub fn early_init() {
    assert_has_not_been_called!("Early x86_64 specific initialization should only be called once.");

    let cpuid = CpuId::new();
    let mut supported = true;

    if let Some(features) = cpuid.get_feature_info() {
        supported &= features.has_apic();
    } else {
        supported = false;
    }

    if let Some(function_info) = cpuid.get_extended_function_info() {
        supported &= function_info.has_syscall_sysret();
        supported &= function_info.has_execute_disable();
    } else {
        supported = false;
    }

    if !supported {
        panic!("Your hardware unfortunately does not supported VeOS.");
    }

    unsafe {
        // Enable syscall/sysret instructions and the NXE bit in the page table.
        wrmsr(msr::IA32_EFER, rdmsr(msr::IA32_EFER) | 1 << 11 | 1);

        // Enable global pages.
        let cr4_flags = control_regs::cr4() | control_regs::Cr4::ENABLE_GLOBAL_PAGES;
        control_regs::cr4_write(cr4_flags);

        // Enable read only pages.
        let cr0_flags = control_regs::cr0() | control_regs::Cr0::WRITE_PROTECT;
        control_regs::cr0_write(cr0_flags);
    }
}

/// Initializes the machine state for the x86_64 architecture to the final
/// state.
pub fn init() {
    assert_has_not_been_called!("x86_64 specific initialization code should only be called once.");

    unsafe {
        GDT.load();
    }

    syscalls::init();
    interrupts::init();
}

/// Returns the ID of the currently running CPU.
pub fn get_cpu_id() -> usize {
    CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize
}

/// Returns the number of addressable CPUs.
pub fn get_cpu_num() -> usize {
    CpuId::new()
        .get_feature_info()
        .unwrap()
        .max_logical_processor_ids() as usize
}

/// This is called once per processor to enter the first user mode thread.
///
/// # Safety
/// - This should only be called once.
pub unsafe fn enter_first_thread() -> ! {
    let stack_pointer = CURRENT_THREAD
        .without_locking()
        .context
        .kernel_stack_pointer;
    TSS.as_mut().privilege_stack_table[0] = ::x86_64::VirtualAddress(stack_pointer.as_usize());
    asm!("mov rsp, $0
          ret"
          : : "r"(stack_pointer) : : "intel", "volatile");
    unreachable!();
}

/// This function starts a scheduling operation.
pub fn schedule() {
    issue_self_interrupt(SCHEDULE_INTERRUPT_NUM);
}
