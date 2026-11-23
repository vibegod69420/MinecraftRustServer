use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: ResourceLocation,
}

impl ResourceKey {
    pub fn new(location: ResourceLocation) -> Self {
        Self { location }
    }
}

fn pools_create_key(name: &str) -> ResourceKey {
    ResourceKey::new(ResourceLocation::with_default_namespace(name))
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedEntry<T> {
    data: T,
    weight: i32,
}

impl<T> WeightedEntry<T> {
    pub fn new(data: T, weight: i32) -> Self {
        Self { data, weight }
    }

    pub fn value(&self) -> &T {
        &self.data
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }
}

#[derive(Clone, Debug)]
pub struct WeightedList<T> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T> Default for WeightedList<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> WeightedList<T> {
    pub fn builder() -> WeightedListBuilder<T> {
        WeightedListBuilder {
            entries: Vec::new(),
        }
    }

    pub fn unwrap(&self) -> &[WeightedEntry<T>] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_random(&self, random: &mut dyn RandomSource) -> Option<&T> {
        let mut total: i32 = 0;
        for entry in &self.entries {
            total = total.wrapping_add(entry.weight);
        }
        if total <= 0 {
            return None;
        }
        let mut value = random.next_int(total);
        for entry in &self.entries {
            value = value.wrapping_sub(entry.weight);
            if value < 0 {
                return Some(&entry.data);
            }
        }
        None
    }
}

pub struct WeightedListBuilder<T> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T> WeightedListBuilder<T> {
    pub fn add(mut self, data: T, weight: i32) -> Self {
        self.entries.push(WeightedEntry { data, weight });
        self
    }

    pub fn build(self) -> WeightedList<T> {
        WeightedList {
            entries: self.entries,
        }
    }
}

/// Placeholder for Mojang serialization codec infrastructure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCodec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectPoolAlias {
    pub alias: ResourceKey,
    pub target: ResourceKey,
}

#[derive(Clone, Debug)]
pub struct RandomPoolAlias {
    pub alias: ResourceKey,
    pub targets: WeightedList<ResourceKey>,
}

#[derive(Clone, Debug)]
pub struct RandomGroupPoolAlias {
    pub groups: WeightedList<Vec<PoolAliasBinding>>,
}

#[derive(Clone, Debug)]
pub enum PoolAliasBinding {
    Direct(DirectPoolAlias),
    Random(RandomPoolAlias),
    RandomGroup(RandomGroupPoolAlias),
}

