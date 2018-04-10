//! Handles configuration of the Local Advanced Programmable Interrupt
//! Controller (LAPIC).

use super::super::memory::map_page_at;
use super::{IRQ8_INTERRUPT_TICKS, SPURIOUS_INTERRUPT_HANDLER_NUM, TIMER_INTERRUPT_HANDLER_NUM};
use memory::{PhysicalAddress, VirtualAddress, NO_CACHE, READABLE, WRITABLE};
use raw_cpuid::CpuId;
use sync::{disable_preemption, restore_preemption_state};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::{inb, outb};

/// The physical base address of the memory mapped LAPIC.
const LAPIC_BASE: PhysicalAddress = PhysicalAddress::from_const(0xfee00000);

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

/// The offset for the timer current count register.
const TIMER_CURRENT_COUNT: usize = 0x390;

/// The offset for the task priority register.
const TASK_PRIORITY_REGISTER: usize = 0x80;

/// The offset for the interrupt command register (bits 0-31).
const INTERRUPT_COMMAND_REGISTER_LOW: usize = 0x300;

/// The offset for the interrupt command register (bits 32-63).
const INTERRUPT_COMMAND_REGISTER_HIGH: usize = 0x310;

/// The offset for the end of interrupt register.
const END_OF_INTERRUPT: usize = 0xb0;

/// The offset of the logical destination register.
const LOGICAL_DESTINATION_REGISTER: usize = 0xd0;

/// The offset of the destination format register.
const DESTINATION_FORMAT_REGISTER: usize = 0xe0;

// TODO: This assumes the LAPICS on all CPUs have the same frequency.
/// The amount of LAPIC timer ticks per milliseconds. Measured at runtime.
///
/// This value is initialized to the value that qemu uses.
static mut TICKS_PER_MS: u32 = 1000000;

/// Initializes the LAPIC.
pub fn init() {
    assert_has_not_been_called!("The LAPIC should only be initialized once.");

    map_page_at(get_lapic_base(), LAPIC_BASE, READABLE | WRITABLE | NO_CACHE);

    let cpu_id = CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id();
    let logical_id = cpu_id % 8;

    let mut inactive_register = LVTRegister::new();
    inactive_register.set_inactive();

    let mut lint0_register = LVTRegister::new();
    lint0_register.set_delivery_mode(EXTINT_DELIVERY_MODE);
    lint0_register.set_trigger_mode(LEVEL_SENSITIVE);

    let mut lint1_register = LVTRegister::new();
    lint1_register.set_delivery_mode(NMI_DELIVERY_MODE);
    lint1_register.set_trigger_mode(EDGE_SENSITIVE);

    let mut timer_register = LVTRegister::new();
    timer_register.set_timer_mode(PERIODIC_TIMER_MODE);
    timer_register.set_vector(TIMER_INTERRUPT_HANDLER_NUM);

    unsafe {
        // Deactivate currently unused interrupts.
        set_lvt_register(CMCI_INTERRUPT, inactive_register);
        set_lvt_register(THERMAL_SENSOR_INTERRUPT, inactive_register);
        set_lvt_register(PERFORMANCE_COUNTER_INTERRUPT, inactive_register);
        set_lvt_register(ERROR_INTERRUPT, inactive_register);

        // Set the local interrupt registers.
        set_lvt_register(LINT0_INTERRUPT, lint0_register);
        set_lvt_register(LINT1_INTERRUPT, lint1_register);

        // Set the timer interrupt register.
        set_lvt_register(TIMER_INTERRUPT, timer_register);
        set_register(TIMER_INITIAL_COUNT, 0);

        // Enable the LAPIC.
        set_register(
            SPURIOUS_INTERRUPT,
            0x100 + SPURIOUS_INTERRUPT_HANDLER_NUM as u32
        );

        // Set the local interrupt registers again, to make sure they have the right
        // value.
        set_lvt_register(LINT0_INTERRUPT, lint0_register);
        set_lvt_register(LINT1_INTERRUPT, lint1_register);

        // Use flat logical destinations.
        set_register(DESTINATION_FORMAT_REGISTER, 0b1111 << 28);

        // Set the processor to its logical destination address.
        set_register(LOGICAL_DESTINATION_REGISTER, (logical_id as u32) << 24);
    }
}

