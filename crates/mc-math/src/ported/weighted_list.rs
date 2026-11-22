use std::hash::{Hash, Hasher};

const FLAT_THRESHOLD: i32 = 64;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Weighted<T> {
    value: T,
    weight: i32,
}

impl<T> Weighted<T> {
    // JAVA: net.minecraft.util.random.Weighted#<init>
    pub fn new(value: T, weight: i32) -> Self {
        Self { value, weight }
    }

    // JAVA: net.minecraft.util.random.Weighted#value
    pub fn value(&self) -> &T {
        &self.value
    }

    // JAVA: net.minecraft.util.random.Weighted#weight
    pub fn weight(&self) -> i32 {
        self.weight
    }

    // JAVA: net.minecraft.util.random.Weighted#map
    pub fn map<U, F>(self, f: F) -> Weighted<U>
    where
        F: FnOnce(T) -> U,
    {
        Weighted {
            value: f(self.value),
            weight: self.weight,
        }
    }
}

// JAVA: net.minecraft.util.random.WeightedRandom#getTotalWeight
pub fn get_total_weight<T>(items: &[Weighted<T>]) -> i32 {
    let mut total: i32 = 0;
    for item in items {
        total = total.wrapping_add(item.weight());
    }
    total
}

#[derive(Clone, Debug)]
enum Selector {
    Flat(Vec<usize>),
    Compact,
}

#[derive(Clone, Debug)]
pub struct WeightedList<T> {
    total_weight: i32,
    items: Vec<Weighted<T>>,
    selector: Option<Selector>,
}

impl<T> WeightedList<T> {
    // JAVA: net.minecraft.util.random.WeightedList#<init>
    pub fn new(items: Vec<Weighted<T>>) -> Self {
        let total_weight = get_total_weight(&items);
        let selector = if total_weight == 0 {
            None
        } else if total_weight < FLAT_THRESHOLD {
            Some(Selector::Flat(build_flat(&items, total_weight)))
        } else {
            Some(Selector::Compact)
        };
        Self {
            total_weight,
            items,
            selector,
        }
    }

    // JAVA: net.minecraft.util.random.WeightedList#of
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    // JAVA: net.minecraft.util.random.WeightedList#of
    pub fn of_value(value: T) -> Self {
        Self::new(vec![Weighted::new(value, 1)])
    }

    // JAVA: net.minecraft.util.random.WeightedList#of
    pub fn of_list(items: Vec<Weighted<T>>) -> Self {
        Self::new(items)
    }

    // JAVA: net.minecraft.util.random.WeightedList#builder
    pub fn builder() -> Builder<T> {
        Builder::new()
    }

    // JAVA: net.minecraft.util.random.WeightedList#isEmpty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    // JAVA: net.minecraft.util.random.WeightedList#map
    pub fn map<U, F>(self, mut f: F) -> WeightedList<U>
    where
        F: FnMut(T) -> U,
    {
        let items = self.items.into_iter().map(|w| w.map(&mut f)).collect();
        WeightedList::new(items)
    }

    // JAVA: net.minecraft.util.random.WeightedList#getRandom
    pub fn get_random<R>(&self, random: &mut R) -> Option<&T>
    where
        R: FnMut(i32) -> i32,
    {
        let selector = self.selector.as_ref()?;
        let i = random(self.total_weight);
        Some(select(self, selector, i))
    }

    // JAVA: net.minecraft.util.random.WeightedList#getRandomOrThrow
    pub fn get_random_or_throw<R>(&self, random: &mut R) -> &T
    where
        R: FnMut(i32) -> i32,
    {
        let Some(selector) = self.selector.as_ref() else {
            panic!("Weighted list has no elements");
        };
        let i = random(self.total_weight);
        select(self, selector, i)
    }

    // JAVA: net.minecraft.util.random.WeightedList#unwrap
    pub fn unwrap_items(&self) -> &[Weighted<T>] {
        &self.items
    }

    // JAVA: net.minecraft.util.random.WeightedList#contains
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.items.iter().any(|w| w.value() == value)
    }

    pub fn total_weight(&self) -> i32 {
        self.total_weight
    }
}

impl<T: PartialEq> PartialEq for WeightedList<T> {
    // JAVA: net.minecraft.util.random.WeightedList#equals
    fn eq(&self, other: &Self) -> bool {
        self.total_weight == other.total_weight && self.items == other.items
    }
}

impl<T: Eq> Eq for WeightedList<T> {}

impl<T: Hash> Hash for WeightedList<T> {
    // JAVA: net.minecraft.util.random.WeightedList#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.total_weight.hash(state);
        self.items.hash(state);
    }
}

fn select<'a, T>(list: &'a WeightedList<T>, selector: &Selector, index: i32) -> &'a T {
    match selector {
        Selector::Flat(entries) => flat_get(&list.items, entries, index),
        Selector::Compact => compact_get(&list.items, index),
    }
}

// JAVA: net.minecraft.util.random.WeightedList$Flat#<init>
fn build_flat<T>(items: &[Weighted<T>], total_weight: i32) -> Vec<usize> {
    let mut entries = vec![0usize; total_weight as usize];
    let mut idx: i32 = 0;
    for (item_idx, item) in items.iter().enumerate() {
        let w = item.weight();
        let start = idx as usize;
        let end = idx.wrapping_add(w) as usize;
        if start < end && end <= entries.len() {
            entries[start..end].fill(item_idx);
        }
        idx = idx.wrapping_add(w);
    }
    entries
}

