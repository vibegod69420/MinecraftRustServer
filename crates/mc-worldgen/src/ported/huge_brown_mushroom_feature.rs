use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    MushroomBlock,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MushroomProperty {
    West,
    East,
    North,
    South,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub west: Option<bool>,
    pub east: Option<bool>,
    pub north: Option<bool>,
    pub south: Option<bool>,
}

impl BlockState {
    pub const fn air() -> Self {
        Self {
            block: Block::Air,
            west: None,
            east: None,
            north: None,
            south: None,
        }
    }

    pub const fn mushroom_cap() -> Self {
        Self {
            block: Block::MushroomBlock,
            west: Some(true),
            east: Some(true),
            north: Some(true),
            south: Some(true),
        }
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#hasProperty
    pub fn has_property(&self, prop: MushroomProperty) -> bool {
        match prop {
            MushroomProperty::West => self.west.is_some(),
            MushroomProperty::East => self.east.is_some(),
            MushroomProperty::North => self.north.is_some(),
            MushroomProperty::South => self.south.is_some(),
        }
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_value(mut self, prop: MushroomProperty, value: bool) -> Self {
        match prop {
            MushroomProperty::West => self.west = Some(value),
            MushroomProperty::East => self.east = Some(value),
            MushroomProperty::North => self.north = Some(value),
            MushroomProperty::South => self.south = Some(value),
        }
        self
    }

    pub fn can_be_replaced(&self) -> bool {
        self.block == Block::Air
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#setWithOffset
    pub fn set_with_offset(&mut self, pos: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = pos.x.wrapping_add(dx);
        self.y = pos.y.wrapping_add(dy);
        self.z = pos.z.wrapping_add(dz);
        self
    }

    pub const fn as_block_pos(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait LevelAccessor {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub struct BlockStateProvider {
    pub state: BlockState,
}

impl BlockStateProvider {
    pub fn get_state<R: RandomSource + ?Sized>(&self, _random: &mut R, _pos: BlockPos) -> BlockState {
        self.state
    }
}

pub struct HugeMushroomFeatureConfiguration {
    pub foliage_radius: i32,
    pub cap_provider: BlockStateProvider,
}

pub struct HugeBrownMushroomFeature;

impl HugeBrownMushroomFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.HugeBrownMushroomFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.AbstractHugeMushroomFeature#placeMushroomBlock
    fn place_mushroom_block<L: LevelAccessor + ?Sized>(
        &self,
        level: &mut L,
        pos: BlockPos,
        state: BlockState,
    ) {
        let current = level.get_block_state(&pos);
        if current.can_be_replaced() {
            level.set_block(pos, state, 2);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.HugeBrownMushroomFeature#makeCap
    pub fn make_cap<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        random: &mut R,
        pos: BlockPos,
        height: i32,
        mutable_pos: &mut MutableBlockPos,
        config: &HugeMushroomFeatureConfiguration,
    ) {
        let foliage_radius = config.foliage_radius;

        let mut j = -foliage_radius;
        while j <= foliage_radius {
            let mut k = -foliage_radius;
            while k <= foliage_radius {
                let is_min_x = j == -foliage_radius;
                let is_max_x = j == foliage_radius;
                let is_min_z = k == -foliage_radius;
                let is_max_z = k == foliage_radius;
                let is_corner_x = is_min_x || is_max_x;
                let is_corner_z = is_min_z || is_max_z;
                if !is_corner_x || !is_corner_z {
                    mutable_pos.set_with_offset(pos, j, height, k);
                    let west = is_min_x || (is_corner_z && j == 1i32.wrapping_sub(foliage_radius));
                    let east = is_max_x || (is_corner_z && j == foliage_radius.wrapping_sub(1));
                    let north = is_min_z || (is_corner_x && k == 1i32.wrapping_sub(foliage_radius));
                    let south = is_max_z || (is_corner_x && k == foliage_radius.wrapping_sub(1));
                    let mut state = config.cap_provider.get_state(random, pos);
                    if state.has_property(MushroomProperty::West)
                        && state.has_property(MushroomProperty::East)
                        && state.has_property(MushroomProperty::North)
                        && state.has_property(MushroomProperty::South)
                    {
                        state = state
                            .set_value(MushroomProperty::West, west)
                            .set_value(MushroomProperty::East, east)
                            .set_value(MushroomProperty::North, north)
                            .set_value(MushroomProperty::South, south);
                    }

                    self.place_mushroom_block(level, mutable_pos.as_block_pos(), state);
                }
                k = k.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.HugeBrownMushroomFeature#getTreeRadiusForHeight
    pub fn get_tree_radius_for_height(
        &self,
        _p0: i32,
        _p1: i32,
        foliage_radius: i32,
        height: i32,
    ) -> i32 {
        if height <= 3 { 0 } else { foliage_radius }
    }
}

impl Default for HugeBrownMushroomFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        fn get(&self, x: i32, y: i32, z: i32) -> BlockState {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::air())
        }
    }

    impl LevelAccessor for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.get(pos.x, pos.y, pos.z)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    struct SeqRandom;

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    #[test]
    fn tree_radius_for_height_matches_java() {
        let f = HugeBrownMushroomFeature::new();
        assert_eq!(f.get_tree_radius_for_height(0, 0, 3, 3), 0);
        assert_eq!(f.get_tree_radius_for_height(0, 0, 3, 4), 3);
        assert_eq!(f.get_tree_radius_for_height(10, -5, 2, 0), 0);
        assert_eq!(f.get_tree_radius_for_height(1, 2, 5, 10), 5);
    }

    #[test]
    fn make_cap_skips_corners_and_places_expected_count() {
        let f = HugeBrownMushroomFeature::new();
        let mut level = MockLevel::new();
        let mut rng = SeqRandom;
        let pos = BlockPos::new(0, 0, 0);
        let mut mpos = MutableBlockPos::new(0, 0, 0);
        let config = HugeMushroomFeatureConfiguration {
            foliage_radius: 2,
            cap_provider: BlockStateProvider {
                state: BlockState::mushroom_cap(),
            },
        };
        f.make_cap(&mut level, &mut rng, pos, 5, &mut mpos, &config);
        assert_eq!(level.blocks.len(), 21); // 5x5 - 4 corners
        assert_eq!(level.get(-2, 5, -2), BlockState::air());
        assert_eq!(level.get(-2, 5, 2), BlockState::air());
        assert_eq!(level.get(2, 5, -2), BlockState::air());
        assert_eq!(level.get(2, 5, 2), BlockState::air());
    }

    #[test]
    fn make_cap_sets_face_properties_correctly() {
        let f = HugeBrownMushroomFeature::new();
        let mut level = MockLevel::new();
        let mut rng = SeqRandom;
        let pos = BlockPos::new(10, 20, 30);
        let mut mpos = MutableBlockPos::new(0, 0, 0);
        let config = HugeMushroomFeatureConfiguration {
            foliage_radius: 2,
            cap_provider: BlockStateProvider {
                state: BlockState::mushroom_cap(),
            },
        };
        f.make_cap(&mut level, &mut rng, pos, 3, &mut mpos, &config);

        let west_edge = level.get(8, 23, 30); // j=-2, k=0
        assert_eq!(west_edge.west, Some(true));
        assert_eq!(west_edge.east, Some(false));
        assert_eq!(west_edge.north, Some(false));
        assert_eq!(west_edge.south, Some(false));

        let north_inner = level.get(9, 23, 28); // j=-1, k=-2
        assert_eq!(north_inner.west, Some(true));
        assert_eq!(north_inner.east, Some(false));
        assert_eq!(north_inner.north, Some(true));
        assert_eq!(north_inner.south, Some(false));

        let center = level.get(10, 23, 30); // j=0, k=0
        assert_eq!(center.west, Some(false));
        assert_eq!(center.east, Some(false));
        assert_eq!(center.north, Some(false));
        assert_eq!(center.south, Some(false));
    }

    #[test]
    fn make_cap_does_not_set_properties_when_missing() {
        let f = HugeBrownMushroomFeature::new();
        let mut level = MockLevel::new();
        let mut rng = SeqRandom;
        let pos = BlockPos::new(0, 0, 0);
        let mut mpos = MutableBlockPos::new(0, 0, 0);
        let plain = BlockState {
            block: Block::Other,
            west: None,
            east: None,
            north: None,
            south: None,
        };
        let config = HugeMushroomFeatureConfiguration {
            foliage_radius: 1,
            cap_provider: BlockStateProvider { state: plain },
        };
        f.make_cap(&mut level, &mut rng, pos, 0, &mut mpos, &config);
        assert_eq!(level.blocks.len(), 5); // 3x3 - 4 corners
        let s = level.get(-1, 0, 0);
        assert_eq!(s.block, Block::Other);
        assert_eq!(s.west, None);
        assert_eq!(s.east, None);
        assert_eq!(s.north, None);
        assert_eq!(s.south, None);
    }

    #[test]
    fn mutable_block_pos_set_with_offset_wraps() {
        let mut m = MutableBlockPos::new(0, 0, 0);
        let base = BlockPos::new(i32::MAX, i32::MIN, 5);
        m.set_with_offset(base, 1, -1, 2);
        assert_eq!(m.x, i32::MIN);
        assert_eq!(m.y, i32::MAX);
        assert_eq!(m.z, 7);
        assert_eq!(m.as_block_pos(), BlockPos::new(i32::MIN, i32::MAX, 7));
    }

    #[test]
    fn place_mushroom_block_does_not_replace_solid() {
        let f = HugeBrownMushroomFeature::new();
        let mut level = MockLevel::new();
        let mut rng = SeqRandom;
        let pos = BlockPos::new(0, 0, 0);
        let mut mpos = MutableBlockPos::new(0, 0, 0);
        level.blocks.insert(
            (0, 1, 0),
            BlockState {
                block: Block::Other,
                west: None,
                east: None,
                north: None,
                south: None,
            },
        );
        let config = HugeMushroomFeatureConfiguration {
            foliage_radius: 0,
            cap_provider: BlockStateProvider {
                state: BlockState::mushroom_cap(),
            },
        };
        // radius 0 skips the single center cell by corner logic, so use radius 1 and check a specific cell
        let config1 = HugeMushroomFeatureConfiguration {
            foliage_radius: 1,
            cap_provider: BlockStateProvider {
                state: BlockState::mushroom_cap(),
            },
        };
        let _ = config; // silence unused
        f.make_cap(&mut level, &mut rng, pos, 1, &mut mpos, &config1);
        // (0,1,0) was solid Other, should remain unchanged
        assert_eq!(level.get(0, 1, 0).block, Block::Other);
    }
}
