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

/// Minimal deterministic random source stub.
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

    pub fn create(seed: i64) -> Self {
        let s = seed as u64;
        let lo = Self::mix64(s);
        let hi = Self::mix64(s.wrapping_add(0x9e3779b97f4a7c15));
        Self {
            state_lo: if lo == 0 && hi == 0 { 1 } else { lo },
            state_hi: hi,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let s0 = self.state_lo;
        let mut s1 = self.state_hi;
        let result = s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        s1 ^= s0;
        self.state_lo = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
        self.state_hi = s1.rotate_left(37);
        result
    }

    pub fn next_i32(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        let bound_u = bound as u32;
        let mut bits: u32;
        let mut val: i32;
        loop {
            bits = (self.next_u64() >> 1) as u32;
            val = (bits % bound_u) as i32;
            if (bits as i32).wrapping_sub(val).wrapping_add(bound.wrapping_sub(1)) >= 0 {
                break;
            }
        }
        val
    }
}

/// Minimal WeightedList stub matching the behavior used by RandomGroupPoolAlias.
#[derive(Clone, Debug)]
pub struct WeightedEntry<T: Clone> {
    data: T,
    weight: i32,
}

impl<T: Clone> WeightedEntry<T> {
    pub fn new(data: T, weight: i32) -> Self {
        Self { data, weight }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }
}

#[derive(Clone, Debug)]
pub struct WeightedList<T: Clone> {
    entries: Vec<WeightedEntry<T>>,
    total_weight: i32,
}

