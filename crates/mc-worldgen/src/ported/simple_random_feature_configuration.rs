/// Stub for `net.minecraft.world.level.levelgen.feature.ConfiguredFeature<?, ?>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeature {
    pub id: u32,
}

/// Marker trait for feature configurations.
pub trait FeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeature> {
        Vec::new()
    }
}

/// Stub for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.core.Holder#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Stub for `net.minecraft.core.HolderSet<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    pub entries: Vec<Holder<T>>,
}

impl<T> HolderSet<T> {
    pub fn new(entries: Vec<Holder<T>>) -> Self {
        Self { entries }
    }

    // JAVA: net.minecraft.core.HolderSet#stream
    pub fn stream(&self) -> impl Iterator<Item = &Holder<T>> {
        self.entries.iter()
    }
}

/// Stub for `net.minecraft.world.level.levelgen.placement.PlacedFeature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeature {
    pub features: Vec<ConfiguredFeature>,
}

impl PlacedFeature {
    pub fn new(features: Vec<ConfiguredFeature>) -> Self {
        Self { features }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacedFeature#getFeatures
    pub fn get_features(&self) -> Vec<ConfiguredFeature> {
        self.features.clone()
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.SimpleRandomFeatureConfiguration`.
///
/// Codec shape (Java): `ExtraCodecs.nonEmptyHolderSet(PlacedFeature.LIST_CODEC)`
///   `.fieldOf("features").xmap(SimpleRandomFeatureConfiguration::new, c -> c.features).codec()`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRandomFeatureConfiguration {
    pub features: HolderSet<PlacedFeature>,
}

impl SimpleRandomFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleRandomFeatureConfiguration#<init>
    pub fn new(features: HolderSet<PlacedFeature>) -> Self {
        Self { features }
    }
}

impl FeatureConfiguration for SimpleRandomFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleRandomFeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeature> {
        self.features
            .stream()
            .flat_map(|holder| holder.value().get_features())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cf(id: u32) -> ConfiguredFeature {
        ConfiguredFeature { id }
    }

    fn pf(ids: &[u32]) -> PlacedFeature {
        PlacedFeature::new(ids.iter().copied().map(cf).collect())
    }

    #[test]
    fn empty_holder_set_returns_no_features() {
        let cfg = SimpleRandomFeatureConfiguration::new(HolderSet::new(vec![]));
        assert!(cfg.get_features().is_empty());
    }

    #[test]
    fn single_placed_feature_returns_its_features() {
        let cfg = SimpleRandomFeatureConfiguration::new(HolderSet::new(vec![Holder::new(pf(&[
            1, 2,
        ]))]));
        assert_eq!(cfg.get_features(), vec![cf(1), cf(2)]);
    }

    #[test]
    fn flattens_multiple_placed_features_in_order() {
        let cfg = SimpleRandomFeatureConfiguration::new(HolderSet::new(vec![
            Holder::new(pf(&[10])),
            Holder::new(pf(&[])),
            Holder::new(pf(&[20, 30])),
        ]));
        assert_eq!(cfg.get_features(), vec![cf(10), cf(20), cf(30)]);
    }

    #[test]
    fn constructor_stores_features() {
        let cfg = SimpleRandomFeatureConfiguration::new(HolderSet::new(vec![Holder::new(pf(&[
            7,
        ]))]));
        assert_eq!(cfg.features.entries.len(), 1);
        assert_eq!(cfg.features.entries[0].value().get_features(), vec![cf(7)]);
    }

    #[test]
    fn placed_feature_with_no_features_contributes_nothing() {
        let cfg = SimpleRandomFeatureConfiguration::new(HolderSet::new(vec![
            Holder::new(pf(&[])),
            Holder::new(pf(&[])),
        ]));
        assert!(cfg.get_features().is_empty());
    }
}
