// === FILE: canyon_world_carver.rs ===

use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn get_block_x(&self, offset: i32) -> i32 {
        self.x.wrapping_shl(4).wrapping_add(offset)
    }

    pub fn get_block_z(&self, offset: i32) -> i32 {
        self.z.wrapping_shl(4).wrapping_add(offset)
    }

    pub fn get_middle_block_x(&self) -> i32 {
        self.get_block_x(8)
    }

    pub fn get_middle_block_z(&self) -> i32 {
        self.get_block_z(8)
    }
}

#[derive(Clone, Debug)]
pub struct RandomSource {
    seed: i64,
}

impl RandomSource {
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1_i64 << 48) - 1;

    pub fn create(seed: i64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: i32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND))
            & Self::MASK;
        ((self.seed as u64) >> (48 - bits as u32)) as i32
    }

    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & bound.wrapping_neg()) == bound {
            return ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound.wrapping_sub(1)) >= 0 {
                return val;
            }
        }
    }

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1_i32 << 24) as f32
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }
}

fn mth_sin(x: f32) -> f32 {
    let idx = ((x * 10430.378_f32) as i32) & 65535;
    ((idx as f64) * std::f64::consts::PI * 2.0 / 65536.0).sin() as f32
}

fn mth_cos(x: f32) -> f32 {
    let idx = (((x * 10430.378_f32) + 16384.0_f32) as i32) & 65535;
    ((idx as f64) * std::f64::consts::PI * 2.0 / 65536.0).sin() as f32
}

fn mth_abs(x: f32) -> f32 {
    x.abs()
}

fn mth_random_between(random: &mut RandomSource, min: f32, max: f32) -> f32 {
    min + random.next_float() * (max - min)
}

pub trait CarvingContext {
    fn get_gen_depth(&self) -> i32;
    fn get_min_gen_y(&self) -> i32;
}

pub trait ChunkAccess {
    fn get_pos(&self) -> ChunkPos;
}

pub trait BiomeSource {}

pub trait Aquifer {}

pub trait CarvingMask {}

pub trait CanyonCarverConfig {
    fn probability(&self) -> f32;
    fn sample_y(&self, random: &mut RandomSource, ctx: &dyn CarvingContext) -> i32;
    fn sample_y_scale(&self, random: &mut RandomSource) -> f64;
    fn sample_vertical_rotation(&self, random: &mut RandomSource) -> f32;
    fn sample_thickness(&self, random: &mut RandomSource) -> f32;
    fn sample_distance_factor(&self, random: &mut RandomSource) -> f32;
    fn sample_horizontal_radius_factor(&self, random: &mut RandomSource) -> f32;
    fn vertical_radius_default_factor(&self) -> f32;
    fn vertical_radius_center_factor(&self) -> f32;
    fn width_smoothness(&self) -> i32;
}

pub trait EllipsoidCarver {
    fn carve_ellipsoid(
        &mut self,
        ctx: &dyn CarvingContext,
        cfg: &dyn CanyonCarverConfig,
        chunk: &dyn ChunkAccess,
        biome_source: &dyn BiomeSource,
        aquifer: &mut dyn Aquifer,
        x: f64,
        y: f64,
        z: f64,
        horizontal_radius: f64,
        vertical_radius: f64,
        mask: &mut dyn CarvingMask,
        skip: &mut dyn FnMut(&dyn CarvingContext, f64, f64, f64, i32) -> bool,
    ) -> bool;
}

// JAVA: net.minecraft.world.level.levelgen.carver.WorldCarver#canReach
pub fn can_reach(
    pos: ChunkPos,
    x: f64,
    z: f64,
    branch_index: i32,
    branch_count: i32,
    radius: f32,
) -> bool {
    let d0 = (pos.get_middle_block_x() as f64) - x;
    let d1 = (pos.get_middle_block_z() as f64) - z;
    let d2 = (branch_count.wrapping_sub(branch_index)) as f64;
    let d3 = (radius + 2.0_f32 + 16.0_f32) as f64;
    d0 * d0 + d1 * d1 - d2 * d2 <= d3 * d3
}

