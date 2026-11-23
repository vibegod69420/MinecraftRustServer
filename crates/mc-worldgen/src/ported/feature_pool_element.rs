use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructurePoolElementType {
    Feature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidSettings {
    ApplyWaterlogging,
    IgnoreWaterlogging,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag {
    entries: BTreeMap<String, String>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_string(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_owned(), value.to_owned());
    }

    pub fn store_string(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_owned(), value.to_owned());
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub block: String,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: CompoundTag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: CompoundTag,
}

impl JigsawBlockInfo {
    pub fn of(info: StructureBlockInfo) -> Self {
        Self {
            pos: info.pos,
            state: info.state,
            nbt: info.nbt,
        }
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

pub trait StructureTemplateManager {}
pub trait WorldGenLevel {}
pub trait StructureManager {}
pub trait ChunkGenerator {}
pub trait RandomSource {}

pub trait PlacedFeature: fmt::Debug {
    fn place(
        &self,
        level: &mut dyn WorldGenLevel,
        chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool;
}

pub struct FeaturePoolElement<F> {
    pub projection: Projection,
    pub feature: Holder<F>,
    pub default_jigsaw_nbt: CompoundTag,
}

impl<F: PlacedFeature> FeaturePoolElement<F> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#<init>
    pub fn new(feature: Holder<F>, projection: Projection) -> Self {
        let mut element = Self {
            projection,
            feature,
            default_jigsaw_nbt: CompoundTag::new(),
        };
        element.default_jigsaw_nbt = element.fill_default_jigsaw_nbt();
        element
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#fillDefaultJigsawNBT
    fn fill_default_jigsaw_nbt(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.store_string("name", "minecraft:bottom");
        tag.put_string("final_state", "minecraft:air");
        tag.store_string("pool", "minecraft:empty");
        tag.store_string("target", "minecraft:empty");
        tag.store_string("joint", "rollable");
        tag
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#getSize
    pub fn get_size(
        &self,
        _template_manager: &dyn StructureTemplateManager,
        _rotation: Rotation,
    ) -> Vec3i {
        Vec3i::ZERO
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#getShuffledJigsawBlocks
    pub fn get_shuffled_jigsaw_blocks(
        &self,
        _template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        _rotation: Rotation,
        _random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        let mut properties = BTreeMap::new();
        properties.insert("orientation".to_owned(), "down_south".to_owned());
        let state = BlockState {
            block: "minecraft:jigsaw".to_owned(),
            properties,
        };
        vec![JigsawBlockInfo::of(StructureBlockInfo {
            pos,
            state,
            nbt: self.default_jigsaw_nbt.clone(),
        })]
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#getBoundingBox
    pub fn get_bounding_box(
        &self,
        template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox {
        let size = self.get_size(template_manager, rotation);
        BoundingBox {
            min_x: pos.x,
            min_y: pos.y,
            min_z: pos.z,
            max_x: pos.x.wrapping_add(size.x),
            max_y: pos.y.wrapping_add(size.y),
            max_z: pos.z.wrapping_add(size.z),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#place
    pub fn place(
        &self,
        _template_manager: &dyn StructureTemplateManager,
        level: &mut dyn WorldGenLevel,
        _structure_manager: &dyn StructureManager,
        chunk_generator: &dyn ChunkGenerator,
        pos: BlockPos,
        _pivot: BlockPos,
        _rotation: Rotation,
        _bounding_box: BoundingBox,
        random: &mut dyn RandomSource,
        _liquid_settings: LiquidSettings,
        _keep_jigsaws: bool,
    ) -> bool {
        self.feature
            .value()
            .place(level, chunk_generator, random, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#getType
    pub fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType::Feature
    }
}

impl<F: fmt::Debug> fmt::Display for FeaturePoolElement<F> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.FeaturePoolElement#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Feature[{:?}]", self.feature.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug)]
    struct MockFeature {
        result: bool,
    }

    impl PlacedFeature for MockFeature {
        fn place(
            &self,
            _level: &mut dyn WorldGenLevel,
            _chunk_generator: &dyn ChunkGenerator,
            _random: &mut dyn RandomSource,
            _pos: BlockPos,
        ) -> bool {
            self.result
        }
    }

    #[derive(Debug)]
    struct RecordingFeature {
        result: bool,
        placed_pos: Cell<Option<BlockPos>>,
    }

    impl PlacedFeature for RecordingFeature {
        fn place(
            &self,
            _level: &mut dyn WorldGenLevel,
            _chunk_generator: &dyn ChunkGenerator,
            _random: &mut dyn RandomSource,
            pos: BlockPos,
        ) -> bool {
            self.placed_pos.set(Some(pos));
            self.result
        }
    }

    struct MockTm;
    impl StructureTemplateManager for MockTm {}
    struct MockLevel;
    impl WorldGenLevel for MockLevel {}
    struct MockSm;
    impl StructureManager for MockSm {}
    struct MockCg;
    impl ChunkGenerator for MockCg {}
    struct MockRng;
    impl RandomSource for MockRng {}

    #[test]
    fn get_size_returns_zero() {
        let elem = FeaturePoolElement::new(
            Holder::new(MockFeature { result: true }),
            Projection::Rigid,
        );
        let tm = MockTm;
        assert_eq!(elem.get_size(&tm, Rotation::Clockwise90), Vec3i::ZERO);
    }

    #[test]
    fn shuffled_jigsaw_blocks_default_nbt_and_negative_pos() {
        let elem = FeaturePoolElement::new(
            Holder::new(MockFeature { result: true }),
            Projection::TerrainMatching,
        );
        let tm = MockTm;
        let mut rng = MockRng;
        let pos = BlockPos {
            x: -5,
            y: 7,
            z: -13,
        };
        let blocks = elem.get_shuffled_jigsaw_blocks(&tm, pos, Rotation::None, &mut rng);
        assert_eq!(blocks.len(), 1);
        let info = &blocks[0];
        assert_eq!(info.pos, pos);
        assert_eq!(info.state.block, "minecraft:jigsaw");
        assert_eq!(
            info.state.properties.get("orientation"),
            Some(&"down_south".to_owned())
        );
        assert_eq!(info.nbt.get_string("name"), Some("minecraft:bottom"));
        assert_eq!(info.nbt.get_string("final_state"), Some("minecraft:air"));
        assert_eq!(info.nbt.get_string("pool"), Some("minecraft:empty"));
        assert_eq!(info.nbt.get_string("target"), Some("minecraft:empty"));
        assert_eq!(info.nbt.get_string("joint"), Some("rollable"));
    }

    #[test]
    fn bounding_box_negative_and_overflow_wrapping() {
        let elem = FeaturePoolElement::new(
            Holder::new(MockFeature { result: true }),
            Projection::Rigid,
        );
        let tm = MockTm;
        let pos = BlockPos {
            x: i32::MIN,
            y: -1,
            z: i32::MAX,
        };
        let bb = elem.get_bounding_box(&tm, pos, Rotation::None);
        assert_eq!(bb.min_x, i32::MIN);
        assert_eq!(bb.min_y, -1);
        assert_eq!(bb.min_z, i32::MAX);
        assert_eq!(bb.max_x, i32::MIN.wrapping_add(0));
        assert_eq!(bb.max_y, (-1i32).wrapping_add(0));
        assert_eq!(bb.max_z, i32::MAX.wrapping_add(0));
    }

    #[test]
    fn bounding_box_preserves_float_bit_patterns() {
        let elem = FeaturePoolElement::new(
            Holder::new(MockFeature { result: true }),
            Projection::Rigid,
        );
        let tm = MockTm;
        let pos = BlockPos {
            x: f32::NAN.to_bits() as i32,
            y: f32::INFINITY.to_bits() as i32,
            z: f64::NEG_INFINITY.to_bits() as i32,
        };
        let bb = elem.get_bounding_box(&tm, pos, Rotation::Clockwise180);
        assert_eq!(bb.min_x, pos.x);
        assert_eq!(bb.min_y, pos.y);
        assert_eq!(bb.min_z, pos.z);
        assert_eq!(bb.max_x, pos.x);
        assert_eq!(bb.max_y, pos.y);
        assert_eq!(bb.max_z, pos.z);
    }

    #[test]
    fn place_delegates_to_feature_and_reports_type_and_string() {
        let feat = RecordingFeature {
            result: true,
            placed_pos: Cell::new(None),
        };
        let elem = FeaturePoolElement::new(Holder::new(feat), Projection::Rigid);
        let tm = MockTm;
        let mut level = MockLevel;
        let sm = MockSm;
        let cg = MockCg;
        let mut rng = MockRng;
        let pos = BlockPos { x: 10, y: 20, z: 30 };
        let pivot = BlockPos { x: 1, y: 2, z: 3 };
        let bb = BoundingBox {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 1,
            max_y: 1,
            max_z: 1,
        };

        let ok = elem.place(
            &tm,
            &mut level,
            &sm,
            &cg,
            pos,
            pivot,
            Rotation::Clockwise180,
            bb,
            &mut rng,
            LiquidSettings::IgnoreWaterlogging,
            true,
        );
        assert!(ok);
        assert_eq!(elem.feature.value().placed_pos.get(), Some(pos));
        assert_eq!(elem.get_type(), StructurePoolElementType::Feature);
        let s = elem.to_string();
        assert!(s.starts_with("Feature["));
        assert!(s.ends_with(']'));
    }
}
