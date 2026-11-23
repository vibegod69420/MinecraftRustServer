use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self(format!("minecraft:{path}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
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

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn as_block_pos(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#getRandom
    pub fn get_random(random: &mut impl RandomSource) -> Self {
        match random.next_int_bound(4) {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            _ => Self::Counterclockwise90,
        }
    }

    pub fn get_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "clockwise_90" => Some(Self::Clockwise90),
            "180" | "clockwise_180" => Some(Self::Clockwise180),
            "counterclockwise_90" => Some(Self::Counterclockwise90),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn from_corners(a: BlockPos, b: BlockPos) -> Self {
        Self {
            min_x: a.x.min(b.x),
            min_y: a.y.min(b.y),
            min_z: a.z.min(b.z),
            max_x: a.x.max(b.x),
            max_y: a.y.max(b.y),
            max_z: a.z.max(b.z),
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int_bound(&mut self, bound: i32) -> i32;
    fn next_long(&mut self) -> i64;
}

// JAVA: net.minecraft.util.Mth#nextInt
pub fn mth_next_int(random: &mut impl RandomSource, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        min.wrapping_add(random.next_int_bound(max.wrapping_sub(min).wrapping_add(1)))
    }
}

// JAVA: net.minecraft.util.Util#getRandom
fn util_get_random_str<'a>(list: &'a [&'a str], random: &mut impl RandomSource) -> &'a str {
    let idx = random.next_int_bound(list.len() as i32) as usize;
    list[idx]
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#transform
pub fn structure_template_transform(
    pos: BlockPos,
    mirror: Mirror,
    rotation: Rotation,
    pivot: BlockPos,
) -> BlockPos {
    let mut x = pos.x;
    let y = pos.y;
    let mut z = pos.z;
    let mut mirrored = true;
    match mirror {
        Mirror::LeftRight => z = z.wrapping_neg(),
        Mirror::FrontBack => x = x.wrapping_neg(),
        Mirror::None => mirrored = false,
    }
    match rotation {
        Rotation::Counterclockwise90 => BlockPos::new(
            pivot.x.wrapping_add(z),
            pivot.y.wrapping_add(y),
            pivot.z.wrapping_sub(x),
        ),
        Rotation::Clockwise90 => BlockPos::new(
            pivot.x.wrapping_sub(z),
            pivot.y.wrapping_add(y),
            pivot.z.wrapping_add(x),
        ),
        Rotation::Clockwise180 => BlockPos::new(
            pivot.x.wrapping_sub(x),
            pivot.y.wrapping_add(y),
            pivot.z.wrapping_sub(z),
        ),
        Rotation::None => {
            if mirrored {
                BlockPos::new(
                    pivot.x.wrapping_add(x),
                    pivot.y.wrapping_add(y),
                    pivot.z.wrapping_add(z),
                )
            } else {
                pos
            }
        }
    }
}

// JAVA: net.minecraft.core.BlockPos#betweenClosed
pub fn block_pos_between_closed(a: BlockPos, b: BlockPos) -> impl Iterator<Item = BlockPos> {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);
    let min_z = a.z.min(b.z);
    let max_x = a.x.max(b.x);
    let max_y = a.y.max(b.y);
    let max_z = a.z.max(b.z);
    let mut x = min_x;
    let mut y = min_y;
    let mut z = min_z;
    let mut started = false;
    std::iter::from_fn(move || {
        if !started {
            started = true;
            if min_x > max_x || min_y > max_y || min_z > max_z {
                return None;
            }
            return Some(BlockPos::new(x, y, z));
        }
        if z < max_z {
            z = z.wrapping_add(1);
        } else if y < max_y {
            z = min_z;
            y = y.wrapping_add(1);
        } else if x < max_x {
            z = min_z;
            y = min_y;
            x = x.wrapping_add(1);
        } else {
            return None;
        }
        Some(BlockPos::new(x, y, z))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    Air,
    Water,
    Chest,
    Ice,
    PackedIce,
    BlueIce,
    FrostedIce,
    Sand,
    Gravel,
    SuspiciousSand,
    SuspiciousGravel,
    Other,
}

impl BlockId {
    pub fn is_air(self) -> bool {
        matches!(self, BlockId::Air)
    }

    pub fn is_ice(self) -> bool {
        matches!(
            self,
            BlockId::Ice | BlockId::PackedIce | BlockId::BlueIce | BlockId::FrostedIce
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: BlockId,
    pub waterlogged: bool,
}

impl BlockState {
    pub const AIR: Self = Self {
        block: BlockId::Air,
        waterlogged: false,
    };
    pub const WATER: Self = Self {
        block: BlockId::Water,
        waterlogged: false,
    };

    pub fn is_air(&self) -> bool {
        self.block.is_air()
    }

    pub fn is_ice(&self) -> bool {
        self.block.is_ice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FluidId {
    Empty,
    Water,
    FlowingWater,
    Lava,
    FlowingLava,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FluidState {
    pub fluid: FluidId,
}

impl FluidState {
    pub const EMPTY: Self = Self {
        fluid: FluidId::Empty,
    };

    pub fn is_water(&self) -> bool {
        matches!(self.fluid, FluidId::Water | FluidId::FlowingWater)
    }
}

pub trait BlockGetter {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn get_fluid_state(&self, pos: BlockPos) -> FluidState;
    fn get_min_y(&self) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloorWg,
}

pub trait WorldGenLevel: BlockGetter {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
}

pub trait ServerLevelAccessor {
    fn get_fluid_state(&self, pos: BlockPos) -> FluidState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
    fn set_chest_loot_table(&mut self, pos: BlockPos, loot_table: Identifier, seed: i64);
    fn get_sea_level(&self) -> i32;
    fn try_spawn_drowned(&mut self, pos: BlockPos, random: &mut dyn RandomSource) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OceanRuinStructureType {
    Warm,
    Cold,
}

impl OceanRuinStructureType {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Warm => "warm",
            Self::Cold => "cold",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "warm" => Some(Self::Warm),
            "cold" => Some(Self::Cold),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OceanRuinStructure {
    pub biome_temp: OceanRuinStructureType,
    pub large_probability: f32,
    pub cluster_probability: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TemplateSize {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Default)]
pub struct StructureTemplateManager {
    sizes: BTreeMap<String, TemplateSize>,
}

impl StructureTemplateManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: &Identifier, size: TemplateSize) {
        self.sizes.insert(id.0.clone(), size);
    }

    pub fn get_size(&self, id: &Identifier) -> TemplateSize {
        self.sizes.get(&id.0).copied().unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructureProcessor {
    BlockRot { integrity: f32 },
    BlockIgnoreStructureAndAir,
    CappedRuleProcessor {
        input: BlockId,
        output: BlockId,
        loot_table: Identifier,
        limit: i32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub processors: Vec<StructureProcessor>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompoundTag {
    strings: BTreeMap<String, String>,
    floats: BTreeMap<String, u32>,
    ints: BTreeMap<String, i32>,
    booleans: BTreeMap<String, bool>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_string(&mut self, key: &str, value: String) {
        self.strings.insert(key.to_string(), value);
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }

    pub fn put_float(&mut self, key: &str, value: f32) {
        self.floats.insert(key.to_string(), value.to_bits());
    }

    pub fn get_float_or(&self, key: &str, default: f32) -> f32 {
        self.floats
            .get(key)
            .copied()
            .map(f32::from_bits)
            .unwrap_or(default)
    }

    pub fn put_int(&mut self, key: &str, value: i32) {
        self.ints.insert(key.to_string(), value);
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        self.ints.get(key).copied().unwrap_or(default)
    }

    pub fn put_boolean(&mut self, key: &str, value: bool) {
        self.booleans.insert(key.to_string(), value);
    }

    pub fn get_boolean_or(&self, key: &str, default: bool) -> bool {
        self.booleans.get(key).copied().unwrap_or(default)
    }

    pub fn store_rotation(&mut self, key: &str, rotation: Rotation) {
        self.put_string(key, rotation.get_name().to_string());
    }

    pub fn read_rotation(&self, key: &str) -> Option<Rotation> {
        self.get_string(key).and_then(Rotation::by_name)
    }

    pub fn store_biome_type(&mut self, key: &str, biome_type: OceanRuinStructureType) {
        self.put_string(key, biome_type.get_name().to_string());
    }

    pub fn read_biome_type(&self, key: &str) -> Option<OceanRuinStructureType> {
        self.get_string(key).and_then(OceanRuinStructureType::by_name)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StructurePieceSerializationContext;

#[derive(Clone, Copy, Debug, Default)]
pub struct StructureManager;

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkGenerator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub const WARM_RUINS: &[&str] = &[
    "underwater_ruin/warm_1",
    "underwater_ruin/warm_2",
    "underwater_ruin/warm_3",
    "underwater_ruin/warm_4",
    "underwater_ruin/warm_5",
    "underwater_ruin/warm_6",
    "underwater_ruin/warm_7",
    "underwater_ruin/warm_8",
];

pub const RUINS_BRICK: &[&str] = &[
    "underwater_ruin/brick_1",
    "underwater_ruin/brick_2",
    "underwater_ruin/brick_3",
    "underwater_ruin/brick_4",
    "underwater_ruin/brick_5",
    "underwater_ruin/brick_6",
    "underwater_ruin/brick_7",
    "underwater_ruin/brick_8",
];

pub const RUINS_CRACKED: &[&str] = &[
    "underwater_ruin/cracked_1",
    "underwater_ruin/cracked_2",
    "underwater_ruin/cracked_3",
    "underwater_ruin/cracked_4",
    "underwater_ruin/cracked_5",
    "underwater_ruin/cracked_6",
    "underwater_ruin/cracked_7",
    "underwater_ruin/cracked_8",
];

pub const RUINS_MOSSY: &[&str] = &[
    "underwater_ruin/mossy_1",
    "underwater_ruin/mossy_2",
    "underwater_ruin/mossy_3",
    "underwater_ruin/mossy_4",
    "underwater_ruin/mossy_5",
    "underwater_ruin/mossy_6",
    "underwater_ruin/mossy_7",
    "underwater_ruin/mossy_8",
];

pub const BIG_RUINS_BRICK: &[&str] = &[
    "underwater_ruin/big_brick_1",
    "underwater_ruin/big_brick_2",
    "underwater_ruin/big_brick_3",
    "underwater_ruin/big_brick_8",
];

pub const BIG_RUINS_MOSSY: &[&str] = &[
    "underwater_ruin/big_mossy_1",
    "underwater_ruin/big_mossy_2",
    "underwater_ruin/big_mossy_3",
    "underwater_ruin/big_mossy_8",
];

pub const BIG_RUINS_CRACKED: &[&str] = &[
    "underwater_ruin/big_cracked_1",
    "underwater_ruin/big_cracked_2",
    "underwater_ruin/big_cracked_3",
    "underwater_ruin/big_cracked_8",
];

pub const BIG_WARM_RUINS: &[&str] = &[
    "underwater_ruin/big_warm_4",
    "underwater_ruin/big_warm_5",
    "underwater_ruin/big_warm_6",
    "underwater_ruin/big_warm_7",
];

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#archyRuleProcessor
fn archy_rule_processor(
    input: BlockId,
    output: BlockId,
    loot_table: Identifier,
) -> StructureProcessor {
    StructureProcessor::CappedRuleProcessor {
        input,
        output,
        loot_table,
        limit: 5,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#WARM_SUSPICIOUS_BLOCK_PROCESSOR
pub fn warm_suspicious_block_processor() -> StructureProcessor {
    archy_rule_processor(
        BlockId::Sand,
        BlockId::SuspiciousSand,
        Identifier::with_default_namespace("archaeology/ocean_ruin_warm"),
    )
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#COLD_SUSPICIOUS_BLOCK_PROCESSOR
pub fn cold_suspicious_block_processor() -> StructureProcessor {
    archy_rule_processor(
        BlockId::Gravel,
        BlockId::SuspiciousGravel,
        Identifier::with_default_namespace("archaeology/ocean_ruin_cold"),
    )
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#getSmallWarmRuin
fn get_small_warm_ruin(random: &mut impl RandomSource) -> Identifier {
    Identifier::with_default_namespace(util_get_random_str(WARM_RUINS, random))
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#getBigWarmRuin
fn get_big_warm_ruin(random: &mut impl RandomSource) -> Identifier {
    Identifier::with_default_namespace(util_get_random_str(BIG_WARM_RUINS, random))
}

#[derive(Clone, Debug)]
pub struct OceanRuinPiece {
    pub template_id: Identifier,
    pub template_position: BlockPos,
    pub template_size: TemplateSize,
    pub place_settings: StructurePlaceSettings,
    pub integrity: f32,
    pub biome_type: OceanRuinStructureType,
    pub is_large: bool,
}

pub trait StructurePieceAccessor {
    fn add_piece(&mut self, piece: OceanRuinPiece);
}

impl StructurePieceAccessor for Vec<OceanRuinPiece> {
    fn add_piece(&mut self, piece: OceanRuinPiece) {
        self.push(piece);
    }
}

impl OceanRuinPiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#<init>
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        template_id: Identifier,
        pos: BlockPos,
        rotation: Rotation,
        integrity: f32,
        biome_type: OceanRuinStructureType,
        is_large: bool,
    ) -> Self {
        let place_settings = Self::make_settings(rotation, integrity, biome_type);
        let template_size = structure_template_manager.get_size(&template_id);
        Self {
            template_id,
            template_position: pos,
            template_size,
            place_settings,
            integrity,
            biome_type,
            is_large,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#<init>
    fn from_tag(
        structure_template_manager: &StructureTemplateManager,
        tag: &CompoundTag,
        rotation: Rotation,
        integrity: f32,
        biome_type: OceanRuinStructureType,
        is_large: bool,
    ) -> Self {
        let place_settings = Self::make_settings(rotation, integrity, biome_type);
        let template_id = Identifier(
            tag.get_string("Template")
                .unwrap_or("minecraft:empty")
                .to_string(),
        );
        let template_position = BlockPos::new(
            tag.get_int_or("TPX", 0),
            tag.get_int_or("TPY", 0),
            tag.get_int_or("TPZ", 0),
        );
        let template_size = structure_template_manager.get_size(&template_id);
        Self {
            template_id,
            template_position,
            template_size,
            place_settings,
            integrity,
            biome_type,
            is_large,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#makeSettings
    fn make_settings(
        rotation: Rotation,
        integrity: f32,
        biome_type: OceanRuinStructureType,
    ) -> StructurePlaceSettings {
        let processor = if biome_type == OceanRuinStructureType::Cold {
            cold_suspicious_block_processor()
        } else {
            warm_suspicious_block_processor()
        };
        StructurePlaceSettings {
            rotation,
            mirror: Mirror::None,
            processors: vec![
                StructureProcessor::BlockRot { integrity },
                StructureProcessor::BlockIgnoreStructureAndAir,
                processor,
            ],
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#create
    pub fn create(structure_template_manager: &StructureTemplateManager, tag: &CompoundTag) -> Self {
        let rotation = tag.read_rotation("Rot").expect("missing Rot");
        let integrity = tag.get_float_or("Integrity", 0.0f32);
        let biome_type = tag
            .read_biome_type("BiomeType")
            .expect("missing BiomeType");
        let is_large = tag.get_boolean_or("IsLarge", false);
        Self::from_tag(
            structure_template_manager,
            tag,
            rotation,
            integrity,
            biome_type,
            is_large,
        )
    }

    fn save_template_base(&self, tag: &mut CompoundTag) {
        tag.put_string("Template", self.template_id.0.clone());
        tag.put_int("TPX", self.template_position.x);
        tag.put_int("TPY", self.template_position.y);
        tag.put_int("TPZ", self.template_position.z);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut CompoundTag,
    ) {
        self.save_template_base(tag);
        tag.store_rotation("Rot", self.place_settings.rotation);
        tag.put_float("Integrity", self.integrity);
        tag.store_biome_type("BiomeType", self.biome_type);
        tag.put_boolean("IsLarge", self.is_large);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#handleDataMarker
    pub fn handle_data_marker(
        &self,
        marker: &str,
        pos: BlockPos,
        level: &mut impl ServerLevelAccessor,
        random: &mut impl RandomSource,
        _bounding_box: BoundingBox,
    ) {
        if marker == "chest" {
            let waterlogged = level.get_fluid_state(pos).is_water();
            level.set_block(
                pos,
                BlockState {
                    block: BlockId::Chest,
                    waterlogged,
                },
                2,
            );
            let loot_table = if self.is_large {
                Identifier::with_default_namespace("chests/underwater_ruin_big")
            } else {
                Identifier::with_default_namespace("chests/underwater_ruin_small")
            };
            level.set_chest_loot_table(pos, loot_table, random.next_long());
        } else if marker == "drowned" {
            if level.try_spawn_drowned(pos, random) {
                if pos.y > level.get_sea_level() {
                    level.set_block(pos, BlockState::AIR, 2);
                } else {
                    level.set_block(pos, BlockState::WATER, 2);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#postProcess
    pub fn post_process(
        &mut self,
        level: &impl WorldGenLevel,
        _structure_manager: &StructureManager,
        _chunk_generator: &ChunkGenerator,
        _random: &mut impl RandomSource,
        _bounding_box: BoundingBox,
        _chunk_pos: ChunkPos,
        _pivot: BlockPos,
    ) {
        let height = level.get_height(
            HeightmapType::OceanFloorWg,
            self.template_position.x,
            self.template_position.z,
        );
        self.template_position =
            BlockPos::new(self.template_position.x, height, self.template_position.z);
        let end = structure_template_transform(
            BlockPos::new(
                self.template_size.x.wrapping_sub(1),
                0,
                self.template_size.z.wrapping_sub(1),
            ),
            Mirror::None,
            self.place_settings.rotation,
            BlockPos::ZERO,
        )
        .offset(
            self.template_position.x,
            self.template_position.y,
            self.template_position.z,
        );
        let new_y = self.get_height_pos(self.template_position, level, end);
        self.template_position =
            BlockPos::new(self.template_position.x, new_y, self.template_position.z);
        // super.postProcess(level, structure_manager, chunk_generator, random, bounding_box, chunk_pos, pivot);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces.OceanRuinPiece#getHeight
    fn get_height_pos(&self, start: BlockPos, level: &impl BlockGetter, end: BlockPos) -> i32 {
        let mut y = start.y;
        let mut min_found = 512i32;
        let threshold = y.wrapping_sub(1);
        let mut below_threshold_count = 0i32;

        for pos in block_pos_between_closed(start, end) {
            let x = pos.x;
            let z = pos.z;
            let mut scan_y = start.y.wrapping_sub(1);
            let mut mutable = MutableBlockPos::new(x, scan_y, z);
            let mut state = level.get_block_state(mutable.as_block_pos());
            let mut fluid = level.get_fluid_state(mutable.as_block_pos());

            while (state.is_air() || fluid.is_water() || state.is_ice())
                && scan_y > level.get_min_y().wrapping_add(1)
            {
                scan_y = scan_y.wrapping_sub(1);
                mutable.set(x, scan_y, z);
                state = level.get_block_state(mutable.as_block_pos());
                fluid = level.get_fluid_state(mutable.as_block_pos());
            }

            min_found = min_found.min(scan_y);
            if scan_y < threshold.wrapping_sub(2) {
                below_threshold_count = below_threshold_count.wrapping_add(1);
            }
        }

        let x_span = start.x.wrapping_sub(end.x).wrapping_abs();
        if threshold.wrapping_sub(min_found) > 2
            && below_threshold_count > x_span.wrapping_sub(2)
        {
            y = min_found.wrapping_add(1);
        }

        y
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addPieces
pub fn add_pieces(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut impl StructurePieceAccessor,
    random: &mut impl RandomSource,
    structure: &OceanRuinStructure,
) {
    let large = random.next_float() <= structure.large_probability;
    let integrity = if large { 0.9f32 } else { 0.8f32 };
    add_piece(
        structure_template_manager,
        pos,
        rotation,
        pieces,
        random,
        structure,
        large,
        integrity,
    );
    if large && random.next_float() <= structure.cluster_probability {
        add_cluster_ruins(
            structure_template_manager,
            random,
            rotation,
            pos,
            structure,
            pieces,
        );
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addClusterRuins
fn add_cluster_ruins(
    structure_template_manager: &StructureTemplateManager,
    random: &mut impl RandomSource,
    rotation: Rotation,
    pos: BlockPos,
    structure: &OceanRuinStructure,
    pieces: &mut impl StructurePieceAccessor,
) {
    let center = BlockPos::new(pos.get_x(), 90, pos.get_z());
    let transformed =
        structure_template_transform(BlockPos::new(15, 0, 15), Mirror::None, rotation, BlockPos::ZERO)
            .offset(center.x, center.y, center.z);
    let bounding_box = BoundingBox::from_corners(center, transformed);
    let min_corner = BlockPos::new(
        center.x.min(transformed.x),
        center.y,
        center.z.min(transformed.z),
    );
    let mut positions = all_positions(random, min_corner);
    let count = mth_next_int(random, 4, 8);

    let mut i = 0i32;
    while i < count {
        if !positions.is_empty() {
            let idx = random.next_int_bound(positions.len() as i32) as usize;
            let candidate = positions.remove(idx);
            let rot = Rotation::get_random(random);
            let candidate_end =
                structure_template_transform(BlockPos::new(5, 0, 6), Mirror::None, rot, BlockPos::ZERO)
                    .offset(candidate.x, candidate.y, candidate.z);
            let candidate_box = BoundingBox::from_corners(candidate, candidate_end);
            if !candidate_box.intersects(&bounding_box) {
                add_piece(
                    structure_template_manager,
                    candidate,
                    rot,
                    pieces,
                    random,
                    structure,
                    false,
                    0.8f32,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#allPositions
fn all_positions(random: &mut impl RandomSource, origin: BlockPos) -> Vec<BlockPos> {
    let mut positions = Vec::with_capacity(8);
    positions.push(origin.offset(
        (-16i32).wrapping_add(mth_next_int(random, 1, 8)),
        0,
        16i32.wrapping_add(mth_next_int(random, 1, 7)),
    ));
    positions.push(origin.offset(
        (-16i32).wrapping_add(mth_next_int(random, 1, 8)),
        0,
        mth_next_int(random, 1, 7),
    ));
    positions.push(origin.offset(
        (-16i32).wrapping_add(mth_next_int(random, 1, 8)),
        0,
        (-16i32).wrapping_add(mth_next_int(random, 4, 8)),
    ));
    positions.push(origin.offset(
        mth_next_int(random, 1, 7),
        0,
        16i32.wrapping_add(mth_next_int(random, 1, 7)),
    ));
    positions.push(origin.offset(
        mth_next_int(random, 1, 7),
        0,
        (-16i32).wrapping_add(mth_next_int(random, 4, 6)),
    ));
    positions.push(origin.offset(
        16i32.wrapping_add(mth_next_int(random, 1, 7)),
        0,
        16i32.wrapping_add(mth_next_int(random, 3, 8)),
    ));
    positions.push(origin.offset(
        16i32.wrapping_add(mth_next_int(random, 1, 7)),
        0,
        mth_next_int(random, 1, 7),
    ));
    positions.push(origin.offset(
        16i32.wrapping_add(mth_next_int(random, 1, 7)),
        0,
        (-16i32).wrapping_add(mth_next_int(random, 4, 8)),
    ));
    positions
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addPiece
fn add_piece(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut impl StructurePieceAccessor,
    random: &mut impl RandomSource,
    structure: &OceanRuinStructure,
    large: bool,
    integrity: f32,
) {
    match structure.biome_temp {
        OceanRuinStructureType::Cold => {
            let brick = if large { BIG_RUINS_BRICK } else { RUINS_BRICK };
            let cracked = if large { BIG_RUINS_CRACKED } else { RUINS_CRACKED };
            let mossy = if large { BIG_RUINS_MOSSY } else { RUINS_MOSSY };
            let idx = random.next_int_bound(brick.len() as i32) as usize;
            pieces.add_piece(OceanRuinPiece::new(
                structure_template_manager,
                Identifier::with_default_namespace(brick[idx]),
                pos,
                rotation,
                integrity,
                structure.biome_temp,
                large,
            ));
            pieces.add_piece(OceanRuinPiece::new(
                structure_template_manager,
                Identifier::with_default_namespace(cracked[idx]),
                pos,
                rotation,
                0.7f32,
                structure.biome_temp,
                large,
            ));
            pieces.add_piece(OceanRuinPiece::new(
                structure_template_manager,
                Identifier::with_default_namespace(mossy[idx]),
                pos,
                rotation,
                0.5f32,
                structure.biome_temp,
                large,
            ));
        }
        OceanRuinStructureType::Warm => {
            let template_id = if large {
                get_big_warm_ruin(random)
            } else {
                get_small_warm_ruin(random)
            };
            pieces.add_piece(OceanRuinPiece::new(
                structure_template_manager,
                template_id,
                pos,
                rotation,
                integrity,
                structure.biome_temp,
                large,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::IntoIter;

    struct SeqRandom {
        ints: IntoIter<i32>,
        floats: IntoIter<f32>,
        longs: IntoIter<i64>,
    }

    impl SeqRandom {
        fn ints(ints: Vec<i32>) -> Self {
            Self {
                ints: ints.into_iter(),
                floats: Vec::new().into_iter(),
                longs: Vec::new().into_iter(),
            }
        }

        fn with_floats(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints: ints.into_iter(),
                floats: floats.into_iter(),
                longs: Vec::new().into_iter(),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            self.floats.next().unwrap_or(0.0f32)
        }

        fn next_int_bound(&mut self, bound: i32) -> i32 {
            let v = self.ints.next().unwrap_or(0);
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_long(&mut self) -> i64 {
            self.longs.next().unwrap_or(0i64)
        }
    }

    struct MockBlockGetter {
        min_y: i32,
        blocks: BTreeMap<(i32, i32, i32), BlockState>,
        fluids: BTreeMap<(i32, i32, i32), FluidState>,
    }

    impl MockBlockGetter {
        fn new(min_y: i32) -> Self {
            Self {
                min_y,
                blocks: BTreeMap::new(),
                fluids: BTreeMap::new(),
            }
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }

        fn set_fluid(&mut self, pos: BlockPos, fluid: FluidState) {
            self.fluids.insert((pos.x, pos.y, pos.z), fluid);
        }
    }

    impl BlockGetter for MockBlockGetter {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState {
                    block: BlockId::Other,
                    waterlogged: false,
                })
        }

        fn get_fluid_state(&self, pos: BlockPos) -> FluidState {
            self.fluids
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(FluidState::EMPTY)
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }
    }

    #[test]
    fn all_positions_negative_origin_and_mth_edge_cases() {
        let mut rng = SeqRandom::ints(vec![0; 16]);
        assert_eq!(mth_next_int(&mut rng, 5, 5), 5);
        assert_eq!(mth_next_int(&mut rng, 10, 3), 10);

        let mut rng = SeqRandom::ints(vec![0; 16]);
        let origin = BlockPos::new(-100, 64, -200);
        let positions = all_positions(&mut rng, origin);
        assert_eq!(positions.len(), 8);
        assert_eq!(positions[0], BlockPos::new(-115, 64, -183));
        assert_eq!(positions[2], BlockPos::new(-115, 64, -212));
        assert_eq!(positions[7], BlockPos::new(-83, 64, -212));
    }

    #[test]
    fn all_positions_overflow_coordinates() {
        let mut rng = SeqRandom::ints(vec![7, 6, 7, 6, 7, 4, 6, 6, 6, 2, 6, 5, 6, 6, 6, 4]);
        let origin = BlockPos::new(i32::MAX, 0, i32::MIN);
        let positions = all_positions(&mut rng, origin);
        assert_eq!(positions[0].x, i32::MAX.wrapping_add(-8));
        assert_eq!(positions[0].z, i32::MIN.wrapping_add(23));
        assert_eq!(positions[0].y, 0);
    }

    #[test]
    fn add_pieces_float_thresholds_and_integrity_bits() {
        let mgr = StructureTemplateManager::new();
        let structure = OceanRuinStructure {
            biome_temp: OceanRuinStructureType::Warm,
            large_probability: 0.5f32,
            cluster_probability: 0.0f32,
        };

        let mut pieces: Vec<OceanRuinPiece> = Vec::new();
        let mut rng = SeqRandom::with_floats(vec![0], vec![0.6f32]);
        add_pieces(
            &mgr,
            BlockPos::new(0, 0, 0),
            Rotation::None,
            &mut pieces,
            &mut rng,
            &structure,
        );
        assert_eq!(pieces.len(), 1);
        assert!(!pieces[0].is_large);
        assert_eq!(pieces[0].integrity.to_bits(), 0.8f32.to_bits());

        let mut pieces: Vec<OceanRuinPiece> = Vec::new();
        let mut rng = SeqRandom::with_floats(vec![0], vec![0.5f32, 1.0f32]);
        add_pieces(
            &mgr,
            BlockPos::new(0, 0, 0),
            Rotation::None,
            &mut pieces,
            &mut rng,
            &structure,
        );
        assert_eq!(pieces.len(), 1);
        assert!(pieces[0].is_large);
        assert_eq!(pieces[0].integrity.to_bits(), 0.9f32.to_bits());
    }

    #[test]
    fn add_piece_cold_creates_three_with_integrities() {
        let mgr = StructureTemplateManager::new();
        let structure = OceanRuinStructure {
            biome_temp: OceanRuinStructureType::Cold,
            large_probability: 0.0f32,
            cluster_probability: 0.0f32,
        };
        let mut pieces: Vec<OceanRuinPiece> = Vec::new();
        let mut rng = SeqRandom::with_floats(vec![2], vec![1.0f32]);
        add_pieces(
            &mgr,
            BlockPos::new(1, 2, 3),
            Rotation::Clockwise90,
            &mut pieces,
            &mut rng,
            &structure,
        );
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0].integrity.to_bits(), 0.8f32.to_bits());
        assert_eq!(pieces[1].integrity.to_bits(), 0.7f32.to_bits());
        assert_eq!(pieces[2].integrity.to_bits(), 0.5f32.to_bits());
        assert_eq!(
            pieces[0].template_id,
            Identifier::with_default_namespace("underwater_ruin/brick_3")
        );
        assert_eq!(
            pieces[1].template_id,
            Identifier::with_default_namespace("underwater_ruin/cracked_3")
        );
        assert_eq!(
            pieces[2].template_id,
            Identifier::with_default_namespace("underwater_ruin/mossy_3")
        );
        for p in &pieces {
            assert_eq!(p.template_position, BlockPos::new(1, 2, 3));
            assert_eq!(p.place_settings.rotation, Rotation::Clockwise90);
            assert_eq!(p.biome_type, OceanRuinStructureType::Cold);
            assert!(!p.is_large);
        }
    }

    #[test]
    fn get_height_pos_min_y_boundary_and_adjustment() {
        let mgr = StructureTemplateManager::new();
        let piece = OceanRuinPiece::new(
            &mgr,
            Identifier::with_default_namespace("underwater_ruin/warm_1"),
            BlockPos::new(0, 5, 0),
            Rotation::None,
            1.0f32,
            OceanRuinStructureType::Warm,
            false,
        );

        let mut level = MockBlockGetter::new(0);
        for yy in 0..=4 {
            level.set_block(
                BlockPos::new(0, yy, 0),
                BlockState {
                    block: BlockId::Air,
                    waterlogged: false,
                },
            );
        }
        let result = piece.get_height_pos(BlockPos::new(0, 5, 0), &level, BlockPos::new(0, 5, 0));
        assert_eq!(result, 2);

        let mut level = MockBlockGetter::new(-64);
        for yy in 6..=9 {
            let p = BlockPos::new(0, yy, 0);
            level.set_block(
                p,
                BlockState {
                    block: BlockId::Water,
                    waterlogged: false,
                },
            );
            level.set_fluid(p, FluidState { fluid: FluidId::Water });
        }
        let result = piece.get_height_pos(BlockPos::new(0, 10, 0), &level, BlockPos::new(1, 10, 0));
        assert_eq!(result, 6);
    }

    #[test]
    fn piece_save_load_round_trip_and_processors() {
        let mut mgr = StructureTemplateManager::new();
        let template_id = Identifier::with_default_namespace("underwater_ruin/warm_1");
        mgr.register(&template_id, TemplateSize { x: 7, y: 5, z: 9 });

        let piece = OceanRuinPiece::new(
            &mgr,
            template_id.clone(),
            BlockPos::new(10, 20, 30),
            Rotation::Clockwise90,
            0.75f32,
            OceanRuinStructureType::Cold,
            true,
        );
        let mut tag = CompoundTag::new();
        piece.add_additional_save_data(&StructurePieceSerializationContext, &mut tag);
        let loaded = OceanRuinPiece::create(&mgr, &tag);

        assert_eq!(loaded.template_id, template_id);
        assert_eq!(loaded.template_position, BlockPos::new(10, 20, 30));
        assert_eq!(loaded.template_size, TemplateSize { x: 7, y: 5, z: 9 });
        assert_eq!(loaded.integrity.to_bits(), 0.75f32.to_bits());
        assert_eq!(loaded.biome_type, OceanRuinStructureType::Cold);
        assert!(loaded.is_large);
        assert_eq!(loaded.place_settings.rotation, Rotation::Clockwise90);
        assert_eq!(
            loaded.place_settings.processors[2],
            cold_suspicious_block_processor()
        );
    }
}
