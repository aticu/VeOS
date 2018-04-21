//! Abstracts architecture details.
//!
//! The job of this module is to have submodules for each architecture and to
//! provide interfaces to them.

use memory::{MemoryArea, PageFlags, PhysicalAddress, VirtualAddress};
use multitasking::stack::StackType;
use sync::time::Timestamp;

trait Architecture {
    /// This type is supposed to manage address spaces for the architecture.
    ///
    /// For more details see the `::memory::address_space::AddressSpaceManager`
    /// trait.
    type AddressSpaceManager: ::memory::address_space_manager::AddressSpaceManager;

    /// This type represents the architecture specific part of an execution
    /// context.
    type Context;

    /// The type of stack this architecture uses.
    const STACK_TYPE: StackType;

    /// This is the first function called during initialization.
    ///
    /// It should set up a stable environment for the rest of the
    /// initialization.
    fn early_init();

    /// This function initializes the memory to operational state.
    fn memory_init();

    /// This is the last function called during initialization.
    ///
    /// It can assume that everything is already initialized, including the
    /// memory.
    fn init();

    /// Returns the number of CPUs available.
    ///
    /// A CPU is anything that can run processes.
    fn get_cpu_num() -> usize;

    /// Returns the ID of the currently running CPU.
    fn get_cpu_id() -> usize;

    /// Invokes the scheduler.
    ///
    /// This function changes the currently running thread on the current CPU
    /// to the thread that should be run next on said CPU (which could be the
    /// same).
    fn schedule();

    /// This function enters user mode for the first time.
    ///
    /// It's job is to transition from the system initialization to normal
    /// operation.
    ///
    /// # Safety
    /// - This function should only be called once (per CPU).
    unsafe fn enter_first_thread();

    /// This function saves power while waiting for resources.
    fn cpu_relax();

    /// This function stops the current CPU.
    ///
    /// The CPU will halt until the next interrupt occurs.
    ///
    /// # Safety
    /// - If interrupts are disabled, this function will render the CPU useless
    /// for the remaining uptime. If this isn't intended, make sure that
    /// interrupts are enabled when calling this function.
    unsafe fn cpu_halt();

    /// Returns true if interrupts are enabled and false otherwise.
    fn get_interrupt_state() -> bool;

    /// Disables all interrupts.
    ///
    /// # Safety
    /// - Make sure to re-enable them later. The best way to do so is by not
    /// calling this function directly but rather
    /// `sync::disable_preemption`.
    unsafe fn disable_interrupts();

    /// Enables all interrupts.
    ///
    /// # Safety
    /// - Make sure that all critial sections have been accessed and that no
    /// locks are held. It is better to just use `sync::PreemptionState`
    /// instead of using this directly.
    unsafe fn enable_interrupts();

    /// Returns the current timestamp.
    fn get_current_timestamp() -> Timestamp;

    /// Switches the execution context and saves the current one.
    ///
    /// `old_context` is where the current context is saved to and
    /// `new_context` is the next context to be loaded.
    ///
    /// # Safety
    /// - To make sure that everything is properly cleaned up after switching
    /// the context, this should only be called by the scheduler.
    /// - Make sure preemption is disabled while calling this.
    unsafe fn switch_context(old_context: &mut Context, new_context: &Context);

    /// Maps the page that contains the given address and the given flags.
    // TODO: Move this into the AddressSpaceManager?
    fn map_page(page_address: VirtualAddress, flags: PageFlags);

    /// Unmaps the page that contains the given address.
    unsafe fn unmap_page(page_address: VirtualAddress);

    /// Returns the physical memory area where the kernel is loaded.
    fn get_kernel_area() -> MemoryArea<PhysicalAddress>;

    /// Returns the physical memory area where the initramfs is loaded.
    fn get_initramfs_area() -> MemoryArea<VirtualAddress>;

    /// Returns the page flags for the page containing the given address.
    fn get_page_flags(page_address: VirtualAddress) -> PageFlags;

