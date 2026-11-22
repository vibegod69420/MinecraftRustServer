use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(self) -> i32 {
        self.x
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn get_z(self) -> i32 {
        self.z
    }
}

pub trait RandomSource: Debug {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacedFeature(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Biome(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Holder<T: Copy>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    BiomeFilter,
}

pub struct LevelRef<'a>(&'a dyn Fn(BlockPos) -> Holder<Biome>);

impl<'a> LevelRef<'a> {
    pub fn get_biome(&self, pos: BlockPos) -> Holder<Biome> {
        (self.0)(pos)
    }
}

pub struct BiomeGenerationSettingsRef<'a> {
    has_feature_fn: &'a dyn Fn(&Holder<Biome>, &PlacedFeature) -> bool,
    biome: Holder<Biome>,
}

impl<'a> BiomeGenerationSettingsRef<'a> {
    pub fn has_feature(&self, feature: &PlacedFeature) -> bool {
        (self.has_feature_fn)(&self.biome, feature)
    }
}

pub struct GeneratorRef<'a> {
    has_feature_fn: &'a dyn Fn(&Holder<Biome>, &PlacedFeature) -> bool,
}

impl<'a> GeneratorRef<'a> {
    pub fn get_biome_generation_settings(
        &self,
        biome: &Holder<Biome>,
    ) -> BiomeGenerationSettingsRef<'_> {
        BiomeGenerationSettingsRef {
            has_feature_fn: self.has_feature_fn,
            biome: *biome,
        }
    }
}

pub struct PlacementContext {
    top_feature: Option<PlacedFeature>,
    biome_at: Box<dyn Fn(BlockPos) -> Holder<Biome>>,
    has_feature: Box<dyn Fn(&Holder<Biome>, &PlacedFeature) -> bool>,
}

impl PlacementContext {
    pub fn new(
        top_feature: Option<PlacedFeature>,
        biome_at: Box<dyn Fn(BlockPos) -> Holder<Biome>>,
        has_feature: Box<dyn Fn(&Holder<Biome>, &PlacedFeature) -> bool>,
    ) -> Self {
        Self {
            top_feature,
            biome_at,
            has_feature,
        }
    }

    pub fn top_feature(&self) -> Option<PlacedFeature> {
        self.top_feature
    }

    pub fn get_level(&self) -> LevelRef<'_> {
        LevelRef(&*self.biome_at)
    }

    pub fn generator(&self) -> GeneratorRef<'_> {
        GeneratorRef {
            has_feature_fn: &*self.has_feature,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeFilter;

impl BiomeFilter {
    // JAVA: net.minecraft.world.level.levelgen.placement.BiomeFilter#<init>
    fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BiomeFilter#biome
    pub fn biome() -> Self {
        Self::new()
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BiomeFilter#shouldPlace
    pub fn should_place(
        &self,
        ctx: &PlacementContext,
        _random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool {
        let feature = ctx.top_feature().unwrap_or_else(|| {
            panic!(
                "Tried to biome check an unregistered feature, or a feature that should not restrict the biome"
            )
        });
        let biome = ctx.get_level().get_biome(pos);
        ctx.generator()
            .get_biome_generation_settings(&biome)
            .has_feature(&feature)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementFilter#getPositions
    pub fn get_positions(
        &self,
        ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        if self.should_place(ctx, random, pos) {
            vec![pos]
        } else {
            Vec::new()
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BiomeFilter#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::BiomeFilter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    #[test]
    fn should_place_when_biome_has_feature() {
        let feature = PlacedFeature(1);
        let biome = Holder(Biome(10));
        let ctx = PlacementContext::new(
            Some(feature),
            Box::new(move |_| biome),
            Box::new(|_, f| *f == PlacedFeature(1)),
        );
        let filter = BiomeFilter::biome();
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, BlockPos::new(0, 64, 0)));
    }

    #[test]
    fn should_not_place_when_biome_lacks_feature() {
        let feature = PlacedFeature(1);
        let biome = Holder(Biome(10));
        let ctx = PlacementContext::new(
            Some(feature),
            Box::new(move |_| biome),
            Box::new(|_, _| false),
        );
        let filter = BiomeFilter::biome();
        let mut rng = DummyRandom;
        assert!(!filter.should_place(&ctx, &mut rng, BlockPos::new(0, 64, 0)));
    }

    #[test]
    #[should_panic(expected = "Tried to biome check an unregistered feature")]
    fn panics_when_no_top_feature() {
        let ctx = PlacementContext::new(
            None,
            Box::new(|_| Holder(Biome(0))),
            Box::new(|_, _| false),
        );
        let filter = BiomeFilter::biome();
        let mut rng = DummyRandom;
        let _ = filter.should_place(&ctx, &mut rng, BlockPos::new(0, 0, 0));
    }

    #[test]
    fn uses_biome_at_position() {
        let feature = PlacedFeature(5);
        let ctx = PlacementContext::new(
            Some(feature),
            Box::new(|pos| Holder(Biome(pos.get_x() as u64))),
            Box::new(|b, _| b.0 .0 == 42),
        );
        let filter = BiomeFilter::biome();
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, BlockPos::new(42, 0, 0)));
        assert!(!filter.should_place(&ctx, &mut rng, BlockPos::new(1, 0, 0)));
    }

    #[test]
    fn get_positions_filters() {
        let feature = PlacedFeature(7);
        let ctx = PlacementContext::new(
            Some(feature),
            Box::new(|_| Holder(Biome(1))),
            Box::new(|_, f| f.0 == 7),
        );
        let filter = BiomeFilter::biome();
        let mut rng = DummyRandom;
        let pos = BlockPos::new(-1, 2, 3);
        assert_eq!(filter.get_positions(&ctx, &mut rng, pos), vec![pos]);
        let ctx2 = PlacementContext::new(
            Some(feature),
            Box::new(|_| Holder(Biome(1))),
            Box::new(|_, _| false),
        );
        assert!(filter.get_positions(&ctx2, &mut rng, pos).is_empty());
    }

    #[test]
    fn placement_type_is_biome_filter() {
        assert_eq!(
            BiomeFilter::biome().placement_type(),
            PlacementModifierType::BiomeFilter
        );
    }
}
