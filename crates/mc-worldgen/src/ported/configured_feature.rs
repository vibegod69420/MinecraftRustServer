use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

pub trait WorldGenLevel {}
pub trait ChunkGenerator {}

pub trait DynConfiguredFeature: fmt::Debug {
    fn get_features(&self) -> Vec<Box<dyn DynConfiguredFeature>>;
}

pub trait FeatureConfiguration: fmt::Debug {
    fn get_features(&self) -> Vec<Box<dyn DynConfiguredFeature>> {
        Vec::new()
    }
}

pub trait Feature<FC: FeatureConfiguration + ?Sized>: fmt::Debug {
    fn place(
        &self,
        config: &FC,
        level: &mut dyn WorldGenLevel,
        chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool;
}

#[derive(Clone, Debug)]
pub struct ConfiguredFeature<FC, F> {
    pub feature: F,
    pub config: FC,
}

impl<FC, F> ConfiguredFeature<FC, F>
where
    FC: FeatureConfiguration,
    F: Feature<FC>,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.ConfiguredFeature#place
    pub fn place(
        &self,
        level: &mut dyn WorldGenLevel,
        chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool {
        self.feature
            .place(&self.config, level, chunk_generator, random, pos)
    }
}

impl<FC, F> ConfiguredFeature<FC, F>
where
    FC: FeatureConfiguration + Clone + 'static,
    F: Feature<FC> + Clone + 'static,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.ConfiguredFeature#getFeatures
    pub fn get_features(&self) -> Vec<Box<dyn DynConfiguredFeature>> {
        let mut out: Vec<Box<dyn DynConfiguredFeature>> = Vec::with_capacity(1);
        out.push(Box::new(self.clone()));
        out.extend(self.config.get_features());
        out
    }
}

impl<FC, F> DynConfiguredFeature for ConfiguredFeature<FC, F>
where
    FC: FeatureConfiguration + Clone + 'static,
    F: Feature<FC> + Clone + 'static,
{
    fn get_features(&self) -> Vec<Box<dyn DynConfiguredFeature>> {
        ConfiguredFeature::get_features(self)
    }
}

impl<FC, F> fmt::Display for ConfiguredFeature<FC, F>
where
    FC: fmt::Debug,
    F: fmt::Debug,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.ConfiguredFeature#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configured: {:?}: {:?}", self.feature, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct LeafConfig;
    impl FeatureConfiguration for LeafConfig {}

    #[derive(Clone, Debug)]
    struct AnyFeature {
        result: bool,
    }

    impl<FC: FeatureConfiguration> Feature<FC> for AnyFeature {
        fn place(
            &self,
            _config: &FC,
            _level: &mut dyn WorldGenLevel,
            _chunk_generator: &dyn ChunkGenerator,
            _random: &mut dyn RandomSource,
            _pos: BlockPos,
        ) -> bool {
            self.result
        }
    }

    #[derive(Clone, Debug)]
    struct ParentConfig;
    impl FeatureConfiguration for ParentConfig {
        fn get_features(&self) -> Vec<Box<dyn DynConfiguredFeature>> {
            vec![Box::new(ConfiguredFeature {
                feature: AnyFeature { result: true },
                config: LeafConfig,
            })]
        }
    }

    struct MockLevel;
    impl WorldGenLevel for MockLevel {}
    struct MockChunkGen;
    impl ChunkGenerator for MockChunkGen {}
    struct MockRandom;
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    #[test]
    fn place_delegates_to_feature() {
        let cf = ConfiguredFeature {
            feature: AnyFeature { result: true },
            config: LeafConfig,
        };
        let mut level = MockLevel;
        let cg = MockChunkGen;
        let mut rng = MockRandom;
        assert!(cf.place(&mut level, &cg, &mut rng, BlockPos { x: 1, y: -2, z: 3 }));

        let cf_false = ConfiguredFeature {
            feature: AnyFeature { result: false },
            config: LeafConfig,
        };
        assert!(!cf_false.place(&mut level, &cg, &mut rng, BlockPos { x: 0, y: 0, z: 0 }));
    }

    #[test]
    fn get_features_includes_self_and_config_features() {
        let cf = ConfiguredFeature {
            feature: AnyFeature { result: false },
            config: LeafConfig,
        };
        assert_eq!(cf.get_features().len(), 1);

        let parent = ConfiguredFeature {
            feature: AnyFeature { result: true },
            config: ParentConfig,
        };
        let feats = parent.get_features();
        assert_eq!(feats.len(), 2);
        assert_eq!(feats[1].get_features().len(), 1);
    }

    #[test]
    fn display_format_matches_java_structure() {
        let cf = ConfiguredFeature {
            feature: AnyFeature { result: true },
            config: LeafConfig,
        };
        let s = cf.to_string();
        assert!(s.starts_with("Configured: "));
        assert!(s.contains(": "));
    }

    #[test]
    fn block_pos_copy_and_negative_values() {
        let p = BlockPos { x: -1, y: i32::MIN, z: i32::MAX };
        let p2 = p;
        assert_eq!(p, p2);
        assert_eq!(p2.y, i32::MIN);
    }
}
