use crate::NbtTag;

/// Self-contained collection tag interface.
/// In Java this extends `Iterable<Tag>, Tag`; here we expose collection operations over `NbtTag`.
/// JAVA: net.minecraft.nbt.CollectionTag
pub trait CollectionTag {
    // JAVA: net.minecraft.nbt.CollectionTag#clear
    fn clear(&mut self);

    // JAVA: net.minecraft.nbt.CollectionTag#setTag
    fn set_tag(&mut self, index: i32, tag: NbtTag) -> bool;

    // JAVA: net.minecraft.nbt.CollectionTag#addTag
    fn add_tag(&mut self, index: i32, tag: NbtTag) -> bool;

    // JAVA: net.minecraft.nbt.CollectionTag#remove
    fn remove(&mut self, index: i32) -> NbtTag;

    // JAVA: net.minecraft.nbt.CollectionTag#get
    fn get(&self, index: i32) -> &NbtTag;

    // JAVA: net.minecraft.nbt.CollectionTag#size
    fn size(&self) -> i32;

    // JAVA: net.minecraft.nbt.CollectionTag#isEmpty
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    // JAVA: net.minecraft.nbt.CollectionTag#iterator
    fn iter(&self) -> CollectionTagIter<'_, Self>
    where
        Self: Sized,
    {
        CollectionTagIter {
            collection: self,
            index: 0,
        }
    }
}

// JAVA: net.minecraft.nbt.CollectionTag#iterator
pub struct CollectionTagIter<'a, T: CollectionTag + ?Sized> {
    collection: &'a T,
    index: i32,
}

impl<'a, T: CollectionTag + ?Sized> Iterator for CollectionTagIter<'a, T> {
    type Item = &'a NbtTag;

    // JAVA: net.minecraft.nbt.CollectionTag#iterator
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.collection.size() {
            let idx = self.index;
            self.index = self.index.wrapping_add(1);
            Some(self.collection.get(idx))
        } else {
            None
        }
    }
}

// JAVA: net.minecraft.nbt.CollectionTag#stream
pub fn stream<'a, T: CollectionTag + ?Sized>(collection: &'a T) -> CollectionTagIter<'a, T> {
    CollectionTagIter {
        collection,
        index: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    #[derive(Debug, Clone, Default)]
    struct SimpleList {
        items: Vec<NbtTag>,
    }

    impl CollectionTag for SimpleList {
        fn clear(&mut self) {
            self.items.clear();
        }

        fn set_tag(&mut self, index: i32, tag: NbtTag) -> bool {
            if index < 0 {
                return false;
            }
            let idx = index as usize;
            if idx >= self.items.len() {
                return false;
            }
            self.items[idx] = tag;
            true
        }

        fn add_tag(&mut self, index: i32, tag: NbtTag) -> bool {
            if index < 0 {
                return false;
            }
            let idx = index as usize;
            if idx > self.items.len() {
                return false;
            }
            self.items.insert(idx, tag);
            true
        }

        fn remove(&mut self, index: i32) -> NbtTag {
            let idx = index as usize;
            self.items.remove(idx)
        }

        fn get(&self, index: i32) -> &NbtTag {
            &self.items[index as usize]
        }

        fn size(&self) -> i32 {
            self.items.len() as i32
        }
    }

    #[test]
    fn is_empty_size_clear() {
        let mut list = SimpleList::default();
        assert!(list.is_empty());
        assert_eq!(list.size(), 0);
        assert!(list.add_tag(0, NbtTag::Int(1)));
        assert!(!list.is_empty());
        assert_eq!(list.size(), 1);
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn set_add_remove_get() {
        let mut list = SimpleList::default();
        assert!(list.add_tag(0, NbtTag::Int(10)));
        assert!(list.add_tag(1, NbtTag::Int(20)));
        assert!(list.set_tag(1, NbtTag::Int(-5)));
        assert_eq!(list.get(1), &NbtTag::Int(-5));
        let removed = list.remove(0);
        assert_eq!(removed, NbtTag::Int(10));
        assert_eq!(list.size(), 1);
        assert_eq!(list.get(0), &NbtTag::Int(-5));
    }

    #[test]
    fn negative_index_rejected_for_set_add() {
        let mut list = SimpleList::default();
        assert!(!list.add_tag(-1, NbtTag::Byte(1)));
        assert!(!list.set_tag(-1, NbtTag::Byte(1)));
        assert!(list.is_empty());
    }

    #[test]
    fn iterator_and_stream_visit_all() {
        let mut list = SimpleList::default();
        assert!(list.add_tag(0, NbtTag::Int(1)));
        assert!(list.add_tag(1, NbtTag::Int(2)));
        assert!(list.add_tag(2, NbtTag::Int(3)));

        let collected: Vec<i32> = list
            .iter()
            .map(|t| match t {
                NbtTag::Int(v) => *v,
                _ => 0,
            })
            .collect();
        assert_eq!(collected, vec![1, 2, 3]);

        let streamed: Vec<i32> = stream(&list)
            .map(|t| match t {
                NbtTag::Int(v) => *v,
                _ => 0,
            })
            .collect();
        assert_eq!(streamed, vec![1, 2, 3]);
    }

    #[test]
    fn iterator_handles_empty() {
        let list = SimpleList::default();
        let mut it = list.iter();
        assert!(it.next().is_none());
    }
}
