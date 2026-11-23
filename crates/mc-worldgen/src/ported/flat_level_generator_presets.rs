use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            path: path.to_owned(),
        }
    }
}

pub struct ResourceKey<T: ?Sized> {
    pub registry: Identifier,
    pub location: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ResourceKey<T> {
    pub fn create(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.location == other.location
    }
}

impl<T: ?Sized> Eq for ResourceKey<T> {}

impl<T: ?Sized> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.location.hash(state);
    }
}

impl<T: ?Sized> std::fmt::Debug for ResourceKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceKey")
            .field("registry", &self.registry)
            .field("location", &self.location)
            .finish()
    }
}

pub struct Holder<T: ?Sized> {
    pub key: ResourceKey<T>,
}

impl<T: ?Sized> Holder<T> {
    pub fn new(key: ResourceKey<T>) -> Self {
        Self { key }
    }
}

impl<T: ?Sized> Clone for Holder<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
        }
    }
}

impl<T: ?Sized> PartialEq for Holder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: ?Sized> Eq for Holder<T> {}

impl<T: ?Sized> std::fmt::Debug for Holder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Holder").field("key", &self.key).finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSet<T: ?Sized> {
    Direct(Vec<Holder<T>>),
}

impl<T: ?Sized> HolderSet<T> {
    pub fn direct(values: Vec<Holder<T>>) -> Self {
        Self::Direct(values)
    }
}

pub struct HolderGetter<'a, T: ?Sized> {
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> HolderGetter<'a, T> {
    pub fn get_or_throw(&self, key: &ResourceKey<T>) -> Holder<T> {
        Holder::new(key.clone())
    }
}

pub struct BootstrapContext<T> {
    pub registrations: Vec<(ResourceKey<T>, T)>,
}

impl<T> BootstrapContext<T> {
    pub fn new() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }

    pub fn lookup<S: ?Sized>(&self, _registry: Identifier) -> HolderGetter<'_, S> {
        HolderGetter {
            _marker: PhantomData,
        }
    }

    pub fn register(&mut self, key: ResourceKey<T>, value: T) -> Holder<T> {
        let holder = Holder::new(key.clone());
        self.registrations.push((key, value));
        holder
    }
}

impl<T> Default for BootstrapContext<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item {
    pub name: &'static str,
}

pub trait ItemLike {
    fn as_item(&self) -> Item;
}

impl ItemLike for Block {
    fn as_item(&self) -> Item {
        Item { name: self.name }
    }
}

impl ItemLike for Item {
    fn as_item(&self) -> Item {
        *self
    }
}

