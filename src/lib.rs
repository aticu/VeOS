#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use]
mod vga_buffer;
mod multiboot;

#[no_mangle]
//pub extern fn main(multiboot_structure : usize) {
pub extern fn main(multiboot_structure : &multiboot::MultibootInformation) {
    vga_buffer::clear_screen();
    println!("Hello from high level Rust! Package name: {}", env!("CARGO_PKG_VERSION"));
//    println!("MultibootInformation address: {:?}", unsafe { (&*(multiboot_structure as *const multiboot::MultibootInformation)).total_size });
    println!("{:?}", multiboot_structure);

    loop {
    }
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC! in file '{}' at line {}:", file, line);
    println!("{}", fmt);
    loop {
    }
}
