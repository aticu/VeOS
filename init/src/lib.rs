#![no_std]

#[macro_use]
extern crate veos_std;
#[allow(unused_extern_crates)]
extern crate rlibc;

use core::time::Duration;

#[no_mangle]
pub fn main() {
    veos_std::process::exec("/bin/test").unwrap();

    loop {
        veos_std::thread::sleep(Duration::from_millis(500));
        println!("Test");
    }
}
