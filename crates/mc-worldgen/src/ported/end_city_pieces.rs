use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_GEN_DEPTH: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

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
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    pub fn get_rotated(self, other: Rotation) -> Rotation {
        Self::from_ordinal((self.ordinal().wrapping_add(other.ordinal())) & 3)
    }

    pub fn rotate(self, dir: Direction) -> Direction {
        match self {
            Rotation::None => dir,
            Rotation::Clockwise90 => match dir {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
                _ => dir,
            },
            Rotation::Clockwise180 => match dir {
                Direction::North => Direction::South,
                Direction::East => Direction::West,
                Direction::South => Direction::North,
                Direction::West => Direction::East,
                _ => dir,
            },
            Rotation::Counterclockwise90 => match dir {
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
                Direction::East => Direction::North,
                _ => dir,
            },
        }
    }

    pub fn ordinal(self) -> i32 {
        match self {
            Rotation::None => 0,
            Rotation::Clockwise90 => 1,
            Rotation::Clockwise180 => 2,
            Rotation::Counterclockwise90 => 3,
        }
    }

    pub fn from_ordinal(v: i32) -> Self {
        match v & 3 {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Rotation::None => "none",
            Rotation::Clockwise90 => "clockwise_90",
            Rotation::Clockwise180 => "180",
            Rotation::Counterclockwise90 => "counterclockwise_90",
        }
    }

    fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Some(Rotation::None),
            "clockwise_90" => Some(Rotation::Clockwise90),
            "180" | "clockwise_180" => Some(Rotation::Clockwise180),
            "counterclockwise_90" => Some(Rotation::Counterclockwise90),
            _ => None,
        }
    }
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
    pub const fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn is_inside(&self, pos: BlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
    }

    pub fn moved(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            min_x: self.min_x.wrapping_add(x),
            min_y: self.min_y.wrapping_add(y),
            min_z: self.min_z.wrapping_add(z),
            max_x: self.max_x.wrapping_add(x),
            max_y: self.max_y.wrapping_add(y),
            max_z: self.max_z.wrapping_add(z),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompoundTag {
    bools: HashMap<String, bool>,
    strings: HashMap<String, String>,
    ints: HashMap<String, i32>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_boolean(&mut self, key: &str, value: bool) {
        self.bools.insert(key.to_string(), value);
    }

    pub fn get_boolean_or(&self, key: &str, default: bool) -> bool {
        self.bools.get(key).copied().unwrap_or(default)
    }

    pub fn put_string(&mut self, key: &str, value: &str) {
        self.strings.insert(key.to_string(), value.to_string());
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }

    pub fn put_int(&mut self, key: &str, value: i32) {
        self.ints.insert(key.to_string(), value);
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        self.ints.get(key).copied().unwrap_or(default)
    }

    pub fn put_rotation(&mut self, key: &str, rot: Rotation) {
        self.put_string(key, rot.name());
    }

    pub fn get_rotation(&self, key: &str) -> Option<Rotation> {
        self.get_string(key).and_then(Rotation::from_name)
    }
}

#[derive(Clone, Debug)]
pub struct StructurePlaceSettings {
    pub ignore_entities: bool,
    pub overwrite: bool,
    pub rotation: Rotation,
}

#[derive(Clone, Debug, Default)]
pub struct StructurePieceSerializationContext;

pub trait StructureTemplate {
    fn calculate_connected_position(
        &self,
        self_settings: &StructurePlaceSettings,
        self_offset: BlockPos,
        other_settings: &StructurePlaceSettings,
        other_offset: BlockPos,
    ) -> BlockPos;

    fn get_bounding_box(&self, settings: &StructurePlaceSettings, pos: BlockPos) -> BoundingBox;
}

pub trait StructureTemplateManager {
    fn get_or_create(&self, location: &Identifier) -> Box<dyn StructureTemplate>;
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_int_unbounded(&mut self) -> i32;
    fn next_boolean(&mut self) -> bool;
}

pub trait ServerLevelAccessor {
    fn set_block_entity_loot_table(
        &mut self,
        pos: BlockPos,
        random: &mut dyn RandomSource,
        loot_table: &str,
    );
    fn is_in_spawnable_bounds(&self, pos: BlockPos) -> bool;
    fn add_shulker(&mut self, x: f64, y: f64, z: f64);
    fn add_elytra_item_frame(&mut self, pos: BlockPos, rotation: Direction);
}

pub struct EndCityPiece {
    pub template_name: String,
    pub template_position: BlockPos,
    pub place_settings: StructurePlaceSettings,
    pub gen_depth: i32,
    pub bounding_box: BoundingBox,
    template: Box<dyn StructureTemplate>,
}

pub type PieceRef = Rc<RefCell<EndCityPiece>>;

impl EndCityPiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#<init>
    pub fn new(
        template_manager: &dyn StructureTemplateManager,
        name: &str,
        pos: BlockPos,
        rotation: Rotation,
        overwrite: bool,
    ) -> Self {
        let settings = make_settings(overwrite, rotation);
        let loc = make_identifier(name);
        let template = template_manager.get_or_create(&loc);
        let bounding_box = template.get_bounding_box(&settings, pos);
        Self {
            template_name: name.to_string(),
            template_position: pos,
            place_settings: settings,
            gen_depth: 0,
            bounding_box,
            template,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#<init>
    pub fn from_tag(template_manager: &dyn StructureTemplateManager, tag: &CompoundTag) -> Self {
        let overwrite = tag.get_boolean_or("OW", false);
        let rotation = tag.get_rotation("Rot").unwrap_or(Rotation::None);
        let template_name = tag.get_string("Template").unwrap_or("base_floor");
        let pos = BlockPos::new(
            tag.get_int_or("TPX", 0),
            tag.get_int_or("TPY", 0),
            tag.get_int_or("TPZ", 0),
        );
        Self::new(template_manager, template_name, pos, rotation, overwrite)
    }

    pub fn template_position(&self) -> BlockPos {
        self.template_position
    }

    pub fn place_settings(&self) -> &StructurePlaceSettings {
        &self.place_settings
    }

    pub fn template(&self) -> &dyn StructureTemplate {
        &*self.template
    }

    pub fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.template_position = BlockPos::new(
            self.template_position.x.wrapping_add(x),
            self.template_position.y.wrapping_add(y),
            self.template_position.z.wrapping_add(z),
        );
        self.bounding_box = self.bounding_box.moved(x, y, z);
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    pub fn get_gen_depth(&self) -> i32 {
        self.gen_depth
    }

    pub fn set_gen_depth(&mut self, depth: i32) {
        self.gen_depth = depth;
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#makeTemplateLocation
    pub fn make_template_location(&self) -> Identifier {
        make_identifier(&self.template_name)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut CompoundTag,
    ) {
        // super.addAdditionalSaveData(context, tag);
        tag.put_rotation("Rot", self.place_settings.rotation);
        tag.put_boolean("OW", self.place_settings.overwrite);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#handleDataMarker
    pub fn handle_data_marker(
        &self,
        name: &str,
        pos: BlockPos,
        level: &mut dyn ServerLevelAccessor,
        random: &mut dyn RandomSource,
        bb: &BoundingBox,
    ) {
        if name.starts_with("Chest") {
            let below = pos.below();
            if bb.is_inside(below) {
                level.set_block_entity_loot_table(
                    below,
                    random,
                    "minecraft:chests/end_city_treasure",
                );
            }
        } else if bb.is_inside(pos) && level.is_in_spawnable_bounds(pos) {
            if name.starts_with("Sentry") {
                let x = pos.get_x() as f64 + 0.5;
                let y = pos.get_y() as f64;
                let z = pos.get_z() as f64 + 0.5;
                level.add_shulker(x, y, z);
            } else if name.starts_with("Elytra") {
                let frame_rot = self.place_settings.rotation.rotate(Direction::South);
                level.add_elytra_item_frame(pos, frame_rot);
            }
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#makeSettings
fn make_settings(overwrite: bool, rotation: Rotation) -> StructurePlaceSettings {
    StructurePlaceSettings {
        ignore_entities: true,
        overwrite,
        rotation,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$EndCityPiece#makeIdentifier
fn make_identifier(name: &str) -> Identifier {
    Identifier::with_default_namespace(&format!("end_city/{name}"))
}

pub fn find_collision_piece(pieces: &[PieceRef], bb: BoundingBox) -> Option<PieceRef> {
    for p in pieces {
        if p.borrow().bounding_box.intersects(&bb) {
            return Some(Rc::clone(p));
        }
    }
    None
}

pub trait SectionGenerator {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#init
    fn init(&self);
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#generate
    fn generate(
        &self,
        template_manager: &dyn StructureTemplateManager,
        depth: i32,
        piece: &PieceRef,
        offset: Option<BlockPos>,
        pieces: &mut Vec<PieceRef>,
        random: &mut dyn RandomSource,
    ) -> bool;
}

pub static TOWER_BRIDGES: [(Rotation, BlockPos); 4] = [
    (Rotation::None, BlockPos::new(1, -1, 0)),
    (Rotation::Clockwise90, BlockPos::new(6, -1, 1)),
    (Rotation::Counterclockwise90, BlockPos::new(0, -1, 5)),
    (Rotation::Clockwise180, BlockPos::new(5, -1, 6)),
];

pub static FAT_TOWER_BRIDGES: [(Rotation, BlockPos); 4] = [
    (Rotation::None, BlockPos::new(4, -1, 0)),
    (Rotation::Clockwise90, BlockPos::new(12, -1, 4)),
    (Rotation::Counterclockwise90, BlockPos::new(0, -1, 8)),
    (Rotation::Clockwise180, BlockPos::new(8, -1, 12)),
];

pub struct HouseTowerGenerator;
pub struct TowerGenerator;
pub struct TowerBridgeGenerator {
    ship_created: AtomicBool,
}
pub struct FatTowerGenerator;

pub static HOUSE_TOWER_GENERATOR: HouseTowerGenerator = HouseTowerGenerator;
pub static TOWER_GENERATOR: TowerGenerator = TowerGenerator;
pub static TOWER_BRIDGE_GENERATOR: TowerBridgeGenerator = TowerBridgeGenerator {
    ship_created: AtomicBool::new(false),
};
pub static FAT_TOWER_GENERATOR: FatTowerGenerator = FatTowerGenerator;

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces#addPiece
pub fn add_piece(
    template_manager: &dyn StructureTemplateManager,
    piece: &PieceRef,
    offset: BlockPos,
    name: &str,
    rotation: Rotation,
    overwrite: bool,
) -> PieceRef {
    let template_position = piece.borrow().template_position;
    let new_piece = Rc::new(RefCell::new(EndCityPiece::new(
        template_manager,
        name,
        template_position,
        rotation,
        overwrite,
    )));
    let connected_pos = {
        let p = piece.borrow();
        p.template.calculate_connected_position(
            &p.place_settings,
            offset,
            &new_piece.borrow().place_settings,
            BlockPos::ZERO,
        )
    };
    new_piece
        .borrow_mut()
        .move_by(connected_pos.x, connected_pos.y, connected_pos.z);
    new_piece
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces#startHouseTower
pub fn start_house_tower(
    template_manager: &dyn StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<PieceRef>,
    random: &mut dyn RandomSource,
) {
    FAT_TOWER_GENERATOR.init();
    HOUSE_TOWER_GENERATOR.init();
    TOWER_BRIDGE_GENERATOR.init();
    TOWER_GENERATOR.init();
    let mut piece = add_helper(
        pieces,
        Rc::new(RefCell::new(EndCityPiece::new(
            template_manager,
            "base_floor",
            pos,
            rotation,
            true,
        ))),
    );
    piece = add_helper(
        pieces,
        add_piece(
            template_manager,
            &piece,
            BlockPos::new(-1, 0, -1),
            "second_floor_1",
            rotation,
            false,
        ),
    );
    piece = add_helper(
        pieces,
        add_piece(
            template_manager,
            &piece,
            BlockPos::new(-1, 4, -1),
            "third_floor_1",
            rotation,
            false,
        ),
    );
    piece = add_helper(
        pieces,
        add_piece(
            template_manager,
            &piece,
            BlockPos::new(-1, 8, -1),
            "third_roof",
            rotation,
            true,
        ),
    );
    recursive_children(
        template_manager,
        &TOWER_GENERATOR,
        1,
        &piece,
        None,
        pieces,
        random,
    );
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces#addHelper
pub fn add_helper(pieces: &mut Vec<PieceRef>, piece: PieceRef) -> PieceRef {
    pieces.push(Rc::clone(&piece));
    piece
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces#recursiveChildren
pub fn recursive_children(
    template_manager: &dyn StructureTemplateManager,
    generator: &dyn SectionGenerator,
    depth: i32,
    piece: &PieceRef,
    offset: Option<BlockPos>,
    pieces: &mut Vec<PieceRef>,
    random: &mut dyn RandomSource,
) -> bool {
    if depth > MAX_GEN_DEPTH {
        return false;
    }

    let mut generated: Vec<PieceRef> = Vec::new();
    if generator.generate(
        template_manager,
        depth,
        piece,
        offset,
        &mut generated,
        random,
    ) {
        let mut collision = false;
        let gen_depth_rand = random.next_int_unbounded();
        let piece_gen_depth = piece.borrow().gen_depth;

        for p in &generated {
            p.borrow_mut().gen_depth = gen_depth_rand;
            let bb = p.borrow().bounding_box;
            if let Some(cp) = find_collision_piece(pieces, bb) {
                if cp.borrow().gen_depth != piece_gen_depth {
                    collision = true;
                    break;
                }
            }
        }

        if !collision {
            pieces.extend(generated);
            return true;
        }
    }

    false
}

impl SectionGenerator for HouseTowerGenerator {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#init
    fn init(&self) {}

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#generate
    fn generate(
        &self,
        template_manager: &dyn StructureTemplateManager,
        depth: i32,
        piece: &PieceRef,
        offset: Option<BlockPos>,
        pieces: &mut Vec<PieceRef>,
        random: &mut dyn RandomSource,
    ) -> bool {
        if depth > MAX_GEN_DEPTH {
            return false;
        }

        let rotation = piece.borrow().place_settings.rotation;
        let offset_pos = offset.unwrap_or(BlockPos::ZERO);
        let mut p = add_helper(
            pieces,
            add_piece(
                template_manager,
                piece,
                offset_pos,
                "base_floor",
                rotation,
                true,
            ),
        );
        let choice = random.next_int(3);
        if choice == 0 {
            add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 4, -1),
                    "base_roof",
                    rotation,
                    true,
                ),
            );
        } else if choice == 1 {
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 0, -1),
                    "second_floor_2",
                    rotation,
                    false,
                ),
            );
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 8, -1),
                    "second_roof",
                    rotation,
                    false,
                ),
            );
            recursive_children(
                template_manager,
                &TOWER_GENERATOR,
                depth.wrapping_add(1),
                &p,
                None,
                pieces,
                random,
            );
        } else if choice == 2 {
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 0, -1),
                    "second_floor_2",
                    rotation,
                    false,
                ),
            );
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 4, -1),
                    "third_floor_2",
                    rotation,
                    false,
                ),
            );
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 8, -1),
                    "third_roof",
                    rotation,
                    true,
                ),
            );
            recursive_children(
                template_manager,
                &TOWER_GENERATOR,
                depth.wrapping_add(1),
                &p,
                None,
                pieces,
                random,
            );
        }

        true
    }
}