pub struct CanyonWorldCarver {
    range: i32,
}

impl Default for CanyonWorldCarver {
    fn default() -> Self {
        Self { range: 4 }
    }
}

impl CanyonWorldCarver {
    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#<init>
    pub fn new(range: i32) -> Self {
        Self { range }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.WorldCarver#getRange
    pub fn get_range(&self) -> i32 {
        self.range
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#isStartChunk
    pub fn is_start_chunk(&self, cfg: &dyn CanyonCarverConfig, random: &mut RandomSource) -> bool {
        random.next_float() <= cfg.probability()
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#carve
    pub fn carve(
        &self,
        ctx: &dyn CarvingContext,
        cfg: &dyn CanyonCarverConfig,
        chunk: &dyn ChunkAccess,
        biome_source: &dyn BiomeSource,
        random: &mut RandomSource,
        aquifer: &mut dyn Aquifer,
        chunk_pos: ChunkPos,
        mask: &mut dyn CarvingMask,
        carver: &mut dyn EllipsoidCarver,
    ) -> bool {
        let range_blocks: i32 = self
            .get_range()
            .wrapping_mul(2)
            .wrapping_sub(1)
            .wrapping_mul(16);
        let x: f64 = chunk_pos.get_block_x(random.next_int_bound(16)) as f64;
        let y: i32 = cfg.sample_y(random, ctx);
        let z: f64 = chunk_pos.get_block_z(random.next_int_bound(16)) as f64;
        let yaw: f32 = random.next_float() * ((std::f64::consts::PI * 2.0) as f32);
        let pitch: f32 = cfg.sample_vertical_rotation(random);
        let y_scale: f64 = cfg.sample_y_scale(random);
        let thickness: f32 = cfg.sample_thickness(random);
        let branch_count: i32 =
            ((range_blocks as f32) * cfg.sample_distance_factor(random)) as i32;
        let seed: i64 = random.next_long();
        self.do_carve(
            ctx,
            cfg,
            chunk,
            biome_source,
            seed,
            aquifer,
            x,
            y as f64,
            z,
            thickness,
            yaw,
            pitch,
            0,
            branch_count,
            y_scale,
            mask,
            carver,
        );
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#doCarve
    fn do_carve(
        &self,
        ctx: &dyn CarvingContext,
        cfg: &dyn CanyonCarverConfig,
        chunk: &dyn ChunkAccess,
        biome_source: &dyn BiomeSource,
        seed: i64,
        aquifer: &mut dyn Aquifer,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        thickness: f32,
        mut yaw: f32,
        mut pitch: f32,
        branch_index: i32,
        branch_count: i32,
        y_scale: f64,
        mask: &mut dyn CarvingMask,
        carver: &mut dyn EllipsoidCarver,
    ) {
        let mut random = RandomSource::create(seed);
        let width_factors = self.init_width_factors(ctx, cfg, &mut random);
        let mut yaw_acc: f32 = 0.0_f32;
        let mut pitch_acc: f32 = 0.0_f32;

        let mut j: i32 = branch_index;
        while j < branch_count {
            let mut horiz_radius: f64 = 1.5_f64
                + (mth_sin((j as f32) * PI / (branch_count as f32)) * thickness) as f64;
            let mut vert_radius: f64 = horiz_radius * y_scale;
            horiz_radius *= cfg.sample_horizontal_radius_factor(&mut random) as f64;
            vert_radius = self.update_vertical_radius(
                cfg,
                &mut random,
                vert_radius,
                branch_count as f32,
                j as f32,
            );
            let cos_pitch = mth_cos(pitch);
            let sin_pitch = mth_sin(pitch);
            x += (mth_cos(yaw) * cos_pitch) as f64;
            y += sin_pitch as f64;
            z += (mth_sin(yaw) * cos_pitch) as f64;
            pitch *= 0.7_f32;
            pitch += pitch_acc * 0.05_f32;
            yaw += yaw_acc * 0.05_f32;
            pitch_acc *= 0.8_f32;
            yaw_acc *= 0.5_f32;
            pitch_acc +=
                (random.next_float() - random.next_float()) * random.next_float() * 2.0_f32;
            yaw_acc +=
                (random.next_float() - random.next_float()) * random.next_float() * 4.0_f32;
            if random.next_int_bound(4) != 0 {
                if !can_reach(chunk.get_pos(), x, z, j, branch_count, thickness) {
                    return;
                }
                let factors = &width_factors;
                carver.carve_ellipsoid(
                    ctx,
                    cfg,
                    chunk,
                    biome_source,
                    aquifer,
                    x,
                    y,
                    z,
                    horiz_radius,
                    vert_radius,
                    mask,
                    &mut |cctx, rel_x, rel_y, rel_z, yy| {
                        self.should_skip(cctx, factors, rel_x, rel_y, rel_z, yy)
                    },
                );
            }
            j = j.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#initWidthFactors
    fn init_width_factors(
        &self,
        ctx: &dyn CarvingContext,
        cfg: &dyn CanyonCarverConfig,
        random: &mut RandomSource,
    ) -> Vec<f32> {
        let gen_depth = ctx.get_gen_depth();
        let mut factors = vec![0.0_f32; gen_depth.max(0) as usize];
        let mut val: f32 = 1.0_f32;

        let mut i: i32 = 0;
        while i < gen_depth {
            if i == 0 || random.next_int_bound(cfg.width_smoothness()) == 0 {
                val = 1.0_f32 + random.next_float() * random.next_float();
            }
            factors[i as usize] = val * val;
            i = i.wrapping_add(1);
        }

        factors
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#updateVerticalRadius
    fn update_vertical_radius(
        &self,
        cfg: &dyn CanyonCarverConfig,
        random: &mut RandomSource,
        vert_radius: f64,
        branch_count: f32,
        branch_index: f32,
    ) -> f64 {
        let center_factor =
            1.0_f32 - mth_abs(0.5_f32 - branch_index / branch_count) * 2.0_f32;
        let scale = cfg.vertical_radius_default_factor()
            + cfg.vertical_radius_center_factor() * center_factor;
        (scale as f64) * vert_radius * (mth_random_between(random, 0.75_f32, 1.0_f32) as f64)
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver#shouldSkip
    fn should_skip(
        &self,
        ctx: &dyn CarvingContext,
        width_factors: &[f32],
        rel_x: f64,
        rel_y: f64,
        rel_z: f64,
        y: i32,
    ) -> bool {
        let idx = y.wrapping_sub(ctx.get_min_gen_y());
        let factor_idx = idx.wrapping_sub(1);
        let factor = width_factors[factor_idx as usize] as f64;
        (rel_x * rel_x + rel_z * rel_z) * factor + rel_y * rel_y / 6.0 >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCtx {
        depth: i32,
        min_y: i32,
    }
    impl CarvingContext for TestCtx {
        fn get_gen_depth(&self) -> i32 {
            self.depth
        }
        fn get_min_gen_y(&self) -> i32 {
            self.min_y
        }
    }

    struct TestCfg {
        prob: f32,
        y: i32,
        y_scale: f64,
        vert_rot: f32,
        thickness: f32,
        dist_factor: f32,
        horiz_factor: f32,
        vr_default: f32,
        vr_center: f32,
        width_smoothness: i32,
    }
    impl CanyonCarverConfig for TestCfg {
        fn probability(&self) -> f32 {
            self.prob
        }
        fn sample_y(&self, _: &mut RandomSource, _: &dyn CarvingContext) -> i32 {
            self.y
        }
        fn sample_y_scale(&self, _: &mut RandomSource) -> f64 {
            self.y_scale
        }
        fn sample_vertical_rotation(&self, _: &mut RandomSource) -> f32 {
            self.vert_rot
        }
        fn sample_thickness(&self, _: &mut RandomSource) -> f32 {
            self.thickness
        }
        fn sample_distance_factor(&self, _: &mut RandomSource) -> f32 {
            self.dist_factor
        }
        fn sample_horizontal_radius_factor(&self, _: &mut RandomSource) -> f32 {
            self.horiz_factor
        }
        fn vertical_radius_default_factor(&self) -> f32 {
            self.vr_default
        }
        fn vertical_radius_center_factor(&self) -> f32 {
            self.vr_center
        }
        fn width_smoothness(&self) -> i32 {
            self.width_smoothness
        }
    }

    struct TestChunk(ChunkPos);
    impl ChunkAccess for TestChunk {
        fn get_pos(&self) -> ChunkPos {
            self.0
        }
    }
    struct TestBiome;
    impl BiomeSource for TestBiome {}
    struct TestAquifer;
    impl Aquifer for TestAquifer {}
    struct TestMask;
    impl CarvingMask for TestMask {}

    struct NoopCarver;
    impl EllipsoidCarver for NoopCarver {
        fn carve_ellipsoid(
            &mut self,
            _: &dyn CarvingContext,
            _: &dyn CanyonCarverConfig,
            _: &dyn ChunkAccess,
            _: &dyn BiomeSource,
            _: &mut dyn Aquifer,
            _: f64,
            _: f64,
            _: f64,
            _: f64,
            _: f64,
            _: &mut dyn CarvingMask,
            _: &mut dyn FnMut(&dyn CarvingContext, f64, f64, f64, i32) -> bool,
        ) -> bool {
            false
        }
    }

    struct RecordingCarver {
        calls: Vec<(f64, f64, f64, f64, f64)>,
    }
    impl EllipsoidCarver for RecordingCarver {
        fn carve_ellipsoid(
            &mut self,
            _: &dyn CarvingContext,
            _: &dyn CanyonCarverConfig,
            _: &dyn ChunkAccess,
            _: &dyn BiomeSource,
            _: &mut dyn Aquifer,
            x: f64,
            y: f64,
            z: f64,
            hr: f64,
            vr: f64,
            _: &mut dyn CarvingMask,
            _: &mut dyn FnMut(&dyn CarvingContext, f64, f64, f64, i32) -> bool,
        ) -> bool {
            self.calls.push((x, y, z, hr, vr));
            true
        }
    }

    fn base_cfg(prob: f32) -> TestCfg {
        TestCfg {
            prob,
            y: 0,
            y_scale: 1.0,
            vert_rot: 0.0,
            thickness: 3.0,
            dist_factor: 1.0,
            horiz_factor: 1.0,
            vr_default: 1.0,
            vr_center: 0.0,
            width_smoothness: 3,
        }
    }

    #[test]
    fn is_start_chunk_probability_and_float_bits() {
        let carver = CanyonWorldCarver::new(4);
        let cfg0 = base_cfg(0.0);
        let cfg1 = base_cfg(1.0);
        let mut r0 = RandomSource::create(0);
        let mut r1 = RandomSource::create(0);
        assert!(!carver.is_start_chunk(&cfg0, &mut r0));
        assert!(carver.is_start_chunk(&cfg1, &mut r1));

        // Mth sin/cos float-bit behavior at key points
        assert_eq!(mth_sin(0.0_f32).to_bits(), 0.0_f32.to_bits());
        assert_eq!(mth_cos(0.0_f32).to_bits(), 1.0_f32.to_bits());
    }

    #[test]
    fn init_width_factors_deterministic_and_bounded() {
        let carver = CanyonWorldCarver::new(4);
        let ctx = TestCtx {
            depth: 16,
            min_y: -64,
        };
        let cfg = base_cfg(1.0);
        let mut r1 = RandomSource::create(12345_i64);
        let mut r2 = RandomSource::create(12345_i64);
        let f1 = carver.init_width_factors(&ctx, &cfg, &mut r1);
        let f2 = carver.init_width_factors(&ctx, &cfg, &mut r2);
        assert_eq!(f1, f2);
        assert_eq!(f1.len(), 16);
        for v in f1 {
            assert!(v >= 1.0_f32 && v <= 4.0_f32);
        }
    }

    #[test]
    fn update_vertical_radius_center_and_edge() {
        let carver = CanyonWorldCarver::new(4);
        let mut cfg = base_cfg(1.0);
        cfg.vr_default = 1.0;
        cfg.vr_center = 1.0;
        let mut rng_center = RandomSource::create(42_i64);
        let mut rng_edge = RandomSource::create(42_i64);
        let center = carver.update_vertical_radius(&cfg, &mut rng_center, 10.0, 100.0_f32, 50.0_f32);
        let edge = carver.update_vertical_radius(&cfg, &mut rng_edge, 10.0, 100.0_f32, 0.0_f32);
        assert!(center >= 15.0 && center <= 20.0); // scale 2.0 * [0.75,1.0] * 10
        assert!(edge >= 7.5 && edge <= 10.0); // scale 1.0 * [0.75,1.0] * 10
    }

    #[test]
    fn should_skip_and_negative_min_y_indexing() {
        let carver = CanyonWorldCarver::new(4);
        let ctx = TestCtx {
            depth: 128,
            min_y: -64,
        };
        let factors = vec![1.0_f32; 128];
        // y=0 -> idx=64, factor index 63
        assert!(!carver.should_skip(&ctx, &factors, 0.0, 0.0, 0.0, 0));
        assert!(carver.should_skip(&ctx, &factors, 1.0, 0.0, 0.0, 0));
        // negative y still maps correctly with wrapping_sub semantics on ints
        assert!(!carver.should_skip(&ctx, &factors, 0.0, 0.0, 0.0, -63));
    }

    #[test]
    fn can_reach_negative_coords_and_carve_integration() {
        let pos = ChunkPos::new(-10, -10);
        let mx = pos.get_middle_block_x() as f64;
        let mz = pos.get_middle_block_z() as f64;
        assert!(can_reach(pos, mx, mz, 0, 10, 1.0));
        assert!(!can_reach(pos, mx + 10_000.0, mz, 0, 10, 1.0));

        let carver = CanyonWorldCarver::new(4);
        let ctx = TestCtx {
            depth: 128,
            min_y: -64,
        };
        let cfg = base_cfg(1.0);
        let chunk = TestChunk(ChunkPos::new(0, 0));
        let biome = TestBiome;
        let mut aquifer = TestAquifer;
        let mut mask = TestMask;
        let mut rng = RandomSource::create(1_i64);
        let mut rec = RecordingCarver { calls: Vec::new() };
        let result = carver.carve(
            &ctx,
            &cfg,
            &chunk,
            &biome,
            &mut rng,
            &mut aquifer,
            ChunkPos::new(0, 0),
            &mut mask,
            &mut rec,
        );
        assert!(result);
        assert!(!rec.calls.is_empty());

        // Noop carver path still returns true from carve()
        let mut rng2 = RandomSource::create(2_i64);
        let mut noop = NoopCarver;
        let result2 = carver.carve(
            &ctx,
            &cfg,
            &chunk,
            &biome,
            &mut rng2,
            &mut aquifer,
            ChunkPos::new(0, 0),
            &mut mask,
            &mut noop,
        );
        assert!(result2);
    }

    #[test]
    fn chunk_pos_and_random_wrapping_edges() {
        let pos = ChunkPos::new(-1, -1);
        assert_eq!(pos.get_block_x(0), -16);
        assert_eq!(pos.get_block_z(15), -1);
        assert_eq!(pos.get_middle_block_x(), -8);

        // Wrapping shift on extreme chunk coordinate should not panic.
        let extreme = ChunkPos::new(i32::MIN >> 4, i32::MAX >> 4);
        let _ = extreme.get_block_x(0);
        let _ = extreme.get_block_z(0);

        let mut r = RandomSource::create(i64::MIN);
        let _f = r.next_float();
        let _l = r.next_long();
        let n = r.next_int_bound(16);
        assert!(n >= 0 && n < 16);
    }
}
