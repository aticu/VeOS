#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use]
mod ffi;
#[macro_use]
mod io;
mod arch;
mod boot;

///Defines the name of the operating system.
static OS_NAME: &str = "VeOS";

///The main entry point for the operating system. This gets called by the loader.
#[no_mangle]
pub extern fn main(magic_number: u32, information_structure_address: usize) -> ! {
    boot::init(magic_number, information_structure_address);
    io::init();
    println!("Booted {} using {}...", OS_NAME, boot::get_bootloader_name());
    boot::print_all();

    loop {
    }
}

//TODO: add doc
#[lang = "eh_personality"]
extern fn eh_personality() {
    unimplemnted!();
}

///This function gets called when the operating system panics. It aims to provide as much information as possible.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC! in file '{}' at line {}:", file, line);
    println!("{}", fmt);
    loop {
    }
}
