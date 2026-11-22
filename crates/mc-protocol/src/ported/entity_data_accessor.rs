//! Port of `net.minecraft.network.syncher.EntityDataAccessor`.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Minimal stub for `net.minecraft.network.syncher.EntityDataSerializer`.
/// This module is self-contained; the canonical type lives in `entity_data_serializer.rs`.
pub trait EntityDataSerializer<T>: Send + Sync {
    fn codec_stub(&self);
    fn copy_value(&self, value: &T) -> T;
}

/// Port of `net.minecraft.network.syncher.EntityDataAccessor`.
#[derive(Clone)]
pub struct EntityDataAccessor<T> {
    pub id: i32,
    pub serializer: Arc<dyn EntityDataSerializer<T>>,
}

impl<T> EntityDataAccessor<T> {
    pub fn new(id: i32, serializer: Arc<dyn EntityDataSerializer<T>>) -> Self {
        Self { id, serializer }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn serializer(&self) -> &Arc<dyn EntityDataSerializer<T>> {
        &self.serializer
    }
}

// JAVA: net.minecraft.network.syncher.EntityDataAccessor#equals
impl<T> PartialEq for EntityDataAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for EntityDataAccessor<T> {}

// JAVA: net.minecraft.network.syncher.EntityDataAccessor#hashCode
impl<T> Hash for EntityDataAccessor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// JAVA: net.minecraft.network.syncher.EntityDataAccessor#toString
impl<T> fmt::Display for EntityDataAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<entity data: {}>", self.id)
    }
}

impl<T> fmt::Debug for EntityDataAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityDataAccessor")
            .field("id", &self.id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    struct MockSerializer;

    impl EntityDataSerializer<i32> for MockSerializer {
        fn codec_stub(&self) {}

        fn copy_value(&self, value: &i32) -> i32 {
            *value
        }
    }

    fn make_accessor(id: i32) -> EntityDataAccessor<i32> {
        EntityDataAccessor::new(id, Arc::new(MockSerializer))
    }

    fn hash_of<T: Hash>(val: &T) -> u64 {
        let mut h = DefaultHasher::new();
        val.hash(&mut h);
        h.finish()
    }

    #[test]
    fn equals_same_id() {
        let a = make_accessor(42);
        let b = make_accessor(42);
        assert_eq!(a, b);
    }

    #[test]
    fn not_equals_different_id() {
        let a = make_accessor(1);
        let b = make_accessor(2);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_code_based_on_id() {
        let a = make_accessor(123);
        let mut id_hasher = DefaultHasher::new();
        123i32.hash(&mut id_hasher);
        let mut acc_hasher = DefaultHasher::new();
        a.hash(&mut acc_hasher);
        assert_eq!(id_hasher.finish(), acc_hasher.finish());
        assert_eq!(hash_of(&make_accessor(123)), hash_of(&make_accessor(123)));
    }

    #[test]
    fn to_string_format() {
        let a = make_accessor(7);
        assert_eq!(a.to_string(), "<entity data: 7>");
    }

    #[test]
    fn negative_and_extreme_ids() {
        let neg = make_accessor(-1);
        let min = make_accessor(i32::MIN);
        let max = make_accessor(i32::MAX);
        assert_eq!(neg.to_string(), "<entity data: -1>");
        assert_eq!(min.to_string(), format!("<entity data: {}>", i32::MIN));
        assert_eq!(max.to_string(), format!("<entity data: {}>", i32::MAX));
        assert_ne!(min, max);
        assert_eq!(make_accessor(-1), neg);
    }

    #[test]
    fn equals_ignores_serializer_identity() {
        let a = EntityDataAccessor::new(5, Arc::new(MockSerializer));
        let b = EntityDataAccessor::new(5, Arc::new(MockSerializer));
        assert_eq!(a, b);
    }
}