// JAVA: net.minecraft.util.random.WeightedList$Flat#get
fn flat_get<'a, T>(items: &'a [Weighted<T>], entries: &[usize], index: i32) -> &'a T {
    let item_idx = entries[index as usize];
    items[item_idx].value()
}

// JAVA: net.minecraft.util.random.WeightedList$Compact#get
fn compact_get<'a, T>(items: &'a [Weighted<T>], mut index: i32) -> &'a T {
    for item in items {
        index = index.wrapping_sub(item.weight());
        if index < 0 {
            return item.value();
        }
    }
    panic!("{} exceeded total weight", index)
}

pub struct Builder<T> {
    result: Vec<Weighted<T>>,
}

impl<T> Builder<T> {
    // JAVA: net.minecraft.util.random.WeightedList$Builder#<init>
    fn new() -> Self {
        Self { result: Vec::new() }
    }

    // JAVA: net.minecraft.util.random.WeightedList$Builder#add
    pub fn add(mut self, value: T) -> Self {
        self.result.push(Weighted::new(value, 1));
        self
    }

    // JAVA: net.minecraft.util.random.WeightedList$Builder#add
    pub fn add_weighted(mut self, value: T, weight: i32) -> Self {
        self.result.push(Weighted::new(value, weight));
        self
    }

    // JAVA: net.minecraft.util.random.WeightedList$Builder#build
    pub fn build(self) -> WeightedList<T> {
        WeightedList::new(self.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_single() {
        let empty: WeightedList<i32> = WeightedList::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.get_random(&mut |_: i32| 0), None);
        assert!(!empty.contains(&1));

        let single = WeightedList::of_value(42);
        assert!(!single.is_empty());
        assert_eq!(single.get_random(&mut |_: i32| 0), Some(&42));
        assert_eq!(single.get_random_or_throw(&mut |_: i32| 0), &42);
        assert!(single.contains(&42));
        assert!(!single.contains(&0));
    }

    #[test]
    fn flat_selection_by_weight() {
        let list = WeightedList::of_list(vec![
            Weighted::new("a", 1),
            Weighted::new("b", 2),
            Weighted::new("c", 3),
        ]);
        assert_eq!(list.total_weight(), 6);
        assert_eq!(list.get_random(&mut |_: i32| 0), Some(&"a"));
        assert_eq!(list.get_random(&mut |_: i32| 1), Some(&"b"));
        assert_eq!(list.get_random(&mut |_: i32| 2), Some(&"b"));
        assert_eq!(list.get_random(&mut |_: i32| 3), Some(&"c"));
        assert_eq!(list.get_random(&mut |_: i32| 5), Some(&"c"));
    }

    #[test]
    fn compact_selection_by_weight() {
        let list = WeightedList::of_list(vec![
            Weighted::new(10i32, 32),
            Weighted::new(20i32, 32),
            Weighted::new(30i32, 10),
        ]);
        assert_eq!(list.total_weight(), 74);
        assert_eq!(list.get_random(&mut |_: i32| 0), Some(&10));
        assert_eq!(list.get_random(&mut |_: i32| 31), Some(&10));
        assert_eq!(list.get_random(&mut |_: i32| 32), Some(&20));
        assert_eq!(list.get_random(&mut |_: i32| 63), Some(&20));
        assert_eq!(list.get_random(&mut |_: i32| 64), Some(&30));
        assert_eq!(list.get_random(&mut |_: i32| 73), Some(&30));
    }

    #[test]
    #[should_panic(expected = "Weighted list has no elements")]
    fn empty_throws() {
        let empty: WeightedList<i32> = WeightedList::empty();
        let _ = empty.get_random_or_throw(&mut |_: i32| 0);
    }

    #[test]
    fn wrapping_and_negative_weights() {
        let total = get_total_weight(&[Weighted::new(1u8, i32::MAX), Weighted::new(2u8, 2)]);
        assert_eq!(total, i32::MAX.wrapping_add(2));

        let list = WeightedList::of_list(vec![Weighted::new("a", 100), Weighted::new("b", -10)]);
        assert_eq!(list.total_weight(), 90);
        assert_eq!(list.get_random(&mut |_: i32| 0), Some(&"a"));
        assert_eq!(list.get_random(&mut |_: i32| 99), Some(&"a"));
    }

    #[test]
    fn map_and_float_bits_preserved() {
        let nan_bits = 0x7FC0_0001u32;
        let nan = f32::from_bits(nan_bits);
        let neg_zero = f32::from_bits(0x8000_0000u32);

        let list = WeightedList::builder()
            .add_weighted(nan, 1)
            .add_weighted(neg_zero, 63)
            .build();
        assert_eq!(list.get_random(&mut |_: i32| 0).unwrap().to_bits(), nan_bits);
        assert_eq!(list.get_random(&mut |_: i32| 1).unwrap().to_bits(), 0x8000_0000u32);

        let mapped = list.map(|v| v.to_bits());
        assert_eq!(*mapped.get_random(&mut |_: i32| 0).unwrap(), nan_bits);
        assert_eq!(mapped.unwrap_items()[1].weight(), 63);
    }
}
