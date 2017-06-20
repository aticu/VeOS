//! Handles configuration of the Local Advanced Programmable Interrupt
//! Controller (LAPIC).

use memory::{NO_CACHE, PhysicalAddress, READABLE, VirtualAddress, WRITABLE, map_page_at};

/// The physical base address of the memory mapped LAPIC.
const LAPIC_BASE: PhysicalAddress = 0xfee00000;

/// The offset for the CMCI interrupt LVT register.
const CMCI_INTERRUPT: usize = 0x2f0;

/// The offset for the timer interrupt LVT register.
const TIMER_INTERRUPT: usize = 0x320;

/// The offset for the thernal sensor interrupt LVT register.
const THERMAL_SENSOR_INTERRUPT: usize = 0x330;

/// The offset for the performance counter interrupt LVT register.
const PERFORMANCE_COUNTER_INTERRUPT: usize = 0x340;

/// The offset for the local interrupt 0 LVT register.
const LINT0_INTERRUPT: usize = 0x350;

/// The offset for the local interrupt 1 LVT register.
const LINT1_INTERRUPT: usize = 0x360;

/// The offset for the error interrupt LVT register.
const ERROR_INTERRUPT: usize = 0x370;

/// The offset for the spurious interrupt register.
const SPURIOUS_INTERRUPT: usize = 0xf0;

/// The offset for the timer inital count register.
const TIMER_INITIAL_COUNT: usize = 0x380;

/// The offset for the end of interrupt register.
const END_OF_INTERRUPT: usize = 0xb0;

/// Initializes the LAPIC.
pub fn init() {
    assert_has_not_been_called!("The LAPIC should only be initialized once.");

    map_page_at(to_virtual!(LAPIC_BASE),
                LAPIC_BASE,
                READABLE | WRITABLE | NO_CACHE);

    let mut inactive_register = LVTRegister::new();
    inactive_register.set_inactive();

    let mut lint0_register = LVTRegister::new();
    lint0_register.set_delivery_mode(EXTINT_DELIVERY_MODE);
    lint0_register.set_trigger_mode(LEVEL_SENSITIVE);
    lint0_register.set_inactive();

    let mut lint1_register = LVTRegister::new();
    lint1_register.set_delivery_mode(NMI_DELIVERY_MODE);
    lint1_register.set_trigger_mode(EDGE_SENSITIVE);

    let mut timer_register = LVTRegister::new();
    timer_register.set_timer_mode(PERIODIC_TIMER_MODE);
    timer_register.set_vector(0x20);

    unsafe {
        set_lvt_register(CMCI_INTERRUPT, inactive_register);
        set_lvt_register(THERMAL_SENSOR_INTERRUPT, inactive_register);
        set_lvt_register(PERFORMANCE_COUNTER_INTERRUPT, inactive_register);
        set_lvt_register(ERROR_INTERRUPT, inactive_register);

        set_lvt_register(LINT0_INTERRUPT, lint0_register);
        set_lvt_register(LINT1_INTERRUPT, lint1_register);

        set_lvt_register(TIMER_INTERRUPT, timer_register);
        set_register(TIMER_INITIAL_COUNT, 0);

        set_register(SPURIOUS_INTERRUPT, 0x12f);

        set_lvt_register(LINT0_INTERRUPT, lint0_register);
        set_lvt_register(LINT1_INTERRUPT, lint1_register);
    }
}

/// Signals the end of the interrupt handler to the LAPIC.
pub fn signal_eoi() {
    unsafe {
        set_register(END_OF_INTERRUPT, 0);
    }
}

/// Sets the periodic lapic timer to the specified delay in milliseconds.
pub fn set_periodic_timer(delay: u32) {
    // TODO: Measure this more accurately.
    unsafe {
        set_register(TIMER_INITIAL_COUNT, delay * 1000000);
    }
}

/// Returns the base address for the LAPIC of this CPU.
fn get_lapic_base() -> VirtualAddress {
    to_virtual!(LAPIC_BASE)
}