    /// Returns whether the given address is a userspace address.
    fn is_userspace_address(address: VirtualAddress) -> bool;

    /// The size, in bytes, of a virtual page on the target architecture.
    const PAGE_SIZE: usize;

    /// The memory area where the heap is located.
    const HEAP_AREA: MemoryArea<VirtualAddress>;

    //TODO: user stacks
    //TODO: get memory information

    //pub use self::$name::memory::new_address_space_manager;
    //pub use self::$name::memory::idle_address_space_manager;

    /// Writes the formatted arguments.
    ///
    /// This takes arguments as dictated by `core::fmt` and prints them to the
    /// screen.
    fn write_fmt(args: fmt::Arguments);

    /// Sets the state of being interruptable to the given state.
    ///
    /// # Safety
    /// - Don't use this function directly, rather use the sync module.
    unsafe fn set_interrupt_state(state: bool) {
        if state {
            enable_interrupts();
        } else {
            disable_interrupts();
        }
    }
}

macro_rules! export_arch {
    ($name:ident) => {
        pub use self::$name::early_init;
        pub use self::$name::enter_first_thread;
        pub use self::$name::get_cpu_id;
        pub use self::$name::get_cpu_num;
        pub use self::$name::init;
        pub use self::$name::init_io;
        pub use self::$name::schedule;
        pub use self::$name::Context;
        pub use self::$name::STACK_TYPE;
        pub use self::$name::KERNEL_LOGGER;

        pub use self::$name::sync::cpu_halt;
        pub use self::$name::sync::cpu_relax;
        pub use self::$name::sync::disable_interrupts;
        pub use self::$name::sync::enable_interrupts;
        pub use self::$name::sync::get_current_timestamp;
        pub use self::$name::sync::interrupts_enabled;

        pub use self::$name::memory::get_free_memory_size;
        pub use self::$name::memory::get_initramfs_area;
        pub use self::$name::memory::get_kernel_area;
        pub use self::$name::memory::get_page_flags;
        pub use self::$name::memory::init as memory_init;
        pub use self::$name::memory::is_userspace_address;
        pub use self::$name::memory::map_page;
        pub use self::$name::memory::unmap_page;
        pub use self::$name::memory::HEAP_MAX_SIZE;
        pub use self::$name::memory::HEAP_START;
        pub use self::$name::memory::KERNEL_STACK_AREA_BASE;
        pub use self::$name::memory::KERNEL_STACK_MAX_SIZE;
        pub use self::$name::memory::KERNEL_STACK_OFFSET;
        pub use self::$name::memory::PAGE_SIZE;
        pub use self::$name::memory::USER_STACK_AREA_BASE;
        pub use self::$name::memory::USER_STACK_MAX_SIZE;
        pub use self::$name::memory::USER_STACK_OFFSET;

        pub use self::$name::memory::idle_address_space_manager;
        pub use self::$name::memory::new_address_space_manager;

        pub use self::$name::context::switch_context;
    };
}

#[cfg(target_arch = "x86_64")]
const CURRENT_ARCH: x86_64::X86_64 = x86_64::X86_64;

#[cfg(target_arch = "x86_64")]
export_arch!(x86_64);

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::vga_buffer;

use core::fmt;
#[cfg(target_arch = "x86_64")]
mod x86_64;

/// Writes the formatted arguments.
///
/// This takes arguments as dictated by `core::fmt` and prints the to the
/// screen using the printing method relevant for the current architecture.
pub fn write_fmt(args: fmt::Arguments) {
    if cfg!(target_arch = "x86_64") {
        use core::fmt::Write;
        x86_64::vga_buffer::WRITER.lock().write_fmt(args).unwrap();
    }
}

/// Sets the state of being interruptable to the given state.
///
/// # Safety
/// - Don't use this function directly, rather use the sync module.
pub unsafe fn set_interrupt_state(state: bool) {
    if state {
        enable_interrupts();
    } else {
        disable_interrupts();
    }
}
