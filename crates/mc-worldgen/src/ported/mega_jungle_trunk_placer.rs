use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#new
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#east
    pub fn east(&self) -> Self {
        Self {
            x: self.x.wrapping_add(1),
            y: self.y,
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#south
    pub fn south(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_add(1),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u32);

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

pub trait LevelSimulatedReader {
    fn is_air(&self, pos: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeConfiguration {
    pub trunk_state: BlockState,
    pub dirt_state: BlockState,
}

impl Default for TreeConfiguration {
    fn default() -> Self {
        Self {
            trunk_state: BlockState(1),
            dirt_state: BlockState(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

impl FoliageAttachment {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#<init>
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrunkPlacerTypeId {
    MegaJungleTrunkPlacer,
}

impl fmt::Display for TrunkPlacerTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrunkPlacerTypeId::MegaJungleTrunkPlacer => write!(f, "mega_jungle_trunk_placer"),
        }
    }
}

// JAVA: net.minecraft.util.Mth#cos
fn mth_cos(v: f32) -> f32 {
    (f64::from(v)).cos() as f32
}

// JAVA: net.minecraft.util.Mth#sin
fn mth_sin(v: f32) -> f32 {
    (f64::from(v)).sin() as f32
}

#[derive(Debug, Clone, Copy)]
pub struct MegaJungleTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl MegaJungleTrunkPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer#<init>
    pub fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer#type
    pub fn type_(&self) -> TrunkPlacerTypeId {
        TrunkPlacerTypeId::MegaJungleTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#setDirtAt
    pub fn set_dirt_at(
        _level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        _random: &mut dyn RandomSource,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) {
        block_setter(pos, config.dirt_state);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLog
    pub fn place_log(
        &self,
        _level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        _random: &mut dyn RandomSource,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> bool {
        block_setter(pos, config.trunk_state);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLogIfFree
    fn place_log_if_free(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> bool {
        self.place_log(level, block_setter, random, pos, config)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.GiantTrunkPlacer#placeLogIfFreeWithOffset
    fn place_log_if_free_with_offset(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        origin: BlockPos,
        x_off: i32,
        y_off: i32,
        z_off: i32,
    ) -> bool {
        let pos = origin.offset(x_off, y_off, z_off);
        self.place_log_if_free(level, block_setter, random, pos, config)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.GiantTrunkPlacer#placeTrunk
    fn giant_place_trunk(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        let below = pos.below();
        Self::set_dirt_at(level, block_setter, random, below, config);
        Self::set_dirt_at(level, block_setter, random, below.east(), config);
        Self::set_dirt_at(level, block_setter, random, below.south(), config);
        Self::set_dirt_at(level, block_setter, random, below.south().east(), config);

        let mut i: i32 = 0;
        while i < free_tree_height {
            self.place_log_if_free_with_offset(level, block_setter, random, config, pos, 0, i, 0);
            if i < free_tree_height.wrapping_sub(1) {
                self.place_log_if_free_with_offset(
                    level, block_setter, random, config, pos, 1, i, 0,
                );
                self.place_log_if_free_with_offset(
                    level, block_setter, random, config, pos, 1, i, 1,
                );
                self.place_log_if_free_with_offset(
                    level, block_setter, random, config, pos, 0, i, 1,
                );
            }
            i = i.wrapping_add(1);
        }

        vec![FoliageAttachment::new(
            pos.above(free_tree_height),
            0,
            true,
        )]
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer#placeTrunk
    pub fn place_trunk(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        let mut attachments: Vec<FoliageAttachment> = Vec::new();
        attachments.extend(self.giant_place_trunk(
            level,
            block_setter,
            random,
            free_tree_height,
            pos,
            config,
        ));

        let mut j: i32 = free_tree_height
            .wrapping_sub(2)
            .wrapping_sub(random.next_int(4));
        let half_height: i32 = free_tree_height / 2;
        while j > half_height {
            let angle: f32 = random.next_float() * ((std::f64::consts::PI * 2.0) as f32);
            let mut dx: i32 = 0;
            let mut dz: i32 = 0;

            let mut k: i32 = 0;
            while k < 5 {
                dx = (1.5f32 + mth_cos(angle) * (k as f32)) as i32;
                dz = (1.5f32 + mth_sin(angle) * (k as f32)) as i32;
                let branch_pos = pos.offset(dx, j.wrapping_sub(3).wrapping_add(k / 2), dz);
                self.place_log(level, block_setter, random, branch_pos, config);
                k = k.wrapping_add(1);
            }

            attachments.push(FoliageAttachment::new(pos.offset(dx, j, dz), -2, false));

            j = j.wrapping_sub(2_i32.wrapping_add(random.next_int(4)));
        }

        attachments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel;
    impl LevelSimulatedReader for MockLevel {
        fn is_air(&self, _pos: BlockPos) -> bool {
            true
        }
    }

    struct SeqRandom {
        ints: Vec<i32>,
        int_idx: usize,
        floats: Vec<f32>,
        float_idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                int_idx: 0,
                floats,
                float_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.ints[self.int_idx % self.ints.len()];
            self.int_idx += 1;
            v
        }
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx % self.floats.len()];
            self.float_idx += 1;
            v
        }
    }

    #[test]
    fn test_constructor_and_type() {
        let p = MegaJungleTrunkPlacer::new(10, 2, 19);
        assert_eq!(p.base_height, 10);
        assert_eq!(p.height_rand_a, 2);
        assert_eq!(p.height_rand_b, 19);
        assert_eq!(p.type_(), TrunkPlacerTypeId::MegaJungleTrunkPlacer);
    }

    #[test]
    fn test_place_trunk_branches_deterministic() {
        let placer = MegaJungleTrunkPlacer::new(10, 2, 19);
        let level = MockLevel;
        let mut random = SeqRandom::new(vec![0, 0, 0, 0], vec![0.0, 0.0]);
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 100, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 6, pos, &config);

        assert_eq!(placed[0], (BlockPos::new(0, 99, 0), config.dirt_state));
        assert_eq!(placed[1], (BlockPos::new(1, 99, 0), config.dirt_state));
        assert_eq!(placed[2], (BlockPos::new(0, 99, 1), config.dirt_state));
        assert_eq!(placed[3], (BlockPos::new(1, 99, 1), config.dirt_state));

        assert_eq!(
            attachments[0],
            FoliageAttachment::new(BlockPos::new(0, 106, 0), 0, true)
        );

        let branch_att = *attachments.last().unwrap();
        assert_eq!(
            branch_att,
            FoliageAttachment::new(BlockPos::new(5, 104, 1), -2, false)
        );

        assert!(placed.contains(&(BlockPos::new(1, 101, 1), config.trunk_state)));
        assert!(placed.contains(&(BlockPos::new(2, 101, 1), config.trunk_state)));
        assert!(placed.contains(&(BlockPos::new(3, 102, 1), config.trunk_state)));
        assert!(placed.contains(&(BlockPos::new(4, 102, 1), config.trunk_state)));
        assert!(placed.contains(&(BlockPos::new(5, 103, 1), config.trunk_state)));
    }

    #[test]
    fn test_place_trunk_small_height_no_branches() {
        let placer = MegaJungleTrunkPlacer::new(1, 0, 0);
        let level = MockLevel;
        let mut random = SeqRandom::new(vec![0], vec![0.0]);
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 10, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 3, pos, &config);

        // j = 3-2-0 = 1, half_height = 3/2 = 1, 1 > 1 is false
        assert_eq!(attachments.len(), 1);
        assert!(attachments[0].double_trunk);
        assert_eq!(attachments[0].pos, BlockPos::new(0, 13, 0));
        assert!(placed.len() >= 4);
    }

    #[test]
    fn test_place_trunk_negative_height() {
        let placer = MegaJungleTrunkPlacer::new(1, 0, 0);
        let level = MockLevel;
        let mut random = SeqRandom::new(vec![0], vec![0.0]);
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 10, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, -2, pos, &config);

        // Giant trunk places 4 dirt blocks; loop does not run for negative height.
        // j = -2-2-0 = -4, half_height = -2/2 = -1, -4 > -1 is false.
        assert_eq!(placed.len(), 4);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].pos, BlockPos::new(0, 8, 0));
    }

    #[test]
    fn test_branch_angle_float_order() {
        let placer = MegaJungleTrunkPlacer::new(10, 2, 19);
        let level = MockLevel;
        let mut random = SeqRandom::new(vec![0, 0], vec![0.5]);
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 64, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 10, pos, &config);

        let angle = 0.5f32 * ((std::f64::consts::PI * 2.0) as f32);
        let dx_k0 = (1.5f32 + mth_cos(angle) * 0.0f32) as i32;
        let dx_k1 = (1.5f32 + mth_cos(angle) * 1.0f32) as i32;
        let dz_k1 = (1.5f32 + mth_sin(angle) * 1.0f32) as i32;

        assert_eq!(dx_k0, 1);
        assert_eq!(dx_k1, 0);
        // sin(pi) is near 0 in f64->f32 conversion; result should be stable for this cast.
        assert_eq!(dz_k1, 1);
        assert!(attachments.len() >= 2);
    }

    #[test]
    fn test_giant_trunk_top_layer_single_log() {
        let placer = MegaJungleTrunkPlacer::new(1, 0, 0);
        let level = MockLevel;
        let mut random = SeqRandom::new(vec![3, 3], vec![0.0]); // large nextInt to avoid branches
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(5, 20, 7);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 2, pos, &config);

        // height 2: i=0 places 4 logs, i=1 places only (0,1,0) because i < 1 is false
        assert!(placed.contains(&(BlockPos::new(5, 20, 7), config.trunk_state))); // i=0, offset 0,0,0
        assert!(placed.contains(&(BlockPos::new(6, 20, 7), config.trunk_state))); // i=0, offset 1,0,0
        assert!(placed.contains(&(BlockPos::new(6, 20, 8), config.trunk_state))); // i=0, offset 1,0,1
        assert!(placed.contains(&(BlockPos::new(5, 20, 8), config.trunk_state))); // i=0, offset 0,0,1
        assert!(placed.contains(&(BlockPos::new(5, 21, 7), config.trunk_state))); // i=1, top only
        assert!(!placed.contains(&(BlockPos::new(6, 21, 7), config.trunk_state)));
        assert_eq!(attachments[0].double_trunk, true);
    }
}
