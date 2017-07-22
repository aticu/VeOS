//! Provides functionality to manage multiple stacks.

use arch::STACK_TYPE;
use core::cmp::{max, min};
use core::fmt;
use memory::{PAGE_SIZE, READABLE, USER_ACCESSIBLE, VirtualAddress, WRITABLE, map_page, unmap_page};

// NOTE: For now only full descending stacks are supported.
/// Represents the different types of stacks that exist.
pub enum StackType {
    /// The value currently pointed to is used and the stack grows downward.
    FullDescending /* // The value currently pointed to is not used and the stack grows
                    * downward.
                    * EmptyDescending,
                    * // The value currently pointed to is used and the stack grows upward.
                    * FullAscending,
                    * // The value currently pointed to is not used and the stack grows upward.
                    * EmptyAscending */
}

/// Determines the type of accesses possible for this stack.
#[derive(PartialEq)]
pub enum AccessType {
    /// The stack can be accessed by usermode code.
    UserAccessible,
    /// The stack can only be accessed by the kernel.
    KernelOnly
}

/// Represents a stack.
pub struct Stack {
    /// Represents the top address of the stack.
    top_address: VirtualAddress,
    /// Represents the bottom address of the stack.
    bottom_address: VirtualAddress,
    /// Represents the maximum stack size.
    max_size: usize,
    /// Represents the current stack pointer.
    ///
    /// # Note
    /// This is only valid when the stack is not currently in use.
    pub base_stack_pointer: VirtualAddress,
    /// The access type for this stack.
    access_type: AccessType
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Bottom: {:x}, Top: {:x}, Max size: {:x}",
               self.bottom_address,
               self.top_address,
               self.max_size)
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        self.resize(0);
    }
}

impl Stack {
    /// Creates a new stack of size zero with the given start address.
    pub fn new(initial_size: usize,
               max_size: usize,
               start_address: VirtualAddress,
               access_type: AccessType)
               -> Stack {
        let mut stack = match STACK_TYPE {
            StackType::FullDescending => {
                Stack {
                    top_address: start_address,
                    bottom_address: start_address,
                    max_size,
                    base_stack_pointer: start_address,
                    access_type
                }
            }
            //_ => unimplemented!()
        };

        stack.resize(initial_size);

        stack
    }

    /// Grows the stack by the given amount.
    pub fn grow(&mut self, amount: usize) {
        match STACK_TYPE {
            StackType::FullDescending => {
                let new_bottom = max(self.top_address - self.max_size,
                                     self.bottom_address - amount);

                let mut flags = READABLE | WRITABLE;

                if self.access_type == AccessType::UserAccessible {
                    flags |= USER_ACCESSIBLE;
                }

                let first_page_to_map = new_bottom / PAGE_SIZE;

                // This should be one less, but the range is exclusive.
                let last_page_to_map = self.bottom_address / PAGE_SIZE;

                for page_num in first_page_to_map..last_page_to_map {
                    map_page(page_num * PAGE_SIZE, flags);
                }

                self.bottom_address = new_bottom;
            }
            //_ => unimplemented!()
        }
    }

    /// Shrinks the stack by the given amount.
    pub fn shrink(&mut self, amount: usize) {
        match STACK_TYPE {
            StackType::FullDescending => {
                let new_bottom = min(self.top_address, self.bottom_address + amount);

                let first_page_to_unmap = self.bottom_address / PAGE_SIZE;

                // This should be one less, but the range is exclusive.
                let last_page_to_unmap = new_bottom / PAGE_SIZE;

                for page_num in first_page_to_unmap..last_page_to_unmap {
                    unsafe {
                        unmap_page(page_num * PAGE_SIZE);
                    }
                }

                self.bottom_address = new_bottom;
            }
            //_ => unimplemented!()
        }
    }

    /// Resizes the stack to the given size.
    pub fn resize(&mut self, new_size: usize) {
        let current_size = (self.top_address - self.bottom_address) as isize;

        let difference = new_size as isize - current_size;

        if difference > 0 {
            self.grow(difference as usize);
        } else {
            self.shrink(-difference as usize);
        }
    }
}
