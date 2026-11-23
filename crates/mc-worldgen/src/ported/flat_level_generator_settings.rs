//! Port of `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings` (Java 1.21.11).
//!
//! This module is self-contained: Minecraft registry/holder/biome/feature types are
//! represented by minimal local data structures that preserve the behavior of the
//! original class.

use std::fmt;

pub const DIMENSION_TYPE_Y_SIZE: i32 = 384;

pub const BIOMES_PLAINS: &str = "minecraft:plains";
pub const BIOMES_THE_VOID: &str = "minecraft:the_void";
pub const BUILTIN_STRUCTURE_SETS_STRONGHOLDS: &str = "minecraft:strongholds";
pub const BUILTIN_STRUCTURE_SETS_VILLAGES: &str = "minecraft:villages";
pub const MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_UNDERGROUND: &str =
    "minecraft:lake_lava_underground";
pub const MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_SURFACE: &str = "minecraft:lake_lava_surface";

pub const GENERATION_STEP_DECORATION_LAKES: usize = 1;
pub const GENERATION_STEP_DECORATION_UNDERGROUND_STRUCTURES: usize = 3;
pub const GENERATION_STEP_DECORATION_SURFACE_STRUCTURES: usize = 4;
pub const GENERATION_STEP_DECORATION_TOP_LAYER_MODIFICATION: usize = 10;
const GENERATION_STEP_DECORATION_COUNT: usize = 11;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: String,
}

impl Block {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn air() -> Self {
        Self::new("minecraft:air")
    }

    pub fn bedrock() -> Self {
        Self::new("minecraft:bedrock")
    }

    pub fn dirt() -> Self {
        Self::new("minecraft:dirt")
    }

    pub fn grass_block() -> Self {
        Self::new("minecraft:grass_block")
    }

    pub fn default_block_state(&self) -> BlockState {
        let motion_blocking_opaque = self.id != "minecraft:air";
        BlockState {
            block: self.clone(),
            motion_blocking_opaque,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    block: Block,
    motion_blocking_opaque: bool,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        block.default_block_state()
    }

    pub fn with_motion_blocking_opaque(block: Block, motion_blocking_opaque: bool) -> Self {
        Self {
            block,
            motion_blocking_opaque,
        }
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn is(&self, block: &Block) -> bool {
        &self.block == block
    }

    pub fn is_air(&self) -> bool {
        self.is(&Block::air())
    }

    pub fn is_motion_blocking_opaque(&self) -> bool {
        self.motion_blocking_opaque
    }
}

/// Minimal port of `FlatLayerInfo` used by `FlatLevelGeneratorSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLayerInfo {
    height: i32,
    block: Block,
}

impl FlatLayerInfo {
    pub fn new(height: i32, block: Block) -> Self {
        Self { height, block }
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn get_block_state(&self) -> BlockState {
        self.block.default_block_state()
    }
}

#[derive(Debug, Clone)]
pub struct Holder<T: Clone + fmt::Debug> {
    key: Option<String>,
    value: T,
}

impl<T: Clone + fmt::Debug> Holder<T> {
    pub fn new(key: impl Into<String>, value: T) -> Self {
        Self {
            key: Some(key.into()),
            value,
        }
    }

