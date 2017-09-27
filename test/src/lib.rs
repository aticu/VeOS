#![no_std]

#[macro_use]
extern crate veos_std;
#[allow(unused_extern_crates)]
extern crate rlibc;

#[no_mangle]
pub fn main() {
    loop {
        veos_std::thread::sleep(1000);
        println!("Nest");
    }
}
