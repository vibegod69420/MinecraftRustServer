use std::rc::Rc;

const NOT_FOUND: i32 = -1;
const LOADFACTOR: f32 = 0.8;

pub struct CrudeIncrementalIntIdentityHashBiMap<K> {
    keys: Vec<Option<Rc<K>>>,
    values: Vec<i32>,
    by_id: Vec<Option<Rc<K>>>,
    next_id: i32,
    size: i32,
}

impl<K> CrudeIncrementalIntIdentityHashBiMap<K> {
    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#<init>(int)
    fn new(capacity: usize) -> Self {
        Self {
            keys: vec![None; capacity],
            values: vec![0; capacity],
            by_id: vec![None; capacity],
            next_id: 0,
            size: 0,
        }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#<init>(Object[],int[],Object[],int,int)
    fn from_parts(
        keys: Vec<Option<Rc<K>>>,
        values: Vec<i32>,
        by_id: Vec<Option<Rc<K>>>,
        next_id: i32,
        size: i32,
    ) -> Self {
        Self {
            keys,
            values,
            by_id,
            next_id,
            size,
        }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#create
    pub fn create(initial_capacity: i32) -> Self {
        let cap = ((initial_capacity as f32) / LOADFACTOR) as i32 as usize;
        Self::new(cap)
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#getId
    pub fn get_id(&self, key: Option<&Rc<K>>) -> i32 {
        let h = self.hash(key);
        self.get_value(self.index_of(key, h))
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#byId
    pub fn by_id(&self, id: i32) -> Option<&Rc<K>> {
        if id >= 0 && (id as usize) < self.by_id.len() {
            self.by_id[id as usize].as_ref()
        } else {
            None
        }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#getValue
    fn get_value(&self, index: i32) -> i32 {
        if index == NOT_FOUND {
            NOT_FOUND
        } else {
            self.values[index as usize]
        }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#contains
    pub fn contains_key(&self, key: Option<&Rc<K>>) -> bool {
        self.get_id(key) != NOT_FOUND
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#contains
    pub fn contains_id(&self, id: i32) -> bool {
        self.by_id(id).is_some()
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#add
    pub fn add(&mut self, key: Rc<K>) -> i32 {
        let id = self.next_id_slot();
        self.add_mapping(key, id);
        id
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#nextId
    fn next_id_slot(&mut self) -> i32 {
        while (self.next_id as usize) < self.by_id.len()
            && self.by_id[self.next_id as usize].is_some()
        {
            self.next_id = self.next_id.wrapping_add(1);
        }
        self.next_id
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#grow
    fn grow(&mut self, new_capacity: usize) {
        let old_keys = std::mem::take(&mut self.keys);
        let old_values = std::mem::take(&mut self.values);
        let _ = std::mem::take(&mut self.by_id);

        let mut new_map = Self::new(new_capacity);
        for i in 0..old_keys.len() {
            if let Some(k) = old_keys[i].clone() {
                new_map.add_mapping(k, old_values[i]);
            }
        }

        self.keys = new_map.keys;
        self.values = new_map.values;
        self.by_id = new_map.by_id;
        self.next_id = new_map.next_id;
        self.size = new_map.size;
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#addMapping
    pub fn add_mapping(&mut self, key: Rc<K>, id: i32) {
        let min_capacity = std::cmp::max(id, self.size.wrapping_add(1));
        if (min_capacity as f32) >= (self.keys.len() as f32 * LOADFACTOR) {
            let mut new_len = (self.keys.len() as i32).wrapping_shl(1) as usize;
            while (new_len as i32) < id {
                new_len = ((new_len as i32).wrapping_shl(1)) as usize;
            }
            self.grow(new_len);
        }

        let h = self.hash(Some(&key));
        let slot = self.find_empty(h);
        self.keys[slot] = Some(key.clone());
        self.values[slot] = id;
        self.by_id[id as usize] = Some(key);
        self.size = self.size.wrapping_add(1);
        if id == self.next_id {
            self.next_id = self.next_id.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#hash
    fn hash(&self, key: Option<&Rc<K>>) -> usize {
        let id_hash = match key {
            Some(k) => identity_hash_code(k),
            None => 0i32,
        };
        let mixed = murmur_hash3_mixer(id_hash);
        (((mixed & i32::MAX) as u32) % (self.keys.len() as u32)) as usize
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#indexOf
    fn index_of(&self, key: Option<&Rc<K>>, start: usize) -> i32 {
        for i in start..self.keys.len() {
            if option_rc_ptr_eq(&self.keys[i], key) {
                return i as i32;
            }
            if self.keys[i].is_none() {
                return NOT_FOUND;
            }
        }
        for i in 0..start {
            if option_rc_ptr_eq(&self.keys[i], key) {
                return i as i32;
            }
            if self.keys[i].is_none() {
                return NOT_FOUND;
            }
        }
        NOT_FOUND
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#findEmpty
    fn find_empty(&self, start: usize) -> usize {
        for i in start..self.keys.len() {
            if self.keys[i].is_none() {
                return i;
            }
        }
        for i in 0..start {
            if self.keys[i].is_none() {
                return i;
            }
        }
        panic!("Overflowed :(");
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#iterator
    pub fn iter(&self) -> impl Iterator<Item = &Rc<K>> {
        self.by_id.iter().filter_map(|x| x.as_ref())
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#clear
    pub fn clear(&mut self) {
        for k in &mut self.keys {
            *k = None;
        }
        for k in &mut self.by_id {
            *k = None;
        }
        self.next_id = 0;
        self.size = 0;
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#size
    pub fn size(&self) -> i32 {
        self.size
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#copy
    pub fn copy(&self) -> Self {
        Self::from_parts(
            self.keys.clone(),
            self.values.clone(),
            self.by_id.clone(),
            self.next_id,
            self.size,
        )
    }
}

fn option_rc_ptr_eq<K>(slot: &Option<Rc<K>>, key: Option<&Rc<K>>) -> bool {
    match (slot, key) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn identity_hash_code<K>(key: &Rc<K>) -> i32 {
    Rc::as_ptr(key) as usize as i32
}

// JAVA: net.minecraft.util.Mth#murmurHash3Mixer
fn murmur_hash3_mixer(mut input: i32) -> i32 {
    input ^= ((input as u32) >> 16) as i32;
    input = input.wrapping_mul(-2048144789i32);
    input ^= ((input as u32) >> 13) as i32;
    input = input.wrapping_mul(-1028477387i32);
    input ^= ((input as u32) >> 16) as i32;
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn basic_add_get_contains_and_iter() {
        let mut map = CrudeIncrementalIntIdentityHashBiMap::create(4);
        let a = Rc::new("a");
        let b = Rc::new("b");
        let id_a = map.add(a.clone());
        let id_b = map.add(b.clone());

        assert_eq!(id_a, 0);
        assert_eq!(id_b, 1);
        assert_eq!(map.get_id(Some(&a)), 0);
        assert_eq!(map.get_id(Some(&b)), 1);
        assert!(map.contains_key(Some(&a)));
        assert!(map.contains_id(0));
        assert!(!map.contains_id(2));
        assert_eq!(map.size(), 2);
        assert!(Rc::ptr_eq(map.by_id(1).unwrap(), &b));
        assert!(map.by_id(-1).is_none());
        assert!(map.by_id(99).is_none());

        let items: Vec<&str> = map.iter().map(|rc| **rc).collect();
        assert_eq!(items, vec!["a", "b"]);
    }

    #[test]
    fn identity_semantics_not_value_equality() {
        let mut map = CrudeIncrementalIntIdentityHashBiMap::create(4);
        let a1 = Rc::new(42i32);
        let a2 = Rc::new(42i32);
        let id1 = map.add(a1.clone());
        let id2 = map.add(a2.clone());

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(map.get_id(Some(&a1)), 0);
        assert_eq!(map.get_id(Some(&a2)), 1);
        assert_eq!(map.size(), 2);
    }

    #[test]
    fn add_mapping_growth_and_next_id_scan() {
        let mut map = CrudeIncrementalIntIdentityHashBiMap::create(1);
        let k1 = Rc::new(1i32);
        let k2 = Rc::new(2i32);
        let k3 = Rc::new(3i32);

        map.add_mapping(k1.clone(), 0);
        map.add_mapping(k2.clone(), 5);

        assert_eq!(map.get_id(Some(&k1)), 0);
        assert_eq!(map.get_id(Some(&k2)), 5);
        assert!(map.contains_id(5));
        assert_eq!(map.size(), 2);

        let id3 = map.add(k3.clone());
        assert_eq!(id3, 1);
        assert_eq!(map.get_id(Some(&k3)), 1);
    }

    #[test]
    fn clear_and_copy_preserve_identity() {
        let mut map = CrudeIncrementalIntIdentityHashBiMap::create(4);
        let a = Rc::new(10i32);
        map.add(a.clone());

        let copy = map.copy();
        assert_eq!(copy.get_id(Some(&a)), 0);
        assert!(Rc::ptr_eq(copy.by_id(0).unwrap(), &a));

        map.clear();
        assert_eq!(map.size(), 0);
        assert_eq!(map.get_id(Some(&a)), -1);
        assert!(!map.contains_id(0));

        assert_eq!(copy.size(), 1);
        assert_eq!(copy.get_id(Some(&a)), 0);
    }

    #[test]
    fn negative_and_missing_lookups() {
        let mut map: CrudeIncrementalIntIdentityHashBiMap<i32> =
            CrudeIncrementalIntIdentityHashBiMap::create(4);
        let a = Rc::new(7i32);
        map.add(a.clone());
        let missing = Rc::new(7i32);

        assert_eq!(map.get_id(Some(&missing)), -1);
        assert!(map.by_id(-1).is_none());
        assert!(map.by_id(i32::MIN).is_none());
        assert!(!map.contains_id(-5));
        assert!(!map.contains_id(1));
    }

    #[test]
    fn create_uses_float_load_factor() {
        let map5: CrudeIncrementalIntIdentityHashBiMap<i32> =
            CrudeIncrementalIntIdentityHashBiMap::create(5);
        let map8: CrudeIncrementalIntIdentityHashBiMap<i32> =
            CrudeIncrementalIntIdentityHashBiMap::create(8);

        // Java: (int)(initialCapacity / 0.8F)
        assert_eq!(map5.keys.len(), 6); // (5 / 0.8f) as int = 6
        assert_eq!(map8.keys.len(), 10); // (8 / 0.8f) as int = 10
    }
}
