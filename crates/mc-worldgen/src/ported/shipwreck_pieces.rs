use std::collections::HashMap;
use std::sync::OnceLock;

pub const NUMBER_OF_BLOCKS_ALLOWED_IN_WORLD_GEN_REGION: i32 = 32;
pub const PIVOT: BlockPos = BlockPos { x: 4, y: 0, z: 15 };

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self(format!("minecraft:{path}"))
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: Identifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
    let min_x = from.x.min(to.x);
    let min_y = from.y.min(to.y);
    let min_z = from.z.min(to.z);
    let max_x = from.x.max(to.x);
    let max_y = from.y.max(to.y);
    let max_z = from.z.max(to.z);
    let mut out = Vec::new();
    let mut x = min_x;
    while x <= max_x {
        let mut y = min_y;
        while y <= max_y {
            let mut z = min_z;
            while z <= max_z {
                out.push(BlockPos::new(x, y, z));
                if z == max_z {
                    break;
                }
                z = z.wrapping_add(1);
            }
            if y == max_y {
                break;
            }
            y = y.wrapping_add(1);
        }
        if x == max_x {
            break;
        }
        x = x.wrapping_add(1);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirror {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurfaceWg,
    OceanFloorWg,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundingBox;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait StructureTemplateManager {
    fn get_template_size(&self, id: &Identifier) -> Vec3i;
}

pub trait StructurePieceAccessor<P> {
    fn add_piece(&mut self, piece: P);
}

pub trait WorldGenLevel {
    fn get_max_y(&self) -> i32;
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
}

pub trait ServerLevelAccessor {
    fn set_block_entity_loot_table(
        &mut self,
        random: &mut dyn RandomSource,
        pos: BlockPos,
        loot_table: &ResourceKey,
    );
}

pub trait StructureManager {}
pub trait ChunkGenerator {}

pub struct StructurePieceSerializationContext;

pub trait CompoundTagWriter {
    fn put_boolean(&mut self, key: &str, value: bool);
    fn store_rotation_legacy(&mut self, key: &str, rotation: Rotation);
}

pub trait CompoundTagReader {
    fn get_boolean_or(&self, key: &str, default: bool) -> bool;
    fn read_rotation_legacy(&self, key: &str) -> Option<Rotation>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub rotation_pivot: BlockPos,
    pub ignore_structure_and_air: bool,
}

impl StructurePlaceSettings {
    pub fn set_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn set_mirror(mut self, mirror: Mirror) -> Self {
        self.mirror = mirror;
        self
    }

    pub fn set_rotation_pivot(mut self, pivot: BlockPos) -> Self {
        self.rotation_pivot = pivot;
        self
    }

    pub fn add_processor_ignore_structure_and_air(mut self) -> Self {
        self.ignore_structure_and_air = true;
        self
    }

    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }
}

pub fn structure_location_beached() -> &'static [Identifier] {
    static V: OnceLock<Vec<Identifier>> = OnceLock::new();
    V.get_or_init(|| {
        vec![
            Identifier::with_default_namespace("shipwreck/with_mast"),
            Identifier::with_default_namespace("shipwreck/sideways_full"),
            Identifier::with_default_namespace("shipwreck/sideways_fronthalf"),
            Identifier::with_default_namespace("shipwreck/sideways_backhalf"),
            Identifier::with_default_namespace("shipwreck/rightsideup_full"),
            Identifier::with_default_namespace("shipwreck/rightsideup_fronthalf"),
            Identifier::with_default_namespace("shipwreck/rightsideup_backhalf"),
            Identifier::with_default_namespace("shipwreck/with_mast_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_full_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_fronthalf_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_backhalf_degraded"),
        ]
    })
}

