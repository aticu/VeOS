///Converts the string at the given address from a c string to a rust &'static str. Optionally if the length is known, the process can be sped up.
macro_rules! from_c_str {
    ($address:expr, $length:expr) => {{
        use core::str;
        use core::slice;
        unsafe {
            assert!(*(($address + $length) as *const u8) == 0);
        }
        let bytes: &[u8] = unsafe { slice::from_raw_parts($address as *const u8, $length as usize - 1) };
        str::from_utf8(bytes)
    }};
    ($address:expr) => {{
        use core::str;
        use core::slice;
        let mut address: usize = $address;
        unsafe {
            while *(address as *const u8) != 0 {
                address += 1;
            }
        }
        from_c_str!($address, (address - $address))
    }};
}
