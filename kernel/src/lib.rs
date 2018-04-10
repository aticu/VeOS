#![feature(lang_items)]
#![feature(const_fn)]
#![feature(const_size_of)]
#![feature(const_unsafe_cell_new)]
#![feature(const_unique_new)]
#![feature(ptr_internals)]
#![feature(repr_transparent)]
#![feature(unique)]
#![feature(asm)]
#![feature(integer_atomics)]
#![feature(alloc)]
#![feature(naked_functions)]
#![feature(use_extern_macros)]
#![feature(allocator_internals)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![warn(missing_docs)]
#![default_lib_allocator]

//! The VeOS operating system kernel.
//!
//! This crate contains all of the rust code for the VeOS kernel.
//!
//! The kernel is aiming to be a microkernel.

#[allow(unused_extern_crates)]
extern crate rlibc;
extern crate volatile;
#[macro_use]
extern crate bitflags;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate once;
#[cfg(not(test))]
extern crate alloc;
extern crate raw_cpuid;

#[macro_use]
mod macros;
#[macro_use]
mod io;
mod arch;
mod boot;
mod elf;
mod file_handle;
mod initramfs;
mod interrupts;
mod memory;
mod multitasking;
mod sync;
mod syscalls;

/// The name of the operating system.
static OS_NAME: &str = "VeOS";

use memory::allocator::Allocator;
/// The global kernel allocator.
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

/// The main entry point for the operating system.
///
/// This is what should get called by the loader.
///
/// #Arguments
/// - `magic_number` should contain a number that identifies a boot loader.
/// - `information_structure_address` is used for boot loaders that pass
/// additional information to the operating system.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main(magic_number: u32, information_structure_address: usize) -> ! {
    unsafe {
        sync::disable_preemption();
    }

    arch::early_init();
    boot::init(magic_number, information_structure_address);
    io::init();
    println!(
        "Booted {} using {}...",
        OS_NAME,
        boot::get_bootloader_name()
    );
    memory::init();
    arch::init();

    let extended_info = raw_cpuid::CpuId::new().get_extended_function_info();
    let unwrapped_info = extended_info.unwrap();
    println!(
        "The processor is a {}",
        unwrapped_info.processor_brand_string().unwrap()
    );
    println!(
        "The available amount of memory is {}MiB.",
        arch::get_free_memory_size() / 1024 / 1024
    );

    elf::process_from_initramfs_file("/bin/init").expect("Initprocess could not be loaded");

    unsafe {
        arch::enter_first_thread();
    }
}

/// The panic handler.
///
/// This function gets called when the operating system panics.
/// It aims to provide as much information as possible.
/// The arguments are passed by the compiler,
/// this is not meant to be called manually anywhere,
/// but through the panic! macro.
#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC! in file '{}' at line {}:", file, line);
    println!("{}", fmt);
    unsafe {
        sync::disable_preemption();
    }
    loop {
        unsafe {
            sync::cpu_halt();
        }
    }
}
