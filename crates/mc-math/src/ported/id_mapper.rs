use std::collections::HashMap;
use std::hash::Hash;

/// Faithful port of Minecraft's `IdMapper<T>`.
///
/// Java uses identity semantics (`Reference2IntOpenHashMap`) and allows null
/// slots in `idToT`. Rust's `HashMap`/`Eq` is used for keys; callers should use
/// types with appropriate identity/value semantics. Values are stored as
/// `Option<T>` to preserve null slots created by sparse `add_mapping`.
#[derive(Clone, Debug)]
pub struct IdMapper<T> {
    next_id: i32,
    t_to_id: HashMap<T, i32>,
    id_to_t: Vec<Option<T>>,
}

impl<T> Default for IdMapper<T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IdMapper<T>
where
    T: Eq + Hash + Clone,
{
    // JAVA: net.minecraft.core.IdMapper#<init>()
    pub fn new() -> Self {
        Self::with_expected_size(512)
    }

    // JAVA: net.minecraft.core.IdMapper#<init>(int)
    pub fn with_expected_size(expected_size: usize) -> Self {
        let mut t_to_id = HashMap::with_capacity(expected_size);
        // Java Reference2IntOpenHashMap.defaultReturnValue(-1) is represented by
        // returning -1 from get_id when absent.
        let _ = &mut t_to_id;
        Self {
            next_id: 0,
            t_to_id,
            id_to_t: Vec::with_capacity(expected_size),
        }
    }

    // JAVA: net.minecraft.core.IdMapper#addMapping
    pub fn add_mapping(&mut self, value: T, id: i32) {
        self.t_to_id.insert(value.clone(), id);

        let needed_len = (id as usize).wrapping_add(1);
        while self.id_to_t.len() < needed_len {
            self.id_to_t.push(None);
        }
        if id >= 0 {
            let idx = id as usize;
            self.id_to_t[idx] = Some(value);
        }

        if self.next_id <= id {
            self.next_id = id.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.core.IdMapper#add
    pub fn add(&mut self, value: T) {
        let id = self.next_id;
        self.add_mapping(value, id);
    }

    // JAVA: net.minecraft.core.IdMapper#getId
    pub fn get_id(&self, value: &T) -> i32 {
        self.t_to_id.get(value).copied().unwrap_or(-1)
    }

    // JAVA: net.minecraft.core.IdMapper#byId
    pub fn by_id(&self, id: i32) -> Option<&T> {
        if id < 0 {
            return None;
        }
        let idx = id as usize;
        if idx >= self.id_to_t.len() {
            return None;
        }
        self.id_to_t[idx].as_ref()
    }

    // JAVA: net.minecraft.core.IdMapper#contains
    pub fn contains(&self, id: i32) -> bool {
        self.by_id(id).is_some()
    }

    // JAVA: net.minecraft.core.IdMapper#size
    pub fn size(&self) -> i32 {
        self.t_to_id.len() as i32
    }

    // JAVA: net.minecraft.core.IdMapper#iterator
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.id_to_t.iter().filter_map(|v| v.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_lookup() {
        let mut mapper = IdMapper::new();
        mapper.add("a");
        mapper.add("b");

        assert_eq!(mapper.get_id(&"a"), 0);
        assert_eq!(mapper.get_id(&"b"), 1);
        assert_eq!(mapper.by_id(0), Some(&"a"));
        assert_eq!(mapper.by_id(1), Some(&"b"));
        assert_eq!(mapper.size(), 2);
    }

    #[test]
    fn add_mapping_sparse_and_contains() {
        let mut mapper = IdMapper::new();
        mapper.add_mapping("x", 3);
        assert_eq!(mapper.by_id(0), None);
        assert_eq!(mapper.by_id(2), None);
        assert_eq!(mapper.by_id(3), Some(&"x"));
        assert!(!mapper.contains(2));
        assert!(mapper.contains(3));
        assert_eq!(mapper.size(), 1);
    }

    #[test]
    fn next_id_advances_after_sparse_mapping() {
        let mut mapper = IdMapper::new();
        mapper.add_mapping("x", 5);
        mapper.add("y");
        assert_eq!(mapper.get_id(&"y"), 6);
        assert_eq!(mapper.by_id(6), Some(&"y"));
    }

    #[test]
    fn missing_and_negative_ids() {
        let mapper: IdMapper<&str> = IdMapper::new();
        assert_eq!(mapper.get_id(&"missing"), -1);
        assert_eq!(mapper.by_id(-1), None);
        assert_eq!(mapper.by_id(i32::MIN), None);
        assert!(!mapper.contains(-1));
    }

    #[test]
    fn iterator_skips_null_slots() {
        let mut mapper = IdMapper::new();
        mapper.add_mapping("a", 0);
        mapper.add_mapping("c", 2);
        let values: Vec<&str> = mapper.iter().copied().collect();
        assert_eq!(values, vec!["a", "c"]);
    }

    #[test]
    fn remap_same_value_updates_id() {
        let mut mapper = IdMapper::new();
        mapper.add("a");
        mapper.add_mapping("a", 10);
        assert_eq!(mapper.get_id(&"a"), 10);
        assert_eq!(mapper.by_id(0), Some(&"a"));
        assert_eq!(mapper.by_id(10), Some(&"a"));
        assert_eq!(mapper.size(), 1);
    }
}