impl SectionGenerator for TowerGenerator {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#init
    fn init(&self) {}

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#generate
    fn generate(
        &self,
        template_manager: &dyn StructureTemplateManager,
        depth: i32,
        piece: &PieceRef,
        _offset: Option<BlockPos>,
        pieces: &mut Vec<PieceRef>,
        random: &mut dyn RandomSource,
    ) -> bool {
        let rotation = piece.borrow().place_settings.rotation;
        let mut p = Rc::clone(piece);
        p = add_helper(
            pieces,
            add_piece(
                template_manager,
                &p,
                BlockPos::new(
                    3i32.wrapping_add(random.next_int(2)),
                    -3,
                    3i32.wrapping_add(random.next_int(2)),
                ),
                "tower_base",
                rotation,
                true,
            ),
        );
        p = add_helper(
            pieces,
            add_piece(
                template_manager,
                &p,
                BlockPos::new(0, 7, 0),
                "tower_piece",
                rotation,
                true,
            ),
        );
        let mut bridge_anchor: Option<PieceRef> = if random.next_int(3) == 0 {
            Some(Rc::clone(&p))
        } else {
            None
        };
        let count = 1i32.wrapping_add(random.next_int(3));

        let mut i = 0i32;
        while i < count {
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(0, 4, 0),
                    "tower_piece",
                    rotation,
                    true,
                ),
            );
            if i < count.wrapping_sub(1) && random.next_boolean() {
                bridge_anchor = Some(Rc::clone(&p));
            }
            i = i.wrapping_add(1);
        }

        if let Some(anchor) = bridge_anchor {
            for &(rot, bpos) in &TOWER_BRIDGES {
                if random.next_boolean() {
                    let bridge_end = add_helper(
                        pieces,
                        add_piece(
                            template_manager,
                            &anchor,
                            bpos,
                            "bridge_end",
                            rotation.get_rotated(rot),
                            true,
                        ),
                    );
                    recursive_children(
                        template_manager,
                        &TOWER_BRIDGE_GENERATOR,
                        depth.wrapping_add(1),
                        &bridge_end,
                        None,
                        pieces,
                        random,
                    );
                }
            }
            add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 4, -1),
                    "tower_top",
                    rotation,
                    true,
                ),
            );
        } else if depth != 7 {
            return recursive_children(
                template_manager,
                &FAT_TOWER_GENERATOR,
                depth.wrapping_add(1),
                &p,
                None,
                pieces,
                random,
            );
        } else {
            add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(-1, 4, -1),
                    "tower_top",
                    rotation,
                    true,
                ),
            );
        }

        true
    }
}

