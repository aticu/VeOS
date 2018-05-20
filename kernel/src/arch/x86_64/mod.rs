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
#[macro_use]
mod serial;

pub use self::context::Context;
use self::gdt::{GDT, TSS};
use self::interrupts::issue_self_interrupt;
use self::interrupts::SCHEDULE_INTERRUPT_NUM;
use self::serial::SerialPort;
use super::Architecture;
use core::fmt;
use core::fmt::Write;
use core::time::Duration;
use log::{set_logger, Level, Log, Metadata, Record};
use memory::{Address, MemoryArea, PageFlags, PhysicalAddress, VirtualAddress};
use multitasking::{StackType, CURRENT_THREAD};
use raw_cpuid::CpuId;
use sync::mutex::Mutex;
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

        debug!("Initializing the GDT...");
        unsafe {
            GDT.load();
        }

        debug!("Initializing the syscall interface...");
        syscalls::init();

        debug!("Initializing interrupts...");
        interrupts::init();
    }

    fn init_io() {
        vga_buffer::init();
        COM1.lock().init();
    }

    fn init_logger() {
        // Ignore the result. If the logger fails to be initialized, logging won't work.
        match set_logger(&KERNEL_LOGGER) {
            _ => ()
        }
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

    fn invoke_scheduler() {
        issue_self_interrupt(SCHEDULE_INTERRUPT_NUM);
    }

    unsafe fn enter_first_thread() -> ! {
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

    #[inline(always)]
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

    fn interrupt_in(duration: Duration) {
        // TODO: allow more fine grained sleeps than milliseconds
        let mut sleep_duration = duration.subsec_millis();
        let second_part = duration.as_secs().saturating_mul(1000);
        sleep_duration = sleep_duration.saturating_add(second_part as u32);

        // FIXME: This doesn't work, as long as the clock source is relying on
        // interrupts.

        interrupts::lapic::set_timer(sleep_duration);
    }

    #[inline(always)]
    unsafe fn switch_context(old_context: &mut Context, new_context: &Context) {
        context::switch_context(old_context, new_context)
    }

    fn get_free_memory_size() -> usize {
        memory::get_free_memory_size()
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

    fn write_fmt(args: fmt::Arguments) {
        vga_buffer::WRITER.lock().write_fmt(args).unwrap();
    }
}

/// The COM1 serial port.
pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3f8));

/// The type of the logger for the kernel.
pub struct KernelLogger;

/// The kernel logger.
pub static KERNEL_LOGGER: KernelLogger = KernelLogger;

/// Determines whether all logging should be to the screen.
const LOG_TO_SCREEN: bool = false;

impl Log for KernelLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let reset = "\x1b[0m";
        let red = "\x1b[31m";
        let yellow = "\x1b[33m";
        let time = Timestamp::get_current();
        match record.metadata().level() {
            Level::Error => {
                println!("{}: {}", record.level(), record.args());
                serial_println!(
                    "{} {}{}{}: {}",
                    time,
                    red,
                    record.level(),
                    reset,
                    record.args()
                );
            },
            Level::Warn => {
                println!("{}: {}", record.level(), record.args());
                serial_println!(
                    "{} {}{}{}: {}",
                    time,
                    yellow,
                    record.level(),
                    reset,
                    record.args()
                );
            },
            Level::Info => {
                println!("{}", record.args());
                serial_println!("{} {}", time, record.args());
            },
            Level::Debug => {
                if LOG_TO_SCREEN {
                    println!("{}: {}", record.level(), record.args());
                }
                serial_println!("{} {}: {}", time, record.level(), record.args());
            },
            Level::Trace => {
                if LOG_TO_SCREEN {
                    println!("{}: {}", record.level(), record.args());
                }
                serial_println!("{} {}: {}", time, record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}
