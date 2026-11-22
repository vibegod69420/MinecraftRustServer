use std::fmt;
use std::marker::PhantomData;

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

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub location: ResourceLocation,
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            location: ResourceLocation::new(namespace, path),
            _marker: PhantomData,
        }
    }

    pub fn identifier(&self) -> &ResourceLocation {
        &self.location
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructureTemplatePool;

#[derive(Clone, Debug)]
pub struct RandomSource {
    state: i64,
}

impl RandomSource {
    pub fn create(seed: i64) -> Self {
        Self { state: seed }
    }
}

pub trait PoolAliasBinding {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapCodecStub<T>(PhantomData<T>);

impl<T> MapCodecStub<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for MapCodecStub<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectPoolAlias {
    pub alias: ResourceKey<StructureTemplatePool>,
    pub target: ResourceKey<StructureTemplatePool>,
}

impl DirectPoolAlias {
    pub fn new(
        alias: ResourceKey<StructureTemplatePool>,
        target: ResourceKey<StructureTemplatePool>,
    ) -> Self {
        Self { alias, target }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias#forEachResolved
    pub fn for_each_resolved(
        &self,
        _random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    ) {
        consumer(self.alias.clone(), self.target.clone());
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias#allTargets
    pub fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
        vec![self.target.clone()]
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias#codec
    pub fn codec() -> MapCodecStub<Self> {
        MapCodecStub::new()
    }
}

impl PoolAliasBinding for DirectPoolAlias {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias#forEachResolved
    fn for_each_resolved(
        &self,
        random: &mut RandomSource,
        consumer: &mut dyn FnMut(
            ResourceKey<StructureTemplatePool>,
            ResourceKey<StructureTemplatePool>,
        ),
    ) {
        DirectPoolAlias::for_each_resolved(self, random, consumer)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias#allTargets
    fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
        DirectPoolAlias::all_targets(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(path: &str) -> ResourceKey<StructureTemplatePool> {
        ResourceKey::new("minecraft", path)
    }

    #[test]
    fn for_each_resolved_emits_alias_target_pair() {
        let alias = DirectPoolAlias::new(key("alias"), key("target"));
        let mut random = RandomSource::create(0_i64);
        let mut out = Vec::new();
        alias.for_each_resolved(&mut random, &mut |k, v| out.push((k, v)));
        assert_eq!(out, vec![(key("alias"), key("target"))]);
    }

    #[test]
    fn all_targets_returns_single_target() {
        let alias = DirectPoolAlias::new(key("alias"), key("target"));
        assert_eq!(alias.all_targets(), vec![key("target")]);
    }

    #[test]
    fn trait_impl_matches_inherent_behavior() {
        let alias = DirectPoolAlias::new(key("a"), key("b"));
        let binding: &dyn PoolAliasBinding = &alias;
        let mut random = RandomSource::create(123_i64);
        let mut out = Vec::new();
        binding.for_each_resolved(&mut random, &mut |k, v| out.push((k, v)));
        assert_eq!(out, vec![(key("a"), key("b"))]);
        assert_eq!(binding.all_targets(), vec![key("b")]);
    }

    #[test]
    fn codec_stub_can_be_created() {
        let _codec: MapCodecStub<DirectPoolAlias> = DirectPoolAlias::codec();
    }
}
