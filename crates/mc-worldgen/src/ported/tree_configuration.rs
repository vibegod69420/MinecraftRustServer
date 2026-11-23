//! Port of `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration`.

use std::fmt::Debug;

/// Marker trait for Minecraft feature configurations.
pub trait FeatureConfiguration: Debug {}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider {
    pub block: String,
}

impl BlockStateProvider {
    /// Creates a simple provider for the given block id/name.
    pub fn simple(block: impl Into<String>) -> Self {
        Self {
            block: block.into(),
        }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrunkPlacer {
    pub id: String,
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoliagePlacer {
    pub id: String,
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootPlacer {
    pub id: String,
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureSize {
    pub id: String,
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDecorator {
    pub id: String,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeConfiguration {
    pub trunk_provider: BlockStateProvider,
    pub dirt_provider: BlockStateProvider,
    pub trunk_placer: TrunkPlacer,
    pub foliage_provider: BlockStateProvider,
    pub foliage_placer: FoliagePlacer,
    pub root_placer: Option<RootPlacer>,
    pub minimum_size: FeatureSize,
    pub decorators: Vec<TreeDecorator>,
    pub ignore_vines: bool,
    pub force_dirt: bool,
}

impl FeatureConfiguration for TreeConfiguration {}

impl TreeConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration#<init>
    pub fn new(
        trunk_provider: BlockStateProvider,
        trunk_placer: TrunkPlacer,
        foliage_provider: BlockStateProvider,
        foliage_placer: FoliagePlacer,
        root_placer: Option<RootPlacer>,
        dirt_provider: BlockStateProvider,
        minimum_size: FeatureSize,
        decorators: Vec<TreeDecorator>,
        ignore_vines: bool,
        force_dirt: bool,
    ) -> Self {
        Self {
            trunk_provider,
            trunk_placer,
            foliage_provider,
            foliage_placer,
            root_placer,
            dirt_provider,
            minimum_size,
            decorators,
            ignore_vines,
            force_dirt,
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration.TreeConfigurationBuilder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeConfigurationBuilder {
    pub trunk_provider: BlockStateProvider,
    trunk_placer: TrunkPlacer,
    pub foliage_provider: BlockStateProvider,
    foliage_placer: FoliagePlacer,
    root_placer: Option<RootPlacer>,
    dirt_provider: BlockStateProvider,
    minimum_size: FeatureSize,
    decorators: Vec<TreeDecorator>,
    ignore_vines: bool,
    force_dirt: bool,
}

impl TreeConfigurationBuilder {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#<init>
    pub fn new(
        trunk_provider: BlockStateProvider,
        trunk_placer: TrunkPlacer,
        foliage_provider: BlockStateProvider,
        foliage_placer: FoliagePlacer,
        root_placer: Option<RootPlacer>,
        minimum_size: FeatureSize,
    ) -> Self {
        Self {
            trunk_provider,
            trunk_placer,
            foliage_provider,
            dirt_provider: BlockStateProvider::simple("minecraft:dirt"),
            foliage_placer,
            root_placer,
            minimum_size,
            decorators: Vec::new(),
            ignore_vines: false,
            force_dirt: false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#<init>
    pub fn without_root(
        trunk_provider: BlockStateProvider,
        trunk_placer: TrunkPlacer,
        foliage_provider: BlockStateProvider,
        foliage_placer: FoliagePlacer,
        minimum_size: FeatureSize,
    ) -> Self {
        Self::new(
            trunk_provider,
            trunk_placer,
            foliage_provider,
            foliage_placer,
            None,
            minimum_size,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#dirt
    pub fn dirt(mut self, dirt_provider: BlockStateProvider) -> Self {
        self.dirt_provider = dirt_provider;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#decorators
    pub fn decorators(mut self, decorators: Vec<TreeDecorator>) -> Self {
        self.decorators = decorators;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#ignoreVines
    pub fn ignore_vines(mut self) -> Self {
        self.ignore_vines = true;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#forceDirt
    pub fn force_dirt(mut self) -> Self {
        self.force_dirt = true;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration$TreeConfigurationBuilder#build
    pub fn build(self) -> TreeConfiguration {
        TreeConfiguration::new(
            self.trunk_provider,
            self.trunk_placer,
            self.foliage_provider,
            self.foliage_placer,
            self.root_placer,
            self.dirt_provider,
            self.minimum_size,
            self.decorators,
            self.ignore_vines,
            self.force_dirt,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_builder() -> TreeConfigurationBuilder {
        TreeConfigurationBuilder::new(
            BlockStateProvider::simple("minecraft:oak_log"),
            TrunkPlacer {
                id: "straight_trunk_placer".to_string(),
            },
            BlockStateProvider::simple("minecraft:oak_leaves"),
            FoliagePlacer {
                id: "blob_foliage_placer".to_string(),
            },
            Some(RootPlacer {
                id: "mangrove_root_placer".to_string(),
            }),
            FeatureSize {
                id: "two_layers_feature_size".to_string(),
            },
        )
    }

    #[test]
    fn builder_defaults_match_java() {
        let cfg = base_builder().build();
        assert_eq!(cfg.trunk_provider.block, "minecraft:oak_log");
        assert_eq!(cfg.foliage_provider.block, "minecraft:oak_leaves");
        assert_eq!(cfg.dirt_provider.block, "minecraft:dirt");
        assert_eq!(
            cfg.root_placer.as_ref().map(|r| r.id.as_str()),
            Some("mangrove_root_placer")
        );
        assert!(cfg.decorators.is_empty());
        assert!(!cfg.ignore_vines);
        assert!(!cfg.force_dirt);
    }

    #[test]
    fn builder_overrides_and_flags() {
        let decorators = vec![
            TreeDecorator {
                id: "beehive".to_string(),
            },
            TreeDecorator {
                id: "cocoa".to_string(),
            },
        ];
        let cfg = base_builder()
            .dirt(BlockStateProvider::simple("minecraft:podzol"))
            .decorators(decorators.clone())
            .ignore_vines()
            .force_dirt()
            .build();

        assert_eq!(cfg.dirt_provider.block, "minecraft:podzol");
        assert_eq!(cfg.decorators, decorators);
        assert!(cfg.ignore_vines);
        assert!(cfg.force_dirt);
    }

    #[test]
    fn builder_without_root_uses_empty_optional() {
        let cfg = TreeConfigurationBuilder::without_root(
            BlockStateProvider::simple("minecraft:birch_log"),
            TrunkPlacer {
                id: "straight".to_string(),
            },
            BlockStateProvider::simple("minecraft:birch_leaves"),
            FoliagePlacer {
                id: "blob".to_string(),
            },
            FeatureSize {
                id: "min".to_string(),
            },
        )
        .build();

        assert_eq!(cfg.root_placer, None);
        assert_eq!(cfg.dirt_provider.block, "minecraft:dirt");
        assert!(!cfg.ignore_vines);
        assert!(!cfg.force_dirt);
    }

    #[test]
    fn constructor_stores_fields_in_java_order() {
        let cfg = TreeConfiguration::new(
            BlockStateProvider::simple("t"),
            TrunkPlacer { id: "tp".to_string() },
            BlockStateProvider::simple("f"),
            FoliagePlacer { id: "fp".to_string() },
            None,
            BlockStateProvider::simple("d"),
            FeatureSize { id: "ms".to_string() },
            vec![TreeDecorator { id: "dec".to_string() }],
            true,
            false,
        );

        assert_eq!(cfg.trunk_provider.block, "t");
        assert_eq!(cfg.trunk_placer.id, "tp");
        assert_eq!(cfg.foliage_provider.block, "f");
        assert_eq!(cfg.foliage_placer.id, "fp");
        assert_eq!(cfg.root_placer, None);
        assert_eq!(cfg.dirt_provider.block, "d");
        assert_eq!(cfg.minimum_size.id, "ms");
        assert_eq!(cfg.decorators.len(), 1);
        assert!(cfg.ignore_vines);
        assert!(!cfg.force_dirt);
    }

    #[test]
    fn edge_case_numeric_semantics_for_ported_data() {
        // Structural edge cases plus integer/float bit semantics required by port rules.
        let neg: i32 = -1;
        let wrapped = i32::MAX.wrapping_add(1);
        let shifted: i32 = -2_i32.wrapping_shl((33_u32) & 31);
        let f_bits = f64::to_bits(-0.0_f64);

        let cfg = TreeConfigurationBuilder::without_root(
            BlockStateProvider::simple(format!("block_{neg}")),
            TrunkPlacer {
                id: format!("tp_{wrapped}"),
            },
            BlockStateProvider::simple(format!("leaf_{shifted}")),
            FoliagePlacer {
                id: format!("fp_{f_bits}"),
            },
            FeatureSize {
                id: "edge".to_string(),
            },
        )
        .decorators(Vec::new())
        .build();

        assert_eq!(cfg.trunk_provider.block, "block_-1");
        assert_eq!(cfg.trunk_placer.id, format!("tp_{}", i32::MIN));
        assert_eq!(cfg.foliage_provider.block, "leaf_-4");
        assert_eq!(cfg.foliage_placer.id, format!("fp_{}", 0x8000_0000_0000_0000_u64));
        assert!(cfg.decorators.is_empty());
    }
}
