//!Handles all x86_64 memory related issues.

mod paging;

///Represents a physical address.
type PhysicalAddress = usize;
///Represents a virtual address.
type VirtualAddress = usize;

pub fn debug() {
    paging::debug();
}
