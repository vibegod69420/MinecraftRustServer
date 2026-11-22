use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for referenced Minecraft types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn to_string_id(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

pub struct ResourceKey<T> {
    pub location: ResourceLocation,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ResourceKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceKey")
            .field("location", &self.location)
            .finish()
    }
}

impl<T> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl<T> Eq for ResourceKey<T> {}

impl<T> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl<T> ResourceKey<T> {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            _marker: PhantomData,
        }
    }

    pub fn identifier(&self) -> String {
        self.location.to_string_id()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructureTemplatePool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal deterministic random source stub.
/// Provides the interface used by PoolAliasLookup/PoolAliasBinding.
#[derive(Clone, Debug)]
pub struct RandomSource {
    state_lo: u64,
    state_hi: u64,
}

impl RandomSource {
    fn mix64(mut z: u64) -> u64 {
        z = (z ^ (z >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        z = (z ^ (z >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
        z ^ (z >> 33)
    }

    // JAVA: net.minecraft.util.RandomSource#create (behavioral stub for deterministic seeding)
    pub fn create(seed: i64) -> Self {
        let s = seed as u64;
        let lo = Self::mix64(s);
        let hi = Self::mix64(s.wrapping_add(0x9e3779b97f4a7c15));
        Self {
            state_lo: if lo == 0 && hi == 0 { 1 } else { lo },
            state_hi: hi,
        }
    }

    pub fn fork_positional(&self) -> PositionalRandomFactory {
        PositionalRandomFactory {
            seed_lo: self.state_lo,
            seed_hi: self.state_hi,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xoshiro256**-style step simplified to 128-bit state for a compact stub
        let s0 = self.state_lo;
        let mut s1 = self.state_hi;
        let result = s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        s1 ^= s0;
        self.state_lo = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
        self.state_hi = s1.rotate_left(37);
        result
    }

    pub fn next_i32(&mut self) -> i32 {
        self.next_u64() as i32
    }

    pub fn next_i64(&mut self) -> i64 {
        self.next_u64() as i64
    }
}

pub struct PositionalRandomFactory {
    seed_lo: u64,
    seed_hi: u64,
}

impl PositionalRandomFactory {
    pub fn at(&self, pos: BlockPos) -> RandomSource {
        let mut s = self.seed_lo ^ (pos.x as i64 as u64).wrapping_mul(0x9E3779B97F4A7C15);
        s ^= (pos.y as i64 as u64).wrapping_mul(0xBF58476D1CE4E5B9);
        s ^= (pos.z as i64 as u64).wrapping_mul(0x94D049BB133111EB);
        s = s.wrapping_add(self.seed_hi);
        RandomSource::create(s as i64)
    }
}

/// Minimal self-contained trait stub for PoolAliasBinding.
pub trait PoolAliasBinding {
    fn for_each_resolved(
        &self,
        random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    );
}

// ---------------------------------------------------------------------------
// Port of PoolAliasLookup
// ---------------------------------------------------------------------------

pub struct PoolAliasLookup {
    lookup_fn: Box<
        dyn Fn(ResourceKey<StructureTemplatePool>) -> ResourceKey<StructureTemplatePool>
            + Send
            + Sync,
    >,
}

impl PoolAliasLookup {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasLookup#lookup
    pub fn lookup(
        &self,
        key: ResourceKey<StructureTemplatePool>,
    ) -> ResourceKey<StructureTemplatePool> {
        (self.lookup_fn)(key)
    }

    /// Equivalent of Java's `PoolAliasLookup EMPTY = $$0 -> $$0`.
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasLookup#EMPTY
    pub fn empty() -> Self {
        Self {
            lookup_fn: Box::new(|key| key),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasLookup#create
    pub fn create(bindings: &[Box<dyn PoolAliasBinding>], pos: BlockPos, seed: i64) -> Self {
        if bindings.is_empty() {
            return Self::empty();
        }

        let mut random = RandomSource::create(seed).fork_positional().at(pos);
        let mut map: HashMap<
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        > = HashMap::new();

        for binding in bindings {
            binding.for_each_resolved(&mut random, &mut |k, v| {
                if map.contains_key(&k) {
                    panic!(
                        "Multiple entries with same key: {}",
                        k.identifier()
                    );
                }
                map.insert(k, v);
            });
        }

        Self {
            lookup_fn: Box::new(move |key| match map.get(&key) {
                Some(value) => value.clone(),
                None => key,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(path: &str) -> ResourceKey<StructureTemplatePool> {
        ResourceKey::new(ResourceLocation::new("minecraft", path))
    }

    struct DirectBinding {
        from: ResourceKey<StructureTemplatePool>,
        to: ResourceKey<StructureTemplatePool>,
    }

    impl PoolAliasBinding for DirectBinding {
        fn for_each_resolved(
            &self,
            _random: &mut RandomSource,
            consumer: &mut dyn FnMut(
                ResourceKey<StructureTemplatePool>,
                ResourceKey<StructureTemplatePool>,
            ),
        ) {
            consumer(self.from.clone(), self.to.clone());
        }
    }

    fn direct(from: &str, to: &str) -> Box<dyn PoolAliasBinding> {
        Box::new(DirectBinding {
            from: key(from),
            to: key(to),
        })
    }

    #[test]
    fn empty_lookup_returns_same_key() {
        let lookup = PoolAliasLookup::empty();
        let k = key("start");
        assert_eq!(lookup.lookup(k.clone()), k);
    }

    #[test]
    fn create_empty_bindings_returns_identity() {
        let lookup = PoolAliasLookup::create(&[], BlockPos::new(0, 0, 0), 0);
        let k = key("pool");
        assert_eq!(lookup.lookup(k.clone()), k);
    }

    #[test]
    fn create_maps_known_key_and_preserves_unknown() {
        let bindings = vec![direct("a", "b")];
        let lookup = PoolAliasLookup::create(&bindings, BlockPos::new(1, 2, 3), 42);
        assert_eq!(lookup.lookup(key("a")), key("b"));
        assert_eq!(lookup.lookup(key("unknown")), key("unknown"));
    }

    #[test]
    fn create_is_deterministic_for_same_seed_and_pos() {
        let bindings = vec![direct("a", "b"), direct("c", "d")];
        let l1 = PoolAliasLookup::create(&bindings, BlockPos::new(10, -5, 7), 12345);
        let l2 = PoolAliasLookup::create(&bindings, BlockPos::new(10, -5, 7), 12345);
        assert_eq!(l1.lookup(key("a")), l2.lookup(key("a")));
        assert_eq!(l1.lookup(key("c")), l2.lookup(key("c")));
        assert_eq!(l1.lookup(key("x")), l2.lookup(key("x")));
    }

    #[test]
    #[should_panic(expected = "Multiple entries with same key")]
    fn create_panics_on_duplicate_key() {
        let bindings = vec![direct("a", "b"), direct("a", "c")];
        let _ = PoolAliasLookup::create(&bindings, BlockPos::new(0, 0, 0), 1);
    }

    #[test]
    fn create_with_negative_pos_and_seed() {
        let bindings = vec![direct("pool", "target")];
        let lookup = PoolAliasLookup::create(
            &bindings,
            BlockPos::new(i32::MIN, -1, i32::MAX),
            i64::MIN,
        );
        assert_eq!(lookup.lookup(key("pool")), key("target"));
        assert_eq!(lookup.lookup(key("other")), key("other"));
    }
}