    pub fn direct(value: T) -> Self {
        Self { key: None, value }
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn is(&self, key: &str) -> bool {
        self.key.as_deref() == Some(key)
    }
}

impl<T: Clone + fmt::Debug + PartialEq> PartialEq for Holder<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.key, &other.key) {
            (Some(a), Some(b)) => a == b,
            _ => self.value == other.value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HolderSet<T: Clone + fmt::Debug + PartialEq> {
    entries: Vec<Holder<T>>,
}

impl<T: Clone + fmt::Debug + PartialEq> HolderSet<T> {
    pub fn direct(entries: Vec<Holder<T>>) -> Self {
        Self { entries }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Holder<T>> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<'a, T: Clone + fmt::Debug + PartialEq> IntoIterator for &'a HolderSet<T> {
    type Item = &'a Holder<T>;
    type IntoIter = std::slice::Iter<'a, Holder<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

pub trait HolderGetter<T: Clone + fmt::Debug> {
    fn get_or_throw(&self, key: &str) -> Holder<T>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureSet {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacedFeatureKind {
    Reference(String),
    FillLayer { layer: i32, state: BlockState },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeature {
    pub kind: PlacedFeatureKind,
}

impl PlacedFeature {
    pub fn reference(id: impl Into<String>) -> Self {
        Self {
            kind: PlacedFeatureKind::Reference(id.into()),
        }
    }

    pub fn fill_layer(layer: i32, state: BlockState) -> Self {
        Self {
            kind: PlacedFeatureKind::FillLayer { layer, state },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BiomeGenerationSettings {
    features_by_step: Vec<HolderSet<PlacedFeature>>,
}

impl BiomeGenerationSettings {
    pub fn new(features_by_step: Vec<HolderSet<PlacedFeature>>) -> Self {
        Self { features_by_step }
    }

    pub fn features(&self) -> &[HolderSet<PlacedFeature>] {
        &self.features_by_step
    }
}

#[derive(Debug, Clone)]
pub struct BiomeGenerationSettingsPlainBuilder {
    features_by_step: Vec<Vec<Holder<PlacedFeature>>>,
}

impl BiomeGenerationSettingsPlainBuilder {
    pub fn new() -> Self {
        Self {
            features_by_step: vec![Vec::new(); GENERATION_STEP_DECORATION_COUNT],
        }
    }

    pub fn add_feature(&mut self, step: usize, feature: Holder<PlacedFeature>) {
        if step >= self.features_by_step.len() {
            self.features_by_step.resize_with(step + 1, Vec::new);
        }
        self.features_by_step[step].push(feature);
    }

    pub fn build(self) -> BiomeGenerationSettings {
        BiomeGenerationSettings {
            features_by_step: self
                .features_by_step
                .into_iter()
                .map(HolderSet::direct)
                .collect(),
        }
    }
}

impl Default for BiomeGenerationSettingsPlainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Biome {
    generation_settings: BiomeGenerationSettings,
}

impl Biome {
    pub fn new(generation_settings: BiomeGenerationSettings) -> Self {
        Self {
            generation_settings,
        }
    }

    pub fn get_generation_settings(&self) -> &BiomeGenerationSettings {
        &self.generation_settings
    }
}

#[derive(Debug, Clone)]
pub struct FlatLevelGeneratorSettings {
    structure_overrides: Option<HolderSet<StructureSet>>,
    layers_info: Vec<FlatLayerInfo>,
    biome: Holder<Biome>,
    layers: Vec<Option<BlockState>>,
    void_gen: bool,
    decoration: bool,
    add_lakes: bool,
    lakes: Vec<Holder<PlacedFeature>>,
}

impl FlatLevelGeneratorSettings {
    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#validateHeight
    fn validate_height(
        settings: FlatLevelGeneratorSettings,
    ) -> Result<FlatLevelGeneratorSettings, String> {
        let total_height: i32 = settings
            .layers_info
            .iter()
            .fold(0i32, |acc, li| acc.wrapping_add(li.get_height()));
        if total_height > DIMENSION_TYPE_Y_SIZE {
            Err(format!(
                "Sum of layer heights is > {}",
                DIMENSION_TYPE_Y_SIZE
            ))
        } else {
            Ok(settings)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#<init>
    pub fn new(
        structure_overrides: Option<HolderSet<StructureSet>>,
        biome: Holder<Biome>,
        lakes: Vec<Holder<PlacedFeature>>,
    ) -> Self {
        Self {
            structure_overrides,
            layers_info: Vec::new(),
            biome,
            layers: Vec::new(),
            void_gen: false,
            decoration: false,
            add_lakes: false,
            lakes,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#<init>
    pub fn new_with_layers(
        structure_overrides: Option<HolderSet<StructureSet>>,
        layers_info: Vec<FlatLayerInfo>,
        add_lakes: bool,
        decoration: bool,
        biome: Option<Holder<Biome>>,
        default_biome: Holder<Biome>,
        lake_lava_underground: Holder<PlacedFeature>,
        lake_lava_surface: Holder<PlacedFeature>,
    ) -> Self {
        let mut settings = Self::new(
            structure_overrides,
            Self::get_biome_or_default(biome, default_biome),
            vec![lake_lava_underground, lake_lava_surface],
        );
        if add_lakes {
            settings.set_add_lakes();
        }
        if decoration {
            settings.set_decoration();
        }
        settings.layers_info.extend(layers_info);
        settings.update_layers();
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getBiome
    fn get_biome_or_default(
        biome: Option<Holder<Biome>>,
        default_biome: Holder<Biome>,
    ) -> Holder<Biome> {
        match biome {
            None => {
                eprintln!("ERROR: Unknown biome, defaulting to plains");
                default_biome
            }
            Some(b) => b,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#withBiomeAndLayers
    pub fn with_biome_and_layers(
        &self,
        layers: Vec<FlatLayerInfo>,
        structure_overrides: Option<HolderSet<StructureSet>>,
        biome: Holder<Biome>,
    ) -> Self {
        let mut settings = Self::new(structure_overrides, biome, self.lakes.clone());
        for layer in &layers {
            settings
                .layers_info
                .push(FlatLayerInfo::new(layer.get_height(), layer.get_block().clone()));
            settings.update_layers();
        }
        if self.decoration {
            settings.set_decoration();
        }
        if self.add_lakes {
            settings.set_add_lakes();
        }
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#setDecoration
    pub fn set_decoration(&mut self) {
        self.decoration = true;
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#setAddLakes
    pub fn set_add_lakes(&mut self) {
        self.add_lakes = true;
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#adjustGenerationSettings
    pub fn adjust_generation_settings(
        &mut self,
        biome: &Holder<Biome>,
    ) -> BiomeGenerationSettings {
        if biome != &self.biome {
            return biome.value().get_generation_settings().clone();
        }

        let base_settings = self.get_biome().value().get_generation_settings().clone();
        let mut builder = BiomeGenerationSettingsPlainBuilder::new();

        if self.add_lakes {
            for lake in &self.lakes {
                builder.add_feature(GENERATION_STEP_DECORATION_LAKES, lake.clone());
            }
        }

        let is_void_biome = biome.is(BIOMES_THE_VOID);
        let add_decoration = (!self.void_gen || is_void_biome) && self.decoration;
        if add_decoration {
            let features = base_settings.features();
            for step in 0..features.len() {
                if step != GENERATION_STEP_DECORATION_UNDERGROUND_STRUCTURES
                    && step != GENERATION_STEP_DECORATION_SURFACE_STRUCTURES
                    && (!self.add_lakes || step != GENERATION_STEP_DECORATION_LAKES)
                {
                    for feature_holder in &features[step] {
                        builder.add_feature(step, feature_holder.clone());
                    }
                }
            }
        }

        let layer_count = self.layers.len();
        for i in 0..layer_count {
            let state = self.layers[i].clone();
            if let Some(s) = state {
                if !s.is_motion_blocking_opaque() {
                    self.layers[i] = None;
                    builder.add_feature(
                        GENERATION_STEP_DECORATION_TOP_LAYER_MODIFICATION,
                        Holder::direct(PlacedFeature::fill_layer(i as i32, s)),
                    );
                }
            }
        }

        builder.build()
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#structureOverrides
    pub fn structure_overrides(&self) -> Option<&HolderSet<StructureSet>> {
        self.structure_overrides.as_ref()
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getBiome
    pub fn get_biome(&self) -> &Holder<Biome> {
        &self.biome
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getLayersInfo
    pub fn get_layers_info(&mut self) -> &mut Vec<FlatLayerInfo> {
        &mut self.layers_info
    }

    pub fn layers_info(&self) -> &[FlatLayerInfo] {
        &self.layers_info
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getLayers
    pub fn get_layers(&self) -> &[Option<BlockState>] {
        &self.layers
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#updateLayers
    pub fn update_layers(&mut self) {
        self.layers.clear();
        for layer_info in &self.layers_info {
            let height = layer_info.get_height();
            let mut i = 0;
            while i < height {
                self.layers.push(Some(layer_info.get_block_state()));
                i += 1;
            }
        }
        self.void_gen = self
            .layers
            .iter()
            .all(|s| matches!(s, Some(state) if state.is_air()));
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getDefault
    pub fn get_default(
        biome_getter: &dyn HolderGetter<Biome>,
        structure_set_getter: &dyn HolderGetter<StructureSet>,
        placed_feature_getter: &dyn HolderGetter<PlacedFeature>,
    ) -> Self {
        let structure_overrides = HolderSet::direct(vec![
            structure_set_getter.get_or_throw(BUILTIN_STRUCTURE_SETS_STRONGHOLDS),
            structure_set_getter.get_or_throw(BUILTIN_STRUCTURE_SETS_VILLAGES),
        ]);
        let mut settings = Self::new(
            Some(structure_overrides),
            Self::get_default_biome(biome_getter),
            Self::create_lakes_list(placed_feature_getter),
        );
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::bedrock()));
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(2, Block::dirt()));
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::grass_block()));
        settings.update_layers();
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#getDefaultBiome
    pub fn get_default_biome(biome_getter: &dyn HolderGetter<Biome>) -> Holder<Biome> {
        biome_getter.get_or_throw(BIOMES_PLAINS)
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings#createLakesList
    pub fn create_lakes_list(
        placed_feature_getter: &dyn HolderGetter<PlacedFeature>,
    ) -> Vec<Holder<PlacedFeature>> {
        vec![
            placed_feature_getter.get_or_throw(MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_UNDERGROUND),
            placed_feature_getter.get_or_throw(MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_SURFACE),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MapGetter<T: Clone + fmt::Debug> {
        map: HashMap<String, T>,
    }

    impl<T: Clone + fmt::Debug> HolderGetter<T> for MapGetter<T> {
        fn get_or_throw(&self, key: &str) -> Holder<T> {
            let value = self
                .map
                .get(key)
                .unwrap_or_else(|| panic!("missing key: {key}"))
                .clone();
            Holder::new(key, value)
        }
    }

    fn biome_holder(key: &str) -> Holder<Biome> {
        Holder::new(key, Biome::new(BiomeGenerationSettings::default()))
    }

    #[test]
    fn validate_height_boundary_and_wrapping_overflow() {
        let biome = biome_holder(BIOMES_PLAINS);
        let mut at_limit = FlatLevelGeneratorSettings::new(None, biome.clone(), vec![]);
        at_limit
            .get_layers_info()
            .push(FlatLayerInfo::new(DIMENSION_TYPE_Y_SIZE, Block::dirt()));
        at_limit.update_layers();
        assert!(FlatLevelGeneratorSettings::validate_height(at_limit).is_ok());

        let mut over_limit = FlatLevelGeneratorSettings::new(None, biome.clone(), vec![]);
        over_limit.get_layers_info().push(FlatLayerInfo::new(
            DIMENSION_TYPE_Y_SIZE + 1,
            Block::dirt(),
        ));
        let err = FlatLevelGeneratorSettings::validate_height(over_limit);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Sum of layer heights is >"));

        // Java int sum wraps: i32::MAX + 1 becomes i32::MIN, which is not > Y_SIZE.
        let mut wrapping = FlatLevelGeneratorSettings::new(None, biome, vec![]);
        wrapping
            .get_layers_info()
            .push(FlatLayerInfo::new(i32::MAX, Block::dirt()));
        wrapping
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::dirt()));
        assert!(FlatLevelGeneratorSettings::validate_height(wrapping).is_ok());
    }

    #[test]
    fn update_layers_handles_negative_height_and_void_gen() {
        let mut settings = FlatLevelGeneratorSettings::new(None, biome_holder(BIOMES_PLAINS), vec![]);
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(-3, Block::dirt()));
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(2, Block::air()));
        settings.update_layers();
        assert_eq!(settings.get_layers().len(), 2);
        assert!(settings
            .get_layers()
            .iter()
            .all(|s| s.as_ref().is_some_and(BlockState::is_air)));
        assert!(settings.void_gen);

        let mut non_void = FlatLevelGeneratorSettings::new(None, biome_holder(BIOMES_PLAINS), vec![]);
        non_void
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::dirt()));
        non_void.update_layers();
        assert!(!non_void.void_gen);
    }

    #[test]
    fn with_biome_and_layers_preserves_flags_and_lakes() {
        let biome = biome_holder(BIOMES_PLAINS);
        let lake = Holder::direct(PlacedFeature::reference("minecraft:lake"));
        let mut settings = FlatLevelGeneratorSettings::new(None, biome, vec![lake.clone()]);
        settings.set_decoration();
        settings.set_add_lakes();
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::bedrock()));
        settings.update_layers();

        let new_biome = biome_holder("minecraft:desert");
        let copied = settings.with_biome_and_layers(
            vec![FlatLayerInfo::new(3, Block::dirt())],
            None,
            new_biome.clone(),
        );

        assert_eq!(copied.get_biome(), &new_biome);
        assert_eq!(copied.get_layers().len(), 3);
        assert!(copied.decoration);
        assert!(copied.add_lakes);
        assert_eq!(copied.lakes, vec![lake]);
    }

    #[test]
    fn adjust_generation_settings_replaces_non_opaque_layers_with_fill_layer() {
        let biome = biome_holder(BIOMES_PLAINS);
        let mut settings = FlatLevelGeneratorSettings::new(None, biome.clone(), vec![]);
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::air()));
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::dirt()));
        settings.update_layers();

        let gs = settings.adjust_generation_settings(&biome);
        assert!(settings.get_layers()[0].is_none());
        assert!(settings.get_layers()[1].is_some());

        let top = &gs.features()[GENERATION_STEP_DECORATION_TOP_LAYER_MODIFICATION];
        assert_eq!(top.len(), 1);
        let feature = top.iter().next().unwrap().value();
        match &feature.kind {
            PlacedFeatureKind::FillLayer { layer, state } => {
                assert_eq!(*layer, 0);
                assert!(state.is_air());
            }
            _ => panic!("expected fill layer feature"),
        }
    }

    #[test]
    fn adjust_generation_settings_decoration_and_lake_filtering() {
        let mut base_features = vec![HolderSet::direct(vec![]); GENERATION_STEP_DECORATION_COUNT];
        base_features[GENERATION_STEP_DECORATION_LAKES] =
            HolderSet::direct(vec![Holder::direct(PlacedFeature::reference("base_lake"))]);
        base_features[GENERATION_STEP_DECORATION_UNDERGROUND_STRUCTURES] = HolderSet::direct(vec![
            Holder::direct(PlacedFeature::reference("underground")),
        ]);
        base_features[GENERATION_STEP_DECORATION_SURFACE_STRUCTURES] = HolderSet::direct(vec![
            Holder::direct(PlacedFeature::reference("surface")),
        ]);
        base_features[GENERATION_STEP_DECORATION_TOP_LAYER_MODIFICATION] = HolderSet::direct(vec![
            Holder::direct(PlacedFeature::reference("top")),
        ]);

        let biome = Holder::new(
            BIOMES_PLAINS,
            Biome::new(BiomeGenerationSettings::new(base_features)),
        );
        let mut settings = FlatLevelGeneratorSettings::new(
            None,
            biome.clone(),
            vec![Holder::direct(PlacedFeature::reference("flat_lake"))],
        );
        settings.set_add_lakes();
        settings.set_decoration();
        settings
            .get_layers_info()
            .push(FlatLayerInfo::new(1, Block::dirt()));
        settings.update_layers();

        let gs = settings.adjust_generation_settings(&biome);
        let lakes: Vec<_> = gs.features()[GENERATION_STEP_DECORATION_LAKES]
            .iter()
            .map(|h| h.value().clone())
            .collect();
        assert_eq!(lakes, vec![PlacedFeature::reference("flat_lake")]);
        assert!(gs.features()[GENERATION_STEP_DECORATION_UNDERGROUND_STRUCTURES].is_empty());
        assert!(gs.features()[GENERATION_STEP_DECORATION_SURFACE_STRUCTURES].is_empty());
        let top: Vec<_> = gs.features()[GENERATION_STEP_DECORATION_TOP_LAYER_MODIFICATION]
            .iter()
            .map(|h| h.value().clone())
            .collect();
        assert_eq!(top, vec![PlacedFeature::reference("top")]);
    }

    #[test]
    fn get_default_builds_expected_layers_and_overrides() {
        let biome_getter = MapGetter {
            map: HashMap::from([(
                BIOMES_PLAINS.to_string(),
                Biome::new(BiomeGenerationSettings::default()),
            )]),
        };
        let structure_getter = MapGetter {
            map: HashMap::from([
                (
                    BUILTIN_STRUCTURE_SETS_STRONGHOLDS.to_string(),
                    StructureSet {
                        id: "strongholds".into(),
                    },
                ),
                (
                    BUILTIN_STRUCTURE_SETS_VILLAGES.to_string(),
                    StructureSet {
                        id: "villages".into(),
                    },
                ),
            ]),
        };
        let placed_getter = MapGetter {
            map: HashMap::from([
                (
                    MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_UNDERGROUND.to_string(),
                    PlacedFeature::reference("lava_underground"),
                ),
                (
                    MISC_OVERWORLD_PLACEMENTS_LAKE_LAVA_SURFACE.to_string(),
                    PlacedFeature::reference("lava_surface"),
                ),
            ]),
        };

        let settings = FlatLevelGeneratorSettings::get_default(
            &biome_getter,
            &structure_getter,
            &placed_getter,
        );

        let layers = settings.get_layers();
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].as_ref().unwrap().get_block().id, "minecraft:bedrock");
        assert_eq!(layers[1].as_ref().unwrap().get_block().id, "minecraft:dirt");
        assert_eq!(layers[2].as_ref().unwrap().get_block().id, "minecraft:dirt");
        assert_eq!(
            layers[3].as_ref().unwrap().get_block().id,
            "minecraft:grass_block"
        );

        let overrides = settings.structure_overrides().unwrap();
        let ids: Vec<_> = overrides
            .iter()
            .map(|h| h.value().id.as_str())
            .collect();
        assert_eq!(ids, vec!["strongholds", "villages"]);
    }
}
