use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

pub const NUMBER_OF_SPIKES: i32 = 10;
pub const SPIKE_DISTANCE: i32 = 42;

pub struct DimensionType;

impl DimensionType {
    pub const MIN_Y: i32 = -2032;
    pub const MAX_Y: i32 = 2032;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

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

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn below(&self) -> Self {
        Self::new(self.x, self.y.wrapping_sub(1), self.z)
    }

    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self::new(mth_floor(x), mth_floor(y), mth_floor(z))
    }

    pub fn dist_to_low_corner_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x as f64 - x;
        let dy = self.y as f64 - y;
        let dz = self.z as f64 - z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
        let min_x = from.x.min(to.x);
        let min_y = from.y.min(to.y);
        let min_z = from.z.min(to.z);
        let max_x = from.x.max(to.x);
        let max_y = from.y.max(to.y);
        let max_z = from.z.max(to.z);
        let mut out = Vec::new();
        let mut z = min_z;
        loop {
            let mut y = min_y;
            loop {
                let mut x = min_x;
                loop {
                    out.push(BlockPos::new(x, y, z));
                    if x == max_x {
                        break;
                    }
                    x = x.wrapping_add(1);
                }
                if y == max_y {
                    break;
                }
                y = y.wrapping_add(1);
            }
            if z == max_z {
                break;
            }
            z = z.wrapping_add(1);
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Obsidian,
    Bedrock,
    Fire,
    IronBars {
        north: bool,
        south: bool,
        west: bool,
        east: bool,
    },
}

#[derive(Clone, Debug)]
pub struct RandomSource {
    seed: u64,
}

impl RandomSource {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    pub fn create(seed: i64) -> Self {
        Self {
            seed: ((seed as u64) ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: i32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48u32.wrapping_sub(bits as u32))) as i32
    }

    pub fn next_long(&mut self) -> i64 {
        let hi = self.next(32) as i64;
        let lo = self.next(32) as i64;
        (hi << 32).wrapping_add(lo)
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & bound.wrapping_neg()) == bound {
            return ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits
                .wrapping_sub(val)
                .wrapping_add(bound.wrapping_sub(1))
                >= 0
            {
                return val;
            }
        }
    }

    pub fn next_float(&mut self) -> f32 {
        (self.next(24) as f32) / ((1i32 << 24) as f32)
    }
}

