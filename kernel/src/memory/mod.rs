//! Handles all memory related things.

pub mod address_space;
pub mod allocator;

pub use arch::get_kernel_area;
pub use arch::get_page_flags;
pub use arch::is_userspace_address;
pub use arch::map_page;
pub use arch::unmap_page;
pub use arch::KERNEL_STACK_AREA_BASE;
pub use arch::KERNEL_STACK_MAX_SIZE;
pub use arch::KERNEL_STACK_OFFSET;
pub use arch::PAGE_SIZE;
pub use arch::USER_STACK_AREA_BASE;
pub use arch::USER_STACK_MAX_SIZE;
pub use arch::USER_STACK_OFFSET;

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Represents something that can act like an address.
pub trait Address: PartialOrd + Ord + Add<usize, Output = Self> + Sized + Clone + Copy {
    /// Returns the value of the address as a `usize`.
    #[inline(always)]
    fn as_usize(&self) -> usize;

    /// Creates a value of the address type from a `usize`.
    #[inline(always)]
    fn from_usize(usize) -> Self;

    /// Aligns the address to the next page border, rounded down.
    fn page_align_down(self) -> Self {
        Self::from_usize(self.as_usize() / PAGE_SIZE * PAGE_SIZE)
    }

    /// Returns the offset of the page from the previous page border.
    fn offset_in_page(self) -> usize {
        self.as_usize() % PAGE_SIZE
    }
}

/// Represents a physical address.
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    /// Returns the physical address corresponding to the constant.
    pub const fn from_const(addr: usize) -> PhysicalAddress {
        PhysicalAddress(addr)
    }

    /// Creates a virtual address from the given physical one.
    pub fn to_virtual(self) -> VirtualAddress {
        VirtualAddress::from_usize(to_virtual!(self.as_usize()))
    }
}

impl Address for PhysicalAddress {
    fn as_usize(&self) -> usize {
        self.0
    }

    fn from_usize(addr: usize) -> PhysicalAddress {
        PhysicalAddress(addr)
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysicalAddress({:x})", self.as_usize())
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn add(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::from_usize(self.as_usize() + rhs)
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn sub(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::from_usize(self.as_usize() - rhs)
    }
}

impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = usize;

    fn sub(self, rhs: PhysicalAddress) -> usize {
        self.as_usize() - rhs.as_usize()
    }
}

impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

/// Represents a virtual address.
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    /// Returns the virtual address corresponding to the constant.
    pub const fn from_const(addr: usize) -> VirtualAddress {
        VirtualAddress(addr)
    }

    /// Returns the start address of the page with the given number.
    pub fn from_page_num(page_num: usize) -> VirtualAddress {
        VirtualAddress::from_usize(page_num * PAGE_SIZE)
    }

    /// Returns the number of the page that the address lies in.
    pub fn page_num(self) -> usize {
        self.as_usize() / PAGE_SIZE
    }

    /// Casts the address as a pointer.
    pub fn as_ptr<T>(self) -> *const T {
        self.as_usize() as *const T
    }

    /// Casts the address as a mutable pointer.
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_usize() as *mut T
    }
}

impl Address for VirtualAddress {
    fn as_usize(&self) -> usize {
        self.0
    }

    fn from_usize(addr: usize) -> VirtualAddress {
        VirtualAddress(addr)
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualAddress({:x})", self.as_usize())
    }
}

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::from_usize(self.as_usize() + rhs)
    }
}

impl AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::from_usize(self.as_usize() - rhs)
    }
}

impl Sub<VirtualAddress> for VirtualAddress {
    type Output = usize;

    fn sub(self, rhs: VirtualAddress) -> usize {
        self.as_usize() - rhs.as_usize()
    }
}

impl SubAssign<usize> for VirtualAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

/// Represents a chunk of virtual memory.
#[derive(Clone, Copy, Default)]
pub struct MemoryArea<AddressType: Address> {
    /// The address at which the chunk starts.
    start_address: AddressType,
    /// The length of the chunk.
    length: usize
}

impl<AddressType: Address> MemoryArea<AddressType> {
    /// Creates a new MemoryArea.
    pub const fn new(start_address: AddressType, length: usize) -> MemoryArea<AddressType> {
        MemoryArea {
            start_address,
            length
        }
    }

    /// Creates a new MemoryArea.
    pub fn from_start_and_end(
        start_address: AddressType,
        end_address: AddressType
    ) -> MemoryArea<AddressType> {
        if start_address > end_address {
            MemoryArea::new(
                start_address,
                start_address.as_usize() - end_address.as_usize()
            )
        } else {
            MemoryArea::new(
                start_address,
                end_address.as_usize() - start_address.as_usize()
            )
        }
    }

    /// Returns the start address of this memory area.
    pub fn start_address(&self) -> AddressType {
        self.start_address
    }

    /// Returns the end address of this memory area.
    ///
    /// The end address is the address of the first byte not contained in it.
    pub fn end_address(&self) -> AddressType {
        self.start_address + self.length
    }

    /// Returns the length in bytes of this memory area.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Checks if the address is contained within the segment.
    fn contains(&self, address: AddressType) -> bool {
        self.start_address() <= address && address < self.end_address()
    }

    /// Checks if the area is contained within another area.
    pub fn is_contained_in(&self, other: MemoryArea<AddressType>) -> bool {
        other.start_address().as_usize() <= self.start_address().as_usize()
            && other.end_address().as_usize() >= self.end_address().as_usize()
    }

    /// Checks if the area overlaps with another area.
    pub fn overlaps_with(&self, other: MemoryArea<AddressType>) -> bool {
        self.contains(other.start_address()) || other.contains(self.start_address())
    }
}

impl MemoryArea<PhysicalAddress> {
    /// Returns the same area except for the first frame.
    pub fn without_first_frame(&self) -> MemoryArea<PhysicalAddress> {
        // The start address should be page aligned.
        assert!(self.start_address.as_usize() % PAGE_SIZE == 0);

        MemoryArea {
            start_address: self.start_address() + PAGE_SIZE,
            length: self.length() - PAGE_SIZE
        }
    }
}

impl MemoryArea<VirtualAddress> {
    /// Creates a constant empty default value for a memory area.
    pub const fn const_default() -> MemoryArea<VirtualAddress> {
        MemoryArea {
            start_address: VirtualAddress::from_const(0),
            length: 0
        }
    }
}

impl<AddressType: Address + fmt::Debug> fmt::Debug for MemoryArea<AddressType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Start: {:?}, Length: {:x}",
            self.start_address,
            self.length()
        )
    }
}

bitflags! {
    /// The flags a page could possibly have.
    pub flags PageFlags: u8 {
        /// Set if the page can be read from.
        const READABLE = 1 << 0,
        /// Set if the page can be written to.
        const WRITABLE = 1 << 1,
        /// Set if code on the page can be executed.
        const EXECUTABLE = 1 << 2,
        /// Set if the page should not be cached.
        const NO_CACHE = 1 << 3,
        /// Set if the page should be accessible from user mode.
        const USER_ACCESSIBLE = 1 << 4,
        /// Set if the page is currently present.
        const PRESENT = 1 << 5
    }
}

/// Initializes the memory managing part of the kernel.
#[cfg(not(test))]
pub fn init() {
    assert_has_not_been_called!("Memory state should only be initialized once.");

    ::arch::memory_init();
}

/// This function gets called when the system is out of memory.
pub fn oom() -> ! {
    panic!("Out of memory!");
}