impl PoolAliasBinding {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#forEachResolved
    pub fn for_each_resolved(
        &self,
        random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(ResourceKey, ResourceKey),
    ) {
        match self {
            PoolAliasBinding::Direct(d) => {
                consumer(d.alias.clone(), d.target.clone());
            }
            PoolAliasBinding::Random(r) => {
                if let Some(target) = r.targets.get_random(random) {
                    consumer(r.alias.clone(), target.clone());
                }
            }
            PoolAliasBinding::RandomGroup(g) => {
                if let Some(group) = g.groups.get_random(random) {
                    for binding in group {
                        binding.for_each_resolved(random, consumer);
                    }
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#allTargets
    pub fn all_targets(&self) -> Box<dyn Iterator<Item = ResourceKey> + '_> {
        match self {
            PoolAliasBinding::Direct(d) => Box::new(std::iter::once(d.target.clone())),
            PoolAliasBinding::Random(r) => {
                Box::new(r.targets.unwrap().iter().map(|e| e.value().clone()))
            }
            PoolAliasBinding::RandomGroup(g) => Box::new(
                g.groups
                    .unwrap()
                    .iter()
                    .flat_map(|e| e.value().iter())
                    .flat_map(|b| b.all_targets()),
            ),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#direct
    pub fn direct(alias: ResourceKey, target: ResourceKey) -> DirectPoolAlias {
        DirectPoolAlias { alias, target }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#direct
    pub fn direct_from_str(alias: &str, target: &str) -> DirectPoolAlias {
        Self::direct(pools_create_key(alias), pools_create_key(target))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#random
    pub fn random(alias: ResourceKey, targets: WeightedList<ResourceKey>) -> RandomPoolAlias {
        RandomPoolAlias { alias, targets }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#random
    pub fn random_from_str(alias: &str, targets: WeightedList<String>) -> RandomPoolAlias {
        let mut builder = WeightedList::<ResourceKey>::builder();
        for entry in targets.unwrap() {
            builder = builder.add(pools_create_key(entry.value()), entry.weight());
        }
        Self::random(pools_create_key(alias), builder.build())
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#randomGroup
    pub fn random_group(groups: WeightedList<Vec<PoolAliasBinding>>) -> RandomGroupPoolAlias {
        RandomGroupPoolAlias { groups }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBinding#codec
    pub fn codec(&self) -> MapCodec {
        MapCodec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.values.len() {
                let v = self.values[self.idx];
                self.idx += 1;
                v
            } else {
                0
            };
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    #[test]
    fn direct_binding_resolves_and_targets() {
        let d = PoolAliasBinding::direct_from_str("alias", "target");
        let binding = PoolAliasBinding::Direct(d);
        let mut results = Vec::new();
        let mut rng = SeqRandom::new(vec![]);
        binding.for_each_resolved(&mut rng, &mut |a, t| results.push((a, t)));

        let expected_alias = ResourceKey::new(ResourceLocation::with_default_namespace("alias"));
        let expected_target = ResourceKey::new(ResourceLocation::with_default_namespace("target"));
        assert_eq!(
            results,
            vec![(expected_alias, expected_target.clone())]
        );

        let targets: Vec<_> = binding.all_targets().collect();
        assert_eq!(targets, vec![expected_target]);
        assert_eq!(binding.codec(), MapCodec);
    }

    #[test]
    fn random_binding_weighted_selection() {
        let targets = WeightedList::<ResourceKey>::builder()
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("a")),
                1,
            )
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("b")),
                2,
            )
            .build();
        let alias = ResourceKey::new(ResourceLocation::with_default_namespace("src"));
        let binding = PoolAliasBinding::Random(PoolAliasBinding::random(alias, targets));

        let mut rng = SeqRandom::new(vec![0]);
        let mut results = Vec::new();
        binding.for_each_resolved(&mut rng, &mut |_, t| results.push(t.location.path.clone()));
        assert_eq!(results, vec!["a"]);

        let mut rng = SeqRandom::new(vec![1]);
        let mut results = Vec::new();
        binding.for_each_resolved(&mut rng, &mut |_, t| results.push(t.location.path.clone()));
        assert_eq!(results, vec!["b"]);

        let mut rng = SeqRandom::new(vec![2]);
        let mut results = Vec::new();
        binding.for_each_resolved(&mut rng, &mut |_, t| results.push(t.location.path.clone()));
        assert_eq!(results, vec!["b"]);

        let all: Vec<_> = binding.all_targets().map(|k| k.location.path).collect();
        assert_eq!(all, vec!["a", "b"]);
    }

    #[test]
    fn random_group_resolves_selected_group() {
        let d1 = PoolAliasBinding::Direct(PoolAliasBinding::direct_from_str("a1", "t1"));
        let d2 = PoolAliasBinding::Direct(PoolAliasBinding::direct_from_str("a2", "t2"));
        let d3 = PoolAliasBinding::Direct(PoolAliasBinding::direct_from_str("b1", "t3"));

        let groups = WeightedList::<Vec<PoolAliasBinding>>::builder()
            .add(vec![d1, d2], 1)
            .add(vec![d3], 1)
            .build();
        let binding = PoolAliasBinding::RandomGroup(PoolAliasBinding::random_group(groups));

        let mut rng = SeqRandom::new(vec![0]);
        let mut results = Vec::new();
        binding.for_each_resolved(&mut rng, &mut |a, t| {
            results.push((a.location.path, t.location.path))
        });
        assert_eq!(
            results,
            vec![
                ("a1".to_string(), "t1".to_string()),
                ("a2".to_string(), "t2".to_string())
            ]
        );

        let mut rng = SeqRandom::new(vec![1]);
        let mut results = Vec::new();
        binding.for_each_resolved(&mut rng, &mut |a, t| {
            results.push((a.location.path, t.location.path))
        });
        assert_eq!(results, vec![("b1".to_string(), "t3".to_string())]);

        let all: Vec<_> = binding.all_targets().map(|k| k.location.path).collect();
        assert_eq!(all, vec!["t1", "t2", "t3"]);
    }

    #[test]
    fn weighted_list_overflow_and_negative_weights() {
        // i32::MAX.wrapping_add(1) == i32::MIN <= 0 -> no selection
        let targets = WeightedList::<ResourceKey>::builder()
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("x")),
                i32::MAX,
            )
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("y")),
                1,
            )
            .build();
        let mut rng = SeqRandom::new(vec![0]);
        assert!(targets.get_random(&mut rng).is_none());

        // Negative total weight -> no selection
        let targets = WeightedList::<ResourceKey>::builder()
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("x")),
                -5,
            )
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("y")),
                2,
            )
            .build();
        let mut rng = SeqRandom::new(vec![0]);
        assert!(targets.get_random(&mut rng).is_none());

        // Zero total weight -> no selection
        let targets = WeightedList::<ResourceKey>::builder()
            .add(
                ResourceKey::new(ResourceLocation::with_default_namespace("x")),
                0,
            )
            .build();
        let mut rng = SeqRandom::new(vec![0]);
        assert!(targets.get_random(&mut rng).is_none());
    }

    #[test]
    fn direct_from_str_uses_default_namespace() {
        let d = PoolAliasBinding::direct_from_str("foo", "bar");
        assert_eq!(d.alias.location.namespace, "minecraft");
        assert_eq!(d.alias.location.path, "foo");
        assert_eq!(d.target.location.namespace, "minecraft");
        assert_eq!(d.target.location.path, "bar");
    }

    #[test]
    fn random_from_str_builds_targets() {
        let targets = WeightedList::<String>::builder()
            .add("alpha".to_string(), 1)
            .add("beta".to_string(), 3)
            .build();
        let r = PoolAliasBinding::random_from_str("src", targets);
        assert_eq!(r.alias.location.path, "src");
        assert_eq!(r.alias.location.namespace, "minecraft");
        let entries = r.targets.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].value().location.path, "alpha");
        assert_eq!(entries[0].weight(), 1);
        assert_eq!(entries[1].value().location.path, "beta");
        assert_eq!(entries[1].weight(), 3);
    }
}
