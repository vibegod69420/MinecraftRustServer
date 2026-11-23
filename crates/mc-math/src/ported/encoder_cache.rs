use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A self-contained faithful port of Minecraft's `EncoderCache`.
///
/// Java uses Guava `LoadingCache` with `maximumSize`, `concurrencyLevel(1)`,
/// `softValues()`, and caches `DataResult<T>` keyed by
/// `(Codec identity, value equality, DynamicOps equality)`.
///
/// Since `mc-math` cannot depend on Mojang DataFixerUpper/Codec types,
/// this module provides a generic, self-contained cache with the same
/// observable key semantics:
/// - codec identity (pointer identity in Rust via `Arc` identity or explicit id)
/// - value equality
/// - ops equality
///
/// The cache stores encoded results as `Arc<dyn EncodedValue>`-like generic
/// payloads through a type-erased trait object is not possible without unsafe
/// or extra deps in a clean way, so we model the Java behavior directly:
/// `wrap` returns an `EncoderCacheCodec` that caches encode results.
///
/// For faithful porting in a math-only crate, we expose a generic cache keyed
/// by a user-provided key and a loader, plus a specialized helper for the
/// common "encode to bytes/string/tag-like clone" pattern.
#[derive(Debug)]
pub struct EncoderCache<K, V> {
    max_size: usize,
    // Java: concurrencyLevel(1), so Mutex is faithful enough.
    // We use a simple insertion-order eviction approximation for maximumSize.
    // Guava's maximumSize eviction is LRU-like; for faithful behavioral porting
    // in this self-contained module we preserve capacity bound and key semantics.
    inner: Mutex<EncoderCacheState<K, V>>,
}

#[derive(Debug)]
struct EncoderCacheState<K, V> {
    map: HashMap<K, Arc<V>>,
    order: Vec<K>,
}

impl<K, V> EncoderCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    // JAVA: net.minecraft.util.EncoderCache#<init>
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size: max_size.max(1),
            inner: Mutex::new(EncoderCacheState {
                map: HashMap::new(),
                order: Vec::new(),
            }),
        }
    }

    /// Equivalent to Guava `LoadingCache.getUnchecked`: returns cached value or loads it.
    /// Java's loader returns `DataResult<?>`; here we return `Arc<V>` to allow cheap cloning
    /// of cached encoded results, mirroring Java's cached object reuse/copy behavior at call site.
    pub fn get_or_load<F, E>(&self, key: K, loader: F) -> Result<Arc<V>, E>
    where
        F: FnOnce(&K) -> Result<V, E>,
    {
        {
            let state = self.inner.lock().expect("encoder cache lock poisoned");
            if let Some(v) = state.map.get(&key) {
                return Ok(Arc::clone(v));
            }
        }

        let loaded = loader(&key)?;
        let value = Arc::new(loaded);

        let mut state = self.inner.lock().expect("encoder cache lock poisoned");
        if let Some(v) = state.map.get(&key) {
            return Ok(Arc::clone(v));
        }

        if state.map.len() >= self.max_size {
            // Evict oldest inserted key to respect maximumSize.
            if let Some(old_key) = state.order.first().cloned() {
                state.order.remove(0);
                state.map.remove(&old_key);
            }
        }

        state.order.push(key.clone());
        state.map.insert(key, Arc::clone(&value));
        Ok(value)
    }

    pub fn len(&self) -> usize {
        let state = self.inner.lock().expect("encoder cache lock poisoned");
        state.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Faithful key semantics for `EncoderCache.Key<A, T>`.
///
/// Java:
/// - equals: same instance fast path, then codec identity (`==`), value.equals, ops.equals
/// - hashCode: identityHashCode(codec) * 31 chain with value.hashCode and ops.hashCode
///
/// In Rust we represent codec identity as a `usize` token (e.g. address/id provided by caller)
/// and ops identity/equality as a comparable value.
#[derive(Clone, Debug)]
pub struct EncoderCacheKey<V, O> {
    pub codec_id: usize,
    pub value: V,
    pub ops: O,
}

impl<V, O> EncoderCacheKey<V, O> {
    // JAVA: net.minecraft.util.EncoderCache$Key#<init>
    pub fn new(codec_id: usize, value: V, ops: O) -> Self {
        Self {
            codec_id,
            value,
            ops,
        }
    }
}

impl<V: PartialEq, O: PartialEq> PartialEq for EncoderCacheKey<V, O> {
    // JAVA: net.minecraft.util.EncoderCache$Key#equals
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        self.codec_id == other.codec_id && self.value == other.value && self.ops == other.ops
    }
}

impl<V: PartialEq, O: PartialEq> Eq for EncoderCacheKey<V, O> {}

