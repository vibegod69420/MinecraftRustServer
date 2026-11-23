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

    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    pub fn relative(&self, dir: Direction) -> Self {
        let n = dir.normal();
        Self {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
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
    pub fn normal(self) -> BlockPos {
        match self {
            Direction::Down => BlockPos::new(0, -1, 0),
            Direction::Up => BlockPos::new(0, 1, 0),
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
        }
    }

    pub fn horizontal() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Water,
    SeaPickle,
    CoralBlock,
    Coral,
    WallCoral,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
    pub pickles: i32,
    pub facing: Option<Direction>,
    pub has_facing_property: bool,
}

impl BlockState {
    pub fn air() -> Self {
        Self {
            kind: BlockKind::Air,
            pickles: 0,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn water() -> Self {
        Self {
            kind: BlockKind::Water,
            pickles: 0,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn sea_pickle() -> Self {
        Self {
            kind: BlockKind::SeaPickle,
            pickles: 1,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn coral_block() -> Self {
        Self {
            kind: BlockKind::CoralBlock,
            pickles: 0,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn coral() -> Self {
        Self {
            kind: BlockKind::Coral,
            pickles: 0,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn wall_coral() -> Self {
        Self {
            kind: BlockKind::WallCoral,
            pickles: 0,
            facing: None,
            has_facing_property: false,
        }
    }

    pub fn is_water(&self) -> bool {
        self.kind == BlockKind::Water
    }

    pub fn is_coral_tag(&self) -> bool {
        matches!(self.kind, BlockKind::Coral | BlockKind::WallCoral)
    }

    pub fn has_facing_property(&self) -> bool {
        self.has_facing_property
    }

    pub fn set_facing(mut self, dir: Direction) -> Self {
        self.facing = Some(dir);
        self
    }

    pub fn set_pickles(mut self, count: i32) -> Self {
        self.pickles = count;
        self
    }
}

pub trait LevelAccess {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait BlockRegistry {
    fn get_random_coral_block(&mut self, random: &mut dyn RandomSource) -> Option<BlockState>;
    fn get_random_coral(&mut self, random: &mut dyn RandomSource) -> Option<BlockState>;
    fn get_random_wall_coral(&mut self, random: &mut dyn RandomSource) -> Option<BlockState>;
}

pub struct FeaturePlaceContext<'a> {
    pub level: &'a mut dyn LevelAccess,
    pub origin: BlockPos,
    pub random: &'a mut dyn RandomSource,
    pub registry: &'a mut dyn BlockRegistry,
}

pub struct CoralFeature<F> {
    place_feature_fn: F,
}

impl<F> CoralFeature<F>
where
    F: FnMut(
        &mut dyn LevelAccess,
        &mut dyn RandomSource,
        &mut dyn BlockRegistry,
        BlockPos,
        BlockState,
    ) -> bool,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.CoralFeature#<init>
    pub fn new(place_feature_fn: F) -> Self {
        Self { place_feature_fn }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.CoralFeature#place
    pub fn place(&mut self, ctx: &mut FeaturePlaceContext<'_>) -> bool {
        let coral_block = ctx.registry.get_random_coral_block(&mut *ctx.random);
        match coral_block {
            None => false,
            Some(state) => self.place_feature(
                &mut *ctx.level,
                &mut *ctx.random,
                &mut *ctx.registry,
                ctx.origin,
                state,
            ),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.CoralFeature#placeFeature
    pub fn place_feature(
        &mut self,
        level: &mut dyn LevelAccess,
        random: &mut dyn RandomSource,
        registry: &mut dyn BlockRegistry,
        pos: BlockPos,
        state: BlockState,
    ) -> bool {
        (self.place_feature_fn)(level, random, registry, pos, state)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.CoralFeature#placeCoralBlock
    pub fn place_coral_block(
        &self,
        level: &mut dyn LevelAccess,
        random: &mut dyn RandomSource,
        registry: &mut dyn BlockRegistry,
        pos: BlockPos,
        state: BlockState,
    ) -> bool {
        let above = pos.above();
        let pos_state = level.get_block_state(pos);
        if (pos_state.is_water() || pos_state.is_coral_tag())
            && level.get_block_state(above).is_water()
        {
            level.set_block(pos, state, 3);
            let r = random.next_float();
            if r < 0.25f32 {
                if let Some(coral) = registry.get_random_coral(random) {
                    level.set_block(above, coral, 2);
                }
            } else if random.next_float() < 0.05f32 {
                let pickles = random.next_int(4).wrapping_add(1);
                let sea_pickle = BlockState::sea_pickle().set_pickles(pickles);
                level.set_block(above, sea_pickle, 2);
            }

            for dir in Direction::horizontal() {
                if random.next_float() < 0.2f32 {
                    let rel = pos.relative(dir);
                    if level.get_block_state(rel).is_water() {
                        if let Some(wall_coral) = registry.get_random_wall_coral(random) {
                            let mut ws = wall_coral;
                            if ws.has_facing_property() {
                                ws = ws.set_facing(dir);
                            }
                            level.set_block(rel, ws, 2);
                        }
                    }
                }
            }

            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::BTreeMap;

    struct MockLevel {
        blocks: BTreeMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: BTreeMap::new(),
            }
        }

        fn set(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    impl LevelAccess for MockLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(BlockState::air)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    struct SeqRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        fi: usize,
        ii: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                fi: 0,
                ii: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.fi).copied().unwrap_or(0.0);
            self.fi += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.ii).copied().unwrap_or(0);
            self.ii += 1;
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    struct MockRegistry {
        coral_block: Option<BlockState>,
        coral: Option<BlockState>,
        wall_coral: Option<BlockState>,
    }

    impl BlockRegistry for MockRegistry {
        fn get_random_coral_block(&mut self, _random: &mut dyn RandomSource) -> Option<BlockState> {
            self.coral_block.clone()
        }

        fn get_random_coral(&mut self, _random: &mut dyn RandomSource) -> Option<BlockState> {
            self.coral.clone()
        }

        fn get_random_wall_coral(&mut self, _random: &mut dyn RandomSource) -> Option<BlockState> {
            self.wall_coral.clone()
        }
    }

    #[test]
    fn place_returns_false_when_no_coral_block() {
        let mut feature = CoralFeature::new(|_, _, _, _, _| true);
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![], vec![]);
        let mut reg = MockRegistry {
            coral_block: None,
            coral: None,
            wall_coral: None,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 0, 0),
            random: &mut rng,
            registry: &mut reg,
        };
        assert!(!feature.place(&mut ctx));
    }

    #[test]
    fn place_coral_block_fails_when_not_water() {
        let feature = CoralFeature::new(|_, _, _, _, _| true);
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![], vec![]);
        let mut reg = MockRegistry {
            coral_block: None,
            coral: None,
            wall_coral: None,
        };
        let pos = BlockPos::new(0, 64, 0);
        assert!(!feature.place_coral_block(
            &mut level,
            &mut rng,
            &mut reg,
            pos,
            BlockState::coral_block(),
        ));
    }

    #[test]
    fn place_coral_block_places_sea_pickle() {
        let feature = CoralFeature::new(|_, _, _, _, _| true);
        let mut level = MockLevel::new();
        let pos = BlockPos::new(0, 64, 0);
        level.set(pos, BlockState::water());
        level.set(pos.above(), BlockState::water());
        // top: 0.5 >= 0.25, else 0.01 < 0.05 => sea pickle, pickles = nextInt(4)+1 = 3
        // horizontal: 4 floats all >= 0.2
        let mut rng = SeqRandom::new(vec![0.5, 0.01, 0.5, 0.5, 0.5, 0.5], vec![2]);
        let mut reg = MockRegistry {
            coral_block: None,
            coral: None,
            wall_coral: None,
        };
        assert!(feature.place_coral_block(
            &mut level,
            &mut rng,
            &mut reg,
            pos,
            BlockState::coral_block(),
        ));
        assert_eq!(level.get_block_state(pos).kind, BlockKind::CoralBlock);
        let above = level.get_block_state(pos.above());
        assert_eq!(above.kind, BlockKind::SeaPickle);
        assert_eq!(above.pickles, 3);
    }

    #[test]
    fn place_coral_block_places_wall_coral_with_facing() {
        let feature = CoralFeature::new(|_, _, _, _, _| true);
        let mut level = MockLevel::new();
        let pos = BlockPos::new(0, 64, 0);
        level.set(pos, BlockState::water());
        level.set(pos.above(), BlockState::water());
        let north = pos.relative(Direction::North);
        level.set(north, BlockState::water());
        // top: 0.5, 0.1 (no pickle); horizontal: North 0.1, East 0.5, South 0.5, West 0.5
        let mut rng = SeqRandom::new(vec![0.5, 0.1, 0.1, 0.5, 0.5, 0.5], vec![]);
        let mut wall = BlockState::wall_coral();
        wall.has_facing_property = true;
        let mut reg = MockRegistry {
            coral_block: None,
            coral: None,
            wall_coral: Some(wall),
        };
        assert!(feature.place_coral_block(
            &mut level,
            &mut rng,
            &mut reg,
            pos,
            BlockState::coral_block(),
        ));
        let placed = level.get_block_state(north);
        assert_eq!(placed.kind, BlockKind::WallCoral);
        assert_eq!(placed.facing, Some(Direction::North));
    }

    #[test]
    fn place_delegates_to_place_feature() {
        let called = Cell::new(false);
        let mut feature = CoralFeature::new(|_, _, _, p, s| {
            called.set(true);
            assert_eq!(p, BlockPos::new(1, 2, 3));
            assert_eq!(s.kind, BlockKind::CoralBlock);
            true
        });
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![], vec![]);
        let mut reg = MockRegistry {
            coral_block: Some(BlockState::coral_block()),
            coral: None,
            wall_coral: None,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(1, 2, 3),
            random: &mut rng,
            registry: &mut reg,
        };
        assert!(feature.place(&mut ctx));
        assert!(called.get());
    }
}
