//! Provides an interface for a linked list allocator.

use super::align;
use arch::PAGE_SIZE;
use core::fmt;
use core::mem::{align_of, size_of};
use memory::{map_page, unmap_page, Address, VirtualAddress, READABLE, WRITABLE};

/// The linked list allocator interface.
pub struct LinkedListAllocator {
    /// The maximum address that this allocator can still manage.
    max_address: VirtualAddress,
    /// The first node of the list of blocks.
    first_node: *mut Node,
    /// The last address that is currently mapped.
    end_address: VirtualAddress
}

// The allocator is locked, so this is okay.
unsafe impl Send for LinkedListAllocator {}

/// Represents a node in the list of blocks.
pub struct Node {
    /// True if the memory in this block is allocated.
    used: bool,
    /// If it exists a pointer to the next node in the list.
    next_node: Option<*mut Node>
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let used_string = if self.used { "Used" } else { "Free" };
        let start_address = self as *const _ as usize;
        if self.next_node.is_none() {
            write!(f, "{}: [{:x}..]", used_string, start_address)
        } else {
            let end_address = self.next_node.unwrap() as usize;
            write!(
                f,
                "{}: [{:x}..{:x}]",
                used_string, start_address, end_address
            )
        }
    }
}

impl Node {
    /// Determines the space wasted by using this node for the given allocation.
    ///
    /// Returns None if the node cannot hold the allocation.
    fn space_waste(
        &self,
        max_address: VirtualAddress,
        size: usize,
        alignment: usize
    ) -> Option<usize> {
        if !self.used {
            let start_address =
                VirtualAddress::from_usize(self as *const _ as usize) + size_of::<Node>();
            let aligned_address = align(start_address, alignment);

            if let Some(node_ptr) = self.next_node {
                let end_address = VirtualAddress::from_usize(node_ptr as usize);
                let next_node_start = align(aligned_address + size, align_of::<Node>());
                if end_address == next_node_start {
                    Some(0)
                } else if end_address > next_node_start + size_of::<Node>() {
                    Some(end_address - size - aligned_address)
                } else {
                    None
                }
            } else {
                if max_address > aligned_address + size {
                    Some(0)
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    /// Checks if this node contains the given allocation.
    fn contains_allocation(&self, ptr: *mut u8, _: usize, alignment: usize) -> bool {
        if self.used {
            let start_address =
                VirtualAddress::from_usize(self as *const _ as usize) + size_of::<Node>();
            let aligned_address = align(start_address, alignment);
            VirtualAddress::from_usize(ptr as usize) == aligned_address
        } else {
            false
        }
    }

    /// Splits this node into two nodes, to fit the given allocation.
    ///
    /// This assumes that the node fits into the space.
    ///
    /// This returns the start address of the allocated area.
    fn split(
        &mut self,
        end_address: &mut VirtualAddress,
        size: usize,
        alignment: usize
    ) -> *mut u8 {
        let start_address =
            VirtualAddress::from_usize(self as *const _ as usize) + size_of::<Node>();
        let aligned_address = align(start_address, alignment);
        let next_node_start = align(aligned_address + size, align_of::<Node>());

        // Map all the necessary pages.
        while next_node_start + size_of::<Node>() > *end_address {
            map_page(*end_address, READABLE | WRITABLE);
            *end_address = (*end_address) + PAGE_SIZE;
        }

        if self.next_node.is_none()
            || VirtualAddress::from_usize(self.next_node.unwrap() as usize) != next_node_start
        {
            let next_node: &mut Node = unsafe { &mut *(next_node_start.as_mut_ptr()) };
            next_node.used = false;
            next_node.next_node = self.next_node;
            self.next_node = Some(next_node);
        }
        self.used = true;

        aligned_address.as_mut_ptr()
    }

    /// Merges the nodes, if they are free.
    fn merge(
        &mut self,
        end_address: &mut VirtualAddress,
        previous: Option<&mut Node>,
        next: Option<&mut Node>
    ) {
        if let Some(next_node) = next {
            if next_node.next_node.is_some() {
                // If the node is between other nodes.
                self.used = false;
                if !next_node.used {
                    // Merge with the next node.
                    self.next_node = next_node.next_node;
                }

                if let Some(previous_node) = previous {
                    // Merge with the previous node.
                    if !previous_node.used {
                        previous_node.next_node = self.next_node;
                    }
                }
            } else {
                // If the end of the list is reached.
                let last_node = if let Some(previous_node) = previous {
                    if !previous_node.used {
                        previous_node
                    } else {
                        self
                    }
                } else {
                    self
                };
                last_node.used = false;
                last_node.next_node = None;

                // Shrink the heap.
                let last_address =
                    VirtualAddress::from_usize(last_node as *mut Node as usize) + size_of::<Node>();
                while (*end_address) - PAGE_SIZE > last_address {
                    *end_address -= PAGE_SIZE;
                    unsafe {
                        unmap_page(*end_address);
                    }
                }
            }
        } else {
            panic!("The last node in the linked list allocator isn't free.");
        }
    }
}

/// Provides an iterator for the linked list allocator.
pub struct LinkedListIterator<'a> {
    /// The current node in the iterator.
    current_node: &'a mut Node,
    /// An indicator whether current node is the last node.
    last_node: bool
}

impl<'a> Iterator for LinkedListIterator<'a> {
    type Item = &'a mut Node;

    fn next(&mut self) -> Option<&'a mut Node> {
        match self.current_node.next_node {
            Some(node_ptr) => {
                let old_node = self.current_node as *mut Node;
                self.current_node = unsafe { &mut *node_ptr };
                Some(unsafe { &mut *old_node })
            },
            None => {
                if self.last_node {
                    None
                } else {
                    self.last_node = true;
                    let node_ptr = self.current_node as *mut Node;
                    Some(unsafe { &mut *node_ptr })
                }
            },
        }
    }
}

impl<'a> IntoIterator for &'a mut LinkedListAllocator {
    type Item = &'a mut Node;
    type IntoIter = LinkedListIterator<'a>;

    fn into_iter(self) -> LinkedListIterator<'a> {
        LinkedListIterator {
            current_node: unsafe { &mut *self.first_node },
            last_node: false
        }
    }
}

impl LinkedListAllocator {
    /// Creates a new linked list allocator.
    pub fn new(start_address: VirtualAddress, max_address: VirtualAddress) -> LinkedListAllocator {
        assert_has_not_been_called!("There should only be one linked list allocator.");
        map_page(start_address, READABLE | WRITABLE);

        let first_node: &mut Node = unsafe { &mut *(start_address.as_mut_ptr()) };

        first_node.used = false;
        first_node.next_node = None;

        LinkedListAllocator {
            max_address,
            first_node,
            end_address: start_address + PAGE_SIZE
        }
    }