impl<V: std::hash::Hash, O: std::hash::Hash> std::hash::Hash for EncoderCacheKey<V, O> {
    // JAVA: net.minecraft.util.EncoderCache$Key#hashCode
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Java uses System.identityHashCode(codec) as first component.
        // We hash the provided codec_id as the identity surrogate.
        let mut h: i32 = self.codec_id as i32;
        // Mix in value and ops hashes using Java's 31*x+y pattern on i32 wrapping.
        let mut hasher_v = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self.value, &mut hasher_v);
        let vh = std::hash::Hasher::finish(&hasher_v) as i32;
        h = h.wrapping_mul(31).wrapping_add(vh);
        let mut hasher_o = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self.ops, &mut hasher_o);
        let oh = std::hash::Hasher::finish(&hasher_o) as i32;
        h = h.wrapping_mul(31).wrapping_add(oh);
        state.write_i32(h);
    }
}

impl<V, O> EncoderCacheKey<V, O> {
    // JAVA: net.minecraft.util.EncoderCache$Key#resolve
    pub fn resolve<T, E, F>(&self, mut encode_start: F) -> Result<T, E>
    where
        F: FnMut(&V, &O) -> Result<T, E>,
    {
        encode_start(&self.value, &self.ops)
    }
}

/// A generic wrapped encoder mirroring `EncoderCache.wrap(codec)`.
///
/// Java `encode` does:
/// `cache.getUnchecked(new Key<>(codec, value, ops)).map(x -> x instanceof Tag ? copy : x)`
///
/// This Rust version caches encoded values and returns a clone via `Clone`.
/// Callers can provide tag-like copy semantics by implementing `Clone` appropriately.
pub struct WrappedEncoder<V, O, T> {
    codec_id: usize,
    cache: Arc<EncoderCache<EncoderCacheKey<V, O>, T>>,
}

impl<V, O, T> WrappedEncoder<V, O, T>
where
    V: std::hash::Hash + Eq + Clone,
    O: std::hash::Hash + Eq + Clone,
    T: Clone,
{
    // JAVA: net.minecraft.util.EncoderCache#wrap
    pub fn new(codec_id: usize, max_size: usize) -> Self {
        Self {
            codec_id,
            cache: Arc::new(EncoderCache::new(max_size)),
        }
    }

    pub fn from_cache(codec_id: usize, cache: Arc<EncoderCache<EncoderCacheKey<V, O>, T>>) -> Self {
        Self { codec_id, cache }
    }

    /// Mirrors Java `encode`: cache lookup/load by key, then return copied/cloned value.
    pub fn encode<F, E>(&self, value: V, ops: O, encode_start: F) -> Result<T, E>
    where
        F: FnOnce(&V, &O) -> Result<T, E>,
    {
        let key = EncoderCacheKey::new(self.codec_id, value, ops);
        let cached = self.cache.get_or_load(key.clone(), |k| encode_start(&k.value, &k.ops))?;
        Ok((*cached).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn key_equality_uses_codec_identity_and_value_ops_equality() {
        let k1 = EncoderCacheKey::new(1, "abc".to_string(), 10i32);
        let k2 = EncoderCacheKey::new(1, "abc".to_string(), 10i32);
        let k3 = EncoderCacheKey::new(2, "abc".to_string(), 10i32);
        let k4 = EncoderCacheKey::new(1, "def".to_string(), 10i32);
        let k5 = EncoderCacheKey::new(1, "abc".to_string(), 11i32);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        assert_ne!(k1, k4);
        assert_ne!(k1, k5);
    }

    #[test]
    fn cache_loads_once_per_key() {
        let cache: EncoderCache<i32, i32> = EncoderCache::new(16);
        let loads = AtomicUsize::new(0);
        let v1 = cache
            .get_or_load(5, |k| {
                loads.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(*k * 2)
            })
            .unwrap();
        let v2 = cache
            .get_or_load(5, |k| {
                loads.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(*k * 3)
            })
            .unwrap();
        assert_eq!(*v1, 10);
        assert_eq!(*v2, 10);
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cache_respects_max_size() {
        let cache: EncoderCache<i32, i32> = EncoderCache::new(2);
        cache.get_or_load(1, |_| Ok::<_, ()>(10)).unwrap();
        cache.get_or_load(2, |_| Ok::<_, ()>(20)).unwrap();
        cache.get_or_load(3, |_| Ok::<_, ()>(30)).unwrap();
        assert!(cache.len() <= 2);
        let v3 = cache.get_or_load(3, |_| Ok::<_, ()>(999)).unwrap();
        assert_eq!(*v3, 30);
    }

    #[test]
    fn wrapped_encoder_caches_and_clones() {
        let enc: WrappedEncoder<String, i32, Vec<u8>> = WrappedEncoder::new(42, 8);
        let loads = AtomicUsize::new(0);
        let r1 = enc
            .encode("a".to_string(), 1, |v, o| {
                loads.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(format!("{}:{}", v, o).into_bytes())
            })
            .unwrap();
        let r2 = enc
            .encode("a".to_string(), 1, |_v, _o| {
                loads.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(b"other".to_vec())
            })
            .unwrap();
        assert_eq!(r1, b"a:1".to_vec());
        assert_eq!(r2, b"a:1".to_vec());
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn key_resolve_invokes_encoder() {
        let key = EncoderCacheKey::new(7, 3i32, 4i32);
        let out = key.resolve(|v, o| Ok::<_, ()>(v + o)).unwrap();
        assert_eq!(out, 7);
    }
}
