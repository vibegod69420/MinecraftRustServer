use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct NonNullList<E> {
    list: Vec<E>,
    default_value: Option<E>,
}

impl<E: fmt::Debug> fmt::Debug for NonNullList<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NonNullList")
            .field("list", &self.list)
            .field("default_value", &self.default_value)
            .finish()
    }
}

impl<E> NonNullList<E> {
    // JAVA: net.minecraft.core.NonNullList#create
    pub fn create() -> Self {
        Self {
            list: Vec::new(),
            default_value: None,
        }
    }

    // JAVA: net.minecraft.core.NonNullList#createWithCapacity
    pub fn create_with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
            default_value: None,
        }
    }

    // JAVA: net.minecraft.core.NonNullList#of
    pub fn of(default_value: E, elements: Vec<E>) -> Self {
        Self {
            list: elements,
            default_value: Some(default_value),
        }
    }

    // JAVA: net.minecraft.core.NonNullList#<init>
    pub fn new(list: Vec<E>, default_value: Option<E>) -> Self {
        Self {
            list,
            default_value,
        }
    }

    // JAVA: net.minecraft.core.NonNullList#get
    pub fn get(&self, index: usize) -> &E {
        &self.list[index]
    }

    // JAVA: net.minecraft.core.NonNullList#set
    pub fn set(&mut self, index: usize, value: E) -> E {
        std::mem::replace(&mut self.list[index], value)
    }

    // JAVA: net.minecraft.core.NonNullList#add
    pub fn add(&mut self, index: usize, value: E) {
        self.list.insert(index, value);
    }

    // JAVA: net.minecraft.core.NonNullList#remove
    pub fn remove(&mut self, index: usize) -> E {
        self.list.remove(index)
    }

    // JAVA: net.minecraft.core.NonNullList#size
    pub fn size(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, E> {
        self.list.iter()
    }
}

impl<E: Clone> NonNullList<E> {
    // JAVA: net.minecraft.core.NonNullList#withSize
    pub fn with_size(size: usize, default_value: E) -> Self {
        Self {
            list: vec![default_value.clone(); size],
            default_value: Some(default_value),
        }
    }

    // JAVA: net.minecraft.core.NonNullList#clear
    pub fn clear(&mut self) {
        if let Some(ref dv) = self.default_value {
            for slot in &mut self.list {
                *slot = dv.clone();
            }
        } else {
            self.list.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NonNullList;

    #[test]
    fn create_and_basic_ops() {
        let mut list: NonNullList<i32> = NonNullList::create();
        assert_eq!(list.size(), 0);
        assert!(list.is_empty());
        list.add(0, 10);
        list.add(1, 20);
        list.add(1, 15);
        assert_eq!(*list.get(0), 10);
        assert_eq!(*list.get(1), 15);
        assert_eq!(*list.get(2), 20);
        assert_eq!(list.set(1, 30), 15);
        assert_eq!(*list.get(1), 30);
        assert_eq!(list.remove(0), 10);
        assert_eq!(list.size(), 2);
    }

    #[test]
    fn with_size_and_clear_with_default() {
        let mut list = NonNullList::with_size(4, 7i32);
        assert_eq!(list.size(), 4);
        list.set(1, 42);
        list.set(3, -1);
        list.clear();
        assert_eq!(*list.get(0), 7);
        assert_eq!(*list.get(1), 7);
        assert_eq!(*list.get(2), 7);
        assert_eq!(*list.get(3), 7);
        assert_eq!(list.size(), 4);
    }

    #[test]
    fn clear_without_default_empties() {
        let mut list: NonNullList<i32> = NonNullList::create_with_capacity(4);
        list.add(0, 1);
        list.add(1, 2);
        list.clear();
        assert_eq!(list.size(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn of_and_with_size_zero() {
        let list = NonNullList::of(0i32, vec![1, 2, 3]);
        assert_eq!(list.size(), 3);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(2), 3);

        let empty = NonNullList::with_size(0, 99i32);
        assert_eq!(empty.size(), 0);
    }

    #[test]
    fn negative_and_overflow_values() {
        let mut list = NonNullList::with_size(2, i32::MIN);
        list.set(1, i32::MAX);
        assert_eq!(*list.get(0), i32::MIN);
        assert_eq!(*list.get(1), i32::MAX);
        list.clear();
        assert_eq!(*list.get(1), i32::MIN);
    }
}
