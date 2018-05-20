//! Manages multitasking in the operating system.

mod cpu_local;
mod pcb;
pub mod scheduler;
pub mod stack;
mod tcb;

pub use self::cpu_local::{CPULocal, CPULocalMut};
pub use self::pcb::{get_current_process, PCB};
pub use self::scheduler::CURRENT_THREAD;
pub use self::stack::{Stack, StackType};
pub use self::tcb::{ThreadState, TCB};
use alloc::btree_map::BTreeMap;
use arch::{self, Architecture};
use memory::address_space::AddressSpace;
use memory::VirtualAddress;
use sync::mutex::MutexGuard;
use sync::Mutex;

/// The type of a process ID.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ProcessID(usize);

impl From<usize> for ProcessID {
    fn from(id: usize) -> ProcessID {
        ProcessID(id)
    }
}

impl From<ProcessID> for usize {
    fn from(id: ProcessID) -> usize {
        id.0
    }
}

/// The type of a thread ID.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ThreadID(usize);

impl From<usize> for ThreadID {
    fn from(id: usize) -> ThreadID {
        ThreadID(id)
    }
}

impl From<ThreadID> for usize {
    fn from(id: ThreadID) -> usize {
        id.0
    }
}

lazy_static! {
    /// The list of all the currently running processes.
    static ref PROCESS_LIST: Mutex<BTreeMap<ProcessID, PCB>> = Mutex::new({
        let mut map = BTreeMap::new();
        map.insert(0.into(), PCB::idle_pcb());

        map
    });
}

/// Finds an unused process ID.
fn find_pid(list: &MutexGuard<BTreeMap<ProcessID, PCB>>) -> ProcessID {
    // UNOPTIMIZED
    let mut pid = 1;
    while list.contains_key(&pid.into()) {
        pid += 1;
    }
    pid.into()
}

/// Creates a new process.
pub fn create_process(address_space: AddressSpace, entry_address: VirtualAddress) -> ProcessID {
    let mut pcb = PCB::new(address_space);

    let mut process_list = PROCESS_LIST.lock();
    let id = find_pid(&process_list);

    let first_tcb = TCB::in_process(id, 0.into(), entry_address, &mut pcb);

    scheduler::READY_LIST.lock().push(first_tcb);

    assert!(
        process_list.insert(id, pcb).is_none(),
        "Trying to use an already used {:?}.",
        id
    );

    id
}

/// Returns the id of the current cpu.
pub fn get_cpu_id() -> usize {
    arch::Current::get_cpu_id()
}

/// Returns the number of available cpus.
pub fn get_cpu_num() -> usize {
    arch::Current::get_cpu_num()
}
