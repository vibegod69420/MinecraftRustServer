use std::fmt;

/// A block state identifier, mirroring Java's `BlockState` reference identity/equality.
/// In Minecraft, surface rules compare states by object identity (`==`); here we use
/// value equality on this lightweight id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u32);

/// Well-known block states used by `SurfaceSystem`.
/// Implementations of [`BlockProperties`] must recognize these ids consistently.
pub const AIR: BlockState = BlockState(0);
pub const WATER: BlockState = BlockState(1);
pub const WHITE_TERRACOTTA: BlockState = BlockState(2);
pub const ORANGE_TERRACOTTA: BlockState = BlockState(3);
pub const TERRACOTTA: BlockState = BlockState(4);
pub const YELLOW_TERRACOTTA: BlockState = BlockState(5);
pub const BROWN_TERRACOTTA: BlockState = BlockState(6);
pub const RED_TERRACOTTA: BlockState = BlockState(7);
pub const LIGHT_GRAY_TERRACOTTA: BlockState = BlockState(8);
pub const PACKED_ICE: BlockState = BlockState(9);
pub const SNOW_BLOCK: BlockState = BlockState(10);

/// Block property queries required by the surface system.
/// This abstracts Java `BlockState.isAir()`, `getFluidState().isEmpty()`,
/// and `is(Block)` checks.
pub trait BlockProperties {
    fn is_air(&self, state: BlockState) -> bool;
    fn has_fluid(&self, state: BlockState) -> bool;
    /// Java: `state.is(block)`. Compares block type identity, not exact state.
    fn is_block(&self, state: BlockState, block: BlockState) -> bool;
}

/// 2D/3D normal noise accessor, mirroring `net.minecraft.world.level.levelgen.synth.NormalNoise`.
pub trait NormalNoise {
    fn get_value(&self, x: f64, y: f64, z: f64) -> f64;
}

/// Random source, mirroring `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
    fn next_double(&mut self) -> f64;
    fn next_boolean(&mut self) -> bool;
}

/// Positional random factory, mirroring `net.minecraft.world.level.levelgen.PositionalRandomFactory`.
pub trait PositionalRandomFactory {
    type Random: RandomSource;
    fn from_hash_of(&self, name: &str) -> Self::Random;
    fn at(&self, x: i32, y: i32, z: i32) -> Self::Random;
}

/// Chunk position, mirroring `net.minecraft.world.level.ChunkPos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.z)
    }
}

/// Minimal chunk access required by `SurfaceSystem`.
/// Mirrors the subset of `ChunkAccess`/`LevelHeightAccessor` used by the Java implementation.
pub trait ChunkAccess {
    fn chunk_pos(&self) -> ChunkPos;
    fn get_min_y(&self) -> i32;
    fn get_height_world_surface_wg(&self, local_x: i32, local_z: i32) -> i32;
    fn get_block_state(&self, x: i32, y: i32, z: i32) -> BlockState;
    fn set_block_state(&mut self, x: i32, y: i32, z: i32, state: BlockState);
    fn is_inside_build_height(&self, y: i32) -> bool;
    fn mark_pos_for_postprocessing(&mut self, x: i32, y: i32, z: i32);
}

/// Block column abstraction, mirroring `net.minecraft.world.level.chunk.BlockColumn`.
pub trait BlockColumn {
    fn get_block(&self, y: i32) -> BlockState;
    fn set_block(&mut self, y: i32, state: BlockState);
}

/// Biome query abstraction for the checks used by `SurfaceSystem`.
pub trait BiomeLike {
    fn is_eroded_badlands(&self) -> bool;
    fn is_frozen_ocean(&self) -> bool;
    fn is_deep_frozen_ocean(&self) -> bool;
    fn should_melt_frozen_ocean_iceberg_slightly(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sea_level: i32,
    ) -> bool;
}

