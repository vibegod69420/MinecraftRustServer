#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#relative
    pub fn relative(&self, direction: Direction) -> Self {
        let (dx, dy, dz) = direction.normal();
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#values
    pub fn values() -> [Direction; 6] {
        [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
    }

    fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VineFaceProperty {
    Up,
    North,
    South,
    West,
    East,
}

// JAVA: net.minecraft.world.level.block.VineBlock#getPropertyForFace
pub fn get_property_for_face(direction: Direction) -> Option<VineFaceProperty> {
    match direction {
        Direction::Up => Some(VineFaceProperty::Up),
        Direction::North => Some(VineFaceProperty::North),
        Direction::South => Some(VineFaceProperty::South),
        Direction::West => Some(VineFaceProperty::West),
        Direction::East => Some(VineFaceProperty::East),
        Direction::Down => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState {
    pub up: bool,
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_value(mut self, prop: VineFaceProperty, value: bool) -> Self {
        match prop {
            VineFaceProperty::Up => self.up = value,
            VineFaceProperty::North => self.north = value,
            VineFaceProperty::South => self.south = value,
            VineFaceProperty::West => self.west = value,
            VineFaceProperty::East => self.east = value,
        }
        self
    }
}

pub struct Blocks;

impl Blocks {
    // JAVA: net.minecraft.world.level.block.Blocks#VINE (defaultBlockState)
    pub fn vine_default_state() -> BlockState {
        BlockState::default()
    }
}

pub trait WorldGenLevel {
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn is_acceptable_vine_neighbour(&self, neighbor_pos: BlockPos, direction: Direction) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub config: NoneFeatureConfiguration,
}

pub struct VinesFeature;

impl VinesFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.VinesFeature#<init>
    pub fn new(_config_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.VinesFeature#place
    pub fn place<L: WorldGenLevel>(&self, ctx: &mut FeaturePlaceContext<'_, L>) -> bool {
        let origin = ctx.origin;
        let _ = &ctx.config;
        if !ctx.level.is_empty_block(origin) {
            return false;
        }

        for direction in Direction::values() {
            if direction != Direction::Down {
                let neighbor_pos = origin.relative(direction);
                if ctx.level.is_acceptable_vine_neighbour(neighbor_pos, direction) {
                    let state = Blocks::vine_default_state().set_value(
                        get_property_for_face(direction).expect("valid vine face"),
                        true,
                    );
                    ctx.level.set_block(origin, state, 2);
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockLevel {
        empty: HashSet<BlockPos>,
        acceptable: HashSet<(BlockPos, Direction)>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
    }

    impl MockLevel {
        fn new(empty: &[BlockPos]) -> Self {
            Self {
                empty: empty.iter().copied().collect(),
                acceptable: HashSet::new(),
                set_calls: Vec::new(),
            }
        }

        fn with_acceptable(mut self, pos: BlockPos, dir: Direction) -> Self {
            self.acceptable.insert((pos, dir));
            self
        }
    }

    impl WorldGenLevel for MockLevel {
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.empty.contains(&pos)
        }

        fn is_acceptable_vine_neighbour(&self, neighbor_pos: BlockPos, direction: Direction) -> bool {
            self.acceptable.contains(&(neighbor_pos, direction))
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.set_calls.push((pos, state, flags));
            true
        }
    }

    #[test]
    fn returns_false_when_origin_not_empty() {
        let mut level = MockLevel::new(&[]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 64, 0),
            config: NoneFeatureConfiguration,
        };
        assert!(!VinesFeature::new(()).place(&mut ctx));
        assert!(ctx.level.set_calls.is_empty());
    }

    #[test]
    fn places_vine_on_first_acceptable_direction() {
        let origin = BlockPos::new(1, 2, 3);
        let north_neighbor = origin.relative(Direction::North);
        let mut level = MockLevel::new(&[origin]).with_acceptable(north_neighbor, Direction::North);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            config: NoneFeatureConfiguration,
        };
        assert!(VinesFeature::new(()).place(&mut ctx));
        assert_eq!(ctx.level.set_calls.len(), 1);
        let (pos, state, flags) = ctx.level.set_calls[0];
        assert_eq!(pos, origin);
        assert_eq!(flags, 2);
        assert!(state.north);
        assert!(!state.up && !state.south && !state.west && !state.east);
    }

    #[test]
    fn skips_down_even_if_acceptable() {
        let origin = BlockPos::new(0, 10, 0);
        let down_neighbor = origin.relative(Direction::Down);
        let mut level = MockLevel::new(&[origin]).with_acceptable(down_neighbor, Direction::Down);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            config: NoneFeatureConfiguration,
        };
        assert!(!VinesFeature::new(()).place(&mut ctx));
        assert!(ctx.level.set_calls.is_empty());
    }

    #[test]
    fn returns_false_when_no_acceptable_neighbor() {
        let origin = BlockPos::new(5, 5, 5);
        let mut level = MockLevel::new(&[origin]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            config: NoneFeatureConfiguration,
        };
        assert!(!VinesFeature::new(()).place(&mut ctx));
    }

    #[test]
    fn relative_wraps_like_java_ints() {
        let pos = BlockPos::new(i32::MAX, 0, i32::MIN);
        let east = pos.relative(Direction::East);
        let north = pos.relative(Direction::North);
        assert_eq!(east.x, i32::MIN);
        assert_eq!(north.z, i32::MAX);
    }
}