impl<T: Clone> WeightedList<T> {
    pub fn of(entries: Vec<WeightedEntry<T>>) -> Self {
        let mut total_weight: i32 = 0;
        for e in &entries {
            total_weight = total_weight.wrapping_add(e.weight);
        }
        Self {
            entries,
            total_weight,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn unwrap_entries(&self) -> &[WeightedEntry<T>] {
        &self.entries
    }

    pub fn get_random(&self, random: &mut RandomSource) -> Option<&T> {
        if self.total_weight <= 0 {
            return None;
        }
        let mut remaining = random.next_i32(self.total_weight);
        for entry in &self.entries {
            remaining = remaining.wrapping_sub(entry.weight);
            if remaining < 0 {
                return Some(&entry.data);
            }
        }
        None
    }
}

/// Minimal self-contained trait stub for PoolAliasBinding.
pub trait PoolAliasBinding: PoolAliasBindingClone {
    fn for_each_resolved(
        &self,
        random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    );

    fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>>;
}

/// Helper trait to allow cloning boxed trait objects.
pub trait PoolAliasBindingClone {
    fn clone_box(&self) -> Box<dyn PoolAliasBinding>;
}

impl<T> PoolAliasBindingClone for T
where
    T: 'static + PoolAliasBinding + Clone,
{
    fn clone_box(&self) -> Box<dyn PoolAliasBinding> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn PoolAliasBinding> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ---------------------------------------------------------------------------
// Port of RandomGroupPoolAlias
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RandomGroupPoolAlias {
    groups: WeightedList<Vec<Box<dyn PoolAliasBinding>>>,
}

impl RandomGroupPoolAlias {
    pub fn new(groups: WeightedList<Vec<Box<dyn PoolAliasBinding>>>) -> Self {
        Self { groups }
    }

    pub fn groups(&self) -> &WeightedList<Vec<Box<dyn PoolAliasBinding>>> {
        &self.groups
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomGroupPoolAlias#forEachResolved
    pub fn for_each_resolved(
        &self,
        random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    ) {
        if let Some(selected) = self.groups.get_random(random) {
            for binding in selected {
                binding.for_each_resolved(random, consumer);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomGroupPoolAlias#allTargets
    pub fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
        let mut out = Vec::new();
        for entry in self.groups.unwrap_entries() {
            for binding in entry.data() {
                out.extend(binding.all_targets());
            }
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomGroupPoolAlias#codec
    // Codec is not ported; this method is a placeholder to preserve the Java method presence.
    pub fn codec_marker(&self) -> &'static str {
        "RandomGroupPoolAlias.CODEC"
    }
}

impl PoolAliasBinding for RandomGroupPoolAlias {
    fn for_each_resolved(
        &self,
        random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    ) {
        RandomGroupPoolAlias::for_each_resolved(self, random, consumer);
    }

    fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
        RandomGroupPoolAlias::all_targets(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(path: &str) -> ResourceKey<StructureTemplatePool> {
        ResourceKey::new(ResourceLocation::new("minecraft", path))
    }

    #[derive(Clone)]
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

        fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
            vec![self.to.clone()]
        }
    }

    fn direct(from: &str, to: &str) -> Box<dyn PoolAliasBinding> {
        Box::new(DirectBinding {
            from: key(from),
            to: key(to),
        })
    }

    #[test]
    fn empty_groups_resolves_nothing() {
        let groups: WeightedList<Vec<Box<dyn PoolAliasBinding>>> = WeightedList::of(vec![]);
        let alias = RandomGroupPoolAlias::new(groups);
        let mut out = Vec::new();
        let mut rng = RandomSource::create(0);
        alias.for_each_resolved(&mut rng, &mut |k, v| out.push((k, v)));
        assert!(out.is_empty());
        assert!(alias.all_targets().is_empty());
    }

    #[test]
    fn single_group_resolves_its_bindings() {
        let groups = WeightedList::of(vec![WeightedEntry::new(
            vec![direct("a", "t1"), direct("b", "t2")],
            1,
        )]);
        let alias = RandomGroupPoolAlias::new(groups);
        let mut out = Vec::new();
        let mut rng = RandomSource::create(42);
        alias.for_each_resolved(&mut rng, &mut |k, v| out.push((k, v)));
        assert_eq!(out, vec![(key("a"), key("t1")), (key("b"), key("t2"))]);
    }

    #[test]
    fn all_targets_collects_across_all_groups() {
        let groups = WeightedList::of(vec![
            WeightedEntry::new(vec![direct("a", "t1")], 1),
            WeightedEntry::new(vec![direct("b", "t2"), direct("c", "t3")], 2),
        ]);
        let alias = RandomGroupPoolAlias::new(groups);
        let targets = alias.all_targets();
        assert_eq!(targets, vec![key("t1"), key("t2"), key("t3")]);
    }

    #[test]
    fn for_each_resolved_is_deterministic_for_same_seed() {
        let groups = WeightedList::of(vec![
            WeightedEntry::new(vec![direct("a", "t1")], 1),
            WeightedEntry::new(vec![direct("a", "t2")], 1),
        ]);
        let alias = RandomGroupPoolAlias::new(groups);

        let mut out1 = Vec::new();
        let mut rng1 = RandomSource::create(123);
        alias.for_each_resolved(&mut rng1, &mut |k, v| out1.push((k, v)));

        let mut out2 = Vec::new();
        let mut rng2 = RandomSource::create(123);
        alias.for_each_resolved(&mut rng2, &mut |k, v| out2.push((k, v)));

        assert_eq!(out1, out2);
        assert_eq!(out1.len(), 1);
    }

    #[test]
    fn zero_weight_group_never_selected_when_positive_exists() {
        let groups = WeightedList::of(vec![
            WeightedEntry::new(vec![direct("a", "zero")], 0),
            WeightedEntry::new(vec![direct("a", "positive")], 10),
        ]);
        let alias = RandomGroupPoolAlias::new(groups);
        for seed in 0..20i64 {
            let mut out = Vec::new();
            let mut rng = RandomSource::create(seed);
            alias.for_each_resolved(&mut rng, &mut |k, v| out.push((k, v)));
            assert_eq!(out, vec![(key("a"), key("positive"))]);
        }
    }

    #[test]
    fn negative_and_overflowing_weights_are_handled_by_wrapping_total() {
        // Exercises wrapping_add in total_weight accumulation.
        let groups = WeightedList::of(vec![
            WeightedEntry::new(vec![direct("a", "t1")], i32::MAX),
            WeightedEntry::new(vec![direct("b", "t2")], 1),
        ]);
        let alias = RandomGroupPoolAlias::new(groups);
        // total_weight wraps in i32; get_random returns None when total_weight <= 0
        let mut out = Vec::new();
        let mut rng = RandomSource::create(1);
        alias.for_each_resolved(&mut rng, &mut |k, v| out.push((k, v)));
        // Wrapped total is i32::MIN, so no selection occurs.
        assert!(out.is_empty());
        // all_targets still reports all targets regardless of weight selection
        assert_eq!(alias.all_targets(), vec![key("t1"), key("t2")]);
    }
}
