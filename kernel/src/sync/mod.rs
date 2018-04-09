//! Handles synchronization within the kernel.

pub mod mutex;
pub mod time;

pub use self::mutex::Mutex;
use arch;

/// Saves the state when disabling preemtion, so it can be restored later.
#[derive(Default)]
pub struct PreemptionState {
    /// Saves whether interrupts were enabled, when preemtion was disabled.
    interrupts_enabled: bool
}

impl PreemptionState {
    /// Reads the current state of preemptability.
    fn new() -> PreemptionState {
        PreemptionState { interrupts_enabled: arch::interrupts_enabled() }
    }

    /// Statically returns a default preemption state.
    const fn default() -> PreemptionState {
        PreemptionState { interrupts_enabled: false }
    }

    /// Restores the saved preemption state.
    unsafe fn restore(&self) {
        arch::set_interrupt_state(self.interrupts_enabled);
    }

    /// Copies the preemption state.
    ///
    /// # Safety
    /// - Make sure that every preemption state is properly restored only once.
    pub unsafe fn copy(&self) -> PreemptionState {
        PreemptionState { interrupts_enabled: self.interrupts_enabled }
    }
}

/// Lightenes CPU load in spin locks.
#[inline(always)]
pub fn cpu_relax() {
    arch::cpu_relax();
}

/// Halts the CPU.
///
/// # Safety
/// - If preemption is disabled, the execution can never be returned.
#[inline(always)]
pub unsafe fn cpu_halt() {
    arch::cpu_halt();
}

/// Disables preemption and returns the previous state.
///
/// # Safety
/// - The returned `PreemptionState` must be restored.
pub unsafe fn disable_preemption() -> PreemptionState {
    let state = PreemptionState::new();

    arch::disable_interrupts();

    state
}

/// Unconditionally enables preemption.
///
/// # Safety
/// This should only be done during initialization. Otherwise the preemption
/// state that
/// was returned by the disable function should be restored.
pub unsafe fn enable_preemption() {
    arch::enable_interrupts();
}

/// Reenables preemption to the saved state.
///
/// # Safety
/// - No locks should be held when restoring the `PreemptionState`.
pub unsafe fn restore_preemption_state(state: &PreemptionState) {
    state.restore();
}