/// Surface rule context abstraction, mirroring the methods `SurfaceSystem` calls on
/// `SurfaceRules.Context`.
pub trait SurfaceContext {
    fn update_xz(&mut self, block_x: i32, block_z: i32);
    fn update_y(
        &mut self,
        stone_depth_above: i32,
        stone_depth_below: i32,
        water_height: i32,
        block_x: i32,
        block_y: i32,
        block_z: i32,
    );
    fn get_min_surface_level(&self) -> i32;
}

/// Surface rule abstraction, mirroring `SurfaceRules.SurfaceRule`.
pub trait SurfaceRule {
    fn try_apply(&self, block_x: i32, block_y: i32, block_z: i32) -> Option<BlockState>;
}

/// Port of `net.minecraft.world.level.levelgen.SurfaceSystem`.
///
/// External Minecraft types are abstracted behind traits to keep this module self-contained.
/// The `build_surface` method accepts a pre-built surface context and rule because
/// `SurfaceRules.Context` construction depends on unported types; the core surface logic
/// remains 1:1 faithful.
pub struct SurfaceSystem<N: NormalNoise, P: PositionalRandomFactory> {
    default_block: BlockState,
    sea_level: i32,
    clay_bands: [BlockState; 192],
    clay_bands_offset_noise: N,
    badlands_pillar_noise: N,
    badlands_pillar_roof_noise: N,
    badlands_surface_noise: N,
    iceberg_pillar_noise: N,
    iceberg_pillar_roof_noise: N,
    iceberg_surface_noise: N,
    noise_random: P,
    surface_noise: N,
    surface_secondary_noise: N,
}

struct ChunkBlockColumn<'a, Ch: ChunkAccess + ?Sized, BP: BlockProperties + ?Sized> {
    chunk: &'a mut Ch,
    x: i32,
    z: i32,
    block_props: &'a BP,
}

impl<Ch: ChunkAccess + ?Sized, BP: BlockProperties + ?Sized> BlockColumn
    for ChunkBlockColumn<'_, Ch, BP>
{
    fn get_block(&self, y: i32) -> BlockState {
        self.chunk.get_block_state(self.x, y, self.z)
    }

    fn set_block(&mut self, y: i32, state: BlockState) {
        if self.chunk.is_inside_build_height(y) {
            self.chunk.set_block_state(self.x, y, self.z, state);
            if self.block_props.has_fluid(state) {
                self.chunk.mark_pos_for_postprocessing(self.x, y, self.z);
            }
        }
    }
}

