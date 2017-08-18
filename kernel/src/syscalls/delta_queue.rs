//! A delta queue to keep track of sleeping threads.

use alloc::boxed::Box;
use multitasking::TCB;

/// Represents a node in a delta queue.
struct DeltaQueueNode {
    /// The thread sleeping in this node.
    thread: TCB,
    /// The time this thread has yet to wait (in ms).
    time: usize,
    /// The next node in the delta queue.
    next_node: Box<Option<DeltaQueueNode>>
}

impl DeltaQueueNode {
    /// Creates a new node for the delta queue.
    fn new(thread: TCB, time: usize, next_node: Box<Option<DeltaQueueNode>>) -> DeltaQueueNode {
        DeltaQueueNode {
            thread,
            time,
            next_node
        }
    }
}

/// Represents a delta queue.
struct DeltaQueue {
    /// The first node in the queue.
    first_node: Option<DeltaQueueNode>
}

impl DeltaQueue {
    /// Inserts the given thread into the delta queue.
    pub fn insert(&mut self, thread: TCB, mut time: usize) {
        if self.first_node.is_none() || self.first_node.as_ref().unwrap().time > time {
            // The entry is the new first entry.
            let first_node = self.first_node.take();
            let new_node = DeltaQueueNode::new(thread, time, Box::new(first_node));
            self.first_node = Some(new_node);
        } else {
            // The entry is inserted somewhere else.
            let mut previous_node = self.first_node.as_mut().unwrap();

            while previous_node.next_node.is_some() {
                time -= previous_node.time;

                if time < previous_node.next_node.as_ref().unwrap().time {
                    let next_node = previous_node.next_node.take();
                    let new_node = DeltaQueueNode::new(thread, time, Box::new(next_node));

                    previous_node.next_node.get_or_insert(new_node);

                    return;
                } else {
                    previous_node = previous_node.next_node.as_mut().as_mut().unwrap();
                }
            }

            time -= previous_node.time;
            // The entry is inserted as the last node.
            *previous_node.next_node = Some(DeltaQueueNode::new(thread, time, Box::new(None)));
        }
    }
}
