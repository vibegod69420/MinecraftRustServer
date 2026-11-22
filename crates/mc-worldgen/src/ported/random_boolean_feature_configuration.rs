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
pub struct RandomBooleanFeatureConfiguration {
    pub feature_true: Holder<PlacedFeature>,
    pub feature_false: Holder<PlacedFeature>,
}

impl FeatureConfiguration for RandomBooleanFeatureConfiguration {
    fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        self.get_features()
    }
}

impl RandomBooleanFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomBooleanFeatureConfiguration#<init>
    pub fn new(feature_true: Holder<PlacedFeature>, feature_false: Holder<PlacedFeature>) -> Self {
        Self {
            feature_true,
            feature_false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomBooleanFeatureConfiguration#getFeatures
    pub fn get_features(&self) -> Box<dyn Iterator<Item = ConfiguredFeatureRef> + '_> {
        Box::new(
            self.feature_true
                .value()
                .get_features()
                .chain(self.feature_false.value().get_features()),
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

    #[test]
    fn constructor_stores_both_features() {
        let t = pf(&[1]);
        let f = pf(&[2]);
        let cfg = RandomBooleanFeatureConfiguration::new(t.clone(), f.clone());
        assert_eq!(cfg.feature_true, t);
        assert_eq!(cfg.feature_false, f);
    }

    #[test]
    fn get_features_concatenates_true_then_false() {
        let cfg = RandomBooleanFeatureConfiguration::new(pf(&[1, 2]), pf(&[3]));
        let ids: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn get_features_empty_sides() {
        let cfg = RandomBooleanFeatureConfiguration::new(pf(&[]), pf(&[]));
        assert_eq!(cfg.get_features().count(), 0);
    }

    #[test]
    fn get_features_one_side_empty_with_edge_ids() {
        let cfg = RandomBooleanFeatureConfiguration::new(pf(&[]), pf(&[0, u32::MAX]));
        let ids: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(ids, vec![0, u32::MAX]);
    }

    #[test]
    fn trait_get_features_matches_inherent() {
        let cfg = RandomBooleanFeatureConfiguration::new(pf(&[7]), pf(&[8, 9]));
        let via_trait: Vec<u32> =
            FeatureConfiguration::get_features(&cfg).map(|f| f.id).collect();
        let via_inherent: Vec<u32> = cfg.get_features().map(|f| f.id).collect();
        assert_eq!(via_trait, via_inherent);
    }
}
