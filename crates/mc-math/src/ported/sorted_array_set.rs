use std::cmp::Ordering;

const DEFAULT_INITIAL_CAPACITY: usize = 10;

pub struct SortedArraySet<T> {
    comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    contents: Vec<Option<T>>,
    size: usize,
}

fn grow_by_half(current: usize, min: usize) -> usize {
    let grown = current + (current >> 1);
    if grown < min {
        min
    } else {
        grown
    }
}

impl<T: Ord + 'static> SortedArraySet<T> {
    // JAVA: net.minecraft.util.SortedArraySet#create
    pub fn create() -> Self {
        Self::create_with_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    // JAVA: net.minecraft.util.SortedArraySet#create
    pub fn create_with_capacity(initial_capacity: usize) -> Self {
        Self::new(initial_capacity, |a: &T, b: &T| a.cmp(b))
    }
}

impl<T: 'static> SortedArraySet<T> {
    // JAVA: net.minecraft.util.SortedArraySet#create
    pub fn create_with_comparator(comparator: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        Self::create_with_comparator_and_capacity(comparator, DEFAULT_INITIAL_CAPACITY)
    }

    // JAVA: net.minecraft.util.SortedArraySet#create
    pub fn create_with_comparator_and_capacity(
        comparator: impl Fn(&T, &T) -> Ordering + 'static,
        initial_capacity: usize,
    ) -> Self {
        Self::new(initial_capacity, comparator)
    }

    // JAVA: net.minecraft.util.SortedArraySet#SortedArraySet
    fn new(initial_capacity: usize, comparator: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        if (initial_capacity as isize) < 0 {
            panic!(
                "Initial capacity ({}) is negative",
                initial_capacity
            );
        }
        let mut contents = Vec::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            contents.push(None);
        }
        Self {
            comparator: Box::new(comparator),
            contents,
            size: 0,
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#findIndex
    fn find_index(&self, value: &T) -> i32 {
        let slice = &self.contents[..self.size];
        match slice.binary_search_by(|elem| (self.comparator)(elem.as_ref().unwrap(), value)) {
            Ok(idx) => idx as i32,
            Err(idx) => -((idx as i32) as i32) - 1,
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#getInsertionPosition
    fn get_insertion_position(index: i32) -> i32 {
        -index - 1
    }

    // JAVA: net.minecraft.util.SortedArraySet#add
    pub fn add(&mut self, value: T) -> bool {
        let index = self.find_index(&value);
        if index >= 0 {
            return false;
        }
        let insertion = Self::get_insertion_position(index);
        self.add_internal(value, insertion);
        true
    }

    // JAVA: net.minecraft.util.SortedArraySet#grow
    fn grow(&mut self, min_capacity: usize) {
        let old_len = self.contents.len();
        if min_capacity > old_len {
            let mut new_cap = min_capacity;
            if old_len > 0 {
                new_cap = grow_by_half(old_len, min_capacity);
            } else if new_cap < DEFAULT_INITIAL_CAPACITY {
                new_cap = DEFAULT_INITIAL_CAPACITY;
            }
            let mut new_contents: Vec<Option<T>> = Vec::with_capacity(new_cap);
            for _ in 0..new_cap {
                new_contents.push(None);
            }
            for i in 0..self.size {
                new_contents[i] = self.contents[i].take();
            }
            self.contents = new_contents;
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#addInternal
    fn add_internal(&mut self, value: T, index: i32) {
        let index = index as usize;
        self.grow(self.size + 1);
        if index != self.size {
            self.contents[index..=self.size].rotate_right(1);
        }
        self.contents[index] = Some(value);
        self.size += 1;
    }

    // JAVA: net.minecraft.util.SortedArraySet#removeInternal
    fn remove_internal(&mut self, index: i32) {
        let index = index as usize;
        self.size -= 1;
        if index != self.size {
            self.contents[index..=self.size].rotate_left(1);
        }
        self.contents[self.size] = None;
    }

    // JAVA: net.minecraft.util.SortedArraySet#getInternal
    fn get_internal(&self, index: i32) -> &T {
        self.contents[index as usize].as_ref().unwrap()
    }

    // JAVA: net.minecraft.util.SortedArraySet#addOrGet
    pub fn add_or_get(&mut self, value: T) -> &T {
        let index = self.find_index(&value);
        if index >= 0 {
            return self.get_internal(index);
        }
        let insertion = Self::get_insertion_position(index);
        self.add_internal(value, insertion);
        self.get_internal(insertion)
    }

    // JAVA: net.minecraft.util.SortedArraySet#remove
    pub fn remove(&mut self, value: &T) -> bool {
        let index = self.find_index(value);
        if index >= 0 {
            self.remove_internal(index);
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#get
    pub fn get(&self, value: &T) -> Option<&T> {
        let index = self.find_index(value);
        if index >= 0 {
            Some(self.get_internal(index))
        } else {
            None
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#first
    pub fn first(&self) -> &T {
        self.get_internal(0)
    }

    // JAVA: net.minecraft.util.SortedArraySet#last
    pub fn last(&self) -> &T {
        self.get_internal(self.size as i32 - 1)
    }

    // JAVA: net.minecraft.util.SortedArraySet#contains
    pub fn contains(&self, value: &T) -> bool {
        self.find_index(value) >= 0
    }

    // JAVA: net.minecraft.util.SortedArraySet#iterator
    pub fn iterator(&mut self) -> ArrayIterator<'_, T> {
        ArrayIterator {
            set: self,
            index: 0,
            last: -1,
        }
    }

    // JAVA: net.minecraft.util.SortedArraySet#size
    pub fn size(&self) -> usize {
        self.size
    }

    // JAVA: net.minecraft.util.SortedArraySet#toArray
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.contents[..self.size]
            .iter()
            .map(|o| o.as_ref().unwrap().clone())
            .collect()
    }

    // JAVA: net.minecraft.util.SortedArraySet#clear
    pub fn clear(&mut self) {
        for i in 0..self.size {
            self.contents[i] = None;
        }
        self.size = 0;
    }

    // JAVA: net.minecraft.util.SortedArraySet#equals
    pub fn equals(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        if std::ptr::eq(self, other) {
            return true;
        }
        self.size == other.size && self.contents[..self.size] == other.contents[..other.size]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.contents[..self.size]
            .iter()
            .map(|o| o.as_ref().unwrap())
    }
}

impl<T: PartialEq + 'static> PartialEq for SortedArraySet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl<T: Eq + 'static> Eq for SortedArraySet<T> {}

pub struct ArrayIterator<'a, T> {
    set: &'a mut SortedArraySet<T>,
    index: usize,
    last: i32,
}

impl<'a, T: 'static> ArrayIterator<'a, T> {
    // JAVA: net.minecraft.util.SortedArraySet$ArrayIterator#hasNext
    pub fn has_next(&self) -> bool {
        self.index < self.set.size
    }

    // JAVA: net.minecraft.util.SortedArraySet$ArrayIterator#next
    pub fn next(&mut self) -> &T {
        if self.index >= self.set.size {
            panic!("NoSuchElementException");
        }
        self.last = self.index as i32;
        let item = self.set.contents[self.index].as_ref().unwrap();
        self.index += 1;
        item
    }

    // JAVA: net.minecraft.util.SortedArraySet$ArrayIterator#remove
    pub fn remove(&mut self) {
        if self.last == -1 {
            panic!("IllegalStateException");
        }
        self.set.remove_internal(self.last);
        self.index -= 1;
        self.last = -1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_add_contains_remove_basic() {
        let mut set = SortedArraySet::<i32>::create();
        assert_eq!(set.size(), 0);
        assert!(set.add(3));
        assert!(set.add(1));
        assert!(set.add(2));
        assert!(!set.add(2));
        assert_eq!(set.size(), 3);
        assert!(set.contains(&1));
        assert!(!set.contains(&4));
        assert_eq!(*set.first(), 1);
        assert_eq!(*set.last(), 3);
        assert!(set.remove(&2));
        assert!(!set.remove(&2));
        assert_eq!(set.size(), 2);
        assert_eq!(set.to_vec(), vec![1, 3]);
    }

    #[test]
    fn test_add_or_get_and_get() {
        let mut set = SortedArraySet::<i32>::create();
        let v = *set.add_or_get(5);
        assert_eq!(v, 5);
        let v2 = *set.add_or_get(5);
        assert_eq!(v2, 5);
        assert_eq!(set.size(), 1);
        assert_eq!(set.get(&5), Some(&5));
        assert_eq!(set.get(&6), None);
    }

    #[test]
    fn test_negative_and_extreme_integers() {
        let mut set = SortedArraySet::<i32>::create();
        assert!(set.add(i32::MIN));
        assert!(set.add(-1));
        assert!(set.add(0));
        assert!(set.add(i32::MAX));
        assert!(set.add(i32::MIN + 1));
        assert_eq!(set.size(), 5);
        assert_eq!(*set.first(), i32::MIN);
        assert_eq!(*set.last(), i32::MAX);
        assert!(set.contains(&-1));
        assert!(set.remove(&i32::MIN));
        assert_eq!(*set.first(), i32::MIN + 1);
    }

    #[test]
    fn test_float_bits_ordering() {
        fn java_double_cmp(a: &f64, b: &f64) -> Ordering {
            a.total_cmp(b)
        }

        let mut set = SortedArraySet::<f64>::create_with_comparator(java_double_cmp);
        let neg_zero = f64::from_bits(0x8000_0000_0000_0000u64);
        let pos_zero = 0.0f64;
        let nan = f64::from_bits(0x7ff8_0000_0000_0001u64);
        let neg_inf = f64::NEG_INFINITY;
        let pos_inf = f64::INFINITY;

        assert!(set.add(pos_zero));
        assert!(set.add(neg_zero));
        assert!(set.add(nan));
        assert!(set.add(pos_inf));
        assert!(set.add(neg_inf));
        assert_eq!(set.size(), 5);

        let vals: Vec<u64> = set.iter().map(|v| v.to_bits()).collect();
        assert_eq!(vals[0], neg_inf.to_bits());
        assert_eq!(vals[1], neg_zero.to_bits());
        assert_eq!(vals[2], pos_zero.to_bits());
        assert_eq!(vals[3], pos_inf.to_bits());
        assert_eq!(vals[4], nan.to_bits());
    }

    #[test]
    fn test_iterator_remove() {
        let mut set = SortedArraySet::<i32>::create();
        for v in [5, 1, 3, 2, 4] {
            set.add(v);
        }
        let mut it = set.iterator();
        let mut collected = Vec::new();
        while it.has_next() {
            let v = *it.next();
            collected.push(v);
            if v % 2 == 0 {
                it.remove();
            }
        }
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
        assert_eq!(set.to_vec(), vec![1, 3, 5]);
    }

    #[test]
    fn test_grow_and_clear() {
        let mut set = SortedArraySet::<i32>::create_with_capacity(0);
        for i in 0..20 {
            assert!(set.add(i));
        }
        assert_eq!(set.size(), 20);
        assert_eq!(*set.first(), 0);
        assert_eq!(*set.last(), 19);
        set.clear();
        assert_eq!(set.size(), 0);
        assert!(!set.contains(&0));
        assert!(set.add(10));
        assert_eq!(set.size(), 1);
        assert_eq!(*set.first(), 10);
    }
}