impl SectionGenerator for TowerBridgeGenerator {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#init
    fn init(&self) {
        self.ship_created.store(false, Ordering::Relaxed);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#generate
    fn generate(
        &self,
        template_manager: &dyn StructureTemplateManager,
        depth: i32,
        piece: &PieceRef,
        _offset: Option<BlockPos>,
        pieces: &mut Vec<PieceRef>,
        random: &mut dyn RandomSource,
    ) -> bool {
        let rotation = piece.borrow().place_settings.rotation;
        let count = random.next_int(4).wrapping_add(1);
        let mut p = add_helper(
            pieces,
            add_piece(
                template_manager,
                piece,
                BlockPos::new(0, 0, -4),
                "bridge_piece",
                rotation,
                true,
            ),
        );
        p.borrow_mut().gen_depth = -1;
        let mut y_offset = 0i32;

        let mut i = 0i32;
        while i < count {
            if random.next_boolean() {
                p = add_helper(
                    pieces,
                    add_piece(
                        template_manager,
                        &p,
                        BlockPos::new(0, y_offset, -4),
                        "bridge_piece",
                        rotation,
                        true,
                    ),
                );
                y_offset = 0;
            } else {
                if random.next_boolean() {
                    p = add_helper(
                        pieces,
                        add_piece(
                            template_manager,
                            &p,
                            BlockPos::new(0, y_offset, -4),
                            "bridge_steep_stairs",
                            rotation,
                            true,
                        ),
                    );
                } else {
                    p = add_helper(
                        pieces,
                        add_piece(
                            template_manager,
                            &p,
                            BlockPos::new(0, y_offset, -8),
                            "bridge_gentle_stairs",
                            rotation,
                            true,
                        ),
                    );
                }
                y_offset = 4;
            }
            i = i.wrapping_add(1);
        }

        let ship_created = self.ship_created.load(Ordering::Relaxed);
        if !ship_created && random.next_int(10i32.wrapping_sub(depth)) == 0 {
            add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(
                        -8i32.wrapping_add(random.next_int(8)),
                        y_offset,
                        -70i32.wrapping_add(random.next_int(10)),
                    ),
                    "ship",
                    rotation,
                    true,
                ),
            );
            self.ship_created.store(true, Ordering::Relaxed);
        } else if !recursive_children(
            template_manager,
            &HOUSE_TOWER_GENERATOR,
            depth.wrapping_add(1),
            &p,
            Some(BlockPos::new(-3, y_offset.wrapping_add(1), -11)),
            pieces,
            random,
        ) {
            return false;
        }

