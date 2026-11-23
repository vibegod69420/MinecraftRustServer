use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: &'static str,
    pub path: &'static str,
}

impl Identifier {
    pub const fn with_default_namespace(path: &'static str) -> Self {
        Self {
            namespace: "minecraft",
            path,
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn get_x_span(&self) -> i32 {
        self.max_x.wrapping_sub(self.min_x).wrapping_add(1)
    }

    pub fn get_y_span(&self) -> i32 {
        self.max_y.wrapping_sub(self.min_y).wrapping_add(1)
    }

    pub fn get_z_span(&self) -> i32 {
        self.max_z.wrapping_sub(self.min_z).wrapping_add(1)
    }

    pub fn get_center(&self) -> BlockPos {
        BlockPos::new(
            self.min_x.wrapping_add(self.get_x_span() / 2),
            self.min_y.wrapping_add(self.get_y_span() / 2),
            self.min_z.wrapping_add(self.get_z_span() / 2),
        )
    }

    pub fn is_inside(&self, pos: BlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }

    pub fn encapsulate(&mut self, other: BoundingBox) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.min_z = self.min_z.min(other.min_z);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
        self.max_z = self.max_z.max(other.max_z);
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    pub const LEGACY_CODEC: &'static str = "Rotation.LEGACY_CODEC";

    // JAVA: net.minecraft.world.level.block.Rotation#getRandom
    pub fn get_random(random: &mut dyn RandomSource) -> Rotation {
        match random.next_int(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
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
pub enum BlockKind {
    Air,
    DriedGhast,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: BlockKind,
    pub rotation: Rotation,
}

impl BlockState {
    pub fn is_air(&self) -> bool {
        self.block == BlockKind::Air
    }

    pub fn rotate(self, rot: Rotation) -> Self {
        Self {
            rotation: rot,
            ..self
        }
    }
}

pub struct Blocks;

impl Blocks {
    pub const DRIED_GHAST: BlockKind = BlockKind::DriedGhast;
}

impl BlockKind {
    pub fn default_block_state(self) -> BlockState {
        BlockState {
            block: self,
            rotation: Rotation::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureProcessor {
    BlockIgnoreStructureAndAir,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub processors: Vec<StructureProcessor>,
}

impl StructurePlaceSettings {
    pub fn new() -> Self {
        Self {
            rotation: Rotation::None,
            mirror: Mirror::None,
            processors: Vec::new(),
        }
    }

    pub fn set_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn set_mirror(mut self, mirror: Mirror) -> Self {
        self.mirror = mirror;
        self
    }

    pub fn add_processor(mut self, processor: StructureProcessor) -> Self {
        self.processors.push(processor);
        self
    }

    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }
}

impl Default for StructurePlaceSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompoundTag {
    rotations: HashMap<String, Rotation>,
    strings: HashMap<String, String>,
    ints: HashMap<String, i32>,
}

impl CompoundTag {
    pub fn store_rotation(&mut self, key: &str, _codec: &str, value: Rotation) {
        self.rotations.insert(key.to_string(), value);
    }

    pub fn read_rotation(&self, key: &str, _codec: &str) -> Option<Rotation> {
        self.rotations.get(key).copied()
    }

    pub fn put_string(&mut self, key: &str, value: String) {
        self.strings.insert(key.to_string(), value);
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }

    pub fn put_int(&mut self, key: &str, value: i32) {
        self.ints.insert(key.to_string(), value);
    }

    pub fn get_int(&self, key: &str) -> i32 {
        self.ints.get(key).copied().unwrap_or(0)
    }
}

pub struct StructurePieceSerializationContext;
pub struct StructureManager;
pub struct ChunkGenerator;

pub trait ServerLevelAccessor {}

pub trait WorldGenLevel: ServerLevelAccessor {
    fn get_seed(&self) -> i64;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    /// Collapses Java's `RandomSource.create(seed).forkPositional().at(pos)`.
    fn fork_positional_random_at(&self, seed: i64, pos: BlockPos) -> Box<dyn RandomSource>;
}

pub trait StructureTemplate {
    fn get_bounding_box(&self, settings: &StructurePlaceSettings, pos: BlockPos) -> BoundingBox;
    fn place_in_world(
        &self,
        level: &mut dyn WorldGenLevel,
        pos: BlockPos,
        pivot: BlockPos,
        settings: &StructurePlaceSettings,
        random: &mut dyn RandomSource,
        flags: i32,
    );
}

pub trait StructureTemplateManager {
    fn get_or_create(&self, id: &Identifier) -> Box<dyn StructureTemplate>;
}

pub trait StructurePieceAccessor {
    fn add_piece(&mut self, piece: NetherFossilPiece);
}

fn util_get_random<T: Copy>(arr: &[T], random: &mut dyn RandomSource) -> T {
    let idx = random.next_int(arr.len() as i32);
    arr[idx as usize]
}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces
// ---------------------------------------------------------------------------

static FOSSILS: [Identifier; 14] = [
    Identifier::with_default_namespace("nether_fossils/fossil_1"),
    Identifier::with_default_namespace("nether_fossils/fossil_2"),
    Identifier::with_default_namespace("nether_fossils/fossil_3"),
    Identifier::with_default_namespace("nether_fossils/fossil_4"),
    Identifier::with_default_namespace("nether_fossils/fossil_5"),
    Identifier::with_default_namespace("nether_fossils/fossil_6"),
    Identifier::with_default_namespace("nether_fossils/fossil_7"),
    Identifier::with_default_namespace("nether_fossils/fossil_8"),
    Identifier::with_default_namespace("nether_fossils/fossil_9"),
    Identifier::with_default_namespace("nether_fossils/fossil_10"),
    Identifier::with_default_namespace("nether_fossils/fossil_11"),
    Identifier::with_default_namespace("nether_fossils/fossil_12"),
    Identifier::with_default_namespace("nether_fossils/fossil_13"),
    Identifier::with_default_namespace("nether_fossils/fossil_14"),
];

// JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces#addPieces
pub fn add_pieces(
    template_manager: &dyn StructureTemplateManager,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    pos: BlockPos,
) {
    let rotation = Rotation::get_random(random);
    let fossil = util_get_random(&FOSSILS, random);
    pieces.add_piece(NetherFossilPiece::new(
        template_manager,
        fossil,
        pos,
        rotation,
    ));
}

pub struct NetherFossilPiece {
    pub template: Box<dyn StructureTemplate>,
    pub place_settings: StructurePlaceSettings,
    pub template_position: BlockPos,
    pub template_name: String,
}

impl NetherFossilPiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#<init>
    pub fn new(
        template_manager: &dyn StructureTemplateManager,
        location: Identifier,
        pos: BlockPos,
        rotation: Rotation,
    ) -> Self {
        let template_name = location.to_string();
        let place_settings = Self::make_settings(rotation);
        let template = template_manager.get_or_create(&location);
        Self {
            template,
            place_settings,
            template_position: pos,
            template_name,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#<init>
    pub fn from_nbt(template_manager: &dyn StructureTemplateManager, tag: &CompoundTag) -> Self {
        let rotation = tag
            .read_rotation("Rot", Rotation::LEGACY_CODEC)
            .expect("missing Rot");
        let template_name = tag
            .get_string("Template")
            .unwrap_or("minecraft:nether_fossils/fossil_1")
            .to_string();
        let template_position = BlockPos::new(tag.get_int("TPX"), tag.get_int("TPY"), tag.get_int("TPZ"));
        let location = parse_identifier(&template_name);
        let place_settings = Self::make_settings(rotation);
        let template = template_manager.get_or_create(&location);
        Self {
            template,
            place_settings,
            template_position,
            template_name,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#makeSettings
    fn make_settings(rotation: Rotation) -> StructurePlaceSettings {
        StructurePlaceSettings::new()
            .set_rotation(rotation)
            .set_mirror(Mirror::None)
            .add_processor(StructureProcessor::BlockIgnoreStructureAndAir)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut CompoundTag,
    ) {
        // super.addAdditionalSaveData(context, tag);
        tag.store_rotation("Rot", Rotation::LEGACY_CODEC, self.place_settings.get_rotation());
        tag.put_string("Template", self.template_name.clone());
        tag.put_int("TPX", self.template_position.x);
        tag.put_int("TPY", self.template_position.y);
        tag.put_int("TPZ", self.template_position.z);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#handleDataMarker
    pub fn handle_data_marker(
        &mut self,
        _name: &str,
        _pos: BlockPos,
        _level: &mut dyn ServerLevelAccessor,
        _random: &mut dyn RandomSource,
        _box: BoundingBox,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#postProcess
    pub fn post_process(
        &mut self,
        level: &mut dyn WorldGenLevel,
        structure_manager: &StructureManager,
        chunk_generator: &ChunkGenerator,
        random: &mut dyn RandomSource,
        bounding_box: &mut BoundingBox,
        chunk_pos: ChunkPos,
        pivot: BlockPos,
    ) {
        let template_bb = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);
        bounding_box.encapsulate(template_bb);
        // super.postProcess(...)
        self.template.place_in_world(
            level,
            self.template_position,
            pivot,
            &self.place_settings,
            random,
            2,
        );
        let _ = (structure_manager, chunk_generator, chunk_pos);
        self.place_dried_ghast(level, random, template_bb, *bounding_box);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#placeDriedGhast
    fn place_dried_ghast(
        &mut self,
        level: &mut dyn WorldGenLevel,
        _random: &mut dyn RandomSource,
        template_bb: BoundingBox,
        bounding_box: BoundingBox,
    ) {
        let mut pos_random =
            level.fork_positional_random_at(level.get_seed(), template_bb.get_center());
        if pos_random.next_float() < 0.5_f32 {
            let x = template_bb
                .min_x
                .wrapping_add(pos_random.next_int(template_bb.get_x_span()));
            let y = template_bb.min_y;
            let z = template_bb
                .min_z
                .wrapping_add(pos_random.next_int(template_bb.get_z_span()));
            let block_pos = BlockPos::new(x, y, z);
            if level.get_block_state(block_pos).is_air() && bounding_box.is_inside(block_pos) {
                let state = Blocks::DRIED_GHAST
                    .default_block_state()
                    .rotate(Rotation::get_random(pos_random.as_mut()));
                level.set_block(block_pos, state, 2);
            }
        }
    }
}

fn parse_identifier(s: &str) -> Identifier {
    // Minimal parser for "namespace:path" or "path"; returns a leaked/static-like
    // representation is not possible, so we map known fossils or fall back.
    // For this self-contained port, match against known fossils first.
    for &f in &FOSSILS {
        let full = format!("{}:{}", f.namespace, f.path);
        if s == full || s == f.path {
            return f;
        }
    }
    // Fallback: treat as default namespace fossil_1 to keep behavior deterministic.
    FOSSILS[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockRandom {
        ints: Vec<i32>,
        floats: Vec<f32>,
        int_idx: usize,
        float_idx: usize,
    }

    impl MockRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                floats,
                int_idx: 0,
                float_idx: 0,
            }
        }
    }

    impl RandomSource for MockRandom {
        fn next_float(&mut self) -> f32 {
            if self.floats.is_empty() {
                return 0.0;
            }
            let v = self.floats[self.float_idx % self.floats.len()];
            self.float_idx = self.float_idx.wrapping_add(1);
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            if self.ints.is_empty() {
                return 0;
            }
            let v = self.ints[self.int_idx % self.ints.len()];
            self.int_idx = self.int_idx.wrapping_add(1);
            ((v % bound) + bound) % bound
        }
    }

    struct MockTemplate {
        bb_offset: i32,
    }

    impl StructureTemplate for MockTemplate {
        fn get_bounding_box(&self, _settings: &StructurePlaceSettings, pos: BlockPos) -> BoundingBox {
            BoundingBox {
                min_x: pos.x,
                min_y: pos.y,
                min_z: pos.z,
                max_x: pos.x.wrapping_add(self.bb_offset),
                max_y: pos.y.wrapping_add(self.bb_offset),
                max_z: pos.z.wrapping_add(self.bb_offset),
            }
        }

        fn place_in_world(
            &self,
            _level: &mut dyn WorldGenLevel,
            _pos: BlockPos,
            _pivot: BlockPos,
            _settings: &StructurePlaceSettings,
            _random: &mut dyn RandomSource,
            _flags: i32,
        ) {
        }
    }

    struct MockTemplateManager;

    impl StructureTemplateManager for MockTemplateManager {
        fn get_or_create(&self, _id: &Identifier) -> Box<dyn StructureTemplate> {
            Box::new(MockTemplate { bb_offset: 4 })
        }
    }

    struct MockPieceAccessor {
        pieces: Vec<NetherFossilPiece>,
    }

    impl StructurePieceAccessor for MockPieceAccessor {
        fn add_piece(&mut self, piece: NetherFossilPiece) {
            self.pieces.push(piece);
        }
    }

    struct MockLevel {
        seed: i64,
        air_positions: HashSet<(i32, i32, i32)>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
        fork_random: MockRandom,
    }

    impl ServerLevelAccessor for MockLevel {}

    impl WorldGenLevel for MockLevel {
        fn get_seed(&self) -> i64 {
            self.seed
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            if self.air_positions.contains(&(pos.x, pos.y, pos.z)) {
                BlockKind::Air.default_block_state()
            } else {
                BlockState {
                    block: BlockKind::Other,
                    rotation: Rotation::None,
                }
            }
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.set_calls.push((pos, state, flags));
            true
        }

        fn fork_positional_random_at(&self, _seed: i64, _pos: BlockPos) -> Box<dyn RandomSource> {
            Box::new(MockRandom::new(
                self.fork_random.ints.clone(),
                self.fork_random.floats.clone(),
            ))
        }
    }

    #[test]
    fn make_settings_and_add_pieces_behavior() {
        let settings = NetherFossilPiece::make_settings(Rotation::Clockwise90);
        assert_eq!(settings.get_rotation(), Rotation::Clockwise90);
        assert_eq!(settings.mirror, Mirror::None);
        assert_eq!(
            settings.processors,
            vec![StructureProcessor::BlockIgnoreStructureAndAir]
        );

        let mgr = MockTemplateManager;
        let mut accessor = MockPieceAccessor { pieces: Vec::new() };
        // first next_int(4)=2 => Clockwise180, second next_int(14)=3 => fossil_4
        let mut random = MockRandom::new(vec![2, 3], vec![]);
        add_pieces(&mgr, &mut accessor, &mut random, BlockPos::new(10, 64, -5));

        assert_eq!(accessor.pieces.len(), 1);
        let piece = &accessor.pieces[0];
        assert_eq!(piece.place_settings.get_rotation(), Rotation::Clockwise180);
        assert_eq!(
            piece.template_name,
            "minecraft:nether_fossils/fossil_4"
        );
        assert_eq!(piece.template_position, BlockPos::new(10, 64, -5));
    }

    #[test]
    fn nbt_rot_round_trip_and_missing_rot_panics() {
        let mgr = MockTemplateManager;
        let piece = NetherFossilPiece::new(
            &mgr,
            Identifier::with_default_namespace("nether_fossils/fossil_7"),
            BlockPos::new(1, 2, 3),
            Rotation::Counterclockwise90,
        );
        let mut tag = CompoundTag::default();
        let ctx = StructurePieceSerializationContext;
        piece.add_additional_save_data(&ctx, &mut tag);

        let loaded = NetherFossilPiece::from_nbt(&mgr, &tag);
        assert_eq!(
            loaded.place_settings.get_rotation(),
            Rotation::Counterclockwise90
        );
        assert_eq!(loaded.template_name, "minecraft:nether_fossils/fossil_7");
        assert_eq!(loaded.template_position, BlockPos::new(1, 2, 3));

        let missing = std::panic::catch_unwind(|| {
            let mgr2 = MockTemplateManager;
            let empty = CompoundTag::default();
            let _ = NetherFossilPiece::from_nbt(&mgr2, &empty);
        });
        assert!(missing.is_err());
    }

    #[test]
    fn place_dried_ghast_float_boundary_and_placement() {
        let mgr = MockTemplateManager;
        let mut piece = NetherFossilPiece::new(
            &mgr,
            Identifier::with_default_namespace("nether_fossils/fossil_1"),
            BlockPos::new(0, 10, 0),
            Rotation::None,
        );
        let template_bb = BoundingBox {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 4,
            max_y: 14,
            max_z: 4,
        };
        let enclosing = template_bb;

        // Exactly 0.5f: condition is nextFloat() < 0.5F, so no placement.
        let mut level_no = MockLevel {
            seed: 1,
            air_positions: HashSet::from([(1, 10, 2)]),
            set_calls: Vec::new(),
            fork_random: MockRandom::new(vec![1, 2, 0], vec![0.5_f32]),
        };
        let mut dummy = MockRandom::new(vec![], vec![]);
        piece.place_dried_ghast(&mut level_no, &mut dummy, template_bb, enclosing);
        assert!(level_no.set_calls.is_empty());

        // Just below 0.5f: should place if air and inside.
        let just_below = f32::from_bits(0x3eff_ffff); // largest f32 < 0.5
        assert!(just_below < 0.5_f32);
        let mut level_yes = MockLevel {
            seed: 1,
            air_positions: HashSet::from([(1, 10, 2)]),
            set_calls: Vec::new(),
            // next_float -> just_below, next_int(xSpan=5)->1, next_int(zSpan=5)->2, next_int(4)->1 (rotation)
            fork_random: MockRandom::new(vec![1, 2, 1], vec![just_below]),
        };
        piece.place_dried_ghast(&mut level_yes, &mut dummy, template_bb, enclosing);
        assert_eq!(level_yes.set_calls.len(), 1);
        let (pos, state, flags) = level_yes.set_calls[0];
        assert_eq!(pos, BlockPos::new(1, 10, 2));
        assert_eq!(state.block, BlockKind::DriedGhast);
        assert_eq!(state.rotation, Rotation::Clockwise90);
        assert_eq!(flags, 2);
    }

    #[test]
    fn place_dried_ghast_skips_when_not_air_or_outside() {
        let mgr = MockTemplateManager;
        let mut piece = NetherFossilPiece::new(
            &mgr,
            Identifier::with_default_namespace("nether_fossils/fossil_1"),
            BlockPos::new(0, 10, 0),
            Rotation::None,
        );
        let template_bb = BoundingBox {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 4,
            max_y: 14,
            max_z: 4,
        };
        let mut dummy = MockRandom::new(vec![], vec![]);

        // Not air
        let mut level_not_air = MockLevel {
            seed: 7,
            air_positions: HashSet::new(),
            set_calls: Vec::new(),
            fork_random: MockRandom::new(vec![0, 0, 0], vec![0.0]),
        };
        piece.place_dried_ghast(&mut level_not_air, &mut dummy, template_bb, template_bb);
        assert!(level_not_air.set_calls.is_empty());

        // Air but outside enclosing bounding box
        let mut level_outside = MockLevel {
            seed: 7,
            air_positions: HashSet::from([(0, 10, 0)]),
            set_calls: Vec::new(),
            fork_random: MockRandom::new(vec![0, 0, 0], vec![0.0]),
        };
        let small_enclosing = BoundingBox {
            min_x: 1,
            min_y: 10,
            min_z: 1,
            max_x: 4,
            max_y: 14,
            max_z: 4,
        };
        piece.place_dried_ghast(&mut level_outside, &mut dummy, template_bb, small_enclosing);
        assert!(level_outside.set_calls.is_empty());
    }

    #[test]
    fn bounding_box_negative_and_wrapping_semantics() {
        let bb = BoundingBox {
            min_x: -10,
            min_y: -20,
            min_z: -30,
            max_x: -6,
            max_y: -15,
            max_z: -21,
        };
        assert_eq!(bb.get_x_span(), 5);
        assert_eq!(bb.get_y_span(), 6);
        assert_eq!(bb.get_z_span(), 10);
        let c = bb.get_center();
        assert_eq!(c, BlockPos::new(-10 + 2, -20 + 3, -30 + 5));

        let mut a = BoundingBox {
            min_x: -1,
            min_y: -1,
            min_z: -1,
            max_x: 1,
            max_y: 1,
            max_z: 1,
        };
        let b = BoundingBox {
            min_x: -5,
            min_y: 0,
            min_z: 2,
            max_x: 3,
            max_y: 4,
            max_z: 2,
        };
        a.encapsulate(b);
        assert_eq!(
            a,
            BoundingBox {
                min_x: -5,
                min_y: -1,
                min_z: -1,
                max_x: 3,
                max_y: 4,
                max_z: 2,
            }
        );

        // Wrapping span/center computation for extreme int values.
        let wrap_bb = BoundingBox {
            min_x: i32::MAX,
            min_y: 0,
            min_z: i32::MIN,
            max_x: i32::MIN,
            max_y: 0,
            max_z: i32::MIN,
        };
        assert_eq!(wrap_bb.get_x_span(), i32::MIN.wrapping_sub(i32::MAX).wrapping_add(1));
        assert_eq!(wrap_bb.get_z_span(), 1);
        let wc = wrap_bb.get_center();
        assert_eq!(wc.z, i32::MIN.wrapping_add(0));
    }

    #[test]
    fn post_process_encapsulates_and_places_dried_ghast() {
        let mgr = MockTemplateManager;
        let mut piece = NetherFossilPiece::new(
            &mgr,
            Identifier::with_default_namespace("nether_fossils/fossil_2"),
            BlockPos::new(5, 20, -3),
            Rotation::None,
        );

        let mut level = MockLevel {
            seed: 123,
            air_positions: HashSet::from([(6, 20, -2)]),
            set_calls: Vec::new(),
            fork_random: MockRandom::new(vec![1, 1, 2], vec![0.1_f32]),
        };
        let mut random = MockRandom::new(vec![], vec![]);
        let mut bb = BoundingBox {
            min_x: 100,
            min_y: 100,
            min_z: 100,
            max_x: 101,
            max_y: 101,
            max_z: 101,
        };
        let sm = StructureManager;
        let cg = ChunkGenerator;

        piece.post_process(
            &mut level,
            &sm,
            &cg,
            &mut random,
            &mut bb,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 0, 0),
        );

        // Encapsulated with template bbox at (5,20,-3) span offset 4 => max (9,24,1)
        assert_eq!(bb.min_x, 5);
        assert_eq!(bb.min_y, 20);
        assert_eq!(bb.min_z, -3);
        assert_eq!(bb.max_x, 101);
        assert_eq!(bb.max_y, 101);
        assert_eq!(bb.max_z, 101);

        assert_eq!(level.set_calls.len(), 1);
        assert_eq!(level.set_calls[0].0, BlockPos::new(6, 20, -2));
        assert_eq!(level.set_calls[0].1.block, BlockKind::DriedGhast);
        assert_eq!(level.set_calls[0].1.rotation, Rotation::Clockwise180);
        assert_eq!(level.set_calls[0].2, 2);
    }
}