/// Calibrates the timer to work properly.
pub fn calibrate_timer() {
    let measure_accuracy_in_ms = 125;

    // Use the RTC to calibrate the LAPIC timer.
    unsafe {
        // Save the NMI enable state to restore it later.
        let nmi_bit = inb(0x70) & 0x80;

        // Read the previous value of status register b.
        outb(0x70, 0x8b);
        let previous_b = inb(0x71);

        // Enable the RTC interrupts with the default frequency of 1024hz.
        outb(0x70, 0x8b);
        outb(0x71, previous_b | 0x40);

        // Read status register c to indicate the interrupt being handled. Just in case.
        outb(0x70, 0x8c);
        inb(0x71);

        let start_tick = *IRQ8_INTERRUPT_TICKS.lock();
        let end_tick = start_tick + 1024 * measure_accuracy_in_ms / 1000;

        // Enable interrupts.
        interrupts::enable();

        // Start LAPIC timer for comparison.
        set_register(TIMER_INITIAL_COUNT, <u32>::max_value());

        // Wait until the specified amount of time has passed.
        while *IRQ8_INTERRUPT_TICKS.lock() < end_tick {
            asm!("pause" : : : : "intel", "volatile");
        }

        // Measure LAPIC timer ticks.
        let timer_ticks_passed = <u32>::max_value() - get_register(TIMER_CURRENT_COUNT);

        // Disable interrupts again.
        interrupts::disable();

        TICKS_PER_MS = timer_ticks_passed / measure_accuracy_in_ms as u32;

        // Disable RTC interrupts after we're done.
        outb(0x70, 0x8b);
        outb(0x71, previous_b);

        // Restore the NMI state.
        outb(0x70, nmi_bit);
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
    unsafe {
        set_register(TIMER_INITIAL_COUNT, delay * TICKS_PER_MS);
    }
}

/// Sets the task priority for the local APIC.
pub fn set_priority(value: u8) {
    unsafe {
        set_register(TASK_PRIORITY_REGISTER, value as u32);
    }
}

/// Gets the current task priority for the local APIC.
pub fn get_priority() -> u8 {
    unsafe { get_register(TASK_PRIORITY_REGISTER) as u8 }
}

/// Sets the ICR to the specified value.
fn set_icr(value: u64) {
    let value_low = value as u32;
    let value_high = (value >> 32) as u32;

    unsafe {
        let preemption_state = disable_preemption();

        set_register(INTERRUPT_COMMAND_REGISTER_HIGH, value_high);
        set_register(INTERRUPT_COMMAND_REGISTER_LOW, value_low);

        restore_preemption_state(&preemption_state);
    }
}

/// Returns the base address for the LAPIC of this CPU.
fn get_lapic_base() -> VirtualAddress {
    LAPIC_BASE.to_virtual()
}

/// Sets a LAPIC register.
///
/// # Safety
/// - Ensure the LAPIC is mapped.
/// - Setting registers incorrectly can cause interrupts to behave unexpected.
unsafe fn set_register(offset: usize, value: u32) {
    assert!(offset < 0x1000);

    *(get_lapic_base() + offset).as_mut_ptr() = value;
}

/// Gets a LAPIC register.
///
/// # Safety
/// - Ensure the LAPIC is mapped.
unsafe fn get_register(offset: usize) -> u32 {
    assert!(offset < 0x1000);

    *(get_lapic_base() + offset).as_mut_ptr()
}

/// Sets an LVT register.
///
/// # Safety
/// - Ensure the LAPIC is mapped.
/// - Setting registers incorrectly can cause interrupts to behave unexpected.
unsafe fn set_lvt_register(offset: usize, register: LVTRegister) {
    set_register(offset, register.0);
}

/// Issues an interrupt to the current CPU.
pub fn issue_self_interrupt(vector: u8) {
    issue_interrupt(SELF, vector);
}

/// Issues the given interrupt for the given target(s).
fn issue_interrupt(target: InterruptDestinationMode, vector: u8) {
    assert!(target.intersects(SELF | ALL | ALL_EXCLUDING_SELF));

    let mut icr = target.bits();
    icr |= vector as u64;

    set_icr(icr);
}

bitflags! {
    /// The possible destination modes for interrupts.
    flags InterruptDestinationMode: u64 {
        /// The destination address for the interrupt is logical.
        const LOGICAL = 1 << 11,
        /// The destination address for the interrupt is physical.
        const PHYSICAL = 0 << 11,
        /// The interrupt addresses the only the current CPU.
        const SELF = 0b01 << 18,
        /// The interrupt addresses all CPUS.
        const ALL = 0b10 << 18,
        /// The interrupt addresses all but the current CPU.
        const ALL_EXCLUDING_SELF = 0b11 << 18
    }
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
