// === FILE: aquifer.rs ===
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u16,
}

impl BlockState {
    pub const fn new(id: u16) -> Self {
        Self { id }
    }

    pub fn is_air(&self) -> bool {
        self.id == blocks::AIR.id
    }

    pub fn is(&self, other: BlockState) -> bool {
        *self == other
    }
}

pub mod blocks {
    use super::BlockState;
    pub const AIR: BlockState = BlockState { id: 0 };
    pub const WATER: BlockState = BlockState { id: 1 };
    pub const LAVA: BlockState = BlockState { id: 2 };
}

pub struct SharedConstants;
impl SharedConstants {
    pub const DEBUG_DISABLE_FLUID_GENERATION: bool = false;
}

pub struct DimensionType;
impl DimensionType {
    pub const WAY_BELOW_MIN_Y: i32 = -1_000_000;
}

pub struct Mth;

impl Mth {
    pub fn floor(value: f64) -> i32 {
        let i = value as i32;
        if value < (i as f64) {
            i.wrapping_sub(1)
        } else {
            i
        }
    }

    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        let mut v = value;
        if v < min {
            v = min;
        }
        if v > max {
            v = max;
        }
        v
    }

    pub fn map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        to_min + (value - from_min) / (from_max - from_min) * (to_max - to_min)
    }

    pub fn clamped_map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        to_min
            + Self::clamp((value - from_min) / (from_max - from_min), 0.0, 1.0)
                * (to_max - to_min)
    }

    pub fn quantize(value: f64, quantizer: i32) -> i32 {
        Self::floor(value / (quantizer as f64)).wrapping_mul(quantizer)
    }

    pub fn square(value: i32) -> i32 {
        value.wrapping_mul(value)
    }
}

pub struct BlockPos;

impl BlockPos {
    pub const PACKED_X_LENGTH: i32 = 26;
    pub const PACKED_Z_LENGTH: i32 = 26;
    pub const PACKED_Y_LENGTH: i32 = 12;
    pub const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_X_LENGTH) - 1;
    pub const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LENGTH) - 1;
    pub const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_Z_LENGTH) - 1;
    pub const Y_OFFSET: i32 = 0;
    pub const Z_OFFSET: i32 = Self::PACKED_Y_LENGTH;
    pub const X_OFFSET: i32 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    pub fn as_long(x: i32, y: i32, z: i32) -> i64 {
        let mut l: i64 = 0i64;
        l |= ((x as i64) & Self::PACKED_X_MASK) << (Self::X_OFFSET & 63);
        l |= ((y as i64) & Self::PACKED_Y_MASK) << (Self::Y_OFFSET & 63);
        l | ((z as i64) & Self::PACKED_Z_MASK) << (Self::Z_OFFSET & 63)
    }

    pub fn get_x(packed: i64) -> i32 {
        (packed << ((64 - Self::X_OFFSET - Self::PACKED_X_LENGTH) & 63)
            >> ((64 - Self::PACKED_X_LENGTH) & 63)) as i32
    }

    pub fn get_y(packed: i64) -> i32 {
        (packed << ((64 - Self::PACKED_Y_LENGTH) & 63) >> ((64 - Self::PACKED_Y_LENGTH) & 63))
            as i32
    }

    pub fn get_z(packed: i64) -> i32 {
        (packed << ((64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH) & 63)
            >> ((64 - Self::PACKED_Z_LENGTH) & 63)) as i32
    }
}

pub struct SectionPos;

impl SectionPos {
    pub fn section_to_block_coord(section: i32) -> i32 {
        section.wrapping_shl(4)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    pub fn get_max_block_x(&self) -> i32 {
        self.x.wrapping_shl(4).wrapping_add(15)
    }

    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }

    pub fn get_max_block_z(&self) -> i32 {
        self.z.wrapping_shl(4).wrapping_add(15)
    }
}