        p = add_helper(
            pieces,
            add_piece(
                template_manager,
                &p,
                BlockPos::new(4, y_offset, 0),
                "bridge_end",
                rotation.get_rotated(Rotation::Clockwise180),
                true,
            ),
        );
        p.borrow_mut().gen_depth = -1;
        true
    }
}

impl SectionGenerator for FatTowerGenerator {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#init
    fn init(&self) {}

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces$SectionGenerator#generate
    fn generate(
        &self,
        template_manager: &dyn StructureTemplateManager,
        depth: i32,
        piece: &PieceRef,
        _offset: Option<BlockPos>,
        pieces: &mut Vec<PieceRef>,
        random: &mut dyn RandomSource,
    ) -> bool {
        let rotation = piece.borrow().place_settings.rotation;
        let mut p = add_helper(
            pieces,
            add_piece(
                template_manager,
                piece,
                BlockPos::new(-3, 4, -3),
                "fat_tower_base",
                rotation,
                true,
            ),
        );
        p = add_helper(
            pieces,
            add_piece(
                template_manager,
                &p,
                BlockPos::new(0, 4, 0),
                "fat_tower_middle",
                rotation,
                true,
            ),
        );

        let mut i = 0i32;
        while i < 2 && random.next_int(3) != 0 {
            p = add_helper(
                pieces,
                add_piece(
                    template_manager,
                    &p,
                    BlockPos::new(0, 8, 0),
                    "fat_tower_middle",
                    rotation,
                    true,
                ),
            );

            for &(rot, bpos) in &FAT_TOWER_BRIDGES {
                if random.next_boolean() {
                    let bridge_end = add_helper(
                        pieces,
                        add_piece(
                            template_manager,
                            &p,
                            bpos,
                            "bridge_end",
                            rotation.get_rotated(rot),
                            true,
                        ),
                    );
                    recursive_children(
                        template_manager,
                        &TOWER_BRIDGE_GENERATOR,
                        depth.wrapping_add(1),
                        &bridge_end,
                        None,
                        pieces,
                        random,
                    );
                }
            }
            i = i.wrapping_add(1);
        }

        add_helper(
            pieces,
            add_piece(
                template_manager,
                &p,
                BlockPos::new(-2, 8, -2),
                "fat_tower_top",
                rotation,
                true,
            ),
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTemplate {
        bb_size: (i32, i32, i32),
    }

    impl StructureTemplate for MockTemplate {
        fn calculate_connected_position(
            &self,
            _self_settings: &StructurePlaceSettings,
            self_offset: BlockPos,
            _other_settings: &StructurePlaceSettings,
            _other_offset: BlockPos,
        ) -> BlockPos {
            self_offset
        }

        fn get_bounding_box(&self, _settings: &StructurePlaceSettings, pos: BlockPos) -> BoundingBox {
            BoundingBox::new(
                pos.x,
                pos.y,
                pos.z,
                pos.x.wrapping_add(self.bb_size.0).wrapping_sub(1),
                pos.y.wrapping_add(self.bb_size.1).wrapping_sub(1),
                pos.z.wrapping_add(self.bb_size.2).wrapping_sub(1),
            )
        }
    }

    struct MockManager;

    impl StructureTemplateManager for MockManager {
        fn get_or_create(&self, _location: &Identifier) -> Box<dyn StructureTemplate> {
            Box::new(MockTemplate { bb_size: (3, 3, 3) })
        }
    }

    struct FixedRandom {
        ints: Vec<i32>,
        bools: Vec<bool>,
        unbounded: Vec<i32>,
        i: usize,
        b: usize,
        u: usize,
    }

    impl FixedRandom {
        fn new(ints: Vec<i32>, bools: Vec<bool>, unbounded: Vec<i32>) -> Self {
            Self {
                ints,
                bools,
                unbounded,
                i: 0,
                b: 0,
                u: 0,
            }
        }
    }

    impl RandomSource for FixedRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.i < self.ints.len() {
                self.ints[self.i]
            } else {
                0
            };
            self.i += 1;
            if bound <= 0 {
                0
            } else {
                ((v % bound) + bound) % bound
            }
        }

        fn next_int_unbounded(&mut self) -> i32 {
            let v = if self.u < self.unbounded.len() {
                self.unbounded[self.u]
            } else {
                0
            };
            self.u += 1;
            v
        }

        fn next_boolean(&mut self) -> bool {
            let v = if self.b < self.bools.len() {
                self.bools[self.b]
            } else {
                false
            };
            self.b += 1;
            v
        }
    }