impl Item {
    pub fn built_in_registry_holder(&self) -> Holder<Item> {
        Holder::new(ResourceKey::create(
            registry_keys::item(),
            Identifier::with_default_namespace(self.name),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biome {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureSet {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacedFeature {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLayerInfo {
    pub height: i32,
    pub block: Block,
}

impl FlatLayerInfo {
    pub fn new(height: i32, block: Block) -> Self {
        Self { height, block }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLevelGeneratorSettings {
    pub structure_overrides: Option<HolderSet<StructureSet>>,
    pub biome: Holder<Biome>,
    pub lakes: Vec<Holder<PlacedFeature>>,
    pub decoration: bool,
    pub add_lakes: bool,
    pub layers_info: Vec<FlatLayerInfo>,
}

impl FlatLevelGeneratorSettings {
    pub fn new(
        structure_overrides: Option<HolderSet<StructureSet>>,
        biome: Holder<Biome>,
        lakes: Vec<Holder<PlacedFeature>>,
    ) -> Self {
        Self {
            structure_overrides,
            biome,
            lakes,
            decoration: false,
            add_lakes: false,
            layers_info: Vec::new(),
        }
    }

    pub fn set_decoration(&mut self) {
        self.decoration = true;
    }

    pub fn set_add_lakes(&mut self) {
        self.add_lakes = true;
    }

    pub fn get_layers_info(&mut self) -> &mut Vec<FlatLayerInfo> {
        &mut self.layers_info
    }

    pub fn create_lakes_list(
        _getter: &HolderGetter<'_, PlacedFeature>,
    ) -> Vec<Holder<PlacedFeature>> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLevelGeneratorPreset {
    pub display_item: Holder<Item>,
    pub settings: FlatLevelGeneratorSettings,
}

pub mod registry_keys {
    use super::Identifier;

    pub fn flat_level_generator_preset() -> Identifier {
        Identifier::with_default_namespace("worldgen/flat_level_generator_preset")
    }

    pub fn structure_set() -> Identifier {
        Identifier::with_default_namespace("worldgen/structure_set")
    }

    pub fn placed_feature() -> Identifier {
        Identifier::with_default_namespace("worldgen/placed_feature")
    }

    pub fn biome() -> Identifier {
        Identifier::with_default_namespace("worldgen/biome")
    }

    pub fn item() -> Identifier {
        Identifier::with_default_namespace("item")
    }
}

pub mod blocks {
    use super::Block;

    pub const GRASS_BLOCK: Block = Block { name: "grass_block" };
    pub const DIRT: Block = Block { name: "dirt" };
    pub const BEDROCK: Block = Block { name: "bedrock" };
    pub const STONE: Block = Block { name: "stone" };
    pub const WATER: Block = Block { name: "water" };
    pub const GRAVEL: Block = Block { name: "gravel" };
    pub const DEEPSLATE: Block = Block { name: "deepslate" };
    pub const SHORT_GRASS: Block = Block { name: "short_grass" };
    pub const SNOW: Block = Block { name: "snow" };
    pub const COBBLESTONE: Block = Block { name: "cobblestone" };
    pub const SAND: Block = Block { name: "sand" };
    pub const SANDSTONE: Block = Block { name: "sandstone" };
    pub const BARRIER: Block = Block { name: "barrier" };
    pub const AIR: Block = Block { name: "air" };
}

pub mod items {
    use super::Item;

    pub const WATER_BUCKET: Item = Item { name: "water_bucket" };
    pub const FEATHER: Item = Item { name: "feather" };
    pub const REDSTONE: Item = Item { name: "redstone" };
}

pub mod biomes {
    use super::{Biome, Identifier, ResourceKey};

    fn key(path: &str) -> ResourceKey<Biome> {
        ResourceKey::create(
            Identifier::with_default_namespace("worldgen/biome"),
            Identifier::with_default_namespace(path),
        )
    }

    pub fn plains() -> ResourceKey<Biome> {
        key("plains")
    }

    pub fn windswept_hills() -> ResourceKey<Biome> {
        key("windswept_hills")
    }

    pub fn deep_ocean() -> ResourceKey<Biome> {
        key("deep_ocean")
    }

    pub fn snowy_plains() -> ResourceKey<Biome> {
        key("snowy_plains")
    }

    pub fn desert() -> ResourceKey<Biome> {
        key("desert")
    }

    pub fn the_void() -> ResourceKey<Biome> {
        key("the_void")
    }
}

pub mod builtin_structure_sets {
    use super::{Identifier, ResourceKey, StructureSet};

    fn key(path: &str) -> ResourceKey<StructureSet> {
        ResourceKey::create(
            Identifier::with_default_namespace("worldgen/structure_set"),
            Identifier::with_default_namespace(path),
        )
    }

    pub fn villages() -> ResourceKey<StructureSet> {
        key("villages")
    }

    pub fn mineshafts() -> ResourceKey<StructureSet> {
        key("mineshafts")
    }

    pub fn strongholds() -> ResourceKey<StructureSet> {
        key("strongholds")
    }

    pub fn ocean_ruins() -> ResourceKey<StructureSet> {
        key("ocean_ruins")
    }

    pub fn shipwrecks() -> ResourceKey<StructureSet> {
        key("shipwrecks")
    }

    pub fn ocean_monuments() -> ResourceKey<StructureSet> {
        key("ocean_monuments")
    }

    pub fn pillager_outposts() -> ResourceKey<StructureSet> {
        key("pillager_outposts")
    }

    pub fn ruined_portals() -> ResourceKey<StructureSet> {
        key("ruined_portals")
    }

    pub fn igloos() -> ResourceKey<StructureSet> {
        key("igloos")
    }

    pub fn desert_pyramids() -> ResourceKey<StructureSet> {
        key("desert_pyramids")
    }
}

// JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets#register
fn register_key(name: &str) -> ResourceKey<FlatLevelGeneratorPreset> {
    ResourceKey::create(
        registry_keys::flat_level_generator_preset(),
        Identifier::with_default_namespace(name),
    )
}

pub static CLASSIC_FLAT: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("classic_flat"));
pub static TUNNELERS_DREAM: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("tunnelers_dream"));
pub static WATER_WORLD: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("water_world"));
pub static OVERWORLD: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("overworld"));
pub static SNOWY_KINGDOM: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("snowy_kingdom"));
pub static BOTTOMLESS_PIT: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("bottomless_pit"));
pub static DESERT: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("desert"));
pub static REDSTONE_READY: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("redstone_ready"));
pub static THE_VOID: LazyLock<ResourceKey<FlatLevelGeneratorPreset>> =
    LazyLock::new(|| register_key("the_void"));

// JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets#bootstrap
pub fn bootstrap(context: &mut BootstrapContext<FlatLevelGeneratorPreset>) {
    Bootstrap::new(context).run();
}

pub struct Bootstrap<'a> {
    context: &'a mut BootstrapContext<FlatLevelGeneratorPreset>,
}

impl<'a> Bootstrap<'a> {
    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets$Bootstrap#<init>
    pub fn new(context: &'a mut BootstrapContext<FlatLevelGeneratorPreset>) -> Self {
        Self { context }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets$Bootstrap#register
    fn register(
        &mut self,
        key: ResourceKey<FlatLevelGeneratorPreset>,
        display: impl ItemLike,
        biome: ResourceKey<Biome>,
        structure_sets: &[ResourceKey<StructureSet>],
        decoration: bool,
        add_lakes: bool,
        layers: &[FlatLayerInfo],
    ) {
        let structure_set_getter: HolderGetter<'_, StructureSet> =
            self.context.lookup(registry_keys::structure_set());
        let placed_feature_getter: HolderGetter<'_, PlacedFeature> =
            self.context.lookup(registry_keys::placed_feature());
        let biome_getter: HolderGetter<'_, Biome> =
            self.context.lookup(registry_keys::biome());

        let structure_overrides = HolderSet::direct(
            structure_sets
                .iter()
                .map(|k| structure_set_getter.get_or_throw(k))
                .collect(),
        );

        let mut settings = FlatLevelGeneratorSettings::new(
            Some(structure_overrides),
            biome_getter.get_or_throw(&biome),
            FlatLevelGeneratorSettings::create_lakes_list(&placed_feature_getter),
        );

        if decoration {
            settings.set_decoration();
        }
        if add_lakes {
            settings.set_add_lakes();
        }

        let mut i = (layers.len() as i32).wrapping_sub(1);
        while i >= 0 {
            settings
                .get_layers_info()
                .push(layers[i as usize].clone());
            i = i.wrapping_sub(1);
        }

        self.context.register(
            key,
            FlatLevelGeneratorPreset {
                display_item: display.as_item().built_in_registry_holder(),
                settings,
            },
        );
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets$Bootstrap#run
    pub fn run(&mut self) {
        self.register(
            CLASSIC_FLAT.clone(),
            blocks::GRASS_BLOCK,
            biomes::plains(),
            &[builtin_structure_sets::villages()],
            false,
            false,
            &[
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
                FlatLayerInfo::new(2, blocks::DIRT),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            TUNNELERS_DREAM.clone(),
            blocks::STONE,
            biomes::windswept_hills(),
            &[
                builtin_structure_sets::mineshafts(),
                builtin_structure_sets::strongholds(),
            ],
            true,
            false,
            &[
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
                FlatLayerInfo::new(5, blocks::DIRT),
                FlatLayerInfo::new(230, blocks::STONE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            WATER_WORLD.clone(),
            items::WATER_BUCKET,
            biomes::deep_ocean(),
            &[
                builtin_structure_sets::ocean_ruins(),
                builtin_structure_sets::shipwrecks(),
                builtin_structure_sets::ocean_monuments(),
            ],
            false,
            false,
            &[
                FlatLayerInfo::new(90, blocks::WATER),
                FlatLayerInfo::new(5, blocks::GRAVEL),
                FlatLayerInfo::new(5, blocks::DIRT),
                FlatLayerInfo::new(5, blocks::STONE),
                FlatLayerInfo::new(64, blocks::DEEPSLATE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            OVERWORLD.clone(),
            blocks::SHORT_GRASS,
            biomes::plains(),
            &[
                builtin_structure_sets::villages(),
                builtin_structure_sets::mineshafts(),
                builtin_structure_sets::pillager_outposts(),
                builtin_structure_sets::ruined_portals(),
                builtin_structure_sets::strongholds(),
            ],
            true,
            true,
            &[
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
                FlatLayerInfo::new(3, blocks::DIRT),
                FlatLayerInfo::new(59, blocks::STONE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            SNOWY_KINGDOM.clone(),
            blocks::SNOW,
            biomes::snowy_plains(),
            &[
                builtin_structure_sets::villages(),
                builtin_structure_sets::igloos(),
            ],
            false,
            false,
            &[
                FlatLayerInfo::new(1, blocks::SNOW),
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
                FlatLayerInfo::new(3, blocks::DIRT),
                FlatLayerInfo::new(59, blocks::STONE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            BOTTOMLESS_PIT.clone(),
            items::FEATHER,
            biomes::plains(),
            &[builtin_structure_sets::villages()],
            false,
            false,
            &[
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
                FlatLayerInfo::new(3, blocks::DIRT),
                FlatLayerInfo::new(2, blocks::COBBLESTONE),
            ],
        );
        self.register(
            DESERT.clone(),
            blocks::SAND,
            biomes::desert(),
            &[
                builtin_structure_sets::villages(),
                builtin_structure_sets::desert_pyramids(),
                builtin_structure_sets::mineshafts(),
                builtin_structure_sets::strongholds(),
            ],
            true,
            false,
            &[
                FlatLayerInfo::new(8, blocks::SAND),
                FlatLayerInfo::new(52, blocks::SANDSTONE),
                FlatLayerInfo::new(3, blocks::STONE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            REDSTONE_READY.clone(),
            items::REDSTONE,
            biomes::desert(),
            &[],
            false,
            false,
            &[
                FlatLayerInfo::new(116, blocks::SANDSTONE),
                FlatLayerInfo::new(3, blocks::STONE),
                FlatLayerInfo::new(1, blocks::BEDROCK),
            ],
        );
        self.register(
            THE_VOID.clone(),
            blocks::BARRIER,
            biomes::the_void(),
            &[],
            true,
            false,
            &[FlatLayerInfo::new(1, blocks::AIR)],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_preset<'a>(
        ctx: &'a BootstrapContext<FlatLevelGeneratorPreset>,
        key: &ResourceKey<FlatLevelGeneratorPreset>,
    ) -> &'a FlatLevelGeneratorPreset {
        ctx.registrations
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .expect("preset not found")
    }

    #[test]
    fn register_key_uses_default_namespace_and_registry() {
        let key = register_key("classic_flat");
        assert_eq!(
            key.registry,
            Identifier::with_default_namespace("worldgen/flat_level_generator_preset")
        );
        assert_eq!(
            key.location,
            Identifier::with_default_namespace("classic_flat")
        );
    }

    #[test]
    fn bootstrap_registers_all_nine_presets_in_order() {
        let mut ctx = BootstrapContext::new();
        bootstrap(&mut ctx);
        assert_eq!(ctx.registrations.len(), 9);
        let names: Vec<&str> = ctx
            .registrations
            .iter()
            .map(|(k, _)| k.location.path.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "classic_flat",
                "tunnelers_dream",
                "water_world",
                "overworld",
                "snowy_kingdom",
                "bottomless_pit",
                "desert",
                "redstone_ready",
                "the_void",
            ]
        );
    }

    #[test]
    fn layers_are_reversed_and_flags_and_display_match() {
        let mut ctx = BootstrapContext::new();
        bootstrap(&mut ctx);

        let classic = find_preset(&ctx, &CLASSIC_FLAT);
        assert!(!classic.settings.decoration);
        assert!(!classic.settings.add_lakes);
        assert_eq!(
            classic.display_item.key.location,
            Identifier::with_default_namespace("grass_block")
        );
        assert_eq!(
            classic.settings.layers_info,
            vec![
                FlatLayerInfo::new(1, blocks::BEDROCK),
                FlatLayerInfo::new(2, blocks::DIRT),
                FlatLayerInfo::new(1, blocks::GRASS_BLOCK),
            ]
        );

        let tunnelers = find_preset(&ctx, &TUNNELERS_DREAM);
        assert!(tunnelers.settings.decoration);
        assert!(!tunnelers.settings.add_lakes);

        let overworld = find_preset(&ctx, &OVERWORLD);
        assert!(overworld.settings.decoration);
        assert!(overworld.settings.add_lakes);
    }

    #[test]
    fn the_void_has_empty_structures_and_single_air_layer() {
        let mut ctx = BootstrapContext::new();
        bootstrap(&mut ctx);
        let void_preset = find_preset(&ctx, &THE_VOID);
        assert_eq!(
            void_preset.settings.structure_overrides,
            Some(HolderSet::Direct(vec![]))
        );
        assert_eq!(
            void_preset.settings.layers_info,
            vec![FlatLayerInfo::new(1, blocks::AIR)]
        );
        assert!(void_preset.settings.decoration);
        assert!(!void_preset.settings.add_lakes);
    }

    #[test]
    fn register_preserves_negative_and_extreme_layer_heights() {
        let mut ctx = BootstrapContext::new();
        let mut b = Bootstrap::new(&mut ctx);
        let key = register_key("test_extreme");
        b.register(
            key.clone(),
            blocks::STONE,
            biomes::plains(),
            &[],
            false,
            false,
            &[
                FlatLayerInfo::new(i32::MIN, blocks::DIRT),
                FlatLayerInfo::new(-1, blocks::STONE),
                FlatLayerInfo::new(0, blocks::SAND),
                FlatLayerInfo::new(i32::MAX, blocks::BEDROCK),
            ],
        );
        let preset = find_preset(&ctx, &key);
        assert_eq!(preset.settings.layers_info[0].height, i32::MAX);
        assert_eq!(preset.settings.layers_info[1].height, 0);
        assert_eq!(preset.settings.layers_info[2].height, -1);
        assert_eq!(preset.settings.layers_info[3].height, i32::MIN);
        // Bit patterns preserved for extreme integer values.
        assert_eq!(preset.settings.layers_info[0].height.to_be_bytes(), i32::MAX.to_be_bytes());
        assert_eq!(preset.settings.layers_info[3].height.to_be_bytes(), i32::MIN.to_be_bytes());
    }

    #[test]
    fn resource_key_clone_and_hash_are_independent_of_type_parameter() {
        let k1: ResourceKey<Biome> = ResourceKey::create(
            Identifier::with_default_namespace("worldgen/biome"),
            Identifier::with_default_namespace("plains"),
        );
        let k2 = k1.clone();
        assert_eq!(k1, k2);

        let mut set = std::collections::HashSet::new();
        set.insert(k1.clone());
        assert!(set.contains(&k2));
        assert_eq!(set.len(), 1);
    }
}
