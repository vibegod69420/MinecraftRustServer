use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

pub type BoxedRunnable = Box<dyn FnOnce() + Send + 'static>;

pub trait StrictQueue<T>: Send + Sync {
    // JAVA: net.minecraft.util.thread.StrictQueue#pop
    fn pop(&self) -> Option<BoxedRunnable>;

    // JAVA: net.minecraft.util.thread.StrictQueue#push
    fn push(&self, value: T) -> bool;

    // JAVA: net.minecraft.util.thread.StrictQueue#isEmpty
    fn is_empty(&self) -> bool;

    // JAVA: net.minecraft.util.thread.StrictQueue#size
    fn size(&self) -> i32;
}

pub struct RunnableWithPriority {
    pub priority: i32,
    pub task: BoxedRunnable,
}

impl RunnableWithPriority {
    // JAVA: net.minecraft.util.thread.StrictQueue$RunnableWithPriority#run
    pub fn run(self) {
        (self.task)();
    }
}

pub struct FixedPriorityQueue {
    queues: Vec<Mutex<VecDeque<BoxedRunnable>>>,
    size: AtomicI32,
}

impl FixedPriorityQueue {
    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#<init>
    pub fn new(priority_count: i32) -> Self {
        let mut queues = Vec::with_capacity(if priority_count > 0 {
            priority_count as usize
        } else {
            0
        });
        let mut i = 0i32;
        while i < priority_count {
            queues.push(Mutex::new(VecDeque::new()));
            i = i.wrapping_add(1);
        }
        Self {
            queues,
            size: AtomicI32::new(0),
        }
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#pop
    pub fn pop(&self) -> Option<BoxedRunnable> {
        for q in &self.queues {
            let mut guard = q.lock().unwrap();
            if let Some(task) = guard.pop_front() {
                drop(guard);
                self.size.fetch_sub(1, Ordering::SeqCst);
                return Some(task);
            }
        }
        None
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#push
    pub fn push(&self, task: RunnableWithPriority) -> bool {
        let p = task.priority;
        let len = self.queues.len() as i32;
        if p < len && p >= 0 {
            self.queues[p as usize]
                .lock()
                .unwrap()
                .push_back(task.task);
            self.size.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            panic!(
                "Priority {} not supported. Expected range [0-{}]",
                p,
                len.wrapping_sub(1)
            );
        }
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#isEmpty
    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::SeqCst) == 0
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#size
    pub fn size(&self) -> i32 {
        self.size.load(Ordering::SeqCst)
    }
}

impl StrictQueue<RunnableWithPriority> for FixedPriorityQueue {
    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#pop
    fn pop(&self) -> Option<BoxedRunnable> {
        FixedPriorityQueue::pop(self)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#push
    fn push(&self, value: RunnableWithPriority) -> bool {
        FixedPriorityQueue::push(self, value)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#isEmpty
    fn is_empty(&self) -> bool {
        FixedPriorityQueue::is_empty(self)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#size
    fn size(&self) -> i32 {
        FixedPriorityQueue::size(self)
    }
}

pub struct QueueStrictQueue {
    queue: Mutex<VecDeque<BoxedRunnable>>,
}

impl QueueStrictQueue {
    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#<init>
    pub fn new(queue: VecDeque<BoxedRunnable>) -> Self {
        Self {
            queue: Mutex::new(queue),
        }
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#pop
    pub fn pop(&self) -> Option<BoxedRunnable> {
        self.queue.lock().unwrap().pop_front()
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#push
    pub fn push(&self, task: BoxedRunnable) -> bool {
        self.queue.lock().unwrap().push_back(task);
        true
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#isEmpty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#size
    pub fn size(&self) -> i32 {
        self.queue.lock().unwrap().len() as i32
    }
}

impl StrictQueue<BoxedRunnable> for QueueStrictQueue {
    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#pop
    fn pop(&self) -> Option<BoxedRunnable> {
        QueueStrictQueue::pop(self)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#push
    fn push(&self, value: BoxedRunnable) -> bool {
        QueueStrictQueue::push(self, value)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#isEmpty
    fn is_empty(&self) -> bool {
        QueueStrictQueue::is_empty(self)
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$QueueStrictQueue#size
    fn size(&self) -> i32 {
        QueueStrictQueue::size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn fixed_priority_queue_orders_by_priority() {
        let q = FixedPriorityQueue::new(3);
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
        for (prio, val) in [(2i32, 20i32), (0, 0), (1, 10), (0, 1)] {
            let log = Arc::clone(&log);
            q.push(RunnableWithPriority {
                priority: prio,
                task: Box::new(move || log.lock().unwrap().push(val)),
            });
        }
        while let Some(task) = q.pop() {
            task();
        }
        assert_eq!(*log.lock().unwrap(), vec![0, 1, 10, 20]);
    }

    #[test]
    fn fixed_priority_queue_size_and_empty() {
        let q = FixedPriorityQueue::new(2);
        assert!(q.is_empty());
        assert_eq!(q.size(), 0);
        q.push(RunnableWithPriority {
            priority: 0,
            task: Box::new(|| {}),
        });
        assert!(!q.is_empty());
        assert_eq!(q.size(), 1);
        let task = q.pop();
        assert!(task.is_some());
        task.unwrap()();
        assert!(q.is_empty());
        assert_eq!(q.size(), 0);
    }

    #[test]
    #[should_panic(expected = "Priority -1 not supported")]
    fn fixed_priority_queue_negative_priority_panics() {
        let q = FixedPriorityQueue::new(2);
        q.push(RunnableWithPriority {
            priority: -1,
            task: Box::new(|| {}),
        });
    }

    #[test]
    #[should_panic(expected = "Priority 2 not supported")]
    fn fixed_priority_queue_out_of_range_panics() {
        let q = FixedPriorityQueue::new(2);
        q.push(RunnableWithPriority {
            priority: 2,
            task: Box::new(|| {}),
        });
    }

    #[test]
    fn queue_strict_queue_basic() {
        let mut dq = VecDeque::new();
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
        let l = Arc::clone(&log);
        dq.push_back(Box::new(move || l.lock().unwrap().push(1)) as BoxedRunnable);
        let q = QueueStrictQueue::new(dq);
        assert_eq!(q.size(), 1);
        assert!(!q.is_empty());
        let l2 = Arc::clone(&log);
        q.push(Box::new(move || l2.lock().unwrap().push(2)));
        assert_eq!(q.size(), 2);
        while let Some(task) = q.pop() {
            task();
        }
        assert_eq!(*log.lock().unwrap(), vec![1, 2]);
        assert!(q.is_empty());
        assert_eq!(q.size(), 0);
    }
}