    #[test]
    fn add_helper_and_add_piece_basic() {
        let mgr = MockManager;
        let mut pieces: Vec<PieceRef> = Vec::new();
        let base = Rc::new(RefCell::new(EndCityPiece::new(
            &mgr,
            "base_floor",
            BlockPos::new(10, -5, 20),
            Rotation::Clockwise90,
            true,
        )));
        let returned = add_helper(&mut pieces, Rc::clone(&base));
        assert!(Rc::ptr_eq(&base, &returned));
        assert_eq!(pieces.len(), 1);
        assert_eq!(base.borrow().template_name, "base_floor");
        assert_eq!(base.borrow().place_settings.rotation, Rotation::Clockwise90);
        assert!(base.borrow().place_settings.overwrite);

        let child = add_piece(
            &mgr,
            &base,
            BlockPos::new(-1, 4, -2),
            "second_floor_1",
            Rotation::Clockwise90,
            false,
        );
        assert_eq!(child.borrow().template_name, "second_floor_1");
        assert_eq!(child.borrow().template_position, BlockPos::new(9, -1, 18));
        assert!(!child.borrow().place_settings.overwrite);
    }

    #[test]
    fn recursive_children_depth_limit_and_collision() {
        let mgr = MockManager;
        let mut pieces: Vec<PieceRef> = Vec::new();
        let piece = Rc::new(RefCell::new(EndCityPiece::new(
            &mgr,
            "base_floor",
            BlockPos::new(0, 0, 0),
            Rotation::None,
            true,
        )));
        piece.borrow_mut().gen_depth = 5;
        let mut random = FixedRandom::new(vec![], vec![], vec![]);

        let ok = recursive_children(
            &mgr,
            &HOUSE_TOWER_GENERATOR,
            9,
            &piece,
            None,
            &mut pieces,
            &mut random,
        );
        assert!(!ok);
        assert!(pieces.is_empty());

        let mut pieces2: Vec<PieceRef> = Vec::new();
        let existing = Rc::new(RefCell::new(EndCityPiece::new(
            &mgr,
            "base_floor",
            BlockPos::new(0, 0, 0),
            Rotation::None,
            true,
        )));
        existing.borrow_mut().gen_depth = 1;
        pieces2.push(existing);

        let mut random2 = FixedRandom::new(vec![0, 0, 0], vec![], vec![42]);
        let ok2 = recursive_children(
            &mgr,
            &HOUSE_TOWER_GENERATOR,
            1,
            &piece,
            Some(BlockPos::ZERO),
            &mut pieces2,
            &mut random2,
        );
        assert!(!ok2);
        assert_eq!(pieces2.len(), 1);
    }