pub trait FunctionContext {
    fn block_x(&self) -> i32;
    fn block_y(&self) -> i32;
    fn block_z(&self) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub struct SinglePointContext {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SinglePointContext {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl FunctionContext for SinglePointContext {
    fn block_x(&self) -> i32 {
        self.x
    }

    fn block_y(&self) -> i32 {
        self.y
    }

    fn block_z(&self) -> i32 {
        self.z
    }
}

pub trait DensityFunction {
    fn compute(&self, ctx: &dyn FunctionContext) -> f64;
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait PositionalRandomFactory {
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource>;
}

pub trait NoiseChunkAccess {
    fn max_preliminary_surface_level(&self, x1: i32, z1: i32, x2: i32, z2: i32) -> i32;
    fn preliminary_surface_level(&self, x: i32, z: i32) -> i32;
}

pub struct NoiseRouter {
    pub barrier_noise: Box<dyn DensityFunction>,
    pub fluid_level_floodedness_noise: Box<dyn DensityFunction>,
    pub fluid_level_spread_noise: Box<dyn DensityFunction>,
    pub lava_noise: Box<dyn DensityFunction>,
    pub erosion: Box<dyn DensityFunction>,
    pub depth: Box<dyn DensityFunction>,
}

pub struct MutableDouble {
    pub value: f64,
}

impl MutableDouble {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn double_value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }
}

pub struct OverworldBiomeBuilder;

impl OverworldBiomeBuilder {
    pub fn is_deep_dark_region(
        erosion: &dyn DensityFunction,
        depth: &dyn DensityFunction,
        ctx: &dyn FunctionContext,
    ) -> bool {
        erosion.compute(ctx) < -0.225 && depth.compute(ctx) > 0.9
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidStatus {
    pub fluid_level: i32,
    pub fluid_type: BlockState,
}

impl FluidStatus {
    // JAVA: net.minecraft.world.level.levelgen.Aquifer$FluidStatus#at
    pub fn at(&self, y: i32) -> BlockState {
        if y < self.fluid_level {
            self.fluid_type
        } else {
            blocks::AIR
        }
    }
}

pub trait FluidPicker {
    fn compute_fluid(&self, x: i32, y: i32, z: i32) -> FluidStatus;
}

pub trait Aquifer {
    // JAVA: net.minecraft.world.level.levelgen.Aquifer#computeSubstance
    fn compute_substance(&mut self, ctx: &dyn FunctionContext, density: f64) -> Option<BlockState>;

    // JAVA: net.minecraft.world.level.levelgen.Aquifer#shouldScheduleFluidUpdate
    fn should_schedule_fluid_update(&self) -> bool;
}

// JAVA: net.minecraft.world.level.levelgen.Aquifer#create
pub fn create(
    noise_chunk: Box<dyn NoiseChunkAccess>,
    chunk_pos: ChunkPos,
    noise_router: NoiseRouter,
    positional_random_factory: Box<dyn PositionalRandomFactory>,
    min_y: i32,
    height: i32,
    fluid_picker: Box<dyn FluidPicker>,
) -> NoiseBasedAquifer {
    NoiseBasedAquifer::new(
        noise_chunk,
        chunk_pos,
        noise_router,
        positional_random_factory,
        min_y,
        height,
        fluid_picker,
    )
}

// JAVA: net.minecraft.world.level.levelgen.Aquifer#createDisabled
pub fn create_disabled(fluid_picker: Box<dyn FluidPicker>) -> DisabledAquifer {
    DisabledAquifer { fluid_picker }
}

pub struct DisabledAquifer {
    fluid_picker: Box<dyn FluidPicker>,
}

impl Aquifer for DisabledAquifer {
    // JAVA: net.minecraft.world.level.levelgen.Aquifer#computeSubstance
    fn compute_substance(&mut self, ctx: &dyn FunctionContext, density: f64) -> Option<BlockState> {
        if density > 0.0 {
            None
        } else {
            Some(
                self.fluid_picker
                    .compute_fluid(ctx.block_x(), ctx.block_y(), ctx.block_z())
                    .at(ctx.block_y()),
            )
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer#shouldScheduleFluidUpdate
    fn should_schedule_fluid_update(&self) -> bool {
        false
    }
}

pub struct NoiseBasedAquifer {
    noise_chunk: Box<dyn NoiseChunkAccess>,
    barrier_noise: Box<dyn DensityFunction>,
    fluid_level_floodedness_noise: Box<dyn DensityFunction>,
    fluid_level_spread_noise: Box<dyn DensityFunction>,
    lava_noise: Box<dyn DensityFunction>,
    positional_random_factory: Box<dyn PositionalRandomFactory>,
    aquifer_cache: Vec<Option<FluidStatus>>,
    aquifer_location_cache: Vec<i64>,
    global_fluid_picker: Box<dyn FluidPicker>,
    erosion: Box<dyn DensityFunction>,
    depth: Box<dyn DensityFunction>,
    should_schedule_fluid_update_flag: bool,
    skip_sampling_above_y: i32,
    min_grid_x: i32,
    min_grid_y: i32,
    min_grid_z: i32,
    grid_size_x: i32,
    grid_size_z: i32,
}

impl fmt::Debug for NoiseBasedAquifer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoiseBasedAquifer")
            .field("skip_sampling_above_y", &self.skip_sampling_above_y)
            .field("min_grid_x", &self.min_grid_x)
            .field("min_grid_y", &self.min_grid_y)
            .field("min_grid_z", &self.min_grid_z)
            .field("grid_size_x", &self.grid_size_x)
            .field("grid_size_z", &self.grid_size_z)
            .finish_non_exhaustive()
    }
}

impl NoiseBasedAquifer {
    pub const X_RANGE: i32 = 10;
    pub const Y_RANGE: i32 = 9;
    pub const Z_RANGE: i32 = 10;
    pub const X_SEPARATION: i32 = 6;
    pub const Y_SEPARATION: i32 = 3;
    pub const Z_SEPARATION: i32 = 6;
    pub const X_SPACING: i32 = 16;
    pub const Y_SPACING: i32 = 12;
    pub const Z_SPACING: i32 = 16;
    pub const X_SPACING_SHIFT: i32 = 4;
    pub const Z_SPACING_SHIFT: i32 = 4;
    pub const MAX_REASONABLE_DISTANCE_TO_AQUIFER_CENTER: i32 = 11;
    pub const FLOWING_UPDATE_SIMULARITY: f64 = 1.0 - (144i32 - 100i32) as f64 / 25.0;
    pub const SAMPLE_OFFSET_X: i32 = -5;
    pub const SAMPLE_OFFSET_Y: i32 = 1;
    pub const SAMPLE_OFFSET_Z: i32 = -5;
    pub const MIN_CELL_SAMPLE_X: i32 = 0;
    pub const MIN_CELL_SAMPLE_Y: i32 = -1;
    pub const MIN_CELL_SAMPLE_Z: i32 = 0;
    pub const MAX_CELL_SAMPLE_X: i32 = 1;
    pub const MAX_CELL_SAMPLE_Y: i32 = 1;
    pub const MAX_CELL_SAMPLE_Z: i32 = 1;

    pub const SURFACE_SAMPLING_OFFSETS_IN_CHUNKS: [[i32; 2]; 13] = [
        [0, 0],
        [-2, -1],
        [-1, -1],
        [0, -1],
        [1, -1],
        [-3, 0],
        [-2, 0],
        [-1, 0],
        [1, 0],
        [-2, 1],
        [-1, 1],
        [0, 1],
        [1, 1],
    ];

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#<init>
    pub fn new(
        noise_chunk: Box<dyn NoiseChunkAccess>,
        chunk_pos: ChunkPos,
        noise_router: NoiseRouter,
        positional_random_factory: Box<dyn PositionalRandomFactory>,
        min_y: i32,
        height: i32,
        global_fluid_picker: Box<dyn FluidPicker>,
    ) -> Self {
        let min_grid_x =
            Self::grid_x(chunk_pos.get_min_block_x().wrapping_add(Self::SAMPLE_OFFSET_X))
                .wrapping_add(Self::MIN_CELL_SAMPLE_X);
        let max_grid_x_plus_one =
            Self::grid_x(chunk_pos.get_max_block_x().wrapping_add(Self::SAMPLE_OFFSET_X))
                .wrapping_add(Self::MAX_CELL_SAMPLE_X);
        let grid_size_x = max_grid_x_plus_one
            .wrapping_sub(min_grid_x)
            .wrapping_add(1);

        let min_grid_y = Self::grid_y(min_y.wrapping_add(Self::SAMPLE_OFFSET_Y))
            .wrapping_add(Self::MIN_CELL_SAMPLE_Y);
        let max_grid_y_plus_one = Self::grid_y(
            min_y
                .wrapping_add(height)
                .wrapping_add(Self::SAMPLE_OFFSET_Y),
        )
        .wrapping_add(Self::MAX_CELL_SAMPLE_Y);
        let grid_size_y = max_grid_y_plus_one
            .wrapping_sub(min_grid_y)
            .wrapping_add(1);

        let min_grid_z =
            Self::grid_z(chunk_pos.get_min_block_z().wrapping_add(Self::SAMPLE_OFFSET_Z))
                .wrapping_add(Self::MIN_CELL_SAMPLE_Z);
        let max_grid_z_plus_one =
            Self::grid_z(chunk_pos.get_max_block_z().wrapping_add(Self::SAMPLE_OFFSET_Z))
                .wrapping_add(Self::MAX_CELL_SAMPLE_Z);
        let grid_size_z = max_grid_z_plus_one
            .wrapping_sub(min_grid_z)
            .wrapping_add(1);

        let total = grid_size_x
            .wrapping_mul(grid_size_y)
            .wrapping_mul(grid_size_z) as usize;
        let aquifer_location_cache = vec![i64::MAX; total];
        let aquifer_cache = vec![None; total];

        let mut this = Self {
            noise_chunk,
            barrier_noise: noise_router.barrier_noise,
            fluid_level_floodedness_noise: noise_router.fluid_level_floodedness_noise,
            fluid_level_spread_noise: noise_router.fluid_level_spread_noise,
            lava_noise: noise_router.lava_noise,
            erosion: noise_router.erosion,
            depth: noise_router.depth,
            positional_random_factory,
            aquifer_cache,
            aquifer_location_cache,
            global_fluid_picker,
            should_schedule_fluid_update_flag: false,
            skip_sampling_above_y: 0,
            min_grid_x,
            min_grid_y,
            min_grid_z,
            grid_size_x,
            grid_size_z,
        };

        let surface_level = this.noise_chunk.max_preliminary_surface_level(
            Self::from_grid_x(this.min_grid_x, 0),
            Self::from_grid_z(this.min_grid_z, 0),
            Self::from_grid_x(max_grid_x_plus_one, 9),
            Self::from_grid_z(max_grid_z_plus_one, 9),
        );
        let adjusted_surface = this.adjust_surface_level(surface_level);
        let max_surface_grid_y = Self::grid_y(adjusted_surface.wrapping_add(12)).wrapping_add(1);
        this.skip_sampling_above_y = Self::from_grid_y(max_surface_grid_y, 11).wrapping_sub(1);
        this
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#getIndex
    fn get_index(&self, x: i32, y: i32, z: i32) -> i32 {
        let dx = x.wrapping_sub(self.min_grid_x);
        let dy = y.wrapping_sub(self.min_grid_y);
        let dz = z.wrapping_sub(self.min_grid_z);
        dy.wrapping_mul(self.grid_size_z)
            .wrapping_add(dz)
            .wrapping_mul(self.grid_size_x)
            .wrapping_add(dx)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#similarity
    pub fn similarity(dist1: i32, dist2: i32) -> f64 {
        1.0 - (dist2.wrapping_sub(dist1)) as f64 / 25.0
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#gridX
    pub fn grid_x(x: i32) -> i32 {
        x >> (4i32 & 31)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#fromGridX
    pub fn from_grid_x(grid_x: i32, offset: i32) -> i32 {
        (grid_x << (4i32 & 31)).wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#gridY
    pub fn grid_y(y: i32) -> i32 {
        i32::div_euclid(y, 12)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#fromGridY
    pub fn from_grid_y(grid_y: i32, offset: i32) -> i32 {
        grid_y.wrapping_mul(12).wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#gridZ
    pub fn grid_z(z: i32) -> i32 {
        x_shift4(z)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#fromGridZ
    pub fn from_grid_z(grid_z: i32, offset: i32) -> i32 {
        (grid_z << (4i32 & 31)).wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#adjustSurfaceLevel
    fn adjust_surface_level(&self, surface_level: i32) -> i32 {
        surface_level.wrapping_add(8)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#getAquiferStatus
    fn get_aquifer_status(&mut self, index: i32) -> FluidStatus {
        let idx = index as usize;
        if let Some(status) = self.aquifer_cache[idx] {
            return status;
        }
        let pos = self.aquifer_location_cache[idx];
        let status = self.compute_fluid(
            BlockPos::get_x(pos),
            BlockPos::get_y(pos),
            BlockPos::get_z(pos),
        );
        self.aquifer_cache[idx] = Some(status);
        status
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#computeFluid
    fn compute_fluid(&self, x: i32, y: i32, z: i32) -> FluidStatus {
        let global_status = self.global_fluid_picker.compute_fluid(x, y, z);
        let mut min_preliminary_surface: i32 = i32::MAX;
        let surface_check_y_top = y.wrapping_add(12);
        let surface_check_y_bottom = y.wrapping_sub(12);
        let mut found_surface_aquifer = false;

        for offset in &Self::SURFACE_SAMPLING_OFFSETS_IN_CHUNKS {
            let sample_x = x.wrapping_add(SectionPos::section_to_block_coord(offset[0]));
            let sample_z = z.wrapping_add(SectionPos::section_to_block_coord(offset[1]));
            let preliminary_surface = self
                .noise_chunk
                .preliminary_surface_level(sample_x, sample_z);
            let adjusted_surface = self.adjust_surface_level(preliminary_surface);
            let is_center = offset[0] == 0 && offset[1] == 0;
            if is_center && surface_check_y_bottom > adjusted_surface {
                return global_status;
            }

            let is_above_surface = surface_check_y_top > adjusted_surface;
            if is_above_surface || is_center {
                let surface_fluid = self
                    .global_fluid_picker
                    .compute_fluid(sample_x, adjusted_surface, sample_z);
                if !surface_fluid.at(adjusted_surface).is_air() {
                    if is_center {
                        found_surface_aquifer = true;
                    }
                    if is_above_surface {
                        return surface_fluid;
                    }
                }
            }

            min_preliminary_surface = min_preliminary_surface.min(preliminary_surface);
        }

        let surface_level = self.compute_surface_level(
            x,
            y,
            z,
            global_status,
            min_preliminary_surface,
            found_surface_aquifer,
        );
        let fluid_type =
            self.compute_fluid_type(x, y, z, global_status, surface_level);
        FluidStatus {
            fluid_level: surface_level,
            fluid_type,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#computeSurfaceLevel
    fn compute_surface_level(
        &self,
        x: i32,
        y: i32,
        z: i32,
        global_status: FluidStatus,
        min_preliminary_surface: i32,
        found_surface_aquifer: bool,
    ) -> i32 {
        let ctx = SinglePointContext::new(x, y, z);
        let (floodedness_threshold, partially_floodedness_threshold) = if OverworldBiomeBuilder::is_deep_dark_region(
            self.erosion.as_ref(),
            self.depth.as_ref(),
            &ctx,
        ) {
            (-1.0f64, -1.0f64)
        } else {
            let distance_above_surface = min_preliminary_surface.wrapping_add(8).wrapping_sub(y);
            let mapped = if found_surface_aquifer {
                Mth::clamped_map(distance_above_surface as f64, 0.0, 64.0, 1.0, 0.0)
            } else {
                0.0
            };
            let floodedness = Mth::clamp(
                self.fluid_level_floodedness_noise.compute(&ctx),
                -1.0,
                1.0,
            );
            let mapped_partially = Mth::map(mapped, 1.0, 0.0, -0.3, 0.8);
            let mapped_fully = Mth::map(mapped, 1.0, 0.0, -0.8, 0.4);
            (floodedness - mapped_fully, floodedness - mapped_partially)
        };

        if partially_floodedness_threshold > 0.0 {
            global_status.fluid_level
        } else if floodedness_threshold > 0.0 {
            self.compute_randomized_fluid_surface_level(x, y, z, min_preliminary_surface)
        } else {
            DimensionType::WAY_BELOW_MIN_Y
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#computeRandomizedFluidSurfaceLevel
    fn compute_randomized_fluid_surface_level(
        &self,
        x: i32,
        y: i32,
        z: i32,
        min_preliminary_surface: i32,
    ) -> i32 {
        let cell_x = i32::div_euclid(x, 16);
        let cell_y = i32::div_euclid(y, 40);
        let cell_z = i32::div_euclid(z, 16);
        let base_y = cell_y.wrapping_mul(40).wrapping_add(20);
        let spread = self
            .fluid_level_spread_noise
            .compute(&SinglePointContext::new(cell_x, cell_y, cell_z))
            * 10.0;
        let quantized = Mth::quantize(spread, 3);
        let surface = base_y.wrapping_add(quantized);
        min_preliminary_surface.min(surface)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#computeFluidType
    fn compute_fluid_type(
        &self,
        x: i32,
        y: i32,
        z: i32,
        global_status: FluidStatus,
        surface_level: i32,
    ) -> BlockState {
        let mut fluid_type = global_status.fluid_type;
        if surface_level <= -10
            && surface_level != DimensionType::WAY_BELOW_MIN_Y
            && global_status.fluid_type != blocks::LAVA
        {
            let cell_x = i32::div_euclid(x, 64);
            let cell_y = i32::div_euclid(y, 40);
            let cell_z = i32::div_euclid(z, 64);
            let lava_noise = self
                .lava_noise
                .compute(&SinglePointContext::new(cell_x, cell_y, cell_z));
            if lava_noise.abs() > 0.3 {
                fluid_type = blocks::LAVA;
            }
        }
        fluid_type
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#calculatePressure
    fn calculate_pressure(
        &self,
        ctx: &dyn FunctionContext,
        barrier_noise_cache: &mut MutableDouble,
        status1: FluidStatus,
        status2: FluidStatus,
    ) -> f64 {
        let y = ctx.block_y();
        let block1 = status1.at(y);
        let block2 = status2.at(y);
        if (!block1.is(blocks::LAVA) || !block2.is(blocks::WATER))
            && (!block1.is(blocks::WATER) || !block2.is(blocks::LAVA))
        {
            let fluid_level_diff = status1
                .fluid_level
                .wrapping_sub(status2.fluid_level)
                .wrapping_abs();
            if fluid_level_diff == 0 {
                return 0.0;
            }
            let avg_fluid_level = 0.5
                * (status1.fluid_level.wrapping_add(status2.fluid_level)) as f64;
            let y_offset_from_avg = (y as f64) + 0.5 - avg_fluid_level;
            let half_diff = (fluid_level_diff as f64) / 2.0;
            let distance_from_edge = half_diff - y_offset_from_avg.abs();
            let pressure_factor = if y_offset_from_avg > 0.0 {
                let value = 0.0 + distance_from_edge;
                if value > 0.0 {
                    value / 1.5
                } else {
                    value / 2.5
                }
            } else {
                let value = 3.0 + distance_from_edge;
                if value > 0.0 {
                    value / 3.0
                } else {
                    value / 10.0
                }
            };

            let barrier_noise = if !(pressure_factor < -2.0) && !(pressure_factor > 2.0) {
                let cached = barrier_noise_cache.double_value();
                if cached.is_nan() {
                    let computed = self.barrier_noise.compute(ctx);
                    barrier_noise_cache.set_value(computed);
                    computed
                } else {
                    cached
                }
            } else {
                0.0
            };
            2.0 * (barrier_noise + pressure_factor)
        } else {
            2.0
        }
    }
}

impl Aquifer for NoiseBasedAquifer {
    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#computeSubstance
    fn compute_substance(&mut self, ctx: &dyn FunctionContext, density: f64) -> Option<BlockState> {
        if density > 0.0 {
            self.should_schedule_fluid_update_flag = false;
            return None;
        }

        let x = ctx.block_x();
        let y = ctx.block_y();
        let z = ctx.block_z();
        let global_status = self.global_fluid_picker.compute_fluid(x, y, z);
        if y > self.skip_sampling_above_y {
            self.should_schedule_fluid_update_flag = false;
            return Some(global_status.at(y));
        }

        if global_status.at(y).is(blocks::LAVA) {
            self.should_schedule_fluid_update_flag = false;
            return Some(if SharedConstants::DEBUG_DISABLE_FLUID_GENERATION {
                blocks::AIR
            } else {
                blocks::LAVA
            });
        }

        let grid_x = Self::grid_x(x.wrapping_add(Self::SAMPLE_OFFSET_X));
        let grid_y = Self::grid_y(y.wrapping_add(Self::SAMPLE_OFFSET_Y));
        let grid_z = Self::grid_z(z.wrapping_add(Self::SAMPLE_OFFSET_Z));
        let mut dist1 = i32::MAX;
        let mut dist2 = i32::MAX;
        let mut dist3 = i32::MAX;
        let mut dist4 = i32::MAX;
        let mut idx1: i32 = 0;
        let mut idx2: i32 = 0;
        let mut idx3: i32 = 0;
        let mut idx4: i32 = 0;

        for dx in 0i32..=1 {
            for dy in -1i32..=1 {
                for dz in 0i32..=1 {
                    let cell_x = grid_x.wrapping_add(dx);
                    let cell_y = grid_y.wrapping_add(dy);
                    let cell_z = grid_z.wrapping_add(dz);
                    let cell_index = self.get_index(cell_x, cell_y, cell_z);
                    let cell_idx_usize = cell_index as usize;
                    let cached_pos = self.aquifer_location_cache[cell_idx_usize];
                    let pos_long = if cached_pos != i64::MAX {
                        cached_pos
                    } else {
                        let mut rng = self
                            .positional_random_factory
                            .at(cell_x, cell_y, cell_z);
                        let px = Self::from_grid_x(cell_x, rng.next_int(Self::X_RANGE));
                        let py = Self::from_grid_y(cell_y, rng.next_int(Self::Y_RANGE));
                        let pz = Self::from_grid_z(cell_z, rng.next_int(Self::Z_RANGE));
                        let new_pos = BlockPos::as_long(px, py, pz);
                        self.aquifer_location_cache[cell_idx_usize] = new_pos;
                        new_pos
                    };

                    let px_diff = BlockPos::get_x(pos_long).wrapping_sub(x);
                    let py_diff = BlockPos::get_y(pos_long).wrapping_sub(y);
                    let pz_diff = BlockPos::get_z(pos_long).wrapping_sub(z);
                    let dist_sq = px_diff
                        .wrapping_mul(px_diff)
                        .wrapping_add(py_diff.wrapping_mul(py_diff))
                        .wrapping_add(pz_diff.wrapping_mul(pz_diff));

                    if dist1 >= dist_sq {
                        idx4 = idx3;
                        idx3 = idx2;
                        idx2 = idx1;
                        idx1 = cell_index;
                        dist4 = dist3;
                        dist3 = dist2;
                        dist2 = dist1;
                        dist1 = dist_sq;
                    } else if dist2 >= dist_sq {
                        idx4 = idx3;
                        idx3 = idx2;
                        idx2 = cell_index;
                        dist4 = dist3;
                        dist3 = dist2;
                        dist2 = dist_sq;
                    } else if dist3 >= dist_sq {
                        idx4 = idx3;
                        idx3 = cell_index;
                        dist4 = dist3;
                        dist3 = dist_sq;
                    } else if dist4 >= dist_sq {
                        idx4 = cell_index;
                        dist4 = dist_sq;
                    }
                }
            }
        }

        let status1 = self.get_aquifer_status(idx1);
        let sim12 = Self::similarity(dist1, dist2);
        let state_at_y = status1.at(y);
        let result_state = if SharedConstants::DEBUG_DISABLE_FLUID_GENERATION {
            blocks::AIR
        } else {
            state_at_y
        };

        if sim12 <= 0.0 {
            if sim12 >= Self::FLOWING_UPDATE_SIMULARITY {
                let status2 = self.get_aquifer_status(idx2);
                self.should_schedule_fluid_update_flag = status1 != status2;
            } else {
                self.should_schedule_fluid_update_flag = false;
            }
            return Some(result_state);
        }

        if state_at_y.is(blocks::WATER)
            && self
                .global_fluid_picker
                .compute_fluid(x, y.wrapping_sub(1), z)
                .at(y.wrapping_sub(1))
                .is(blocks::LAVA)
        {
            self.should_schedule_fluid_update_flag = true;
            return Some(result_state);
        }

        let mut barrier_cache = MutableDouble::new(f64::NAN);
        let status2 = self.get_aquifer_status(idx2);
        let pressure12 = sim12 * self.calculate_pressure(ctx, &mut barrier_cache, status1, status2);
        if density + pressure12 > 0.0 {
            self.should_schedule_fluid_update_flag = false;
            return None;
        }

        let status3 = self.get_aquifer_status(idx3);
        let sim13 = Self::similarity(dist1, dist3);
        if sim13 > 0.0 {
            let pressure13 =
                sim12 * sim13 * self.calculate_pressure(ctx, &mut barrier_cache, status1, status3);
            if density + pressure13 > 0.0 {
                self.should_schedule_fluid_update_flag = false;
                return None;
            }
        }

        let sim23 = Self::similarity(dist2, dist3);
        if sim23 > 0.0 {
            let pressure23 =
                sim12 * sim23 * self.calculate_pressure(ctx, &mut barrier_cache, status2, status3);
            if density + pressure23 > 0.0 {
                self.should_schedule_fluid_update_flag = false;
                return None;
            }
        }

        let different12 = status1 != status2;
        let different23 =
            sim23 >= Self::FLOWING_UPDATE_SIMULARITY && status2 != status3;
        let different13 =
            sim13 >= Self::FLOWING_UPDATE_SIMULARITY && status1 != status3;
        if !different12 && !different23 && !different13 {
            let status4 = self.get_aquifer_status(idx4);
            self.should_schedule_fluid_update_flag = sim13 >= Self::FLOWING_UPDATE_SIMULARITY
                && Self::similarity(dist1, dist4) >= Self::FLOWING_UPDATE_SIMULARITY
                && status1 != status4;
        } else {
            self.should_schedule_fluid_update_flag = true;
        }

        Some(result_state)
    }

    // JAVA: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer#shouldScheduleFluidUpdate
    fn should_schedule_fluid_update(&self) -> bool {
        self.should_schedule_fluid_update_flag
    }
}

fn x_shift4(x: i32) -> i32 {
    x >> (4i32 & 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstDensity(f64);
    impl DensityFunction for ConstDensity {
        fn compute(&self, _ctx: &dyn FunctionContext) -> f64 {
            self.0
        }
    }

    struct ConstFluidPicker(FluidStatus);
    impl FluidPicker for ConstFluidPicker {
        fn compute_fluid(&self, _x: i32, _y: i32, _z: i32) -> FluidStatus {
            self.0
        }
    }

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    struct FixedPosFactory(i32);
    impl PositionalRandomFactory for FixedPosFactory {
        fn at(&self, _x: i32, _y: i32, _z: i32) -> Box<dyn RandomSource> {
            Box::new(FixedRandom(self.0))
        }
    }

    struct FlatNoiseChunk(i32);
    impl NoiseChunkAccess for FlatNoiseChunk {
        fn max_preliminary_surface_level(&self, _x1: i32, _z1: i32, _x2: i32, _z2: i32) -> i32 {
            self.0
        }

        fn preliminary_surface_level(&self, _x: i32, _z: i32) -> i32 {
            self.0
        }
    }

    struct Ctx(i32, i32, i32);
    impl FunctionContext for Ctx {
        fn block_x(&self) -> i32 {
            self.0
        }
        fn block_y(&self) -> i32 {
            self.1
        }
        fn block_z(&self) -> i32 {
            self.2
        }
    }

    fn make_test_aquifer(surface: i32, fluid_level: i32, fluid_type: BlockState) -> NoiseBasedAquifer {
        let router = NoiseRouter {
            barrier_noise: Box::new(ConstDensity(0.0)),
            fluid_level_floodedness_noise: Box::new(ConstDensity(0.0)),
            fluid_level_spread_noise: Box::new(ConstDensity(0.0)),
            lava_noise: Box::new(ConstDensity(0.0)),
            erosion: Box::new(ConstDensity(0.0)),
            depth: Box::new(ConstDensity(0.0)),
        };
        NoiseBasedAquifer::new(
            Box::new(FlatNoiseChunk(surface)),
            ChunkPos::new(0, 0),
            router,
            Box::new(FixedPosFactory(0)),
            -64,
            384,
            Box::new(ConstFluidPicker(FluidStatus {
                fluid_level,
                fluid_type,
            })),
        )
    }

    #[test]
    fn block_pos_roundtrip_and_negative_inputs() {
        let cases = [
            (0i32, 0i32, 0i32),
            (1, -1, 2),
            (-100, 63, -200),
            (33_554_431, -2048, -33_554_432),
            (-33_554_432, 2047, 33_554_431),
        ];
        for (x, y, z) in cases {
            let packed = BlockPos::as_long(x, y, z);
            assert_eq!(BlockPos::get_x(packed), x);
            assert_eq!(BlockPos::get_y(packed), y);
            assert_eq!(BlockPos::get_z(packed), z);
        }
    }

    #[test]
    fn grid_functions_handle_negative_inputs() {
        assert_eq!(NoiseBasedAquifer::grid_x(-1), -1);
        assert_eq!(NoiseBasedAquifer::grid_x(-16), -1);
        assert_eq!(NoiseBasedAquifer::grid_x(-17), -2);
        assert_eq!(NoiseBasedAquifer::grid_y(-1), -1);
        assert_eq!(NoiseBasedAquifer::grid_y(-12), -1);
        assert_eq!(NoiseBasedAquifer::grid_y(-13), -2);
        assert_eq!(NoiseBasedAquifer::from_grid_x(-1, 5), -11);
        assert_eq!(NoiseBasedAquifer::from_grid_y(-1, 5), -7);
        assert_eq!(NoiseBasedAquifer::from_grid_z(-2, 3), -29);
    }

    #[test]
    fn similarity_and_float_bits() {
        let expected = -0.76f64;
        assert_eq!(
            NoiseBasedAquifer::FLOWING_UPDATE_SIMULARITY.to_bits(),
            expected.to_bits()
        );
        assert_eq!(
            NoiseBasedAquifer::similarity(100, 144).to_bits(),
            expected.to_bits()
        );
        assert_eq!(NoiseBasedAquifer::similarity(0, 0).to_bits(), 1.0f64.to_bits());
        // integer subtraction wrapping case
        let wrap = NoiseBasedAquifer::similarity(i32::MAX, i32::MIN);
        let expected_wrap = 1.0 - (i32::MIN.wrapping_sub(i32::MAX) as f64) / 25.0;
        assert_eq!(wrap.to_bits(), expected_wrap.to_bits());
    }

    #[test]
    fn fluid_status_at_and_disabled_aquifer() {
        let water = FluidStatus {
            fluid_level: 63,
            fluid_type: blocks::WATER,
        };
        assert_eq!(water.at(62), blocks::WATER);
        assert_eq!(water.at(63), blocks::AIR);
        assert_eq!(water.at(64), blocks::AIR);

        let picker = Box::new(ConstFluidPicker(FluidStatus {
            fluid_level: 0,
            fluid_type: blocks::WATER,
        }));
        let mut aq = create_disabled(picker);
        let below = Ctx(1, -5, 2);
        let above = Ctx(1, 5, 2);
        assert_eq!(aq.compute_substance(&below, 1.0), None);
        assert_eq!(aq.compute_substance(&below, 0.0), Some(blocks::WATER));
        assert_eq!(aq.compute_substance(&above, -1.0), Some(blocks::AIR));
        assert!(!aq.should_schedule_fluid_update());
    }

    #[test]
    fn noise_based_aquifer_basic_behavior_and_overflow_safe() {
        let mut aq = make_test_aquifer(64, 63, blocks::WATER);
        let solid_ctx = Ctx(0, 50, 0);
        assert_eq!(aq.compute_substance(&solid_ctx, 1.0), None);
        assert!(!aq.should_schedule_fluid_update());

        let fluid_ctx = Ctx(0, 0, 0);
        let result = aq.compute_substance(&fluid_ctx, -1.0);
        assert!(matches!(result, Some(s) if s == blocks::WATER || s == blocks::AIR || s == blocks::LAVA));

        // Large/negative coordinates should not panic and should respect integer semantics.
        let mut aq2 = make_test_aquifer(0, -10, blocks::LAVA);
        let edge_ctx = Ctx(i32::MIN / 2, -64, i32::MAX / 2);
        let _ = aq2.compute_substance(&edge_ctx, -0.5);
        let _ = aq2.should_schedule_fluid_update();
    }
}
