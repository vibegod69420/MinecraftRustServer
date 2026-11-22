use std::marker::PhantomData;

/// Minimal codec stub for self-contained compilation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Codec<T>(pub PhantomData<T>);

/// Marker configuration matching `NoneFeatureConfiguration`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoneFeatureConfiguration;

/// Minimal feature placement context stub.
#[derive(Debug)]
pub struct FeaturePlaceContext<C> {
    pub config: C,
}

impl<C> FeaturePlaceContext<C> {
    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#config
    pub fn config(&self) -> &C {
        &self.config
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.NoOpFeature`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpFeature {
    _codec: PhantomData<Codec<NoneFeatureConfiguration>>,
}

impl NoOpFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.NoOpFeature#<init>
    pub fn new(_codec: Codec<NoneFeatureConfiguration>) -> Self {
        Self {
            _codec: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.NoOpFeature#place
    pub fn place(&self, _ctx: FeaturePlaceContext<NoneFeatureConfiguration>) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_always_returns_true() {
        let feature = NoOpFeature::new(Codec(PhantomData));
        let ctx = FeaturePlaceContext {
            config: NoneFeatureConfiguration,
        };
        assert!(feature.place(ctx));
    }

    #[test]
    fn place_returns_true_with_default_config() {
        let feature = NoOpFeature::new(Codec::default());
        let ctx = FeaturePlaceContext {
            config: NoneFeatureConfiguration::default(),
        };
        assert!(feature.place(ctx));
    }

    #[test]
    fn multiple_places_are_stable() {
        let feature = NoOpFeature::new(Codec(PhantomData));
        for _ in 0..4 {
            let ctx = FeaturePlaceContext {
                config: NoneFeatureConfiguration,
            };
            assert!(feature.place(ctx));
        }
    }
}