    #[test]
    fn blockpos_move_by_wrapping_and_negative() {
        let mgr = MockManager;
        let mut piece = EndCityPiece::new(
            &mgr,
            "base_floor",
            BlockPos::new(i32::MAX, i32::MIN, -10),
            Rotation::None,
            true,
        );
        piece.move_by(1, -1, -20);
        assert_eq!(piece.template_position.x, i32::MIN);
        assert_eq!(piece.template_position.y, i32::MAX);
        assert_eq!(piece.template_position.z, -30);
        let bb = piece.get_bounding_box();
        assert_eq!(bb.min_x, i32::MIN);
        assert_eq!(bb.min_y, i32::MAX);
        assert_eq!(bb.min_z, -30);
    }

    #[test]
    fn handle_data_marker_float_bits_and_chest() {
        #[derive(Default)]
        struct MockLevel {
            chests: Vec<(BlockPos, String)>,
            shulkers: Vec<(u64, u64, u64)>,
            frames: Vec<(BlockPos, Direction)>,
            spawnable: bool,
        }

        impl ServerLevelAccessor for MockLevel {
            fn set_block_entity_loot_table(
                &mut self,
                pos: BlockPos,
                _random: &mut dyn RandomSource,
                loot_table: &str,
            ) {
                self.chests.push((pos, loot_table.to_string()));
            }

            fn is_in_spawnable_bounds(&self, _pos: BlockPos) -> bool {
                self.spawnable
            }

            fn add_shulker(&mut self, x: f64, y: f64, z: f64) {
                self.shulkers.push((x.to_bits(), y.to_bits(), z.to_bits()));
            }

            fn add_elytra_item_frame(&mut self, pos: BlockPos, rotation: Direction) {
                self.frames.push((pos, rotation));
            }
        }

        let mgr = MockManager;
        let piece = EndCityPiece::new(&mgr, "base_floor", BlockPos::ZERO, Rotation::Clockwise90, true);
        let bb = BoundingBox::new(-100, -100, -100, 100, 100, 100);
        let mut random = FixedRandom::new(vec![], vec![], vec![]);

        let mut level = MockLevel {
            spawnable: true,
            ..Default::default()
        };
        piece.handle_data_marker("Chest", BlockPos::new(1, 5, 2), &mut level, &mut random, &bb);
        assert_eq!(level.chests.len(), 1);
        assert_eq!(level.chests[0].0, BlockPos::new(1, 4, 2));
        assert_eq!(level.chests[0].1, "minecraft:chests/end_city_treasure");

        let mut level2 = MockLevel {
            spawnable: true,
            ..Default::default()
        };
        piece.handle_data_marker("Sentry", BlockPos::new(3, 7, -2), &mut level2, &mut random, &bb);
        assert_eq!(level2.shulkers.len(), 1);
        assert_eq!(level2.shulkers[0].0, (3.0f64 + 0.5).to_bits());
        assert_eq!(level2.shulkers[0].1, (7.0f64).to_bits());
        assert_eq!(level2.shulkers[0].2, (-2.0f64 + 0.5).to_bits());

        let mut level3 = MockLevel {
            spawnable: true,
            ..Default::default()
        };
        piece.handle_data_marker("Elytra", BlockPos::new(0, 0, 0), &mut level3, &mut random, &bb);
        assert_eq!(level3.frames.len(), 1);
        assert_eq!(level3.frames[0].1, Direction::West);
    }

    #[test]
    fn tower_bridge_ship_created_state() {
        let mgr = MockManager;
        TOWER_BRIDGE_GENERATOR.init();
        assert!(!TOWER_BRIDGE_GENERATOR.ship_created.load(Ordering::Relaxed));

        let mut pieces: Vec<PieceRef> = Vec::new();
        let base = Rc::new(RefCell::new(EndCityPiece::new(
            &mgr,
            "bridge_end",
            BlockPos::ZERO,
            Rotation::None,
            true,
        )));
        // count=1, one straight bridge piece, then ship check passes (next_int(9)==0)
        let mut random = FixedRandom::new(vec![0, 0, 0, 0], vec![true], vec![]);
        let ok = TOWER_BRIDGE_GENERATOR.generate(&mgr, 1, &base, None, &mut pieces, &mut random);
        assert!(ok);
        assert!(TOWER_BRIDGE_GENERATOR.ship_created.load(Ordering::Relaxed));
        assert!(pieces.iter().any(|p| p.borrow().template_name == "ship"));

        TOWER_BRIDGE_GENERATOR.init();
        assert!(!TOWER_BRIDGE_GENERATOR.ship_created.load(Ordering::Relaxed));
    }
}