pub fn structure_location_ocean() -> &'static [Identifier] {
    static V: OnceLock<Vec<Identifier>> = OnceLock::new();
    V.get_or_init(|| {
        vec![
            Identifier::with_default_namespace("shipwreck/with_mast"),
            Identifier::with_default_namespace("shipwreck/upsidedown_full"),
            Identifier::with_default_namespace("shipwreck/upsidedown_fronthalf"),
            Identifier::with_default_namespace("shipwreck/upsidedown_backhalf"),
            Identifier::with_default_namespace("shipwreck/sideways_full"),
            Identifier::with_default_namespace("shipwreck/sideways_fronthalf"),
            Identifier::with_default_namespace("shipwreck/sideways_backhalf"),
            Identifier::with_default_namespace("shipwreck/rightsideup_full"),
            Identifier::with_default_namespace("shipwreck/rightsideup_fronthalf"),
            Identifier::with_default_namespace("shipwreck/rightsideup_backhalf"),
            Identifier::with_default_namespace("shipwreck/with_mast_degraded"),
            Identifier::with_default_namespace("shipwreck/upsidedown_full_degraded"),
            Identifier::with_default_namespace("shipwreck/upsidedown_fronthalf_degraded"),
            Identifier::with_default_namespace("shipwreck/upsidedown_backhalf_degraded"),
            Identifier::with_default_namespace("shipwreck/sideways_full_degraded"),
            Identifier::with_default_namespace("shipwreck/sideways_fronthalf_degraded"),
            Identifier::with_default_namespace("shipwreck/sideways_backhalf_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_full_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_fronthalf_degraded"),
            Identifier::with_default_namespace("shipwreck/rightsideup_backhalf_degraded"),
        ]
    })
}

pub fn markers_to_loot() -> &'static HashMap<String, ResourceKey> {
    static V: OnceLock<HashMap<String, ResourceKey>> = OnceLock::new();
    V.get_or_init(|| {
        HashMap::from([
            (
                "map_chest".to_string(),
                ResourceKey {
                    location: Identifier::with_default_namespace("chests/shipwreck_map"),
                },
            ),
            (
                "treasure_chest".to_string(),
                ResourceKey {
                    location: Identifier::with_default_namespace("chests/shipwreck_treasure"),
                },
            ),
            (
                "supply_chest".to_string(),
                ResourceKey {
                    location: Identifier::with_default_namespace("chests/shipwreck_supply"),
                },
            ),
        ])
    })
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces#addRandomPiece
pub fn add_random_piece<M, A>(
    structure_template_manager: &M,
    pos: BlockPos,
    rotation: Rotation,
    piece_accessor: &mut A,
    random: &mut dyn RandomSource,
    is_beached: bool,
) -> ShipwreckPiece
where
    M: StructureTemplateManager,
    A: StructurePieceAccessor<ShipwreckPiece>,
{
    let locations = if is_beached {
        structure_location_beached()
    } else {
        structure_location_ocean()
    };
    let idx = random.next_int(locations.len() as i32) as usize;
    let id = locations[idx].clone();
    let piece = ShipwreckPiece::new(structure_template_manager, id, pos, rotation, is_beached);
    piece_accessor.add_piece(piece.clone());
    piece
}

#[derive(Clone, Debug)]
pub struct ShipwreckPiece {
    pub template_name: Identifier,
    pub template_position: BlockPos,
    pub place_settings: StructurePlaceSettings,
    pub template_size: Vec3i,
    pub is_beached: bool,
}

