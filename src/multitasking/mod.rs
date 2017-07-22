//! Manages multitasking in the operating system.

mod tcb;
mod stack;
pub mod scheduler;
mod cpu_local;

pub use self::cpu_local::{CPULocal, CPULocalMut};
pub use self::scheduler::CURRENT_THREAD;
pub use self::stack::{Stack, StackType};
pub use self::tcb::{TCB, ThreadState};
use alloc::binary_heap::BinaryHeap;
pub use arch::{get_cpu_id, get_cpu_num};
use sync::Mutex;

lazy_static! {
    pub static ref READY_LIST: Mutex<BinaryHeap<TCB>> = Mutex::new(BinaryHeap::from(vec![
                                         TCB::test(1, thread as u64, 10, '1' as u64, 0, 0, 0, 0),
                                         TCB::test(2, thread as u64, 20, '2' as u64, 0, 0, 0, 0),
                                         TCB::test(3, thread as u64, 30, '3' as u64, 0, 0, 0, 0),
                                         TCB::test(4, thread as u64, 40, '4' as u64, 0, 0, 0, 0)
]));
}

pub fn thread(amount: u64, character: char) {
    let mut curr_amount = 0;
    while curr_amount < amount {
        unsafe {
            asm!("mov rax, 0
                  mov edi, $0
                  syscall"
                  : : "r"(character) : "rax", "rcx", "r11", "r10" : "intel", "volatile");
        }
        curr_amount += 1;
        let mut curr_val = 0;
        while curr_val < 3000000 {
            let _ = 13434;
            curr_val += 1;
        }
    }
    unsafe {
        asm!("mov rax, 1
              syscall"
              : : : : "intel", "volatile");
    }
}