    /// Returns an iterator over the elements.
    pub fn iter_mut(&mut self) -> LinkedListIterator {
        LinkedListIterator {
            current_node: unsafe { &mut *self.first_node },
            last_node: false
        }
    }

    /// Allocates the first chunk of memory that fits the given size and
    /// alignment.
    pub fn allocate_first_fit(&mut self, size: usize, alignment: usize) -> *mut u8 {
        let max_address = self.max_address;
        let mut end_address = self.end_address;
        let mut return_address = 0 as *mut u8;

        for entry in self.iter_mut() {
            if let Some(_) = entry.space_waste(max_address, size, alignment) {
                return_address = entry.split(&mut end_address, size, alignment);
                break;
            }
        }

        self.end_address = end_address;

        return_address
    }

    /// Frees the previously allocated memory chunk pointed to by ptr.
    pub fn free(&mut self, ptr: *mut u8, size: usize, alignment: usize) {
        let mut end_address = self.end_address;

        {
            let mut iterator = self.iter_mut();
            let mut previous = iterator.next().unwrap();

            if previous.contains_allocation(ptr, size, alignment) {
                // The first entry is freed.
                previous.merge(&mut end_address, None, iterator.next());
            } else {
                // Some other entry is freed.
                while let Some(entry) = iterator.next() {
                    if entry.contains_allocation(ptr, size, alignment) {
                        // Entry is the entry to be freed.
                        entry.merge(&mut end_address, Some(previous), iterator.next());
                        break;
                    }
                    previous = entry;
                }
            }
        }

        self.end_address = end_address;
    }
}
