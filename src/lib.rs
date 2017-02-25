#![feature(lang_items)]
#![no_std]

extern crate rlibc;

#[no_mangle]
pub extern fn rust_main()
{
    let string = b"Hello from Rust!";
    let color_byte = 0x0f;

    let mut string_colored = [color_byte; 24];
    for (i, char_byte) in string.into_iter().enumerate() {
        string_colored[i * 2] = *char_byte;
    }

    let buffer_ptr = (0xb8000 + 1988) as * mut _;
    unsafe { *buffer_ptr = string_colored };

    loop {}
}

#[lang = "eh_personality"]
extern fn eh_personality()
{
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> !
{
    loop
    {
    }
}
