use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructureTemplatePool;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    location: String,
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            _marker: PhantomData,
        }
    }

    pub fn location(&self) -> &str {
        &self.location
    }
}

pub trait RandomSource {
    fn next_i32(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weighted<T> {
    data: T,
    weight: i32,
}

impl<T> Weighted<T> {
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
    entries: Vec<Weighted<T>>,
    total_weight: i32,
}

impl<T: Clone> WeightedList<T> {
    pub fn new(entries: Vec<Weighted<T>>) -> Self {
        let mut total_weight: i32 = 0;
        for e in &entries {
            total_weight = total_weight.wrapping_add(e.weight);
        }
        Self {
            entries,
            total_weight,
        }
    }

    pub fn get_random(&self, random: &mut dyn RandomSource) -> Option<T> {
        if self.total_weight <= 0 {
            return None;
        }
        let mut i = random.next_i32(self.total_weight);
        for entry in &self.entries {
            i = i.wrapping_sub(entry.weight);
            if i < 0 {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub fn unwrap(&self) -> &[Weighted<T>] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MapCodecPlaceholder;

#[derive(Clone, Debug)]
pub struct RandomPoolAlias {
    alias: ResourceKey<StructureTemplatePool>,
    targets: WeightedList<ResourceKey<StructureTemplatePool>>,
}

impl RandomPoolAlias {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#<init>
    pub fn new(
        alias: ResourceKey<StructureTemplatePool>,
        targets: WeightedList<ResourceKey<StructureTemplatePool>>,
    ) -> Self {
        Self { alias, targets }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#alias
    pub fn alias(&self) -> &ResourceKey<StructureTemplatePool> {
        &self.alias
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#targets
    pub fn targets(&self) -> &WeightedList<ResourceKey<StructureTemplatePool>> {
        &self.targets
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#forEachResolved
    pub fn for_each_resolved(
        &self,
        random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    ) {
        if let Some(target) = self.targets.get_random(random) {
            consumer(self.alias.clone(), target);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#allTargets
    pub fn all_targets(&self) -> Box<dyn Iterator<Item = ResourceKey<StructureTemplatePool>> + '_> {
        Box::new(self.targets.unwrap().iter().map(|w| w.value().clone()))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.RandomPoolAlias#codec
    pub fn codec(&self) -> MapCodecPlaceholder {
        MapCodecPlaceholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_i32(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    fn key(name: &str) -> ResourceKey<StructureTemplatePool> {
        ResourceKey::new(name)
    }

    #[test]
    fn for_each_resolved_selects_target() {
        let alias = RandomPoolAlias::new(
            key("alias"),
            WeightedList::new(vec![
                Weighted::new(key("a"), 1),
                Weighted::new(key("b"), 2),
            ]),
        );
        let mut out = Vec::new();
        alias.for_each_resolved(&mut FixedRandom(1), &mut |a, t| {
            out.push((a.location().to_string(), t.location().to_string()));
        });
        assert_eq!(out, vec![("alias".to_string(), "b".to_string())]);
    }

    #[test]
    fn for_each_resolved_empty_when_zero_total_weight() {
        let alias = RandomPoolAlias::new(
            key("alias"),
            WeightedList::new(vec![Weighted::new(key("a"), 0)]),
        );
        let mut count = 0;
        alias.for_each_resolved(&mut FixedRandom(0), &mut |_, _| count += 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn all_targets_returns_all_values() {
        let alias = RandomPoolAlias::new(
            key("alias"),
            WeightedList::new(vec![
                Weighted::new(key("a"), 1),
                Weighted::new(key("b"), 1),
            ]),
        );
        let targets: Vec<String> = alias.all_targets().map(|k| k.location().to_string()).collect();
        assert_eq!(targets, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn overflow_total_weight_wraps_to_non_positive() {
        let alias = RandomPoolAlias::new(
            key("alias"),
            WeightedList::new(vec![
                Weighted::new(key("a"), i32::MAX),
                Weighted::new(key("b"), 1),
            ]),
        );
        let mut count = 0;
        alias.for_each_resolved(&mut FixedRandom(0), &mut |_, _| count += 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn negative_weight_uses_wrapping_sub() {
        let alias = RandomPoolAlias::new(
            key("alias"),
            WeightedList::new(vec![
                Weighted::new(key("a"), -1),
                Weighted::new(key("b"), 2),
            ]),
        );
        let mut out = Vec::new();
        alias.for_each_resolved(&mut FixedRandom(0), &mut |_, t| {
            out.push(t.location().to_string());
        });
        assert_eq!(out, vec!["b".to_string()]);
    }
}
