use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn at_y(self, y: i32) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
    }

    pub fn subtract(self, v: Vec3i) -> Self {
        Self {
            x: self.x.wrapping_sub(v.x),
            y: self.y.wrapping_sub(v.y),
            z: self.z.wrapping_sub(v.z),
        }
    }

    pub fn offset(self, v: Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(v.x),
            y: self.y.wrapping_add(v.y),
            z: self.z.wrapping_add(v.z),
        }
    }

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
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

    pub fn between_closed_stream(bb: &BoundingBox) -> Vec<BlockPos> {
        let mut out = Vec::new();
        let mut x = bb.min_x;
        loop {
            let mut y = bb.min_y;
            loop {
                let mut z = bb.min_z;
                loop {
                    out.push(BlockPos::new(x, y, z));
                    if z == bb.max_z {
                        break;
                    }
                    z = z.wrapping_add(1);
                }
                if y == bb.max_y {
                    break;
                }
                y = y.wrapping_add(1);
            }
            if x == bb.max_x {
                break;
            }
            x = x.wrapping_add(1);
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const HORIZONTAL: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub fn normal(self) -> Vec3i {
        match self {
            Direction::Down => Vec3i::new(0, -1, 0),
            Direction::Up => Vec3i::new(0, 1, 0),
            Direction::North => Vec3i::new(0, 0, -1),
            Direction::South => Vec3i::new(0, 0, 1),
            Direction::West => Vec3i::new(-1, 0, 0),
            Direction::East => Vec3i::new(1, 0, 0),
        }
    }

    pub fn get_opposite(self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Air,
    Water,
    MagmaBlock,
    Other,
}

impl BlockKind {
    pub fn default_block_state(self) -> BlockState {
        BlockState { kind: self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
}

impl BlockState {
    pub fn is(&self, block: BlockKind) -> bool {
        self.kind == block
    }

    pub fn is_air(&self) -> bool {
        self.kind == BlockKind::Air
    }

    pub fn get_face_occlusion_shape(&self, _dir: Direction) -> VoxelShape {
        match self.kind {
            BlockKind::Air | BlockKind::Water => VoxelShape::Empty,
            BlockKind::MagmaBlock | BlockKind::Other => VoxelShape::Full,
        }
    }
}

pub struct Blocks;

impl Blocks {
    pub const WATER: BlockKind = BlockKind::Water;
    pub const MAGMA_BLOCK: BlockKind = BlockKind::MagmaBlock;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelShape {
    Empty,
    Full,
}

pub struct Shapes;

impl Shapes {
    pub fn empty() -> VoxelShape {
        VoxelShape::Empty
    }
}

pub struct Block;

impl Block {
    pub fn is_shape_full_block(shape: VoxelShape) -> bool {
        shape == VoxelShape::Full
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

pub trait LevelAccessor {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

pub trait WorldGenLevel: LevelAccessor {
    fn set_block(&mut self, pos: &BlockPos, state: BlockState, flags: i32) -> bool;
}

#[derive(Clone, Debug)]
pub struct UnderwaterMagmaConfiguration {
    pub floor_search_range: i32,
    pub placement_radius_around_floor: i32,
    pub placement_probability_per_valid_position: f32,
}

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub config: &'a UnderwaterMagmaConfiguration,
    pub random: &'a mut R,
}

struct Column {
    floor: i32,
}

impl Column {
    fn scan<L: LevelAccessor + ?Sized>(
        level: &L,
        pos: BlockPos,
        range: i32,
        inside: &dyn Fn(&BlockState) -> bool,
        outside: &dyn Fn(&BlockState) -> bool,
    ) -> Option<Column> {
        let mut y = pos.y.wrapping_add(range);
        let mut remaining = range.wrapping_mul(2);
        loop {
            let p = BlockPos::new(pos.x, y, pos.z);
            let below_p = p.below();
            let state = level.get_block_state(&p);
            let below_state = level.get_block_state(&below_p);
            if inside(&state) && outside(&below_state) {
                return Some(Column {
                    floor: y.wrapping_sub(1),
                });
            }
            if remaining == 0 {
                break;
            }
            remaining = remaining.wrapping_sub(1);
            y = y.wrapping_sub(1);
        }
        None
    }

    fn get_floor(&self) -> Option<i32> {
        Some(self.floor)
    }
}

pub struct UnderwaterMagmaFeature;

impl UnderwaterMagmaFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#<init>
    pub fn new(_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let origin = ctx.origin;
        let config = ctx.config.clone();
        let floor_y_opt = Self::get_floor_y(ctx.level, origin, &config);
        if floor_y_opt.is_none() {
            return false;
        }

        let floor_y = floor_y_opt.unwrap();
        let pos = origin.at_y(floor_y);
        let radius = config.placement_radius_around_floor;
        let vec = Vec3i::new(radius, radius, radius);
        let bb = BoundingBox::from_corners(pos.subtract(vec), pos.offset(vec));
        let positions = BlockPos::between_closed_stream(&bb);
        let mut count: i32 = 0;
        for p in positions {
            if ctx.random.next_float() < config.placement_probability_per_valid_position
                && self.is_valid_placement(ctx.level, p)
            {
                ctx.level
                    .set_block(&p, Blocks::MAGMA_BLOCK.default_block_state(), 2);
                count = count.wrapping_add(1);
            }
        }
        count > 0
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#getFloorY
    fn get_floor_y<L: LevelAccessor + ?Sized>(
        level: &L,
        pos: BlockPos,
        config: &UnderwaterMagmaConfiguration,
    ) -> Option<i32> {
        let is_water = |s: &BlockState| s.is(Blocks::WATER);
        let not_water = |s: &BlockState| !s.is(Blocks::WATER);
        let col = Column::scan(
            level,
            pos,
            config.floor_search_range,
            &is_water,
            &not_water,
        );
        col.and_then(|c| c.get_floor())
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isValidPlacement
    fn is_valid_placement<L: WorldGenLevel + ?Sized>(&self, level: &L, pos: BlockPos) -> bool {
        if !Self::is_water_or_air(&level.get_block_state(&pos))
            && !self.is_visible_from_outside(level, pos.below(), Direction::Up)
        {
            for &dir in &Direction::HORIZONTAL {
                if self.is_visible_from_outside(level, pos.relative(dir), dir.get_opposite()) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isWaterOrAir
    fn is_water_or_air(state: &BlockState) -> bool {
        state.is(Blocks::WATER) || state.is_air()
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isVisibleFromOutside
    fn is_visible_from_outside<L: LevelAccessor + ?Sized>(
        &self,
        level: &L,
        pos: BlockPos,
        dir: Direction,
    ) -> bool {
        let state = level.get_block_state(&pos);
        let shape = state.get_face_occlusion_shape(dir);
        shape == Shapes::empty() || !Block::is_shape_full_block(shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                set_calls: Vec::new(),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, kind: BlockKind) {
            self.blocks.insert((x, y, z), BlockState { kind });
        }
    }

    impl LevelAccessor for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState {
                    kind: BlockKind::Air,
                })
        }
    }

    impl WorldGenLevel for MockLevel {
        fn set_block(&mut self, pos: &BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self.set_calls.push((*pos, state, flags));
            true
        }
    }

    struct SeqRandom {
        floats: Vec<f32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>) -> Self {
            Self { floats, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.idx % self.floats.len()];
            self.idx = self.idx.wrapping_add(1);
            v
        }
    }

    fn enclosed_floor_level() -> MockLevel {
        let mut level = MockLevel::new();
        level.set(0, 5, 0, BlockKind::Water);
        level.set(0, 4, 0, BlockKind::Other);
        level.set(0, 3, 0, BlockKind::Other);
        level.set(1, 4, 0, BlockKind::Other);
        level.set(-1, 4, 0, BlockKind::Other);
        level.set(0, 4, 1, BlockKind::Other);
        level.set(0, 4, -1, BlockKind::Other);
        level
    }

    #[test]
    fn place_returns_false_when_no_floor() {
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![0.0]);
        let config = UnderwaterMagmaConfiguration {
            floor_search_range: 5,
            placement_radius_around_floor: 1,
            placement_probability_per_valid_position: 1.0,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 10, 0),
            config: &config,
            random: &mut rng,
        };
        let feature = UnderwaterMagmaFeature::new(());
        assert!(!feature.place(&mut ctx));
    }

    #[test]
    fn places_magma_in_valid_enclosed_position() {
        let mut level = enclosed_floor_level();
        let mut rng = SeqRandom::new(vec![0.0]);
        let config = UnderwaterMagmaConfiguration {
            floor_search_range: 5,
            placement_radius_around_floor: 0,
            placement_probability_per_valid_position: 1.0,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 5, 0),
            config: &config,
            random: &mut rng,
        };
        let feature = UnderwaterMagmaFeature::new(());
        assert!(feature.place(&mut ctx));
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, 4, 0)).kind,
            BlockKind::MagmaBlock
        );
        assert_eq!(level.set_calls.len(), 1);
        assert_eq!(level.set_calls[0].2, 2);
    }

    #[test]
    fn is_valid_placement_rejects_water_air_and_exposed() {
        let mut level = MockLevel::new();
        let feature = UnderwaterMagmaFeature::new(());
        level.set(0, 0, 0, BlockKind::Water);
        assert!(!feature.is_valid_placement(&level, BlockPos::new(0, 0, 0)));

        level.set(1, 0, 0, BlockKind::Air);
        assert!(!feature.is_valid_placement(&level, BlockPos::new(1, 0, 0)));

        level.set(2, 0, 0, BlockKind::Other);
        level.set(2, -1, 0, BlockKind::Air);
        assert!(!feature.is_valid_placement(&level, BlockPos::new(2, 0, 0)));
    }

    #[test]
    fn probability_comparison_is_strictly_less_and_consumes_random() {
        let mut level = enclosed_floor_level();
        let mut rng = SeqRandom::new(vec![0.5]);
        let config = UnderwaterMagmaConfiguration {
            floor_search_range: 5,
            placement_radius_around_floor: 0,
            placement_probability_per_valid_position: 0.5,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 5, 0),
            config: &config,
            random: &mut rng,
        };
        let feature = UnderwaterMagmaFeature::new(());
        assert!(!feature.place(&mut ctx));
        assert_eq!(rng.idx, 1);
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, 4, 0)).kind,
            BlockKind::Other
        );
    }

    #[test]
    fn handles_negative_coordinates_and_wrapping_arithmetic() {
        let mut level = MockLevel::new();
        level.set(-10, -5, -20, BlockKind::Water);
        level.set(-10, -6, -20, BlockKind::Other);
        level.set(-10, -7, -20, BlockKind::Other);
        level.set(-9, -6, -20, BlockKind::Other);
        level.set(-11, -6, -20, BlockKind::Other);
        level.set(-10, -6, -19, BlockKind::Other);
        level.set(-10, -6, -21, BlockKind::Other);

        let mut rng = SeqRandom::new(vec![0.0_f32.to_bits() as f32]); // 0.0
        let config = UnderwaterMagmaConfiguration {
            floor_search_range: 5,
            placement_radius_around_floor: 0,
            placement_probability_per_valid_position: 1.0,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(-10, -5, -20),
            config: &config,
            random: &mut rng,
        };
        let feature = UnderwaterMagmaFeature::new(());
        assert!(feature.place(&mut ctx));
        assert_eq!(
            level.get_block_state(&BlockPos::new(-10, -6, -20)).kind,
            BlockKind::MagmaBlock
        );

        let min = BlockPos::new(i32::MIN, 0, 0);
        let below = min.below();
        assert_eq!(below.y, -1);
        let rel = min.relative(Direction::West);
        assert_eq!(rel.x, i32::MAX);
    }
}
