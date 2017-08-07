//! This module defines a process control block (PCB).

use memory::address_space::AddressSpace;

/// A process control block (PCB) holds all data required to manage a process.
pub struct PCB {
    /// The address space of the process.
    pub address_space: AddressSpace
}

impl PCB {
    /// Creates a new PCB with the given parameters.
    pub fn new(address_space: AddressSpace) -> PCB {
        PCB { address_space }
    }

    /// Creates a pcb for the idle threads.
    pub fn idle_pcb() -> PCB {
        assert_has_not_been_called!("There should only be one idle PCB.");
        PCB { address_space: AddressSpace::idle_address_space() }
    }
}
