// === FILE: array_list_deque.rs ===

pub trait ListAndDeque<T> {}

pub struct ArrayListDeque<T> {
    contents: Vec<Option<T>>,
    head: i32,
    size: i32,
    mod_count: i32,
}

impl<T> ArrayListDeque<T> {
    // JAVA: net.minecraft.util.ArrayListDeque#<init>
    pub fn new() -> Self {
        Self::with_capacity(1)
    }

    // JAVA: net.minecraft.util.ArrayListDeque#<init>
    pub fn with_capacity(capacity: i32) -> Self {
        let cap = if capacity < 1 { 1usize } else { capacity as usize };
        let mut contents = Vec::with_capacity(cap);
        contents.resize_with(cap, || None);
        Self {
            contents,
            head: 0,
            size: 0,
            mod_count: 0,
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#size
    pub fn size(&self) -> i32 {
        self.size
    }

    // JAVA: net.minecraft.util.ArrayListDeque#capacity
    pub fn capacity(&self) -> i32 {
        self.contents.len() as i32
    }

    // JAVA: net.minecraft.util.ArrayListDeque#getIndex
    fn get_index(&self, index: i32) -> usize {
        let len = self.contents.len() as i32;
        let mut r = index.wrapping_add(self.head) % len;
        if r < 0 {
            r = r.wrapping_add(len);
        }
        r as usize
    }

    // JAVA: net.minecraft.util.ArrayListDeque#verifyIndexInRange
    fn verify_index_in_range_static(index: i32, size: i32) {
        if index < 0 || index >= size {
            panic!("IndexOutOfBoundsException: {index}");
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#verifyIndexInRange
    fn verify_index_in_range(&self, index: i32) {
        Self::verify_index_in_range_static(index, self.size);
    }

    // JAVA: net.minecraft.util.ArrayListDeque#getInner
    fn get_inner(&self, index: usize) -> &T {
        self.contents[index]
            .as_ref()
            .expect("element should be present")
    }

    // JAVA: net.minecraft.util.ArrayListDeque#get
    pub fn get(&self, index: i32) -> &T {
        self.verify_index_in_range(index);
        let idx = self.get_index(index);
        self.get_inner(idx)
    }

    // JAVA: net.minecraft.util.ArrayListDeque#set
    pub fn set(&mut self, index: i32, value: T) -> T {
        self.verify_index_in_range(index);
        let idx = self.get_index(index);
        let old = self.contents[idx]
            .take()
            .expect("element should be present");
        self.contents[idx] = Some(value);
        old
    }

    // JAVA: net.minecraft.util.ArrayListDeque#add
    pub fn add(&mut self, index: i32, value: T) {
        Self::verify_index_in_range_static(index, self.size.wrapping_add(1));
        if self.size == self.contents.len() as i32 {
            self.grow();
        }

        let idx = self.get_index(index);
        if index == self.size {
            self.contents[idx] = Some(value);
        } else if index == 0 {
            self.head = self.head.wrapping_sub(1);
            if self.head < 0 {
                self.head = self.head.wrapping_add(self.contents.len() as i32);
            }
            let i0 = self.get_index(0);
            self.contents[i0] = Some(value);
        } else {
            let mut i = self.size.wrapping_sub(1);
            while i >= index {
                let from = self.get_index(i);
                let to = self.get_index(i.wrapping_add(1));
                let val = self.contents[from].take();
                self.contents[to] = val;
                i = i.wrapping_sub(1);
            }
            self.contents[idx] = Some(value);
        }

        self.mod_count = self.mod_count.wrapping_add(1);
        self.size = self.size.wrapping_add(1);
    }

    // JAVA: net.minecraft.util.ArrayListDeque#grow
    fn grow(&mut self) {
        let old_len = self.contents.len() as i32;
        let growth = std::cmp::max(old_len >> 1, 1);
        let new_len = old_len.wrapping_add(growth);
        if new_len <= old_len {
            panic!("ArrayListDeque capacity overflow");
        }
        let new_len_usize = new_len as usize;
        let mut new_contents: Vec<Option<T>> = Vec::with_capacity(new_len_usize);
        new_contents.resize_with(new_len_usize, || None);
        self.copy_count(&mut new_contents, self.size);
        self.head = 0;
        self.contents = new_contents;
    }

    // JAVA: net.minecraft.util.ArrayListDeque#copyCount
    fn copy_count(&mut self, dest: &mut [Option<T>], count: i32) {
        let mut i: i32 = 0;
        while i < count {
            let src = self.get_index(i);
            dest[i as usize] = self.contents[src].take();
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#remove
    pub fn remove(&mut self, index: i32) -> T {
        self.verify_index_in_range(index);
        let idx = self.get_index(index);
        let removed = self.contents[idx]
            .take()
            .expect("element should be present");
        if index == 0 {
            self.head = self.head.wrapping_add(1);
        } else if index == self.size.wrapping_sub(1) {
            // slot already cleared by take
        } else {
            let mut i = index.wrapping_add(1);
            while i < self.size {
                let from = self.get_index(i);
                let to = self.get_index(i.wrapping_sub(1));
                let val = self.contents[from].take();
                self.contents[to] = val;
                i = i.wrapping_add(1);
            }
            let last = self.get_index(self.size.wrapping_sub(1));
            self.contents[last] = None;
        }

        self.mod_count = self.mod_count.wrapping_add(1);
        self.size = self.size.wrapping_sub(1);
        removed
    }

    // JAVA: net.minecraft.util.ArrayListDeque#removeIf
    pub fn remove_if<F>(&mut self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let mut removed_count: i32 = 0;
        let mut i: i32 = 0;
        while i < self.size {
            let val_ref = self.get(i);
            if predicate(val_ref) {
                removed_count = removed_count.wrapping_add(1);
            } else if removed_count != 0 {
                let target = self.get_index(i.wrapping_sub(removed_count));
                let source = self.get_index(i);
                let val = self.contents[source].take();
                self.contents[target] = val;
            }
            i = i.wrapping_add(1);
        }

        self.mod_count = self.mod_count.wrapping_add(removed_count);
        let old_size = self.size;
        self.size = self.size.wrapping_sub(removed_count);
        let mut j = self.size;
        while j < old_size {
            let idx = self.get_index(j);
            self.contents[idx] = None;
            j = j.wrapping_add(1);
        }
        removed_count != 0
    }

    // JAVA: net.minecraft.util.ArrayListDeque#replaceAll
    pub fn replace_all<F>(&mut self, mut operator: F)
    where
        F: FnMut(T) -> T,
    {
        let mut i: i32 = 0;
        while i < self.size {
            let idx = self.get_index(i);
            let old = self.contents[idx]
                .take()
                .expect("element should be present");
            let new_val = operator(old);
            self.contents[idx] = Some(new_val);
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#forEach
    pub fn for_each<F>(&self, mut consumer: F)
    where
        F: FnMut(&T),
    {
        let mut i: i32 = 0;
        while i < self.size {
            consumer(self.get(i));
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#addFirst
    pub fn add_first(&mut self, value: T) {
        self.add(0, value);
    }

    // JAVA: net.minecraft.util.ArrayListDeque#addLast
    pub fn add_last(&mut self, value: T) {
        self.add(self.size, value);
    }

    // JAVA: net.minecraft.util.ArrayListDeque#offerFirst
    pub fn offer_first(&mut self, value: T) -> bool {
        self.add_first(value);
        true
    }

    // JAVA: net.minecraft.util.ArrayListDeque#offerLast
    pub fn offer_last(&mut self, value: T) -> bool {
        self.add_last(value);
        true
    }

    // JAVA: net.minecraft.util.ArrayListDeque#removeFirst
    pub fn remove_first(&mut self) -> T {
        if self.size == 0 {
            panic!("NoSuchElementException");
        }
        self.remove(0)
    }

    // JAVA: net.minecraft.util.ArrayListDeque#removeLast
    pub fn remove_last(&mut self) -> T {
        if self.size == 0 {
            panic!("NoSuchElementException");
        }
        self.remove(self.size.wrapping_sub(1))
    }

    // JAVA: net.minecraft.util.ArrayListDeque#reversed
    pub fn reversed(&mut self) -> ReversedView<'_, T> {
        ReversedView { source: self }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#pollFirst
    pub fn poll_first(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            Some(self.remove_first())
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#pollLast
    pub fn poll_last(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            Some(self.remove_last())
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#getFirst
    pub fn get_first(&self) -> &T {
        if self.size == 0 {
            panic!("NoSuchElementException");
        }
        self.get(0)
    }

    // JAVA: net.minecraft.util.ArrayListDeque#getLast
    pub fn get_last(&self) -> &T {
        if self.size == 0 {
            panic!("NoSuchElementException");
        }
        self.get(self.size.wrapping_sub(1))
    }

    // JAVA: net.minecraft.util.ArrayListDeque#peekFirst
    pub fn peek_first(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            Some(self.get_first())
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#peekLast
    pub fn peek_last(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            Some(self.get_last())
        }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#removeFirstOccurrence
    pub fn remove_first_occurrence(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let mut i: i32 = 0;
        while i < self.size {
            if self.get(i) == value {
                self.remove(i);
                return true;
            }
            i = i.wrapping_add(1);
        }
        false
    }

    // JAVA: net.minecraft.util.ArrayListDeque#removeLastOccurrence
    pub fn remove_last_occurrence(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let mut i = self.size.wrapping_sub(1);
        while i >= 0 {
            if self.get(i) == value {
                self.remove(i);
                return true;
            }
            i = i.wrapping_sub(1);
        }
        false
    }

    // JAVA: net.minecraft.util.ArrayListDeque#descendingIterator
    pub fn descending_iterator(&mut self) -> DescendingIterator<'_, T> {
        let index = self.size().wrapping_sub(1);
        DescendingIterator { deque: self, index }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#iterator
    pub fn iterator(&self) -> ArrayListDequeIter<'_, T> {
        ArrayListDequeIter { deque: self, index: 0 }
    }

    // JAVA: net.minecraft.util.ArrayListDeque#isEmpty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    // JAVA: net.minecraft.util.ArrayListDeque#contains
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.index_of(value) != -1
    }

    // JAVA: net.minecraft.util.ArrayListDeque#indexOf
    pub fn index_of(&self, value: &T) -> i32
    where
        T: PartialEq,
    {
        let mut i: i32 = 0;
        while i < self.size {
            if self.get(i) == value {
                return i;
            }
            i = i.wrapping_add(1);
        }
        -1
    }

    // JAVA: net.minecraft.util.ArrayListDeque#lastIndexOf
    pub fn last_index_of(&self, value: &T) -> i32
    where
        T: PartialEq,
    {
        let mut i = self.size.wrapping_sub(1);
        while i >= 0 {
            if self.get(i) == value {
                return i;
            }
            i = i.wrapping_sub(1);
        }
        -1
    }

    // JAVA: net.minecraft.util.ArrayListDeque#clear
    pub fn clear(&mut self) {
        let mut i: i32 = 0;
        while i < self.size {
            let idx = self.get_index(i);
            self.contents[idx] = None;
            i = i.wrapping_add(1);
        }
        self.head = 0;
        self.size = 0;
        self.mod_count = self.mod_count.wrapping_add(1);
    }
}

impl<T> Default for ArrayListDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ListAndDeque<T> for ArrayListDeque<T> {}

pub struct ArrayListDequeIter<'a, T> {
    deque: &'a ArrayListDeque<T>,
    index: i32,
}

impl<'a, T> Iterator for ArrayListDequeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.deque.size() {
            let val = self.deque.get(self.index);
            self.index = self.index.wrapping_add(1);
            Some(val)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.deque.size().wrapping_sub(self.index).max(0) as usize;
        (remaining, Some(remaining))
    }
}

pub struct DescendingIterator<'a, T> {
    deque: &'a mut ArrayListDeque<T>,
    index: i32,
}

impl<'a, T> DescendingIterator<'a, T> {
    // JAVA: net.minecraft.util.ArrayListDeque$DescendingIterator#hasNext
    pub fn has_next(&self) -> bool {
        self.index >= 0
    }

    // JAVA: net.minecraft.util.ArrayListDeque$DescendingIterator#next
    pub fn next(&mut self) -> &T {
        let i = self.index;
        self.index = self.index.wrapping_sub(1);
        self.deque.get(i)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$DescendingIterator#remove
    pub fn remove(&mut self) {
        self.deque.remove(self.index.wrapping_add(1));
    }
}

pub struct ReversedView<'a, T> {
    source: &'a mut ArrayListDeque<T>,
}

impl<'a, T> ReversedView<'a, T> {
    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#reversed
    pub fn reversed(self) -> &'a mut ArrayListDeque<T> {
        self.source
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#getFirst
    pub fn get_first(&self) -> &T {
        self.source.get_last()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#getLast
    pub fn get_last(&self) -> &T {
        self.source.get_first()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#addFirst
    pub fn add_first(&mut self, value: T) {
        self.source.add_last(value);
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#addLast
    pub fn add_last(&mut self, value: T) {
        self.source.add_first(value);
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#offerFirst
    pub fn offer_first(&mut self, value: T) -> bool {
        self.source.offer_last(value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#offerLast
    pub fn offer_last(&mut self, value: T) -> bool {
        self.source.offer_first(value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#pollFirst
    pub fn poll_first(&mut self) -> Option<T> {
        self.source.poll_last()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#pollLast
    pub fn poll_last(&mut self) -> Option<T> {
        self.source.poll_first()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#peekFirst
    pub fn peek_first(&self) -> Option<&T> {
        self.source.peek_last()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#peekLast
    pub fn peek_last(&self) -> Option<&T> {
        self.source.peek_first()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#removeFirst
    pub fn remove_first(&mut self) -> T {
        self.source.remove_last()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#removeLast
    pub fn remove_last(&mut self) -> T {
        self.source.remove_first()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#removeFirstOccurrence
    pub fn remove_first_occurrence(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.source.remove_last_occurrence(value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#removeLastOccurrence
    pub fn remove_last_occurrence(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.source.remove_first_occurrence(value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#descendingIterator
    pub fn descending_iterator(&self) -> ArrayListDequeIter<'_, T> {
        self.source.iterator()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#size
    pub fn size(&self) -> i32 {
        self.source.size()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#isEmpty
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#contains
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.source.contains(value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#get
    pub fn get(&self, index: i32) -> &T {
        let ri = self.reverse_index(index);
        self.source.get(ri)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#set
    pub fn set(&mut self, index: i32, value: T) -> T {
        let ri = self.reverse_index(index);
        self.source.set(ri, value)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#add
    pub fn add(&mut self, index: i32, value: T) {
        let ri = self.reverse_index(index);
        self.source.add(ri.wrapping_add(1), value);
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#remove
    pub fn remove(&mut self, index: i32) -> T {
        let ri = self.reverse_index(index);
        self.source.remove(ri)
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#indexOf
    pub fn index_of(&self, value: &T) -> i32
    where
        T: PartialEq,
    {
        self.reverse_index(self.source.last_index_of(value))
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#lastIndexOf
    pub fn last_index_of(&self, value: &T) -> i32
    where
        T: PartialEq,
    {
        self.reverse_index(self.source.index_of(value))
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#iterator
    pub fn iterator(&mut self) -> DescendingIterator<'_, T> {
        self.source.descending_iterator()
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#clear
    pub fn clear(&mut self) {
        self.source.clear();
    }

    // JAVA: net.minecraft.util.ArrayListDeque$ReversedView#reverseIndex
    fn reverse_index(&self, index: i32) -> i32 {
        if index == -1 {
            -1
        } else {
            self.source.size().wrapping_sub(1).wrapping_sub(index)
        }
    }
}

impl<'a, T> ListAndDeque<T> for ReversedView<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_deque_operations() {
        let mut d = ArrayListDeque::with_capacity(2);
        d.add_last(10);
        d.add_first(5);
        d.add_last(20);
        assert_eq!(d.size(), 3);
        assert_eq!(*d.get(0), 5);
        assert_eq!(*d.get(1), 10);
        assert_eq!(*d.get(2), 20);
        assert_eq!(*d.get_first(), 5);
        assert_eq!(*d.get_last(), 20);

        let old = d.set(1, 99);
        assert_eq!(old, 10);
        assert_eq!(*d.get(1), 99);

        let removed = d.remove(0);
        assert_eq!(removed, 5);
        assert_eq!(d.size(), 2);
        assert_eq!(*d.get(0), 99);
        assert_eq!(*d.get(1), 20);
    }

    #[test]
    fn wrap_around_and_grow() {
        let mut d = ArrayListDeque::with_capacity(3);
        d.add_last(1);
        d.add_last(2);
        d.add_last(3);
        assert_eq!(d.remove_first(), 1);
        assert_eq!(d.remove_first(), 2);
        d.add_last(4);
        d.add_last(5);
        assert_eq!(d.size(), 3);
        assert_eq!(*d.get(0), 3);
        assert_eq!(*d.get(1), 4);
        assert_eq!(*d.get(2), 5);

        d.add_last(6);
        assert_eq!(d.size(), 4);
        assert!(d.capacity() >= 4);
        for i in 0..4i32 {
            assert_eq!(*d.get(i), i.wrapping_add(3));
        }
    }

    #[test]
    fn remove_if_and_occurrences() {
        let mut d = ArrayListDeque::with_capacity(4);
        for v in [1, 2, 3, 4, 5] {
            d.add_last(v);
        }
        let changed = d.remove_if(|v| *v % 2 == 0);
        assert!(changed);
        assert_eq!(d.size(), 3);
        assert_eq!(*d.get(0), 1);
        assert_eq!(*d.get(1), 3);
        assert_eq!(*d.get(2), 5);

        assert!(d.remove_first_occurrence(&3));
        assert_eq!(d.size(), 2);
        assert!(!d.remove_last_occurrence(&99));

        let mut d2 = ArrayListDeque::new();
        d2.add_last(7);
        d2.add_last(8);
        assert!(d2.remove_if(|_| true));
        assert!(d2.is_empty());
    }

    #[test]
    fn descending_iterator_remove() {
        let mut d = ArrayListDeque::new();
        d.add_last(1);
        d.add_last(2);
        d.add_last(3);
        {
            let mut it = d.descending_iterator();
            assert!(it.has_next());
            assert_eq!(*it.next(), 3);
            assert_eq!(*it.next(), 2);
            it.remove();
            assert_eq!(*it.next(), 1);
            assert!(!it.has_next());
        }
        assert_eq!(d.size(), 2);
        assert_eq!(*d.get(0), 1);
        assert_eq!(*d.get(1), 3);
    }

    #[test]
    fn reversed_view_operations() {
        let mut d = ArrayListDeque::new();
        d.add_last(1);
        d.add_last(2);
        d.add_last(3);
        {
            let mut r = d.reversed();
            assert_eq!(*r.get_first(), 3);
            assert_eq!(*r.get_last(), 1);
            assert_eq!(*r.get(0), 3);
            assert_eq!(*r.get(2), 1);
            r.add_first(4);
            let old = r.set(0, 40);
            assert_eq!(old, 4);
        }
        assert_eq!(d.size(), 4);
        assert_eq!(*d.get(0), 1);
        assert_eq!(*d.get(3), 40);
    }

    #[test]
    fn edge_cases_negative_and_float_bits() {
        let mut empty: ArrayListDeque<i32> = ArrayListDeque::new();
        assert!(empty.is_empty());
        assert_eq!(empty.poll_first(), None);
        assert_eq!(empty.poll_last(), None);
        assert_eq!(empty.peek_first(), None);
        assert_eq!(empty.peek_last(), None);

        let neg = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let d: ArrayListDeque<i32> = ArrayListDeque::new();
            d.get(-1);
        }));
        assert!(neg.is_err());

        let mut df: ArrayListDeque<f32> = ArrayListDeque::new();
        let nan_bits: u32 = 0x7FC0_0001;
        let neg_zero_bits: u32 = 0x8000_0000;
        df.add_last(f32::from_bits(nan_bits));
        df.add_last(f32::from_bits(neg_zero_bits));
        df.add_last(f32::NEG_INFINITY);
        assert_eq!(df.get(0).to_bits(), nan_bits);
        assert_eq!(df.get(1).to_bits(), neg_zero_bits);
        assert_eq!(df.get(2).to_bits(), f32::NEG_INFINITY.to_bits());
    }
}