impl<N: NormalNoise, P: PositionalRandomFactory> SurfaceSystem<N, P> {
    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#<init>
    ///
    /// Java constructor takes `(RandomState, BlockState, int, PositionalRandomFactory)`.
    /// This port takes the resolved noises directly to remain self-contained.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        default_block: BlockState,
        sea_level: i32,
        noise_random: P,
        clay_bands_offset_noise: N,
        surface_noise: N,
        surface_secondary_noise: N,
        badlands_pillar_noise: N,
        badlands_pillar_roof_noise: N,
        badlands_surface_noise: N,
        iceberg_pillar_noise: N,
        iceberg_pillar_roof_noise: N,
        iceberg_surface_noise: N,
    ) -> Self
    where
        P::Random: 'static,
    {
        let mut clay_rand = noise_random.from_hash_of("minecraft:clay_bands");
        let clay_bands = Self::generate_bands(&mut clay_rand);
        Self {
            default_block,
            sea_level,
            clay_bands,
            clay_bands_offset_noise,
            badlands_pillar_noise,
            badlands_pillar_roof_noise,
            badlands_surface_noise,
            iceberg_pillar_noise,
            iceberg_pillar_roof_noise,
            iceberg_surface_noise,
            noise_random,
            surface_noise,
            surface_secondary_noise,
        }
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#buildSurface
    ///
    /// The Java method constructs `SurfaceRules.Context` and `SurfaceRule` internally.
    /// Here they are passed in pre-built to keep the module self-contained while preserving
    /// the surface-building logic 1:1.
    #[allow(clippy::too_many_arguments)]
    pub fn build_surface<Ch, Bi, Ctx, R, BP>(
        &self,
        chunk: &mut Ch,
        biome_at: &mut impl FnMut(i32, i32, i32) -> Bi,
        is_low_noise: bool,
        context: &mut Ctx,
        rule: &R,
        block_props: &BP,
    ) where
        Ch: ChunkAccess,
        Bi: BiomeLike,
        Ctx: SurfaceContext,
        R: SurfaceRule,
        BP: BlockProperties,
    {
        let chunk_pos = chunk.chunk_pos();
        let min_block_x = chunk_pos.get_min_block_x();
        let min_block_z = chunk_pos.get_min_block_z();

        for local_x in 0..16_i32 {
            for local_z in 0..16_i32 {
                let block_x = min_block_x.wrapping_add(local_x);
                let block_z = min_block_z.wrapping_add(local_z);
                let surface_height =
                    chunk.get_height_world_surface_wg(local_x, local_z).wrapping_add(1);

                let biome_y = if is_low_noise { 0 } else { surface_height };
                let biome = biome_at(block_x, biome_y, block_z);

                if biome.is_eroded_badlands() {
                    self.eroded_badlands_extension_at(
                        chunk,
                        block_x,
                        block_z,
                        surface_height,
                        block_props,
                    );
                }

                let top_height =
                    chunk.get_height_world_surface_wg(local_x, local_z).wrapping_add(1);
                context.update_xz(block_x, block_z);
                let mut stone_depth_above: i32 = 0;
                let mut water_height: i32 = i32::MIN;
                let mut min_surface_level: i32 = i32::MAX;
                let min_y = chunk.get_min_y();

                let mut y = top_height;
                while y >= min_y {
                    let state = chunk.get_block_state(block_x, y, block_z);
                    if block_props.is_air(state) {
                        stone_depth_above = 0;
                        water_height = i32::MIN;
                    } else if block_props.has_fluid(state) {
                        if water_height == i32::MIN {
                            water_height = y.wrapping_add(1);
                        }
                    } else {
                        if min_surface_level >= y {
                            min_surface_level = i32::MIN; // DimensionType.WAY_BELOW_MIN_Y

                            let mut scan_y = y.wrapping_sub(1);
                            let scan_limit = min_y.wrapping_sub(1);
                            while scan_y >= scan_limit {
                                let scan_state = chunk.get_block_state(block_x, scan_y, block_z);
                                if !self.is_stone(scan_state, block_props) {
                                    min_surface_level = scan_y.wrapping_add(1);
                                    break;
                                }
                                scan_y = scan_y.wrapping_sub(1);
                            }
                        }

                        stone_depth_above = stone_depth_above.wrapping_add(1);
                        let stone_depth_below = y.wrapping_sub(min_surface_level).wrapping_add(1);
                        context.update_y(
                            stone_depth_above,
                            stone_depth_below,
                            water_height,
                            block_x,
                            y,
                            block_z,
                        );
                        if state == self.default_block {
                            if let Some(new_state) = rule.try_apply(block_x, y, block_z) {
                                if chunk.is_inside_build_height(y) {
                                    chunk.set_block_state(block_x, y, block_z, new_state);
                                    if block_props.has_fluid(new_state) {
                                        chunk.mark_pos_for_postprocessing(block_x, y, block_z);
                                    }
                                }
                            }
                        }
                    }
                    y = y.wrapping_sub(1);
                }

                if biome.is_frozen_ocean() || biome.is_deep_frozen_ocean() {
                    self.frozen_ocean_extension_at(
                        context.get_min_surface_level(),
                        &biome,
                        chunk,
                        block_x,
                        block_z,
                        surface_height,
                        block_props,
                    );
                }
            }
        }
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#getSurfaceDepth
    pub fn get_surface_depth(&self, x: i32, z: i32) -> i32 {
        let d2 = self.surface_noise.get_value(x as f64, 0.0, z as f64);
        let rand = self.noise_random.at(x, 0, z).next_double();
        (d2 * 2.75 + 3.0 + rand * 0.25) as i32
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#getSurfaceSecondary
    pub fn get_surface_secondary(&self, x: i32, z: i32) -> f64 {
        self.surface_secondary_noise.get_value(x as f64, 0.0, z as f64)
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#isStone
    fn is_stone(&self, state: BlockState, block_props: &impl BlockProperties) -> bool {
        !block_props.is_air(state) && !block_props.has_fluid(state)
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#getSeaLevel
    pub fn get_sea_level(&self) -> i32 {
        self.sea_level
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#topMaterial
    ///
    /// Deprecated in Java. The `CarvingContext`-dependent construction of
    /// `SurfaceRules.Context` is abstracted: callers provide the pre-built context and rule.
    #[allow(clippy::too_many_arguments)]
    pub fn top_material<Ctx, R>(
        &self,
        context: &mut Ctx,
        rule: &R,
        x: i32,
        y: i32,
        z: i32,
        is_low_noise: bool,
    ) -> Option<BlockState>
    where
        Ctx: SurfaceContext,
        R: SurfaceRule,
    {
        context.update_xz(x, z);
        let water_height = if is_low_noise {
            y.wrapping_add(1)
        } else {
            i32::MIN
        };
        context.update_y(1, 1, water_height, x, y, z);
        rule.try_apply(x, y, z)
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#erodedBadlandsExtension
    fn eroded_badlands_extension<BC: BlockColumn + ?Sized, BP: BlockProperties + ?Sized>(
        &self,
        column: &mut BC,
        x: i32,
        z: i32,
        height: i32,
        min_y: i32,
        block_props: &BP,
    ) {
        let _d5: f64 = 0.2;
        let d6 = f64::min(
            (self.badlands_surface_noise.get_value(x as f64, 0.0, z as f64) * 8.25).abs(),
            self.badlands_pillar_noise
                .get_value((x as f64) * 0.2, 0.0, (z as f64) * 0.2)
                * 15.0,
        );
        if !(d6 <= 0.0) {
            let _d7: f64 = 0.75;
            let _d8: f64 = 1.5;
            let d9 = (self
                .badlands_pillar_roof_noise
                .get_value((x as f64) * 0.75, 0.0, (z as f64) * 0.75)
                * 1.5)
                .abs();
            let d10 = 64.0 + f64::min(d6 * d6 * 2.5, (d9 * 50.0).ceil() + 24.0);
            let i11 = floor_i32(d10);
            if height <= i11 {
                let mut y = i11;
                while y >= min_y {
                    let state = column.get_block(y);
                    if block_props.is_block(state, self.default_block) {
                        break;
                    }
                    if block_props.is_block(state, WATER) {
                        return;
                    }
                    y = y.wrapping_sub(1);
                }

                let mut y = i11;
                while y >= min_y && block_props.is_air(column.get_block(y)) {
                    column.set_block(y, self.default_block);
                    y = y.wrapping_sub(1);
                }
            }
        }
    }

    /// Helper that adapts chunk access to the `BlockColumn` interface used by
    /// `eroded_badlands_extension`, preserving the Java method's behavior.
    fn eroded_badlands_extension_at<Ch: ChunkAccess, BP: BlockProperties>(
        &self,
        chunk: &mut Ch,
        x: i32,
        z: i32,
        height: i32,
        block_props: &BP,
    ) {
        let min_y = chunk.get_min_y();
        let mut column = ChunkBlockColumn {
            chunk,
            x,
            z,
            block_props,
        };
        self.eroded_badlands_extension(&mut column, x, z, height, min_y, block_props);
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#frozenOceanExtension
    #[allow(clippy::too_many_arguments)]
    fn frozen_ocean_extension<BC: BlockColumn + ?Sized, B: BiomeLike + ?Sized, R: RandomSource>(
        &self,
        min_surface_level: i32,
        biome: &B,
        column: &mut BC,
        x: i32,
        z: i32,
        height: i32,
        random: &mut R,
    ) {
        let _d7: f64 = 1.28;
        let d8 = f64::min(
            (self.iceberg_surface_noise.get_value(x as f64, 0.0, z as f64) * 8.25).abs(),
            self.iceberg_pillar_noise
                .get_value((x as f64) * 1.28, 0.0, (z as f64) * 1.28)
                * 15.0,
        );
        if !(d8 <= 1.8) {
            let _d9: f64 = 1.17;
            let _d10: f64 = 1.5;
            let d11 = (self
                .iceberg_pillar_roof_noise
                .get_value((x as f64) * 1.17, 0.0, (z as f64) * 1.17)
                * 1.5)
                .abs();
            let mut d12 = f64::min(d8 * d8 * 1.2, (d11 * 40.0).ceil() + 14.0);
            if biome.should_melt_frozen_ocean_iceberg_slightly(x, self.sea_level, z, self.sea_level)
            {
                d12 -= 2.0;
            }

            let (d13, d15): (f64, f64);
            if d12 > 2.0 {
                let sea = self.sea_level as f64;
                d13 = sea - d12 - 7.0;
                d12 += sea;
                d15 = d12;
            } else {
                d12 = 0.0;
                d13 = 0.0;
                d15 = d12;
            }

            let i17 = 2_i32.wrapping_add(random.next_int(4));
            let i18 = self.sea_level.wrapping_add(18).wrapping_add(random.next_int(10));
            let mut i19: i32 = 0;

            let start_y = i32::max(height, (d15 as i32).wrapping_add(1));
            let mut y = start_y;
            while y >= min_surface_level {
                let state = column.get_block(y);
                let is_air = state == AIR;
                let is_water = state == WATER;
                if (is_air && y < (d15 as i32) && random.next_double() > 0.01)
                    || (is_water
                        && y > (d13 as i32)
                        && y < self.sea_level
                        && d13 != 0.0
                        && random.next_double() > 0.15)
                {
                    if i19 <= i17 && y > i18 {
                        column.set_block(y, SNOW_BLOCK);
                        i19 = i19.wrapping_add(1);
                    } else {
                        column.set_block(y, PACKED_ICE);
                    }
                }
                y = y.wrapping_sub(1);
            }
        }
    }

    /// Helper that adapts chunk access and positional random to
    /// `frozen_ocean_extension`, preserving the Java method's behavior.
    #[allow(clippy::too_many_arguments)]
    fn frozen_ocean_extension_at<Ch: ChunkAccess, B: BiomeLike, BP: BlockProperties>(
        &self,
        min_surface_level: i32,
        biome: &B,
        chunk: &mut Ch,
        x: i32,
        z: i32,
        height: i32,
        block_props: &BP,
    ) {
        let mut column = ChunkBlockColumn {
            chunk,
            x,
            z,
            block_props,
        };
        let mut random = self.noise_random.at(x, 0, z);
        self.frozen_ocean_extension(min_surface_level, biome, &mut column, x, z, height, &mut random);
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#generateBands
    fn generate_bands(random: &mut impl RandomSource) -> [BlockState; 192] {
        let mut bands = [TERRACOTTA; 192];

        let mut i: usize = 0;
        while i < bands.len() {
            i = i.wrapping_add(random.next_int(5) as usize).wrapping_add(1);
            if i < bands.len() {
                bands[i] = ORANGE_TERRACOTTA;
            }
        }

        Self::make_bands(random, &mut bands, 1, YELLOW_TERRACOTTA);
        Self::make_bands(random, &mut bands, 2, BROWN_TERRACOTTA);
        Self::make_bands(random, &mut bands, 1, RED_TERRACOTTA);

        let count = random.next_int_between_inclusive(9, 15);
        let mut placed: i32 = 0;
        let mut j: usize = 0;
        while placed < count && j < bands.len() {
            bands[j] = WHITE_TERRACOTTA;
            if j >= 1 && random.next_boolean() {
                bands[j - 1] = LIGHT_GRAY_TERRACOTTA;
            }
            if j.wrapping_add(1) < bands.len() && random.next_boolean() {
                bands[j + 1] = LIGHT_GRAY_TERRACOTTA;
            }
            placed = placed.wrapping_add(1);
            j = j.wrapping_add(random.next_int(16) as usize).wrapping_add(4);
        }

        bands
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#makeBands
    fn make_bands(
        random: &mut impl RandomSource,
        bands: &mut [BlockState; 192],
        base_width: i32,
        state: BlockState,
    ) {
        let count = random.next_int_between_inclusive(6, 15);
        let mut i: i32 = 0;
        while i < count {
            let width = base_width.wrapping_add(random.next_int(3));
            let start = random.next_int(bands.len() as i32) as usize;
            let mut k: i32 = 0;
            while start.wrapping_add(k as usize) < bands.len() && k < width {
                bands[start.wrapping_add(k as usize)] = state;
                k = k.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }

    /// JAVA: net.minecraft.world.level.levelgen.SurfaceSystem#getBand
    pub fn get_band(&self, x: i32, y: i32, z: i32) -> BlockState {
        let offset = (self
            .clay_bands_offset_noise
            .get_value(x as f64, 0.0, z as f64)
            * 4.0)
            .round() as i32;
        let len = self.clay_bands.len() as i32;
        let idx = y.wrapping_add(offset).wrapping_add(len).rem_euclid(len);
        self.clay_bands[idx as usize]
    }
}

/// Java `Mth.floor(double)`: floor toward negative infinity.
fn floor_i32(v: f64) -> i32 {
    let i = v as i32;
    if (i as f64) <= v { i } else { i.wrapping_sub(1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct ConstNoise(f64);
    impl NormalNoise for ConstNoise {
        fn get_value(&self, _x: f64, _y: f64, _z: f64) -> f64 {
            self.0
        }
    }

    #[derive(Clone, Debug)]
    struct SeqRandom {
        ints: std::vec::Vec<i32>,
        doubles: std::vec::Vec<f64>,
        bools: std::vec::Vec<bool>,
        i: usize,
        d: usize,
        b: usize,
    }

    impl SeqRandom {
        fn new(ints: &[i32], doubles: &[f64], bools: &[bool]) -> Self {
            Self {
                ints: ints.to_vec(),
                doubles: doubles.to_vec(),
                bools: bools.to_vec(),
                i: 0,
                d: 0,
                b: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.i).copied().unwrap_or(0);
            self.i += 1;
            if bound <= 0 { 0 } else { v.rem_euclid(bound) }
        }
        fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32 {
            let v = self.ints.get(self.i).copied().unwrap_or(min);
            self.i += 1;
            let span = max.wrapping_sub(min).wrapping_add(1);
            if span <= 0 { min } else { min.wrapping_add(v.rem_euclid(span)) }
        }
        fn next_double(&mut self) -> f64 {
            let v = self.doubles.get(self.d).copied().unwrap_or(0.0);
            self.d += 1;
            v
        }
        fn next_boolean(&mut self) -> bool {
            let v = self.bools.get(self.b).copied().unwrap_or(false);
            self.b += 1;
            v
        }
    }

    #[derive(Clone, Debug)]
    struct TestFactory {
        seq: SeqRandom,
    }

    impl PositionalRandomFactory for TestFactory {
        type Random = SeqRandom;
        fn from_hash_of(&self, _name: &str) -> Self::Random {
            self.seq.clone()
        }
        fn at(&self, _x: i32, _y: i32, _z: i32) -> Self::Random {
            self.seq.clone()
        }
    }

    struct TestBlockProps;
    impl BlockProperties for TestBlockProps {
        fn is_air(&self, state: BlockState) -> bool {
            state == AIR
        }
        fn has_fluid(&self, state: BlockState) -> bool {
            state == WATER
        }
        fn is_block(&self, state: BlockState, block: BlockState) -> bool {
            state == block
        }
    }

    fn make_system(noise: f64, rand_seq: SeqRandom) -> SurfaceSystem<ConstNoise, TestFactory> {
        let factory = TestFactory { seq: rand_seq };
        SurfaceSystem::new(
            BlockState(100), // default block
            63,
            factory,
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
            ConstNoise(noise),
        )
    }

    #[test]
    fn get_band_wraps_with_negative_offset() {
        let sys = make_system(-1.0, SeqRandom::new(&[0], &[0.0], &[false]));
        // offset = round(-1.0 * 4.0) = -4
        let band = sys.get_band(0, 0, 0);
        let expected_idx = (0_i32).wrapping_add(-4).wrapping_add(192).rem_euclid(192) as usize;
        assert_eq!(band, sys.clay_bands[expected_idx]);
    }

    #[test]
    fn get_band_handles_extreme_y() {
        let sys = make_system(0.0, SeqRandom::new(&[0], &[0.0], &[false]));
        let b1 = sys.get_band(0, i32::MAX, 0);
        let b2 = sys.get_band(0, i32::MIN, 0);
        let len = 192_i32;
        let idx1 = i32::MAX.wrapping_add(0).wrapping_add(len).rem_euclid(len) as usize;
        let idx2 = i32::MIN.wrapping_add(0).wrapping_add(len).rem_euclid(len) as usize;
        assert_eq!(b1, sys.clay_bands[idx1]);
        assert_eq!(b2, sys.clay_bands[idx2]);
    }

    #[test]
    fn get_surface_depth_formula_and_negative_coords() {
        let sys = make_system(2.0, SeqRandom::new(&[], &[0.5], &[]));
        // (2.0 * 2.75 + 3.0 + 0.5 * 0.25) = (5.5 + 3.0 + 0.125) = 8.625 -> 8 as i32
        assert_eq!(sys.get_surface_depth(-10, -20), 8);
    }

    #[test]
    fn get_surface_secondary_returns_noise_value() {
        let sys = make_system(1.25, SeqRandom::new(&[0], &[0.0], &[false]));
        let v = sys.get_surface_secondary(123, -456);
        assert_eq!(v.to_bits(), 1.25_f64.to_bits());
    }

    #[test]
    fn generate_bands_is_deterministic_and_bounded() {
        let mut r1 = SeqRandom::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], &[0.0], &[true, false]);
        let mut r2 = SeqRandom::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], &[0.0], &[true, false]);
        let b1 = SurfaceSystem::<ConstNoise, TestFactory>::generate_bands(&mut r1);
        let b2 = SurfaceSystem::<ConstNoise, TestFactory>::generate_bands(&mut r2);
        assert_eq!(b1, b2);
        assert_eq!(b1.len(), 192);
        assert!(b1.iter().all(|s| matches!(
            *s,
            TERRACOTTA
                | ORANGE_TERRACOTTA
                | YELLOW_TERRACOTTA
                | BROWN_TERRACOTTA
                | RED_TERRACOTTA
                | WHITE_TERRACOTTA
                | LIGHT_GRAY_TERRACOTTA
        )));
    }

    #[test]
    fn floor_i32_matches_java_behavior() {
        assert_eq!(floor_i32(2.0), 2);
        assert_eq!(floor_i32(2.1), 2);
        assert_eq!(floor_i32(-2.0), -2);
        assert_eq!(floor_i32(-2.1), -3);
        assert_eq!(floor_i32(0.0_f64), 0);
    }

    #[test]
    fn is_stone_excludes_air_and_fluids() {
        let sys = make_system(0.0, SeqRandom::new(&[0], &[0.0], &[false]));
        let props = TestBlockProps;
        assert!(!sys.is_stone(AIR, &props));
        assert!(!sys.is_stone(WATER, &props));
        assert!(sys.is_stone(BlockState(100), &props));
    }
}
