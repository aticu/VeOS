//! Handles synchronization within the kernel.
pub mod mutex;

use arch;

/// Saves the state when disabling preemtion, so it can be restored later.
#[derive(Default)]
pub struct PreemptionState {
    /// Saves whether interrupts were enabled, when preemtion was disabled.
    interrupts_enabled: bool
}

impl PreemptionState {
    /// Reads the current state of preemption.
    pub fn new() -> PreemptionState {
        PreemptionState { interrupts_enabled: arch::sync::interrupts_enabled() }
    }

    /// Statically returns a default preemption state.
    pub const fn default() -> PreemptionState {
        PreemptionState { interrupts_enabled: false }
    }

    /// Restores the saved preemption state.
    unsafe fn restore(&self) {
        arch::set_interrupt_state(self.interrupts_enabled);
    }
}

/// Lightenes CPU load in spin locks.
#[inline(always)]
pub fn cpu_relax() {
    arch::sync::cpu_relax();
}

/// Disables preemption and returns the previous state.
pub unsafe fn disable_preemption() -> PreemptionState {
    let state = PreemptionState::new();

    arch::sync::disable_interrupts();

    state
}

/// Reenables preemption to the saved state.
pub unsafe fn restore_preemption_state(state: &PreemptionState) {
    state.restore();
}

/// Enables preemption.
pub unsafe fn enable_preemption() {
    arch::sync::enable_interrupts();
}
