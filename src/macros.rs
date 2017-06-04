/// Creates an &'static str from a c string.
///
/// Converts the string at the given address from a c string to a rust &'static
/// str.
/// Optionally if the length is known, the process can be sped up, by passing
/// it.
#[macro_export]
macro_rules! from_c_str {
    ($address: expr, $length: expr) => {{
        use core::str;
        use core::slice;
        unsafe {
            assert_eq!(*(($address + $length) as *const u8), 0);
        }
        let bytes: &[u8] = unsafe {
                                   slice::from_raw_parts($address
                                       as *const u8, $length as usize - 1)
                           };
        str::from_utf8(bytes)
    }};
    ($address: expr) => {{
        let mut address: usize = $address;
        unsafe {
            while *(address as *const u8) != 0 {
                address += 1;
            }
        }
        from_c_str!($address, (address - $address))
    }};
}

/// Converts to a virtual address.
///
/// Converts a given physical address within the kernel part of memory to its
/// corresponding
/// virtual address.
#[macro_export]
#[cfg(target_arch = "x86_64")]
macro_rules! to_virtual {
    ($address: expr) => {{
        const KERNEL_OFFSET: usize = 0xffff800000000000;
        $address as usize + KERNEL_OFFSET
    }};
}

/// Returns true for a valid virtual address.
#[macro_export]
macro_rules! valid_address {
    ($address: expr) => {{
        if cfg!(arch = "x86_64") {
            use arch::x86_64::memory::{VIRTUAL_LOW_MAX_ADDRESS, VIRTUAL_HIGH_MIN_ADDRESS};
            (VIRTUAL_LOW_MAX_ADDRESS >= $address || $address >= VIRTUAL_HIGH_MIN_ADDRESS)
        } else {
            true
        }
    }};
}

