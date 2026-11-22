#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeatureRef {
    pub id: u32,
}

pub trait FeatureConfiguration {
    fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        Box::new(std::iter::empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeature {
    pub features: Vec<ConfiguredFeatureRef>,
}

impl PlacedFeature {
    pub fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        Box::new(self.features.iter().cloned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct WeightedPlacedFeature {
    pub feature: Holder<PlacedFeature>,
    pub chance: f32,
}

impl WeightedPlacedFeature {
    pub fn new(feature: Holder<PlacedFeature>, chance: f32) -> Self {
        Self { feature, chance }
    }
}

#[derive(Debug, Clone)]
pub struct RandomFeatureConfiguration {
    pub features: Vec<WeightedPlacedFeature>,
    pub default_feature: Holder<PlacedFeature>,
}

impl FeatureConfiguration for RandomFeatureConfiguration {
    fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        self.get_features()
    }
}

impl RandomFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomFeatureConfiguration#<init>
    pub fn new(
        features: Vec<WeightedPlacedFeature>,
        default_feature: Holder<PlacedFeature>,
    ) -> Self {
        Self {
            features,
            default_feature,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomFeatureConfiguration#getFeatures
    pub fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        Box::new(
            self.features
                .iter()
                .flat_map(|wf| wf.feature.value().get_features())
                .chain(self.default_feature.value().get_features()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pf(ids: &[u32]) -> Holder<PlacedFeature> {
        Holder::new(PlacedFeature {
            features: ids.iter().map(|&id| ConfiguredFeatureRef { id }).collect(),
        })
    }

    fn wf(ids: &[u32], chance: f32) -> WeightedPlacedFeature {
        WeightedPlacedFeature::new(pf(ids), chance)
    }

    #[test]
    fn constructor_stores_fields() {
        let feats = vec![wf(&[1], 0.5)];
        let default = pf(&[9]);
        let cfg = RandomFeatureConfiguration::new(feats.clone(), default.clone());
        assert_eq!(cfg.features.len(), 1);
        assert_eq!(cfg.default_feature, default);
        assert_eq!(cfg.features[0].chance.to_bits(), 0.5f32.to_bits());
    }

    #[test]
    fn get_features_chains_weighted_then_default() {
        let cfg = RandomFeatureConfiguration::new(
            vec![wf(&[1, 2], 0.25), wf(&[3], 0.75)],
            pf(&[10]),
        );
        let ids: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 10]);
    }

    #[test]
    fn get_features_empty_weighted_list() {
        let cfg = RandomFeatureConfiguration::new(vec![], pf(&[5, 6]));
        let ids: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(ids, vec![5, 6]);
    }

    #[test]
    fn get_features_empty_everything() {
        let cfg = RandomFeatureConfiguration::new(vec![], pf(&[]));
        assert_eq!(cfg.get_features().count(), 0);
    }

    #[test]
    fn get_features_preserves_order_with_empty_entries_and_float_bits() {
        let neg_zero = -0.0f32;
        let cfg = RandomFeatureConfiguration::new(
            vec![wf(&[], neg_zero), wf(&[u32::MAX], 0.2), wf(&[], f32::NAN)],
            pf(&[]),
        );
        let ids: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(ids, vec![u32::MAX]);
        assert_eq!(cfg.features[0].chance.to_bits(), neg_zero.to_bits());
        assert!(cfg.features[2].chance.is_nan());
    }
}
