use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

struct ClassBucket<T> {
    checker: Box<dyn Fn(&dyn Any) -> bool>,
    items: Vec<Rc<T>>,
}

pub struct ClassInstanceMultiMap<T: Any + 'static> {
    base_class: TypeId,
    by_class: HashMap<TypeId, ClassBucket<T>>,
    all_instances: Vec<Rc<T>>,
}

impl<T: Any + 'static> ClassInstanceMultiMap<T> {
    // JAVA: net.minecraft.util.ClassInstanceMultiMap#<init>
    pub fn new() -> Self {
        Self {
            base_class: TypeId::of::<T>(),
            by_class: HashMap::new(),
            all_instances: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#add
    pub fn add(&mut self, value: T) -> bool {
        let rc = Rc::new(value);
        let any_ref: &dyn Any = &*rc;
        let mut added = false;
        for bucket in self.by_class.values_mut() {
            if (bucket.checker)(any_ref) {
                bucket.items.push(Rc::clone(&rc));
                added = true;
            }
        }
        self.all_instances.push(rc);
        let _ = added;
        true
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#remove
    pub fn remove(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let any_ref: &dyn Any = value;
        let mut removed = false;
        for bucket in self.by_class.values_mut() {
            if (bucket.checker)(any_ref) {
                if let Some(pos) = bucket.items.iter().position(|item| &**item == value) {
                    bucket.items.remove(pos);
                    removed = true;
                }
            }
        }
        if let Some(pos) = self.all_instances.iter().position(|item| &**item == value) {
            self.all_instances.remove(pos);
            removed = true;
        }
        removed
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#contains
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.all_instances.iter().any(|item| &**item == value)
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#find
    pub fn find<S: Any + 'static>(&mut self) -> Vec<&S> {
        let tid = TypeId::of::<S>();
        if tid != self.base_class {
            panic!("Don't know how to search for {}", std::any::type_name::<S>());
        }
        self.all_instances
            .iter()
            .filter_map(|item| {
                let any: &dyn Any = &**item;
                any.downcast_ref::<S>()
            })
            .collect()
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#iterator
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.all_instances.iter().map(|rc| &**rc)
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#getAllInstances
    pub fn get_all_instances(&self) -> Vec<&T> {
        self.all_instances.iter().map(|rc| &**rc).collect()
    }

    // JAVA: net.minecraft.util.ClassInstanceMultiMap#size
    pub fn size(&self) -> usize {
        self.all_instances.len()
    }
}

impl<T: Any + 'static> Default for ClassInstanceMultiMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Animal(i32);
    #[derive(Debug, PartialEq)]
    struct Other(i32);

    #[test]
    fn add_and_size() {
        let mut map: ClassInstanceMultiMap<Animal> = ClassInstanceMultiMap::new();
        assert!(map.add(Animal(1)));
        assert!(map.add(Animal(2)));
        assert_eq!(map.size(), 2);
    }

    #[test]
    fn find_base_type() {
        let mut map = ClassInstanceMultiMap::new();
        map.add(Animal(10));
        map.add(Animal(20));
        let found: Vec<&Animal> = map.find::<Animal>();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, 10);
        assert_eq!(found[1].0, 20);
    }

    #[test]
    fn remove_and_contains() {
        let mut map = ClassInstanceMultiMap::new();
        map.add(Animal(1));
        map.add(Animal(2));
        assert!(map.contains(&Animal(1)));
        assert!(map.remove(&Animal(1)));
        assert!(!map.contains(&Animal(1)));
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn iter_and_get_all() {
        let mut map = ClassInstanceMultiMap::new();
        map.add(Animal(5));
        map.add(Animal(6));
        let all = map.get_all_instances();
        assert_eq!(all, vec![&Animal(5), &Animal(6)]);
        let collected: Vec<i32> = map.iter().map(|a| a.0).collect();
        assert_eq!(collected, vec![5, 6]);
    }

    #[test]
    #[should_panic(expected = "Don't know how to search for")]
    fn find_unrelated_type_panics() {
        let mut map = ClassInstanceMultiMap::new();
        map.add(Animal(1));
        let _: Vec<&Other> = map.find::<Other>();
    }
}
