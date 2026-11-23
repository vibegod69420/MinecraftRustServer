use std::collections::{HashMap, VecDeque};

pub struct SequencedPriorityIterator<T> {
    highest_prio_key: Option<i32>,
    highest_prio: i32,
    queues_by_priority: HashMap<i32, VecDeque<T>>,
}

impl<T> SequencedPriorityIterator<T> {
    // JAVA: net.minecraft.util.SequencedPriorityIterator#<init>
    pub fn new() -> Self {
        Self {
            highest_prio_key: None,
            highest_prio: i32::MIN,
            queues_by_priority: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.SequencedPriorityIterator#add
    pub fn add(&mut self, value: T, priority: i32) {
        if Some(priority) == self.highest_prio_key {
            if let Some(q) = self.queues_by_priority.get_mut(&priority) {
                q.push_back(value);
                return;
            }
        }

        let q = self
            .queues_by_priority
            .entry(priority)
            .or_insert_with(VecDeque::new);
        q.push_back(value);
        if priority >= self.highest_prio {
            self.highest_prio = priority;
            self.highest_prio_key = Some(priority);
        }
    }

    // JAVA: net.minecraft.util.SequencedPriorityIterator#computeNext
    fn compute_next(&mut self) -> Option<T> {
        let key = self.highest_prio_key?;
        let val = {
            let q = self.queues_by_priority.get_mut(&key)?;
            q.pop_front()?
        };

        let is_empty = self
            .queues_by_priority
            .get(&key)
            .is_some_and(|q| q.is_empty());
        if is_empty {
            self.switch_cache_to_next_highest_prio_queue();
        }

        Some(val)
    }

    // JAVA: net.minecraft.util.SequencedPriorityIterator#switchCacheToNextHighestPrioQueue
    fn switch_cache_to_next_highest_prio_queue(&mut self) {
        let mut best_prio: i32 = i32::MIN;
        let mut found: Option<i32> = None;
        let old_highest = self.highest_prio;

        for (&prio, q) in &self.queues_by_priority {
            if prio > best_prio && !q.is_empty() {
                best_prio = prio;
                found = Some(prio);
                if prio == old_highest.wrapping_sub(1) {
                    break;
                }
            }
        }

        self.highest_prio = best_prio;
        self.highest_prio_key = found;
    }
}

impl<T> Default for SequencedPriorityIterator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for SequencedPriorityIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.compute_next()
    }
}

#[cfg(test)]
mod tests {
    use super::SequencedPriorityIterator;

    #[test]
    fn basic_priority_ordering() {
        let mut it = SequencedPriorityIterator::new();
        it.add("low", 0);
        it.add("high", 10);
        it.add("mid", 5);
        assert_eq!(it.next(), Some("high"));
        assert_eq!(it.next(), Some("mid"));
        assert_eq!(it.next(), Some("low"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn fifo_within_same_priority() {
        let mut it = SequencedPriorityIterator::new();
        it.add(1, 5);
        it.add(2, 5);
        it.add(3, 5);
        assert_eq!(it.collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn negative_and_extreme_priorities() {
        let mut it = SequencedPriorityIterator::new();
        it.add("min", i32::MIN);
        it.add("neg", -1);
        it.add("max", i32::MAX);
        it.add("zero", 0);
        assert_eq!(it.next(), Some("max"));
        assert_eq!(it.next(), Some("zero"));
        assert_eq!(it.next(), Some("neg"));
        // Priority i32::MIN cannot be rediscovered by switch due to `> i32::MIN` sentinel check.
        assert_eq!(it.next(), None);
    }

    #[test]
    fn min_value_priority_cache_behavior() {
        let mut it = SequencedPriorityIterator::new();
        it.add(1i32, i32::MIN);
        it.add(2, i32::MIN);
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next(), None);
        it.add(3, i32::MIN);
        assert_eq!(it.next(), Some(3));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn interleaved_add_and_next() {
        let mut it = SequencedPriorityIterator::new();
        it.add(1, 1);
        assert_eq!(it.next(), Some(1));
        it.add(2, 0);
        it.add(3, 10);
        assert_eq!(it.next(), Some(3));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn empty_iterator_returns_none() {
        let mut it: SequencedPriorityIterator<i32> = SequencedPriorityIterator::new();
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }
}
