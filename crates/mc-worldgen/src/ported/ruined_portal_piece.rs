use std::fmt;

/// Java: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece
pub const PROBABILITY_OF_GOLD_GONE: f32 = 0.3;
pub const PROBABILITY_OF_MAGMA_INSTEAD_OF_NETHERRACK: f32 = 0.07;
pub const PROBABILITY_OF_MAGMA_INSTEAD_OF_LAVA: f32 = 0.2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
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

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn above(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    pub fn relative(self, dir: Direction) -> Self {
        let n = dir.normal();
        Self {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
        }
    }

    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        let n = dir.normal();
        self.x = self.x.wrapping_add(n.x);
        self.y = self.y.wrapping_add(n.y);
        self.z = self.z.wrapping_add(n.z);
        self
    }

    pub fn immutable(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const fn normal(self) -> Vec3i {
        match self {
            Self::Down => Vec3i { x: 0, y: -1, z: 0 },
            Self::Up => Vec3i { x: 0, y: 1, z: 0 },
            Self::North => Vec3i { x: 0, y: 0, z: -1 },
            Self::South => Vec3i { x: 0, y: 0, z: 1 },
            Self::West => Vec3i { x: -1, y: 0, z: 0 },
            Self::East => Vec3i { x: 1, y: 0, z: 0 },
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    GoldBlock,
    Netherrack,
    MagmaBlock,
    Lava,
    Vine,
    JungleLeaves,
    Obsidian,
    Other(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockTag {
    FeaturesCannotReplace,
}

pub const VINE_NORTH: u16 = 1 << 0;
pub const VINE_SOUTH: u16 = 1 << 1;
pub const VINE_EAST: u16 = 1 << 2;
pub const VINE_WEST: u16 = 1 << 3;
pub const VINE_UP: u16 = 1 << 4;
pub const LEAVES_PERSISTENT: u16 = 1 << 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub flags: u16,
}

impl BlockState {
    pub const fn new(block: Block, flags: u16) -> Self {
        Self { block, flags }
    }

    pub fn default_state(block: Block) -> Self {
        Self { block, flags: 0 }
    }

    pub fn is_air(&self) -> bool {
        self.block == Block::Air
    }

    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }

    pub fn set_value(mut self, flag: u16, value: bool) -> Self {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
        self
    }

    pub fn has_flag(self, flag: u16) -> bool {
        (self.flags & flag) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    OceanFloorWg,
    WorldSurfaceWg,
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_double(&mut self) -> f64;
}

pub trait LevelAccessor {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn is_face_full(&self, pos: BlockPos, direction: Direction) -> bool;
    fn has_block_tag(&self, block: Block, tag: BlockTag) -> bool;
}

pub trait WorldGenLevel: LevelAccessor {}
pub trait ServerLevelAccessor: LevelAccessor {}
pub trait StructureManager {}
pub trait ChunkGenerator {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
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
    pub const fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
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
            && pos.z >= self.min_z
            && pos.z <= self.max_z
            && pos.y >= self.min_y
            && pos.y <= self.max_y
    }

    pub fn encapsulate(&mut self, other: BoundingBox) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.min_z = self.min_z.min(other.min_z);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
        self.max_z = self.max_z.max(other.max_z);
    }

    pub fn get_center(&self) -> BlockPos {
        BlockPos::new(
            self.min_x.wrapping_add(self.get_x_span() / 2),
            self.min_y.wrapping_add(self.get_y_span() / 2),
            self.min_z.wrapping_add(self.get_z_span() / 2),
        )
    }

    pub fn get_x_span(&self) -> i32 {
        self.max_x.wrapping_sub(self.min_x).wrapping_add(1)
    }

    pub fn get_y_span(&self) -> i32 {
        self.max_y.wrapping_sub(self.min_y).wrapping_add(1)
    }

    pub fn get_z_span(&self) -> i32 {
        self.max_z.wrapping_sub(self.min_z).wrapping_add(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub rotation_pivot: BlockPos,
    pub air_pocket: bool,
    pub mossiness: f32,
    pub replace_with_blackstone: bool,
    pub processor_rules: Vec<ProcessorRule>,
    pub has_protected_block_processor: bool,
    pub has_lava_submerged_block_processor: bool,
    pub has_block_age_processor: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessorRule {
    RandomBlockReplace {
        input: Block,
        probability: f32,
        output: Block,
    },
    BlockReplace {
        input: Block,
        output: Block,
    },
}

impl StructurePlaceSettings {
    pub fn new(
        rotation: Rotation,
        mirror: Mirror,
        rotation_pivot: BlockPos,
    ) -> Self {
        Self {
            rotation,
            mirror,
            rotation_pivot,
            air_pocket: false,
            mossiness: 0.0,
            replace_with_blackstone: false,
            processor_rules: Vec::new(),
            has_protected_block_processor: false,
            has_lava_submerged_block_processor: false,
            has_block_age_processor: false,
        }
    }
}

/// Java: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece$Properties
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Properties {
    pub cold: bool,
    pub mossiness: f32,
    pub air_pocket: bool,
    pub overgrown: bool,
    pub vines: bool,
    pub replace_with_blackstone: bool,
}

impl Properties {
    pub fn new(
        cold: bool,
        mossiness: f32,
        air_pocket: bool,
        overgrown: bool,
        vines: bool,
        replace_with_blackstone: bool,
    ) -> Self {
        Self {
            cold,
            mossiness,
            air_pocket,
            overgrown,
            vines,
            replace_with_blackstone,
        }
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            cold: false,
            mossiness: 0.0,
            air_pocket: false,
            overgrown: false,
            vines: false,
            replace_with_blackstone: false,
        }
    }
}

/// Java: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece$VerticalPlacement
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VerticalPlacement {
    OnLandSurface,
    PartlyBuried,
    OnOceanFloor,
    InMountain,
    Underground,
    InNether,
}

impl VerticalPlacement {
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::OnLandSurface => "on_land_surface",
            Self::PartlyBuried => "partly_buried",
            Self::OnOceanFloor => "on_ocean_floor",
            Self::InMountain => "in_mountain",
            Self::Underground => "underground",
            Self::InNether => "in_nether",
        }
    }

    pub const fn get_serialized_name(self) -> &'static str {
        self.get_name()
    }
}