/// Sets a LAPIC register.
unsafe fn set_register(offset: usize, value: u32) {
    assert!(offset < 0x1000);

    *((get_lapic_base() + offset) as *mut u32) = value;
}

/// Sets an LVT register.
unsafe fn set_lvt_register(offset: usize, register: LVTRegister) {
    set_register(offset, register.0);
}

/// Represents a register belonging to the local vector table.
#[repr(C)]
#[derive(Clone, Copy)]
struct LVTRegister(u32);

bitflags! {
    /// Contains the possible flags for an LVT register.
    flags LVTRegisterFlags: u32 {
        /// Corresponds to the interrupt vector in the IVT.
        const VECTOR = 0xff,
        /// The delivery mode of the interrupt.
        const DELIVERY_MODE = 0b111 << 8,
        /// Delivers the interrupt to the specified vector.
        const FIXED_DELIVERY_MODE = 0b000 << 8,
        /// Delivers an SMI interrupt.
        const SMI_DELIVERY_MODE = 0b010 << 8,
        /// Delivers an NMI interrupt.
        const NMI_DELIVERY_MODE = 0b100 << 8,
        /// For external interrupts.
        const EXTINT_DELIVERY_MODE = 0b111 << 8,
        /// Delivers an INIT request.
        const INIT_DELIVERY_MODE = 0b101 << 8,
        /// The delivery status of the interrupt.
        ///
        /// Read only.
        const DELIVERY_STATUS = 1 << 12,
        /// Specifies when the pin is active.
        const PIN_POLARITY = 1 << 13,
        /// The pin is active when high.
        const HIGH_ACTIVE_PIN_POLARITY = 0 << 13,
        /// The pin is active when low.
        const LOW_ACTIVE_PIN_POLARITY = 1 << 13,
        /// Indicates if the interrupt is being serviced.
        ///
        /// Read only.
        const REMOTRE_IRR = 1 << 14,
        /// Specifies the trigger mode for the interrupt.
        const TRIGGER_MODE = 1 << 15,
        /// For edge sensitive interrupts.
        const EDGE_SENSITIVE = 0 << 15,
        /// For level sensitive interrupts.
        const LEVEL_SENSITIVE = 1 << 15,
        /// Masks the interrupt.
        const MASK = 1 << 16,
        /// Sets the mode for the timer.
        ///
        /// Only valid for the timer interrupt register.
        const TIMER_MODE = 0b11 << 17,
        /// Sets the timer as a one shot timer.
        const ONE_SHOT_TIMER_MODE = 0b00 << 17,
        /// Sets the timer to be periodic.
        const PERIODIC_TIMER_MODE = 0b01 << 17,
        /// Sets the timer to a deadline timer.
        const DEADLINE_TIMER_MODE = 0b10 << 17
    }
}

impl LVTRegister {
    /// Creates a new LVT register.
    fn new() -> LVTRegister {
        let mut register = LVTRegister(0);
        register.set_active();
        register.set_delivery_mode(FIXED_DELIVERY_MODE);

        register
    }

    /// Sets the vector of this interrupt.
    fn set_vector(&mut self, num: u8) {
        self.0 &= !VECTOR.bits();
        self.0 |= num as u32;
    }

    /// Sets the delivery mode for this interrupt.
    fn set_delivery_mode(&mut self, mode: LVTRegisterFlags) {
        self.0 &= !DELIVERY_MODE.bits();
        self.0 |= mode.bits();
    }

    /// Sets the trigger mode for this interrupt.
    fn set_trigger_mode(&mut self, mode: LVTRegisterFlags) {
        self.0 &= !TRIGGER_MODE.bits();
        self.0 |= mode.bits();
    }

    /// Deactivates this interrupt.
    fn set_inactive(&mut self) {
        self.0 |= MASK.bits();
    }

    /// Activates this interrupt.
    fn set_active(&mut self) {
        self.0 &= !MASK.bits();
    }

    /// Sets the timer mode for this interrupt.
    fn set_timer_mode(&mut self, timer_mode: LVTRegisterFlags) {
        self.0 &= !TIMER_MODE.bits();
        self.0 |= timer_mode.bits();
    }
}
