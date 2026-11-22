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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BootstrapContext<T> {
    pub registered: Vec<(String, T)>,
}

impl<T> BootstrapContext<T> {
    pub fn new() -> Self {
        Self {
            registered: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapCodecStub;

#[derive(Clone, Debug, Default)]
pub struct Registry<T> {
    pub entries: Vec<(String, T)>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

fn registry_register<T: Clone>(registry: &mut Registry<T>, name: &str, value: T) -> T {
    registry.entries.push((name.to_string(), value.clone()));
    value
}

pub trait PoolAliasBinding {
    fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructurePoolElement {
    pub name: String,
}

impl StructurePoolElement {
    pub fn single(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Projection {
    Rigid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplatePool {
    pub fallback: Holder<StructureTemplatePool>,
    pub elements: Vec<(StructurePoolElement, i32)>,
    pub projection: Projection,
}

pub struct Pools;

impl Pools {
    pub fn register(ctx: &mut BootstrapContext<TemplatePool>, name: &str, pool: TemplatePool) {
        ctx.registered.push((name.to_string(), pool));
    }
}

pub struct PoolAliasBindings;

impl PoolAliasBindings {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBindings#bootstrap
    pub fn bootstrap(registry: &mut Registry<MapCodecStub>) -> MapCodecStub {
        registry_register(registry, "random", MapCodecStub);
        registry_register(registry, "random_group", MapCodecStub);
        registry_register(registry, "direct", MapCodecStub)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBindings#registerTargetsAsPools
    pub fn register_targets_as_pools(
        ctx: &mut BootstrapContext<TemplatePool>,
        fallback: Holder<StructureTemplatePool>,
        aliases: &[Box<dyn PoolAliasBinding>],
    ) {
        for binding in aliases {
            for target in binding.all_targets() {
                let path = target.identifier().get_path().to_string();
                Pools::register(
                    ctx,
                    &path,
                    TemplatePool {
                        fallback: fallback.clone(),
                        elements: vec![(StructurePoolElement::single(&path), 1)],
                        projection: Projection::Rigid,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBinding {
        targets: Vec<ResourceKey<StructureTemplatePool>>,
    }

    impl PoolAliasBinding for MockBinding {
        fn all_targets(&self) -> Vec<ResourceKey<StructureTemplatePool>> {
            self.targets.clone()
        }
    }

    fn key(path: &str) -> ResourceKey<StructureTemplatePool> {
        ResourceKey::new("minecraft", path)
    }

    #[test]
    fn bootstrap_registers_codecs_in_order() {
        let mut registry = Registry::<MapCodecStub>::new();
        let returned = PoolAliasBindings::bootstrap(&mut registry);
        assert_eq!(returned, MapCodecStub);
        let names: Vec<&str> = registry.entries.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["random", "random_group", "direct"]);
    }

    #[test]
    fn register_targets_as_pools_registers_all_targets() {
        let mut ctx = BootstrapContext::<TemplatePool>::new();
        let fallback = Holder::new(StructureTemplatePool);
        let aliases: Vec<Box<dyn PoolAliasBinding>> = vec![
            Box::new(MockBinding {
                targets: vec![key("house/start"), key("house/end")],
            }),
            Box::new(MockBinding {
                targets: vec![key("road")],
            }),
        ];

        PoolAliasBindings::register_targets_as_pools(&mut ctx, fallback, &aliases);

        let names: Vec<&str> = ctx.registered.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["house/start", "house/end", "road"]);
        for (name, pool) in &ctx.registered {
            assert_eq!(pool.elements.len(), 1);
            assert_eq!(pool.elements[0].0.name, *name);
            assert_eq!(pool.elements[0].1, 1);
            assert_eq!(pool.projection, Projection::Rigid);
        }
    }

    #[test]
    fn register_targets_as_pools_handles_empty_aliases() {
        let mut ctx = BootstrapContext::<TemplatePool>::new();
        let fallback = Holder::new(StructureTemplatePool);
        let aliases: Vec<Box<dyn PoolAliasBinding>> = Vec::new();
        PoolAliasBindings::register_targets_as_pools(&mut ctx, fallback, &aliases);
        assert!(ctx.registered.is_empty());
    }

    #[test]
    fn register_targets_as_pools_preserves_duplicate_targets() {
        let mut ctx = BootstrapContext::<TemplatePool>::new();
        let fallback = Holder::new(StructureTemplatePool);
        let aliases: Vec<Box<dyn PoolAliasBinding>> = vec![Box::new(MockBinding {
            targets: vec![key("dup"), key("dup")],
        })];
        PoolAliasBindings::register_targets_as_pools(&mut ctx, fallback, &aliases);
        let names: Vec<&str> = ctx.registered.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["dup", "dup"]);
    }
}