impl fmt::Display for VerticalPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_name())
    }
}

/// Java: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece
#[derive(Clone, Debug)]
pub struct RuinedPortalPiece {
    pub template_position: BlockPos,
    pub bounding_box: BoundingBox,
    pub place_settings: StructurePlaceSettings,
    pub vertical_placement: VerticalPlacement,
    pub properties: Properties,
}

impl RuinedPortalPiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#<init>
    pub fn new(
        template_position: BlockPos,
        bounding_box: BoundingBox,
        vertical_placement: VerticalPlacement,
        properties: Properties,
        mirror: Mirror,
        rotation: Rotation,
        rotation_pivot: BlockPos,
    ) -> Self {
        let place_settings = Self::make_settings(mirror, rotation, vertical_placement, rotation_pivot, &properties);
        Self {
            template_position,
            bounding_box,
            place_settings,
            vertical_placement,
            properties,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#makeSettings
    fn make_settings(
        mirror: Mirror,
        rotation: Rotation,
        vertical_placement: VerticalPlacement,
        rotation_pivot: BlockPos,
        properties: &Properties,
    ) -> StructurePlaceSettings {
        let mut settings = StructurePlaceSettings::new(rotation, mirror, rotation_pivot);
        settings.air_pocket = properties.air_pocket;
        settings.mossiness = properties.mossiness;
        settings.replace_with_blackstone = properties.replace_with_blackstone;

        let mut rules = Vec::new();
        rules.push(Self::get_block_replace_rule_with_probability(
            Block::GoldBlock,
            PROBABILITY_OF_GOLD_GONE,
            Block::Air,
        ));
        rules.push(Self::get_lava_processor_rule(vertical_placement, properties));
        if !properties.cold {
            rules.push(Self::get_block_replace_rule_with_probability(
                Block::Netherrack,
                PROBABILITY_OF_MAGMA_INSTEAD_OF_NETHERRACK,
                Block::MagmaBlock,
            ));
        }

        settings.processor_rules = rules;
        settings.has_block_age_processor = true;
        settings.has_protected_block_processor = true;
        settings.has_lava_submerged_block_processor = true;
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#getLavaProcessorRule
    fn get_lava_processor_rule(
        vertical_placement: VerticalPlacement,
        properties: &Properties,
    ) -> ProcessorRule {
        if vertical_placement == VerticalPlacement::OnOceanFloor {
            Self::get_block_replace_rule(Block::Lava, Block::MagmaBlock)
        } else if properties.cold {
            Self::get_block_replace_rule(Block::Lava, Block::Netherrack)
        } else {
            Self::get_block_replace_rule_with_probability(
                Block::Lava,
                PROBABILITY_OF_MAGMA_INSTEAD_OF_LAVA,
                Block::MagmaBlock,
            )
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#getBlockReplaceRule
    fn get_block_replace_rule_with_probability(input: Block, probability: f32, output: Block) -> ProcessorRule {
        ProcessorRule::RandomBlockReplace {
            input,
            probability,
            output,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#getBlockReplaceRule
    fn get_block_replace_rule(input: Block, output: Block) -> ProcessorRule {
        ProcessorRule::BlockReplace { input, output }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#postProcess
    pub fn post_process(
        &mut self,
        level: &mut dyn WorldGenLevel,
        _structure_manager: &dyn StructureManager,
        _chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        structure_bb: &mut BoundingBox,
        _chunk_pos: ChunkPos,
        _pivot: BlockPos,
        template_bb: BoundingBox,
        mut super_post_process: impl FnMut(&mut dyn WorldGenLevel, &mut BoundingBox),
    ) {
        if structure_bb.is_inside(template_bb.get_center()) {
            structure_bb.encapsulate(template_bb);
            super_post_process(level, structure_bb);
            self.spread_netherrack(random, level);
            self.add_netherrack_drip_columns_below_portal(random, level);
            if self.properties.vines || self.properties.overgrown {
                let bb = self.bounding_box;
                let mut y = bb.min_y;
                while y <= bb.max_y {
                    let mut z = bb.min_z;
                    while z <= bb.max_z {
                        let mut x = bb.min_x;
                        while x <= bb.max_x {
                            let pos = BlockPos::new(x, y, z);
                            if self.properties.vines {
                                self.maybe_add_vines(random, level, pos);
                            }
                            if self.properties.overgrown {
                                self.maybe_add_leaves_above(random, level, pos);
                            }
                            x = x.wrapping_add(1);
                        }
                        z = z.wrapping_add(1);
                    }
                    y = y.wrapping_add(1);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#handleDataMarker
    pub fn handle_data_marker(
        &self,
        _name: &str,
        _pos: BlockPos,
        _level: &mut dyn ServerLevelAccessor,
        _random: &mut dyn RandomSource,
        _box_limit: BoundingBox,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#maybeAddVines
    fn maybe_add_vines(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor, pos: BlockPos) {
        let state = level.get_block_state(pos);
        if !state.is_air() && !state.is(Block::Vine) {
            let dir = get_random_horizontal_direction(random);
            let pos2 = pos.relative(dir);
            let state2 = level.get_block_state(pos2);
            if state2.is_air() {
                if level.is_face_full(pos, dir) {
                    let vine_flag = vine_property_for_face(dir.opposite());
                    let vine_state = BlockState::default_state(Block::Vine).set_value(vine_flag, true);
                    level.set_block(pos2, vine_state, 3);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#maybeAddLeavesAbove
    fn maybe_add_leaves_above(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor, pos: BlockPos) {
        if random.next_float() < 0.5f32
            && level.get_block_state(pos).is(Block::Netherrack)
            && level.get_block_state(pos.above()).is_air()
        {
            let leaves = BlockState::default_state(Block::JungleLeaves).set_value(LEAVES_PERSISTENT, true);
            level.set_block(pos.above(), leaves, 3);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#addNetherrackDripColumnsBelowPortal
    fn add_netherrack_drip_columns_below_portal(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor) {
        let mut x = self.bounding_box.min_x.wrapping_add(1);
        while x < self.bounding_box.max_x {
            let mut z = self.bounding_box.min_z.wrapping_add(1);
            while z < self.bounding_box.max_z {
                let pos = BlockPos::new(x, self.bounding_box.min_y, z);
                if level.get_block_state(pos).is(Block::Netherrack) {
                    self.add_netherrack_drip_column(random, level, pos.below());
                }
                z = z.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#addNetherrackDripColumn
    fn add_netherrack_drip_column(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor, pos: BlockPos) {
        let mut cursor = pos.mutable();
        self.place_netherrack_or_magma(random, level, cursor.immutable());
        let mut remaining: i32 = 8;
        while remaining > 0 && random.next_float() < 0.5f32 {
            cursor.move_dir(Direction::Down);
            remaining = remaining.wrapping_sub(1);
            self.place_netherrack_or_magma(random, level, cursor.immutable());
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#spreadNetherrack
    fn spread_netherrack(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor) {
        let on_surface = self.vertical_placement == VerticalPlacement::OnLandSurface
            || self.vertical_placement == VerticalPlacement::OnOceanFloor;
        let center = self.bounding_box.get_center();
        let center_x = center.x;
        let center_z = center.z;
        let falloff: [f32; 14] = [
            1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 0.9f32, 0.9f32, 0.8f32, 0.7f32,
            0.6f32, 0.4f32, 0.2f32,
        ];
        let radius = falloff.len() as i32;
        let span_sum = self
            .bounding_box
            .get_x_span()
            .wrapping_add(self.bounding_box.get_z_span());
        let span_avg = span_sum / 2;
        let bound = 8i32.wrapping_sub(span_avg / 2).max(1);
        let offset = random.next_int(bound);

        let mut x = center_x.wrapping_sub(radius);
        let end_x = center_x.wrapping_add(radius);
        while x <= end_x {
            let mut z = center_z.wrapping_sub(radius);
            let end_z = center_z.wrapping_add(radius);
            while z <= end_z {
                let dx = (x.wrapping_sub(center_x)).abs();
                let dz = (z.wrapping_sub(center_z)).abs();
                let dist = dx.wrapping_add(dz);
                let idx = dist.wrapping_add(offset).max(0);
                if idx < radius {
                    let prob = falloff[idx as usize];
                    if random.next_double() < prob as f64 {
                        let surface_y = Self::get_surface_y(level, x, z, self.vertical_placement);
                        let y = if on_surface {
                            surface_y
                        } else {
                            self.bounding_box.min_y.min(surface_y)
                        };
                        let pos = BlockPos::new(x, y, z);
                        if (y.wrapping_sub(self.bounding_box.min_y)).abs() <= 3
                            && self.can_block_be_replaced_by_netherrack_or_magma(level, pos)
                        {
                            self.place_netherrack_or_magma(random, level, pos);
                            if self.properties.overgrown {
                                self.maybe_add_leaves_above(random, level, pos);
                            }
                            self.add_netherrack_drip_column(random, level, pos.below());
                        }
                    }
                }
                z = z.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#canBlockBeReplacedByNetherrackOrMagma
    fn can_block_be_replaced_by_netherrack_or_magma(&self, level: &dyn LevelAccessor, pos: BlockPos) -> bool {
        let state = level.get_block_state(pos);
        !state.is(Block::Air)
            && !state.is(Block::Obsidian)
            && !level.has_block_tag(state.block, BlockTag::FeaturesCannotReplace)
            && (self.vertical_placement == VerticalPlacement::InNether || !state.is(Block::Lava))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#placeNetherrackOrMagma
    fn place_netherrack_or_magma(&self, random: &mut dyn RandomSource, level: &mut dyn LevelAccessor, pos: BlockPos) {
        if !self.properties.cold && random.next_float() < PROBABILITY_OF_MAGMA_INSTEAD_OF_NETHERRACK {
            level.set_block(pos, BlockState::default_state(Block::MagmaBlock), 3);
        } else {
            level.set_block(pos, BlockState::default_state(Block::Netherrack), 3);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#getSurfaceY
    pub fn get_surface_y(level: &dyn LevelAccessor, x: i32, z: i32, placement: VerticalPlacement) -> i32 {
        level.get_height(Self::get_height_map_type(placement), x, z).wrapping_sub(1)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#getHeightMapType
    pub fn get_height_map_type(placement: VerticalPlacement) -> HeightmapType {
        if placement == VerticalPlacement::OnOceanFloor {
            HeightmapType::OceanFloorWg
        } else {
            HeightmapType::WorldSurfaceWg
        }
    }
}

fn get_random_horizontal_direction(random: &mut dyn RandomSource) -> Direction {
    match random.next_int(4) & 3 {
        0 => Direction::North,
        1 => Direction::South,
        2 => Direction::West,
        _ => Direction::East,
    }
}

fn vine_property_for_face(dir: Direction) -> u16 {
    match dir {
        Direction::North => VINE_NORTH,
        Direction::South => VINE_SOUTH,
        Direction::East => VINE_EAST,
        Direction::West => VINE_WEST,
        Direction::Up => VINE_UP,
        Direction::Down => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        doubles: Vec<f64>,
        fi: usize,
        ii: usize,
        di: usize,
    }

    impl TestRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>, doubles: Vec<f64>) -> Self {
            Self { floats, ints, doubles, fi: 0, ii: 0, di: 0 }
        }
    }

    impl RandomSource for TestRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.fi).copied().unwrap_or(0.0);
            self.fi += 1;
            v
        }
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.ii).copied().unwrap_or(0);
            self.ii += 1;
            if bound <= 0 { 0 } else { v.rem_euclid(bound) }
        }
        fn next_double(&mut self) -> f64 {
            let v = self.doubles.get(self.di).copied().unwrap_or(0.0);
            self.di += 1;
            v
        }
    }

    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        heights: HashMap<(i32, i32, HeightmapType), i32>,
        face_full: HashMap<(i32, i32, i32, Direction), bool>,
        protected: Vec<Block>,
    }

    impl TestLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                heights: HashMap::new(),
                face_full: HashMap::new(),
                protected: Vec::new(),
            }
        }
        fn set(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
        fn get(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState::default_state(Block::Air))
        }
    }

    impl LevelAccessor for TestLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.get(pos)
        }
        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.set(pos, state);
            true
        }
        fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            self.heights.get(&(x, z, heightmap)).copied().unwrap_or(64)
        }
        fn is_face_full(&self, pos: BlockPos, direction: Direction) -> bool {
            self.face_full.get(&(pos.x, pos.y, pos.z, direction)).copied().unwrap_or(false)
        }
        fn has_block_tag(&self, block: Block, tag: BlockTag) -> bool {
            match tag {
                BlockTag::FeaturesCannotReplace => self.protected.contains(&block),
            }
        }
    }

    impl WorldGenLevel for TestLevel {}

    struct NoopStructureManager;
    impl StructureManager for NoopStructureManager {}
    struct NoopChunkGenerator;
    impl ChunkGenerator for NoopChunkGenerator {}

    fn make_piece(placement: VerticalPlacement, properties: Properties) -> RuinedPortalPiece {
        RuinedPortalPiece::new(
            BlockPos::new(0, 64, 0),
            BoundingBox::new(0, 60, 0, 10, 70, 10),
            placement,
            properties,
            Mirror::None,
            Rotation::None,
            BlockPos::new(5, 0, 5),
        )
    }

    #[test]
    fn test_vertical_placement_names_and_heightmap() {
        assert_eq!(VerticalPlacement::OnLandSurface.get_name(), "on_land_surface");
        assert_eq!(VerticalPlacement::InNether.get_serialized_name(), "in_nether");
        assert_eq!(
            RuinedPortalPiece::get_height_map_type(VerticalPlacement::OnOceanFloor),
            HeightmapType::OceanFloorWg
        );
        assert_eq!(
            RuinedPortalPiece::get_height_map_type(VerticalPlacement::Underground),
            HeightmapType::WorldSurfaceWg
        );
    }

    #[test]
    fn test_make_settings_rules() {
        let props = Properties::new(false, 0.2, false, false, false, true);
        let piece = make_piece(VerticalPlacement::OnLandSurface, props);
        assert!(piece.place_settings.replace_with_blackstone);
        assert!(piece.place_settings.has_block_age_processor);
        assert!(piece.place_settings.has_protected_block_processor);
        assert!(piece.place_settings.has_lava_submerged_block_processor);
        // gold, lava, netherrack->magma (not cold)
        assert_eq!(piece.place_settings.processor_rules.len(), 3);
        match piece.place_settings.processor_rules[0] {
            ProcessorRule::RandomBlockReplace { input, probability, output } => {
                assert_eq!(input, Block::GoldBlock);
                assert_eq!(probability.to_bits(), PROBABILITY_OF_GOLD_GONE.to_bits());
                assert_eq!(output, Block::Air);
            }
            _ => panic!("expected random replace"),
        }
        match piece.place_settings.processor_rules[1] {
            ProcessorRule::RandomBlockReplace { input, probability, output } => {
                assert_eq!(input, Block::Lava);
                assert_eq!(probability.to_bits(), PROBABILITY_OF_MAGMA_INSTEAD_OF_LAVA.to_bits());
                assert_eq!(output, Block::MagmaBlock);
            }
            _ => panic!("expected lava random replace"),
        }
    }

    #[test]
    fn test_lava_processor_rule_variants() {
        let cold = Properties::new(true, 0.0, false, false, false, false);
        let ocean = RuinedPortalPiece::get_lava_processor_rule(VerticalPlacement::OnOceanFloor, &cold);
        match ocean {
            ProcessorRule::BlockReplace { input, output } => {
                assert_eq!(input, Block::Lava);
                assert_eq!(output, Block::MagmaBlock);
            }
            _ => panic!("expected block replace"),
        }
        let cold_rule = RuinedPortalPiece::get_lava_processor_rule(VerticalPlacement::OnLandSurface, &cold);
        match cold_rule {
            ProcessorRule::BlockReplace { input, output } => {
                assert_eq!(input, Block::Lava);
                assert_eq!(output, Block::Netherrack);
            }
            _ => panic!("expected block replace"),
        }
        let warm = Properties::new(false, 0.0, false, false, false, false);
        let warm_rule = RuinedPortalPiece::get_lava_processor_rule(VerticalPlacement::OnLandSurface, &warm);
        match warm_rule {
            ProcessorRule::RandomBlockReplace { probability, .. } => {
                assert_eq!(probability.to_bits(), PROBABILITY_OF_MAGMA_INSTEAD_OF_LAVA.to_bits());
            }
            _ => panic!("expected random replace"),
        }
    }

    #[test]
    fn test_maybe_add_vines_and_leaves() {
        let mut level = TestLevel::new();
        let pos = BlockPos::new(0, 64, 0);
        level.set(pos, BlockState::default_state(Block::Netherrack));
        level.face_full.insert((0, 64, 0, Direction::North), true);
        let mut random = TestRandom::new(vec![0.0, 0.0], vec![0], vec![]);
        let piece = make_piece(VerticalPlacement::OnLandSurface, Properties::new(false, 0.0, false, true, true, false));

        piece.maybe_add_vines(&mut random, &mut level, pos);
        let vine_pos = pos.relative(Direction::North);
        let vine = level.get(vine_pos);
        assert_eq!(vine.block, Block::Vine);
        assert!(vine.has_flag(VINE_SOUTH)); // opposite of North

        // maybeAddLeavesAbove: nextFloat < 0.5, netherrack below, air above
        let mut random2 = TestRandom::new(vec![0.1], vec![], vec![]);
        piece.maybe_add_leaves_above(&mut random2, &mut level, pos);
        let above = level.get(pos.above());
        assert_eq!(above.block, Block::JungleLeaves);
        assert!(above.has_flag(LEAVES_PERSISTENT));
    }

    #[test]
    fn test_can_replace_and_place_netherrack_or_magma() {
        let mut level = TestLevel::new();
        level.protected.push(Block::Other(99));
        let piece_nether = make_piece(VerticalPlacement::InNether, Properties::new(false, 0.0, false, false, false, false));
        let pos = BlockPos::new(1, 2, 3);
        level.set(pos, BlockState::default_state(Block::Lava));
        assert!(piece_nether.can_block_be_replaced_by_netherrack_or_magma(&level, pos));
        let piece_surface = make_piece(VerticalPlacement::OnLandSurface, Properties::new(false, 0.0, false, false, false, false));
        assert!(!piece_surface.can_block_be_replaced_by_netherrack_or_magma(&level, pos));

        level.set(pos, BlockState::default_state(Block::Other(99)));
        assert!(!piece_surface.can_block_be_replaced_by_netherrack_or_magma(&level, pos));

        let mut random_magma = TestRandom::new(vec![0.01], vec![], vec![]);
        let target = BlockPos::new(5, 5, 5);
        piece_surface.place_netherrack_or_magma(&mut random_magma, &mut level, target);
        assert_eq!(level.get(target).block, Block::MagmaBlock);

        let mut random_neth = TestRandom::new(vec![0.5], vec![], vec![]);
        let target2 = BlockPos::new(6, 5, 5);
        piece_surface.place_netherrack_or_magma(&mut random_neth, &mut level, target2);
        assert_eq!(level.get(target2).block, Block::Netherrack);
    }

    #[test]
    fn test_spread_netherrack_negative_coords_and_surface() {
        let mut level = TestLevel::new();
        let props = Properties::new(false, 0.0, false, false, false, false);
        let mut piece = RuinedPortalPiece::new(
            BlockPos::new(-10, 0, -10),
            BoundingBox::new(-5, -2, -5, 5, 5, 5),
            VerticalPlacement::OnLandSurface,
            props,
            Mirror::None,
            Rotation::None,
            BlockPos::new(0, 0, 0),
        );
        // Place replaceable ground at surface
        let center = piece.bounding_box.get_center();
        for dx in -2..=2 {
            for dz in -2..=2 {
                let x = center.x.wrapping_add(dx);
                let z = center.z.wrapping_add(dz);
                level.heights.insert((x, z, HeightmapType::WorldSurfaceWg), 1);
                level.set(BlockPos::new(x, 0, z), BlockState::default_state(Block::Other(1)));
            }
        }
        let mut random = TestRandom::new(vec![0.0; 512], vec![0; 128], vec![0.0; 512]);
        piece.spread_netherrack(&mut random, &mut level);
        // At least center should be replaced when random gives 0.0
        let replaced = level.get(BlockPos::new(center.x, 0, center.z));
        assert!(replaced.is(Block::Netherrack) || replaced.is(Block::MagmaBlock));
        assert_eq!(
            RuinedPortalPiece::get_surface_y(&level, 7, -3, VerticalPlacement::OnLandSurface),
            63
        );
    }
}
