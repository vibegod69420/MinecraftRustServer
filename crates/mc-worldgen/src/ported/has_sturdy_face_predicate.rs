#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(&self, offset: &Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(offset.x),
            y: self.y.wrapping_add(offset.y),
            z: self.z.wrapping_add(offset.z),
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

/// Self-contained stub for `BlockState`.
/// `sturdy_faces` models the result of `isFaceSturdy` for testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
    pub sturdy_faces: Vec<Direction>,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            sturdy_faces: Vec::new(),
        }
    }

    pub fn with_sturdy_faces(mut self, faces: &[Direction]) -> Self {
        self.sturdy_faces = faces.to_vec();
        self
    }

    // Stub for Java BlockState#isFaceSturdy(BlockGetter, BlockPos, Direction).
    pub fn is_face_sturdy(
        &self,
        _level: &dyn WorldGenLevel,
        _pos: &BlockPos,
        direction: Direction,
    ) -> bool {
        self.sturdy_faces.contains(&direction)
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    HasSturdyFace,
}

pub trait BlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

/// Port of `net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate`.
#[derive(Debug, Clone)]
pub struct HasSturdyFacePredicate {
    offset: Vec3i,
    direction: Direction,
}

impl HasSturdyFacePredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate#<init>(Vec3i,Direction)
    pub fn new(offset: Vec3i, direction: Direction) -> Self {
        Self { offset, direction }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate#test(WorldGenLevel,BlockPos)
    pub fn test_at(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        let target = pos.offset(&self.offset);
        let state = level.get_block_state(&target);
        state.is_face_sturdy(level, &target, self.direction)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate#type()
    pub fn get_type(&self) -> BlockPredicateType {
        BlockPredicateType::HasSturdyFace
    }
}

impl BlockPredicate for HasSturdyFacePredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate#test(WorldGenLevel,BlockPos)
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.test_at(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate#type()
    fn predicate_type(&self) -> BlockPredicateType {
        self.get_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
        default: BlockState,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                states: HashMap::new(),
                default: BlockState::new(0),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
            self.states.insert((x, y, z), state);
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    #[test]
    fn sturdy_face_matches_direction() {
        let pred = HasSturdyFacePredicate::new(Vec3i::ZERO, Direction::Up);
        let mut level = MockLevel::new();
        level.set(
            1,
            2,
            3,
            BlockState::new(1).with_sturdy_faces(&[Direction::Up, Direction::North]),
        );
        assert!(pred.test(&level, &BlockPos::new(1, 2, 3)));
    }

    #[test]
    fn wrong_direction_fails() {
        let pred = HasSturdyFacePredicate::new(Vec3i::ZERO, Direction::Down);
        let mut level = MockLevel::new();
        level.set(0, 0, 0, BlockState::new(1).with_sturdy_faces(&[Direction::Up]));
        assert!(!pred.test(&level, &BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn applies_offset() {
        let pred = HasSturdyFacePredicate::new(Vec3i::new(0, -1, 0), Direction::Up);
        let mut level = MockLevel::new();
        level.set(5, 9, 5, BlockState::new(1).with_sturdy_faces(&[Direction::Up]));
        assert!(pred.test(&level, &BlockPos::new(5, 10, 5)));
    }

    #[test]
    fn negative_coordinates() {
        let pred = HasSturdyFacePredicate::new(Vec3i::new(-1, -1, -1), Direction::South);
        let mut level = MockLevel::new();
        level.set(
            -11,
            -21,
            -31,
            BlockState::new(1).with_sturdy_faces(&[Direction::South]),
        );
        assert!(pred.test(&level, &BlockPos::new(-10, -20, -30)));
    }

    #[test]
    fn offset_wrapping_overflow() {
        let pred = HasSturdyFacePredicate::new(Vec3i::new(1, 0, 0), Direction::East);
        let mut level = MockLevel::new();
        level.set(
            i32::MIN,
            0,
            0,
            BlockState::new(1).with_sturdy_faces(&[Direction::East]),
        );
        assert!(pred.test(&level, &BlockPos::new(i32::MAX, 0, 0)));
    }

    #[test]
    fn predicate_type_is_has_sturdy_face() {
        let pred = HasSturdyFacePredicate::new(Vec3i::ZERO, Direction::Up);
        assert_eq!(pred.predicate_type(), BlockPredicateType::HasSturdyFace);
    }
}