fn mth_floor(v: f64) -> i32 {
    let i = v as i32;
    if v < i as f64 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

fn mth_abs(v: i32) -> i32 {
    if v >= 0 {
        v
    } else {
        v.wrapping_neg()
    }
}

fn block_to_section_coord(block_coord: i32) -> i32 {
    block_coord >> 4
}

fn to_shuffled_list(start: i32, end: i32, random: &mut RandomSource) -> Vec<i32> {
    let mut list: Vec<i32> = (start..end).collect();
    let mut i = list.len() as i32;
    while i > 1 {
        let j = random.next_int(i);
        i = i.wrapping_sub(1);
        list.swap(i as usize, j as usize);
    }
    list
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndCrystal {
    pub beam_target: Option<BlockPos>,
    pub invulnerable: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
}

impl EndCrystal {
    pub fn new() -> Self {
        Self {
            beam_target: None,
            invulnerable: false,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
        }
    }

    pub fn set_beam_target(&mut self, target: Option<BlockPos>) {
        self.beam_target = target;
    }

    pub fn set_invulnerable(&mut self, invulnerable: bool) {
        self.invulnerable = invulnerable;
    }

    pub fn snap_to(&mut self, x: f64, y: f64, z: f64, y_rot: f32, x_rot: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.y_rot = y_rot;
        self.x_rot = x_rot;
    }

    pub fn block_position(&self) -> BlockPos {
        BlockPos::containing(self.x, self.y, self.z)
    }
}

impl Default for EndCrystal {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ServerLevelAccessor {
    fn get_min_y(&self) -> i32;
    fn set_block(&mut self, pos: BlockPos, state: Block, flags: i32);
    fn get_fire_state(&self, pos: BlockPos) -> Block;
    fn add_end_crystal(&mut self, crystal: EndCrystal) -> bool;
}

pub trait WorldGenLevel: ServerLevelAccessor {
    fn get_seed(&self) -> i64;
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndSpike {
    center_x: i32,
    center_z: i32,
    radius: i32,
    height: i32,
    guarded: bool,
    top_bounding_box: AABB,
}

impl EndSpike {
    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#<init>
    pub fn new(center_x: i32, center_z: i32, radius: i32, height: i32, guarded: bool) -> Self {
        let top_bounding_box = AABB::new(
            center_x.wrapping_sub(radius) as f64,
            DimensionType::MIN_Y as f64,
            center_z.wrapping_sub(radius) as f64,
            center_x.wrapping_add(radius) as f64,
            DimensionType::MAX_Y as f64,
            center_z.wrapping_add(radius) as f64,
        );
        Self {
            center_x,
            center_z,
            radius,
            height,
            guarded,
            top_bounding_box,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#isCenterWithinChunk
    pub fn is_center_within_chunk(&self, pos: BlockPos) -> bool {
        block_to_section_coord(pos.get_x()) == block_to_section_coord(self.center_x)
            && block_to_section_coord(pos.get_z()) == block_to_section_coord(self.center_z)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#getCenterX
    pub fn get_center_x(&self) -> i32 {
        self.center_x
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#getCenterZ
    pub fn get_center_z(&self) -> i32 {
        self.center_z
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#getRadius
    pub fn get_radius(&self) -> i32 {
        self.radius
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#getHeight
    pub fn get_height(&self) -> i32 {
        self.height
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#isGuarded
    pub fn is_guarded(&self) -> bool {
        self.guarded
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$EndSpike#getTopBoundingBox
    pub fn get_top_bounding_box(&self) -> AABB {
        self.top_bounding_box
    }
}

#[derive(Clone, Debug)]
pub struct SpikeConfiguration {
    spikes: Vec<EndSpike>,
    crystal_beam_target: Option<BlockPos>,
    crystal_invulnerable: bool,
}

impl SpikeConfiguration {
    pub fn new(
        spikes: Vec<EndSpike>,
        crystal_beam_target: Option<BlockPos>,
        crystal_invulnerable: bool,
    ) -> Self {
        Self {
            spikes,
            crystal_beam_target,
            crystal_invulnerable,
        }
    }

    pub fn get_spikes(&self) -> &[EndSpike] {
        &self.spikes
    }

    pub fn get_crystal_beam_target(&self) -> Option<BlockPos> {
        self.crystal_beam_target
    }

    pub fn is_crystal_invulnerable(&self) -> bool {
        self.crystal_invulnerable
    }
}

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized> {
    pub config: &'a SpikeConfiguration,
    pub level: &'a mut L,
    pub random: &'a mut RandomSource,
    pub origin: BlockPos,
}

struct SpikeCacheEntry {
    inserted: Instant,
    spikes: Vec<EndSpike>,
}

static SPIKE_CACHE: LazyLock<Mutex<HashMap<i64, SpikeCacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature$SpikeCacheLoader#load
fn spike_cache_load(key: i64) -> Vec<EndSpike> {
    let mut random = RandomSource::create(key);
    let shuffled = to_shuffled_list(0, 10, &mut random);
    let mut spikes = Vec::with_capacity(10);
    let mut i: i32 = 0;
    while i < 10 {
        let angle = 2.0f64
            * (-std::f64::consts::PI + (std::f64::consts::PI / 10.0f64) * (i as f64));
        let cx = mth_floor(42.0f64 * angle.cos());
        let cz = mth_floor(42.0f64 * angle.sin());
        let val = shuffled[i as usize];
        let radius = 2i32.wrapping_add(val / 3);
        let height = 76i32.wrapping_add(val.wrapping_mul(3));
        let guarded = val == 1 || val == 2;
        spikes.push(EndSpike::new(cx, cz, radius, height, guarded));
        i = i.wrapping_add(1);
    }
    spikes
}

fn spike_cache_get(key: i64) -> Vec<EndSpike> {
    let mut cache = SPIKE_CACHE.lock().expect("spike cache lock poisoned");
    let now = Instant::now();
    if let Some(entry) = cache.get(&key) {
        if now.duration_since(entry.inserted) < Duration::from_secs(5 * 60) {
            return entry.spikes.clone();
        }
    }
    let spikes = spike_cache_load(key);
    cache.insert(
        key,
        SpikeCacheEntry {
            inserted: now,
            spikes: spikes.clone(),
        },
    );
    spikes
}

pub struct SpikeFeature;

impl SpikeFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature#getSpikesForLevel
    pub fn get_spikes_for_level<L: WorldGenLevel + ?Sized>(level: &L) -> Vec<EndSpike> {
        let mut random = RandomSource::create(level.get_seed());
        let key = random.next_long() & 65535i64;
        spike_cache_get(key)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature#place
    pub fn place<L: WorldGenLevel + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L>,
    ) -> bool {
        let mut spikes: Vec<EndSpike> = ctx.config.get_spikes().to_vec();
        if spikes.is_empty() {
            spikes = Self::get_spikes_for_level(ctx.level);
        }

        for spike in &spikes {
            if spike.is_center_within_chunk(ctx.origin) {
                self.place_spike(ctx.level, ctx.random, ctx.config, spike);
            }
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature#setBlock
    fn set_block<L: ServerLevelAccessor + ?Sized>(
        &self,
        level: &mut L,
        pos: BlockPos,
        state: Block,
    ) {
        level.set_block(pos, state, 2);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpikeFeature#placeSpike
    fn place_spike<L: ServerLevelAccessor + ?Sized>(
        &self,
        level: &mut L,
        random: &mut RandomSource,
        config: &SpikeConfiguration,
        spike: &EndSpike,
    ) {
        let radius = spike.get_radius();
        let center_x = spike.get_center_x();
        let center_z = spike.get_center_z();
        let height = spike.get_height();
        let min_y = level.get_min_y();

        let from = BlockPos::new(
            center_x.wrapping_sub(radius),
            min_y,
            center_z.wrapping_sub(radius),
        );
        let to = BlockPos::new(
            center_x.wrapping_add(radius),
            height.wrapping_add(10),
            center_z.wrapping_add(radius),
        );
        let r2_plus_1 = radius.wrapping_mul(radius).wrapping_add(1) as f64;

        for pos in BlockPos::between_closed(from, to) {
            let dist = pos.dist_to_low_corner_sqr(
                center_x as f64,
                pos.get_y() as f64,
                center_z as f64,
            );
            if dist <= r2_plus_1 && pos.get_y() < height {
                self.set_block(level, pos, Block::Obsidian);
            } else if pos.get_y() > 65 {
                self.set_block(level, pos, Block::Air);
            }
        }

        if spike.is_guarded() {
            let mut dx: i32 = -2;
            while dx <= 2 {
                let mut dz: i32 = -2;
                while dz <= 2 {
                    let mut dy: i32 = 0;
                    while dy <= 3 {
                        let flag_x = mth_abs(dx) == 2;
                        let flag_z = mth_abs(dz) == 2;
                        let flag_top = dy == 3;
                        if flag_x || flag_z || flag_top {
                            let flag16 = dx == -2 || dx == 2 || flag_top;
                            let flag17 = dz == -2 || dz == 2 || flag_top;
                            let state = Block::IronBars {
                                north: flag16 && dz != -2,
                                south: flag16 && dz != 2,
                                west: flag17 && dx != -2,
                                east: flag17 && dx != 2,
                            };
                            let p = BlockPos::new(
                                center_x.wrapping_add(dx),
                                height.wrapping_add(dy),
                                center_z.wrapping_add(dz),
                            );
                            self.set_block(level, p, state);
                        }
                        dy = dy.wrapping_add(1);
                    }
                    dz = dz.wrapping_add(1);
                }
                dx = dx.wrapping_add(1);
            }
        }

        let mut crystal = EndCrystal::new();
        crystal.set_beam_target(config.get_crystal_beam_target());
        crystal.set_invulnerable(config.is_crystal_invulnerable());
        crystal.snap_to(
            center_x as f64 + 0.5,
            height.wrapping_add(1) as f64,
            center_z as f64 + 0.5,
            random.next_float() * 360.0f32,
            0.0f32,
        );
        if level.add_end_crystal(crystal.clone()) {
            let crystal_pos = crystal.block_position();
            self.set_block(level, crystal_pos.below(), Block::Bedrock);
            let fire = level.get_fire_state(crystal_pos);
            self.set_block(level, crystal_pos, fire);
        }
    }
}

impl Default for SpikeFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        min_y: i32,
        seed: i64,
        blocks: HashMap<(i32, i32, i32), Block>,
        crystals: Vec<EndCrystal>,
    }

    impl MockLevel {
        fn new(min_y: i32, seed: i64) -> Self {
            Self {
                min_y,
                seed,
                blocks: HashMap::new(),
                crystals: Vec::new(),
            }
        }

        fn block_at(&self, pos: BlockPos) -> Option<Block> {
            self.blocks.get(&(pos.x, pos.y, pos.z)).copied()
        }
    }

    impl ServerLevelAccessor for MockLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn set_block(&mut self, pos: BlockPos, state: Block, _flags: i32) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }

        fn get_fire_state(&self, _pos: BlockPos) -> Block {
            Block::Fire
        }

        fn add_end_crystal(&mut self, crystal: EndCrystal) -> bool {
            self.crystals.push(crystal);
            true
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_seed(&self) -> i64 {
            self.seed
        }
    }

    #[test]
    fn spike_cache_load_matches_formula_and_float_order() {
        let spikes = spike_cache_load(0);
        assert_eq!(spikes.len(), 10);
        let mut random = RandomSource::create(0);
        let shuffled = to_shuffled_list(0, 10, &mut random);
        for i in 0..10usize {
            let idx = i as i32;
            let angle = 2.0f64
                * (-std::f64::consts::PI + (std::f64::consts::PI / 10.0f64) * (idx as f64));
            let cx = mth_floor(42.0f64 * angle.cos());
            let cz = mth_floor(42.0f64 * angle.sin());
            let val = shuffled[i];
            assert_eq!(spikes[i].get_center_x(), cx);
            assert_eq!(spikes[i].get_center_z(), cz);
            assert_eq!(spikes[i].get_radius(), 2i32.wrapping_add(val / 3));
            assert_eq!(spikes[i].get_height(), 76i32.wrapping_add(val.wrapping_mul(3)));
            assert_eq!(spikes[i].is_guarded(), val == 1 || val == 2);
        }
    }

    #[test]
    fn end_spike_center_within_chunk_negative() {
        let spike = EndSpike::new(-33, 48, 2, 76, false);
        assert!(spike.is_center_within_chunk(BlockPos::new(-48, 0, 48)));
        assert!(spike.is_center_within_chunk(BlockPos::new(-33, 100, 63)));
        assert!(!spike.is_center_within_chunk(BlockPos::new(-32, 0, 48)));
    }

    #[test]
    fn end_spike_aabb_wrapping_overflow() {
        let spike = EndSpike::new(i32::MAX, i32::MIN, 10, 76, false);
        let aabb = spike.get_top_bounding_box();
        assert_eq!(aabb.min_x, i32::MAX.wrapping_sub(10) as f64);
        assert_eq!(aabb.max_x, i32::MAX.wrapping_add(10) as f64);
        assert_eq!(aabb.min_z, i32::MIN.wrapping_sub(10) as f64);
        assert_eq!(aabb.max_z, i32::MIN.wrapping_add(10) as f64);
        assert_eq!(aabb.min_y, DimensionType::MIN_Y as f64);
        assert_eq!(aabb.max_y, DimensionType::MAX_Y as f64);
    }

    #[test]
    fn place_spike_places_blocks_cage_and_crystal() {
        let mut level = MockLevel::new(0, 0);
        let mut random = RandomSource::create(12345);
        let config =
            SpikeConfiguration::new(vec![], Some(BlockPos::new(1, 2, 3)), true);
        let spike = EndSpike::new(0, 0, 3, 80, true);
        let feature = SpikeFeature::new();
        feature.place_spike(&mut level, &mut random, &config, &spike);

        assert_eq!(
            level.block_at(BlockPos::new(0, 0, 0)),
            Some(Block::Obsidian)
        );
        assert_eq!(
            level.block_at(BlockPos::new(2, 79, 0)),
            Some(Block::Obsidian)
        );
        assert_eq!(level.block_at(BlockPos::new(3, 80, 3)), Some(Block::Air));

        assert_eq!(
            level.block_at(BlockPos::new(-2, 83, -2)),
            Some(Block::IronBars {
                north: false,
                south: true,
                west: false,
                east: true,
            })
        );

        assert_eq!(level.crystals.len(), 1);
        let c = &level.crystals[0];
        assert_eq!(c.beam_target, Some(BlockPos::new(1, 2, 3)));
        assert!(c.invulnerable);
        assert_eq!(c.x, 0.5);
        assert_eq!(c.y, 81.0);
        assert_eq!(c.z, 0.5);
        assert_eq!(c.x_rot.to_bits(), 0.0f32.to_bits());
        let mut r2 = RandomSource::create(12345);
        let expected_y_rot = r2.next_float() * 360.0f32;
        assert_eq!(c.y_rot.to_bits(), expected_y_rot.to_bits());

        let cp = c.block_position();
        assert_eq!(level.block_at(cp.below()), Some(Block::Bedrock));
        assert_eq!(level.block_at(cp), Some(Block::Fire));
    }

    #[test]
    fn place_falls_back_to_level_spikes() {
        let mut level = MockLevel::new(-64, 42);
        let spikes = SpikeFeature::get_spikes_for_level(&level);
        let s0 = &spikes[0];
        let origin = BlockPos::new(s0.get_center_x(), 0, s0.get_center_z());
        let config = SpikeConfiguration::new(vec![], None, false);
        let mut random = RandomSource::create(7);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            random: &mut random,
            origin,
        };
        let feature = SpikeFeature::new();
        assert!(feature.place(&mut ctx));
        assert!(!ctx.level.crystals.is_empty());
    }

    #[test]
    fn between_closed_order_negative_and_mth_edges() {
        let positions =
            BlockPos::between_closed(BlockPos::new(0, 0, 0), BlockPos::new(1, 1, 1));
        assert_eq!(
            positions,
            vec![
                BlockPos::new(0, 0, 0),
                BlockPos::new(1, 0, 0),
                BlockPos::new(0, 1, 0),
                BlockPos::new(1, 1, 0),
                BlockPos::new(0, 0, 1),
                BlockPos::new(1, 0, 1),
                BlockPos::new(0, 1, 1),
                BlockPos::new(1, 1, 1),
            ]
        );
        let p = BlockPos::new(-1, -2, -3);
        assert_eq!(
            p.dist_to_low_corner_sqr(-1.0, -2.0, -3.0).to_bits(),
            0.0f64.to_bits()
        );
        assert_eq!(mth_abs(i32::MIN), i32::MIN);
        assert_eq!(mth_floor(-0.1), -1);
        assert_eq!(mth_floor(-1.0), -1);
    }
}
