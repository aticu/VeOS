//! Handles mutual exclusion to data.
//!
//! This is a modification of the Mutex code from the spin crate (see
//! https://crates.io/crates/spin).

use super::{PreemptionState, cpu_relax, disable_preemption, restore_preemption_state};
use core::cell::UnsafeCell;
use core::default::Default;
use core::fmt;
use core::marker::Sync;
use core::ops::{Deref, DerefMut, Drop};
use core::option::Option::{self, None, Some};
use core::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};

/// This type provides MUTual EXclusion based on spinning.
///
/// # Description
///
/// This structure behaves a lot like a normal Mutex. There are some
/// differences:
///
/// - It may be used outside the runtime.
/// - A normal mutex will fail when used without the runtime, this will just
/// lock
/// - When the runtime is present, it will call the deschedule function when
/// appropriate
/// - No lock poisoning. When a fail occurs when the lock is held, no
/// guarantees are made
///
/// When calling rust functions from bare threads, such as C `pthread`s, this
/// lock will be very
/// helpful. In other cases however, you are encouraged to use the locks from
/// the standard
/// library.
///
/// # Simple examples
///
/// ```
/// use spin;
/// let spin_mutex = spin::Mutex::new(0);
///
/// // Modify the data
/// {
///     let mut data = spin_mutex.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spin_mutex.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
///
/// # Thread-safety example
///
/// ```
/// use spin;
/// use std::sync::{Arc, Barrier};
///
/// let numthreads = 1000;
/// let spin_mutex = Arc::new(spin::Mutex::new(0));
///
/// // We use a barrier to ensure the readout happens after all writing
/// let barrier = Arc::new(Barrier::new(numthreads + 1));
///
/// for _ in (0..numthreads)
/// {
///     let my_barrier = barrier.clone();
///     let my_lock = spin_mutex.clone();
///     std::thread::spawn(move||
///     {
///         let mut guard = my_lock.lock();
///         *guard += 1;
///
///         // Release the lock to prevent a deadlock
///         drop(guard);
///         my_barrier.wait();
///     });
/// }
///
/// barrier.wait();
///
/// let answer = { *spin_mutex.lock() };
/// assert_eq!(answer, numthreads);
/// ```
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    preemption_state: UnsafeCell<PreemptionState>,
    data: UnsafeCell<T>
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    preemption_state: &'a PreemptionState,
    data: &'a mut T
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new spinlock wrapping the supplied data.
    ///
    /// May be used statically:
    ///
    /// ```
    /// #![feature(const_fn)]
    /// use spin;
    ///
    /// static MUTEX: spin::Mutex<()> = spin::Mutex::new(());
    ///
    /// fn demo() {
    ///     let lock = MUTEX.lock();
    ///     // do something with lock
    ///     drop(lock);
    /// }
    /// ```
    pub const fn new(user_data: T) -> Mutex<T> {
        Mutex {
            lock: ATOMIC_BOOL_INIT,
            preemption_state: UnsafeCell::new(PreemptionState::default()),
            data: UnsafeCell::new(user_data)
        }
    }

    /// Consumes this mutex, returning the underlying data.
    #[allow(dead_code)]
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let Mutex { data, .. } = self;
        unsafe { data.into_inner() }
    }
}

impl<T: ?Sized> Mutex<T> {
    fn obtain_lock(&self) {
        // while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false
        //
        let mut preemption_state;
        loop {
            unsafe {
                preemption_state = disable_preemption();
            }
            let lock_switch = !self.lock
                                   .compare_and_swap(false, true, Ordering::Acquire);
            if lock_switch {
                break;
            } else {
                unsafe {
                    restore_preemption_state(&preemption_state);
                }
            }

            // Wait until the lock looks unlocked before retrying
            while self.lock.load(Ordering::Relaxed) {
                cpu_relax();
            }
        }

        unsafe {
            *self.preemption_state.get() = preemption_state;
        }
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    ///
    /// ```
    /// let mylock = spin::Mutex::new(0);
    /// {
    ///     let mut data = mylock.lock();
    ///     // The lock is now locked and the data can be accessed
    ///     *data += 1;
    ///     // The lock is implicitly dropped
    /// }
    ///
    /// ```
    pub fn lock(&self) -> MutexGuard<T> {
        self.obtain_lock();
        MutexGuard {
            lock: &self.lock,
            preemption_state: unsafe { &*self.preemption_state.get() },
            data: unsafe { &mut *self.data.get() }
        }
    }

    /// Force unlock the spinlock.
    ///
    /// This is *extremely* unsafe if the lock is not held by the current
    /// thread. However, this can be useful in some instances for exposing the
    /// lock to FFI that doesn't know how to deal with RAII.
    ///
    /// If the lock isn't held, this is a no-op.
    #[allow(dead_code)]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
        restore_preemption_state(&*self.preemption_state.get());
    }

    /// Tries to lock the mutex. If it is already locked, it will return None.
    /// Otherwise it returns
    /// a guard within Some.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        let preemption_state = unsafe { disable_preemption() };
        let lock_switch = !self.lock
                               .compare_and_swap(false, true, Ordering::Acquire);
        // if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false
        if lock_switch {
            unsafe {
                *self.preemption_state.get() = preemption_state;
            }
            Some(MutexGuard {
                     lock: &self.lock,
                     preemption_state: unsafe { &*self.preemption_state.get() },
                     data: unsafe { &mut *self.data.get() }
                 })
        } else {
            unsafe {
                restore_preemption_state(&preemption_state);
            }
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "Mutex {{ data: {:?} }}", &*guard),
            None => write!(f, "Mutex {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created
    /// from.
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
        unsafe {
            restore_preemption_state(self.preemption_state);
        }
    }
}