impl ShipwreckPiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#<init>
    pub fn new(
        structure_template_manager: &dyn StructureTemplateManager,
        template_name: Identifier,
        pos: BlockPos,
        rotation: Rotation,
        is_beached: bool,
    ) -> Self {
        let template_size = structure_template_manager.get_template_size(&template_name);
        let place_settings = Self::make_settings(rotation);
        Self {
            template_name,
            template_position: pos,
            place_settings,
            template_size,
            is_beached,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#<init>
    pub fn from_tag(
        structure_template_manager: &dyn StructureTemplateManager,
        tag: &dyn CompoundTagReader,
        template_name: Identifier,
    ) -> Self {
        let rotation = tag
            .read_rotation_legacy("Rot")
            .expect("missing Rot in shipwreck piece tag");
        let place_settings = Self::make_settings(rotation);
        let template_size = structure_template_manager.get_template_size(&template_name);
        let is_beached = tag.get_boolean_or("isBeached", false);
        Self {
            template_name,
            template_position: BlockPos::new(0, 0, 0),
            place_settings,
            template_size,
            is_beached,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut dyn CompoundTagWriter,
    ) {
        tag.put_boolean("isBeached", self.is_beached);
        tag.store_rotation_legacy("Rot", self.place_settings.get_rotation());
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#makeSettings
    fn make_settings(rotation: Rotation) -> StructurePlaceSettings {
        StructurePlaceSettings {
            rotation,
            mirror: Mirror::None,
            rotation_pivot: PIVOT,
            ignore_structure_and_air: false,
        }
        .set_rotation(rotation)
        .set_mirror(Mirror::None)
        .set_rotation_pivot(PIVOT)
        .add_processor_ignore_structure_and_air()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#handleDataMarker
    pub fn handle_data_marker(
        &self,
        marker: &str,
        pos: BlockPos,
        level: &mut dyn ServerLevelAccessor,
        random: &mut dyn RandomSource,
        _bounding_box: &BoundingBox,
    ) {
        if let Some(loot_table) = markers_to_loot().get(marker) {
            level.set_block_entity_loot_table(random, pos.below(), loot_table);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#postProcess
    pub fn post_process(
        &mut self,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        bounding_box: &BoundingBox,
        chunk_pos: &ChunkPos,
        pos: &BlockPos,
        super_post_process: &mut dyn FnMut(
            &mut dyn WorldGenLevel,
            &dyn StructureManager,
            &dyn ChunkGenerator,
            &mut dyn RandomSource,
            &BoundingBox,
            &ChunkPos,
            &BlockPos,
        ),
    ) {
        if self.is_too_big_to_fit_in_world_gen_region() {
            super_post_process(
                level,
                structure_manager,
                chunk_generator,
                random,
                bounding_box,
                chunk_pos,
                pos,
            );
        } else {
            let mut max_y_plus_1: i32 = level.get_max_y().wrapping_add(1);
            let mut sum_height: i32 = 0;
            let size = self.template_size;
            let heightmap = if self.is_beached {
                HeightmapType::WorldSurfaceWg
            } else {
                HeightmapType::OceanFloorWg
            };
            let area: i32 = size.get_x().wrapping_mul(size.get_z());
            if area == 0 {
                sum_height = level.get_height(
                    heightmap,
                    self.template_position.x,
                    self.template_position.z,
                );
            } else {
                let corner = self.template_position.offset(
                    size.get_x().wrapping_sub(1),
                    0,
                    size.get_z().wrapping_sub(1),
                );
                for p in between_closed(self.template_position, corner) {
                    let h = level.get_height(heightmap, p.x, p.z);
                    sum_height = sum_height.wrapping_add(h);
                    max_y_plus_1 = max_y_plus_1.min(h);
                }
                sum_height /= area;
            }

            let target_y = if self.is_beached {
                self.calculate_beached_position(max_y_plus_1, random)
            } else {
                sum_height
            };
            self.adjust_position_height(target_y);
            super_post_process(
                level,
                structure_manager,
                chunk_generator,
                random,
                bounding_box,
                chunk_pos,
                pos,
            );
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#isTooBigToFitInWorldGenRegion
    pub fn is_too_big_to_fit_in_world_gen_region(&self) -> bool {
        let size = self.template_size;
        size.get_x() > 32 || size.get_y() > 32
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#calculateBeachedPosition
    pub fn calculate_beached_position(&self, y: i32, random: &mut dyn RandomSource) -> i32 {
        y.wrapping_sub(self.template_size.get_y() / 2)
            .wrapping_sub(random.next_int(3))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece#adjustPositionHeight
    pub fn adjust_position_height(&mut self, y: i32) {
        self.template_position = BlockPos::new(
            self.template_position.x,
            y,
            self.template_position.z,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.0.rem_euclid(bound)
            }
        }
    }

    struct SeqRandom {
        vals: Vec<i32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(vals: Vec<i32>) -> Self {
            Self { vals, idx: 0 }
        }
    }
    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.vals.len() {
                let v = self.vals[self.idx];
                self.idx += 1;
                v
            } else {
                0
            };
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    struct MockManager {
        size: Vec3i,
    }
    impl StructureTemplateManager for MockManager {
        fn get_template_size(&self, _id: &Identifier) -> Vec3i {
            self.size
        }
    }

    struct MockAccessor {
        pieces: Vec<ShipwreckPiece>,
    }
    impl StructurePieceAccessor<ShipwreckPiece> for MockAccessor {
        fn add_piece(&mut self, piece: ShipwreckPiece) {
            self.pieces.push(piece);
        }
    }

    struct MockLevel {
        max_y: i32,
        heights: HashMap<(i32, i32), i32>,
        default_height: i32,
    }
    impl MockLevel {
        fn new(max_y: i32, default_height: i32) -> Self {
            Self {
                max_y,
                heights: HashMap::new(),
                default_height,
            }
        }
        fn set_height(&mut self, x: i32, z: i32, h: i32) {
            self.heights.insert((x, z), h);
        }
    }
    impl WorldGenLevel for MockLevel {
        fn get_max_y(&self) -> i32 {
            self.max_y
        }
        fn get_height(&self, _heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            self.heights
                .get(&(x, z))
                .copied()
                .unwrap_or(self.default_height)
        }
    }

    struct MockServerLevel {
        loot_calls: Vec<(BlockPos, Identifier)>,
    }
    impl MockServerLevel {
        fn new() -> Self {
            Self {
                loot_calls: Vec::new(),
            }
        }
    }
    impl ServerLevelAccessor for MockServerLevel {
        fn set_block_entity_loot_table(
            &mut self,
            _random: &mut dyn RandomSource,
            pos: BlockPos,
            loot_table: &ResourceKey,
        ) {
            self.loot_calls.push((pos, loot_table.location.clone()));
        }
    }

    struct NoopSM;
    impl StructureManager for NoopSM {}
    struct NoopCG;
    impl ChunkGenerator for NoopCG {}

    #[derive(Default)]
    struct MockTagWriter {
        bools: HashMap<String, bool>,
        rotations: HashMap<String, Rotation>,
    }
    impl CompoundTagWriter for MockTagWriter {
        fn put_boolean(&mut self, key: &str, value: bool) {
            self.bools.insert(key.to_string(), value);
        }
        fn store_rotation_legacy(&mut self, key: &str, rotation: Rotation) {
            self.rotations.insert(key.to_string(), rotation);
        }
    }

    struct MockTagReader {
        bools: HashMap<String, bool>,
        rotations: HashMap<String, Rotation>,
    }
    impl MockTagReader {
        fn new() -> Self {
            Self {
                bools: HashMap::new(),
                rotations: HashMap::new(),
            }
        }
    }
    impl CompoundTagReader for MockTagReader {
        fn get_boolean_or(&self, key: &str, default: bool) -> bool {
            self.bools.get(key).copied().unwrap_or(default)
        }
        fn read_rotation_legacy(&self, key: &str) -> Option<Rotation> {
            self.rotations.get(key).copied()
        }
    }

    fn default_piece(is_beached: bool, size: Vec3i) -> ShipwreckPiece {
        let mgr = MockManager { size };
        ShipwreckPiece::new(
            &mgr,
            Identifier::with_default_namespace("shipwreck/test"),
            BlockPos::new(10, 0, 20),
            Rotation::None,
            is_beached,
        )
    }

    #[test]
    fn test_add_random_piece_selects_from_correct_pool_and_adds() {
        let mgr = MockManager {
            size: Vec3i::new(5, 5, 5),
        };
        let mut acc = MockAccessor { pieces: Vec::new() };
        let mut rng = FixedRandom(0);
        let piece = add_random_piece(
            &mgr,
            BlockPos::new(1, 2, 3),
            Rotation::Clockwise90,
            &mut acc,
            &mut rng,
            true,
        );
        assert_eq!(acc.pieces.len(), 1);
        assert_eq!(piece.template_name, structure_location_beached()[0]);
        assert!(piece.is_beached);
        assert_eq!(piece.place_settings.rotation, Rotation::Clockwise90);

        let mut acc2 = MockAccessor { pieces: Vec::new() };
        let mut rng2 = FixedRandom(1);
        let piece2 = add_random_piece(
            &mgr,
            BlockPos::new(0, 0, 0),
            Rotation::None,
            &mut acc2,
            &mut rng2,
            false,
        );
        assert_eq!(piece2.template_name, structure_location_ocean()[1]);
        assert!(!piece2.is_beached);
    }

    #[test]
    fn test_is_too_big_to_fit_in_world_gen_region() {
        let p1 = default_piece(false, Vec3i::new(32, 32, 10));
        assert!(!p1.is_too_big_to_fit_in_world_gen_region());
        let p2 = default_piece(false, Vec3i::new(33, 10, 10));
        assert!(p2.is_too_big_to_fit_in_world_gen_region());
        let p3 = default_piece(false, Vec3i::new(10, 33, 10));
        assert!(p3.is_too_big_to_fit_in_world_gen_region());
    }

    #[test]
    fn test_calculate_beached_position_and_negative_overflow() {
        let mut piece = default_piece(true, Vec3i::new(5, 7, 5));
        let mut rng = FixedRandom(2); // next_int(3) -> 2
        // y - (size.y/2) - rand = 10 - 3 - 2 = 5
        assert_eq!(piece.calculate_beached_position(10, &mut rng), 5);

        // negative / wrapping case
        piece.template_size = Vec3i::new(1, i32::MAX, 1);
        let mut rng2 = FixedRandom(0);
        let y = i32::MIN;
        let expected = y
            .wrapping_sub(i32::MAX / 2)
            .wrapping_sub(0);
        assert_eq!(piece.calculate_beached_position(y, &mut rng2), expected);
    }

    #[test]
    fn test_post_process_averages_height_and_adjusts_position() {
        let mut piece = default_piece(false, Vec3i::new(2, 5, 2));
        piece.template_position = BlockPos::new(0, 0, 0);
        let mut level = MockLevel::new(100, 0);
        level.set_height(0, 0, 10);
        level.set_height(1, 0, 20);
        level.set_height(0, 1, 30);
        level.set_height(1, 1, 40);
        let mut rng = FixedRandom(0);
        let mut super_called = false;
        piece.post_process(
            &mut level,
            &NoopSM,
            &NoopCG,
            &mut rng,
            &BoundingBox,
            &ChunkPos::default(),
            &BlockPos::new(0, 0, 0),
            &mut |_, _, _, _, _, _, _| {
                super_called = true;
            },
        );
        // average = (10+20+30+40)/4 = 25
        assert_eq!(piece.template_position.y, 25);
        assert!(super_called);
    }

    #[test]
    fn test_post_process_beached_uses_min_height_and_random_offset() {
        let mut piece = default_piece(true, Vec3i::new(2, 6, 1));
        piece.template_position = BlockPos::new(5, 0, 7);
        let mut level = MockLevel::new(64, 0);
        level.set_height(5, 7, 50);
        level.set_height(6, 7, 40);
        let mut rng = FixedRandom(1); // next_int(3)->1
        piece.post_process(
            &mut level,
            &NoopSM,
            &NoopCG,
            &mut rng,
            &BoundingBox,
            &ChunkPos::default(),
            &BlockPos::new(0, 0, 0),
            &mut |_, _, _, _, _, _, _| {},
        );
        // min height = 40, beached = 40 - (6/2) - 1 = 36
        assert_eq!(piece.template_position, BlockPos::new(5, 36, 7));
    }

    #[test]
    fn test_handle_data_marker_sets_loot_below() {
        let piece = default_piece(false, Vec3i::new(1, 1, 1));
        let mut level = MockServerLevel::new();
        let mut rng = FixedRandom(0);
        piece.handle_data_marker(
            "map_chest",
            BlockPos::new(1, 5, 2),
            &mut level,
            &mut rng,
            &BoundingBox,
        );
        assert_eq!(level.loot_calls.len(), 1);
        assert_eq!(level.loot_calls[0].0, BlockPos::new(1, 4, 2));
        assert_eq!(
            level.loot_calls[0].1,
            Identifier::with_default_namespace("chests/shipwreck_map")
        );

        piece.handle_data_marker(
            "unknown",
            BlockPos::new(0, 0, 0),
            &mut level,
            &mut rng,
            &BoundingBox,
        );
        assert_eq!(level.loot_calls.len(), 1);
    }
}
