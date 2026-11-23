#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoliagePlacerType {
    name: &'static str,
}

impl FoliagePlacerType {
    pub const BLOB_FOLIAGE_PLACER: Self = Self::register("blob_foliage_placer");
    pub const SPRUCE_FOLIAGE_PLACER: Self = Self::register("spruce_foliage_placer");
    pub const PINE_FOLIAGE_PLACER: Self = Self::register("pine_foliage_placer");
    pub const ACACIA_FOLIAGE_PLACER: Self = Self::register("acacia_foliage_placer");
    pub const BUSH_FOLIAGE_PLACER: Self = Self::register("bush_foliage_placer");
    pub const FANCY_FOLIAGE_PLACER: Self = Self::register("fancy_foliage_placer");
    pub const MEGA_JUNGLE_FOLIAGE_PLACER: Self = Self::register("jungle_foliage_placer");
    pub const MEGA_PINE_FOLIAGE_PLACER: Self = Self::register("mega_pine_foliage_placer");
    pub const DARK_OAK_FOLIAGE_PLACER: Self = Self::register("dark_oak_foliage_placer");
    pub const RANDOM_SPREAD_FOLIAGE_PLACER: Self = Self::register("random_spread_foliage_placer");
    pub const CHERRY_FOLIAGE_PLACER: Self = Self::register("cherry_foliage_placer");

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacerType#register
    const fn register(name: &'static str) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacerType#<init>
    const fn new(name: &'static str) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacerType#codec
    pub fn codec(&self) -> &'static str {
        self.name
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_correct_names() {
        assert_eq!(FoliagePlacerType::BLOB_FOLIAGE_PLACER.name(), "blob_foliage_placer");
        assert_eq!(FoliagePlacerType::SPRUCE_FOLIAGE_PLACER.name(), "spruce_foliage_placer");
        assert_eq!(FoliagePlacerType::MEGA_JUNGLE_FOLIAGE_PLACER.name(), "jungle_foliage_placer");
        assert_eq!(FoliagePlacerType::MEGA_PINE_FOLIAGE_PLACER.name(), "mega_pine_foliage_placer");
        assert_eq!(FoliagePlacerType::CHERRY_FOLIAGE_PLACER.name(), "cherry_foliage_placer");
    }

    #[test]
    fn all_constants_present() {
        let all = [
            FoliagePlacerType::BLOB_FOLIAGE_PLACER,
            FoliagePlacerType::SPRUCE_FOLIAGE_PLACER,
            FoliagePlacerType::PINE_FOLIAGE_PLACER,
            FoliagePlacerType::ACACIA_FOLIAGE_PLACER,
            FoliagePlacerType::BUSH_FOLIAGE_PLACER,
            FoliagePlacerType::FANCY_FOLIAGE_PLACER,
            FoliagePlacerType::MEGA_JUNGLE_FOLIAGE_PLACER,
            FoliagePlacerType::MEGA_PINE_FOLIAGE_PLACER,
            FoliagePlacerType::DARK_OAK_FOLIAGE_PLACER,
            FoliagePlacerType::RANDOM_SPREAD_FOLIAGE_PLACER,
            FoliagePlacerType::CHERRY_FOLIAGE_PLACER,
        ];
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn register_codec_and_new() {
        let t = FoliagePlacerType::register("test_placer");
        assert_eq!(t.name(), "test_placer");
        assert_eq!(t.codec(), "test_placer");
        let t2 = FoliagePlacerType::new("other");
        assert_eq!(t2.name(), "other");
    }

    #[test]
    fn equality_and_copy_semantics() {
        let a = FoliagePlacerType::BLOB_FOLIAGE_PLACER;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, FoliagePlacerType::PINE_FOLIAGE_PLACER);
    }
}
