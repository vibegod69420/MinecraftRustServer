#[derive(Debug)]
pub struct SingleKeyCache<K, V, F> {
    compute_value: F,
    cache_key: Option<K>,
    cached_value: Option<V>,
}

impl<K, V, F> SingleKeyCache<K, V, F>
where
    K: PartialEq + Clone,
    F: FnMut(&K) -> V,
{
    // JAVA: net.minecraft.util.SingleKeyCache#<init>
    pub fn new(compute_value: F) -> Self {
        Self {
            compute_value,
            cache_key: None,
            cached_value: None,
        }
    }

    // JAVA: net.minecraft.util.SingleKeyCache#getValue
    pub fn get_value(&mut self, key: K) -> &V {
        let miss = match self.cached_value {
            None => true,
            Some(_) => self.cache_key.as_ref() != Some(&key),
        };
        if miss {
            let value = (self.compute_value)(&key);
            self.cache_key = Some(key);
            self.cached_value = Some(value);
        }
        self.cached_value
            .as_ref()
            .expect("cached value must be present after computation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn computes_on_first_access() {
        let calls = Cell::new(0i32);
        let mut cache = SingleKeyCache::new(|k: &i32| {
            calls.set(calls.get().wrapping_add(1));
            k.wrapping_mul(10)
        });
        assert_eq!(*cache.get_value(3), 30);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn caches_same_key() {
        let calls = Cell::new(0i32);
        let mut cache = SingleKeyCache::new(|k: &i32| {
            calls.set(calls.get().wrapping_add(1));
            *k
        });
        assert_eq!(*cache.get_value(7), 7);
        assert_eq!(*cache.get_value(7), 7);
        assert_eq!(*cache.get_value(7), 7);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn recomputes_on_different_key() {
        let calls = Cell::new(0i32);
        let mut cache = SingleKeyCache::new(|k: &i32| {
            calls.set(calls.get().wrapping_add(1));
            k.wrapping_add(100)
        });
        assert_eq!(*cache.get_value(1), 101);
        assert_eq!(*cache.get_value(2), 102);
        assert_eq!(*cache.get_value(1), 101);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn works_with_owned_string_keys() {
        let calls = Cell::new(0i32);
        let mut cache = SingleKeyCache::new(|k: &String| {
            calls.set(calls.get().wrapping_add(1));
            k.len()
        });
        assert_eq!(*cache.get_value("abc".to_string()), 3);
        assert_eq!(*cache.get_value("abc".to_string()), 3);
        assert_eq!(*cache.get_value("wxyz".to_string()), 4);
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn negative_and_extreme_keys() {
        let mut cache = SingleKeyCache::new(|k: &i32| *k);
        assert_eq!(*cache.get_value(i32::MIN), i32::MIN);
        assert_eq!(*cache.get_value(i32::MAX), i32::MAX);
        assert_eq!(*cache.get_value(-1), -1);
    }
}
