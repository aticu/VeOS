#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]
#![feature(integer_atomics)]
#![feature(alloc, collections)]
#![feature(oom)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(use_extern_macros)]
#![no_std]
#![warn(missing_docs)]

//! The VeOS operating system.
//!
//! This crate contains all of the rust code for VeOS.

extern crate rlibc;
extern crate volatile;
extern crate spin;
#[macro_use]
extern crate bitflags;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate once;
extern crate allocator_stub;
extern crate raw_cpuid;
#[cfg(not(test))]
extern crate alloc;
#[cfg(not(test))]
#[macro_use]
extern crate collections;

#[macro_use]
mod macros;
#[macro_use]
mod io;
mod arch;
mod boot;
mod sync;
mod memory;

/// The name of the operating system.
static OS_NAME: &str = "VeOS";

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
    println!("Booted {} using {}...",
             OS_NAME,
             boot::get_bootloader_name());
    memory::init();
    arch::init();

    let extended_info = raw_cpuid::CpuId::new().get_extended_function_info();
    let unwrapped_info = extended_info.unwrap();
    println!("The processor is a {}",
             unwrapped_info.processor_brand_string().unwrap());

    unsafe {
        sync::enable_preemption();
    }
    loop {}
}

// TODO: add support for stack unwinding
#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {
    unimplemented!();
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
