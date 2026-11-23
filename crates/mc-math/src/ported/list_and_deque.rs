/// Faithful port of the `ListAndDeque` interface default methods.
///
/// Java extends `Serializable, Cloneable, Deque<T>, List<T>, RandomAccess`.
/// This Rust trait models the methods declared in the Java source and the
/// Deque operations required by its default methods.
pub trait ListAndDeque<T> {
    // JAVA: net.minecraft.util.ListAndDeque#reversed
    fn reversed(&self) -> Self
    where
        Self: Sized;

    // JAVA: net.minecraft.util.ListAndDeque#getFirst
    fn get_first(&self) -> &T;

    // JAVA: net.minecraft.util.ListAndDeque#getLast
    fn get_last(&self) -> &T;

    // JAVA: net.minecraft.util.ListAndDeque#addFirst
    fn add_first(&mut self, value: T);

    // JAVA: net.minecraft.util.ListAndDeque#addLast
    fn add_last(&mut self, value: T);

    // JAVA: net.minecraft.util.ListAndDeque#removeFirst
    fn remove_first(&mut self) -> T;

    // JAVA: net.minecraft.util.ListAndDeque#removeLast
    fn remove_last(&mut self) -> T;

    /// Required by Java `Deque#offerLast`, used by `offer`.
    fn offer_last(&mut self, value: T) -> bool;

    /// Required by Java `Deque#pollFirst`, used by `poll`.
    fn poll_first(&mut self) -> Option<T>;

    /// Required by Java `Deque#peekFirst`, used by `peek`.
    fn peek_first(&self) -> Option<&T>;

    // JAVA: net.minecraft.util.ListAndDeque#offer
    fn offer(&mut self, value: T) -> bool {
        self.offer_last(value)
    }

    // JAVA: net.minecraft.util.ListAndDeque#remove
    fn remove(&mut self) -> T {
        self.remove_first()
    }

    // JAVA: net.minecraft.util.ListAndDeque#poll
    fn poll(&mut self) -> Option<T> {
        self.poll_first()
    }

    // JAVA: net.minecraft.util.ListAndDeque#element
    fn element(&self) -> &T {
        self.get_first()
    }

    // JAVA: net.minecraft.util.ListAndDeque#peek
    fn peek(&self) -> Option<&T> {
        self.peek_first()
    }

    // JAVA: net.minecraft.util.ListAndDeque#push
    fn push(&mut self, value: T) {
        self.add_first(value)
    }

    // JAVA: net.minecraft.util.ListAndDeque#pop
    fn pop(&mut self) -> T {
        self.remove_first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestDeque<T> {
        inner: VecDeque<T>,
    }

    impl<T> TestDeque<T> {
        fn new() -> Self {
            Self {
                inner: VecDeque::new(),
            }
        }
    }

    impl<T: Clone> ListAndDeque<T> for TestDeque<T> {
        fn reversed(&self) -> Self {
            let mut inner = self.inner.clone();
            let slice = inner.make_contiguous();
            slice.reverse();
            Self { inner }
        }

        fn get_first(&self) -> &T {
            self.inner.front().expect("NoSuchElementException")
        }

        fn get_last(&self) -> &T {
            self.inner.back().expect("NoSuchElementException")
        }

        fn add_first(&mut self, value: T) {
            self.inner.push_front(value);
        }

        fn add_last(&mut self, value: T) {
            self.inner.push_back(value);
        }

        fn remove_first(&mut self) -> T {
            self.inner.pop_front().expect("NoSuchElementException")
        }

        fn remove_last(&mut self) -> T {
            self.inner.pop_back().expect("NoSuchElementException")
        }

        fn offer_last(&mut self, value: T) -> bool {
            self.inner.push_back(value);
            true
        }

        fn poll_first(&mut self) -> Option<T> {
            self.inner.pop_front()
        }

        fn peek_first(&self) -> Option<&T> {
            self.inner.front()
        }
    }

    #[test]
    fn default_methods_match_deque_semantics() {
        let mut d = TestDeque::new();
        assert!(d.offer(1));
        d.add_last(2);
        d.push(0);

        assert_eq!(*d.element(), 0);
        assert_eq!(d.peek(), Some(&0));
        assert_eq!(d.pop(), 0);
        assert_eq!(d.remove(), 1);
        assert_eq!(d.poll(), Some(2));
        assert_eq!(d.poll(), None);
        assert_eq!(d.peek(), None);
    }

    #[test]
    fn first_last_and_reversed_operations() {
        let mut d = TestDeque::new();
        d.add_first(2);
        d.add_last(3);
        d.add_first(1);
        assert_eq!(*d.get_first(), 1);
        assert_eq!(*d.get_last(), 3);

        let r = d.reversed();
        assert_eq!(*r.get_first(), 3);
        assert_eq!(*r.get_last(), 1);

        assert_eq!(d.remove_last(), 3);
        assert_eq!(d.remove_first(), 1);
        assert_eq!(d.remove_first(), 2);
    }

    #[test]
    #[should_panic(expected = "NoSuchElementException")]
    fn element_panics_on_empty() {
        let d: TestDeque<i32> = TestDeque::new();
        let _ = d.element();
    }

    #[test]
    #[should_panic(expected = "NoSuchElementException")]
    fn pop_panics_on_empty() {
        let mut d: TestDeque<i32> = TestDeque::new();
        let _ = d.pop();
    }
}
