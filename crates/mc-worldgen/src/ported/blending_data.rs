use std::cmp;
use std::fmt;

pub const BLENDING_DENSITY_FACTOR: f64 = 0.1;
pub const CELL_WIDTH: i32 = 4;
pub const CELL_HEIGHT: i32 = 8;
pub const CELL_RATIO: i32 = 2;
const SOLID_DENSITY: f64 = 1.0;
const AIR_DENSITY: f64 = -1.0;
const CELLS_PER_SECTION_Y: i32 = 2;
const QUARTS_PER_SECTION: i32 = 4; // QuartPos.fromBlock(16)
const CELL_HORIZONTAL_MAX_INDEX_INSIDE: i32 = QUARTS_PER_SECTION - 1;
const CELL_HORIZONTAL_MAX_INDEX_OUTSIDE: i32 = QUARTS_PER_SECTION;
const CELL_COLUMN_INSIDE_COUNT: i32 = 2 * CELL_HORIZONTAL_MAX_INDEX_INSIDE + 1;
const CELL_COLUMN_OUTSIDE_COUNT: i32 = 2 * CELL_HORIZONTAL_MAX_INDEX_OUTSIDE + 1;
pub const CELL_COLUMN_COUNT: usize =
    (CELL_COLUMN_INSIDE_COUNT + CELL_COLUMN_OUTSIDE_COUNT) as usize;
pub const NO_VALUE: f64 = f64::MAX;

fn section_to_block_coord(section: i32) -> i32 {
    section.wrapping_shl(4)
}

fn block_to_section_coord(block: i32) -> i32 {
    block >> 4
}

fn quart_pos_from_block(block: i32) -> i32 {
    block >> 2
}

fn quart_pos_from_section(section: i32) -> i32 {
    section.wrapping_shl(2)
}

fn mth_floor(v: f64) -> i32 {
    let i = v as i32;
    if v < i as f64 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

fn mth_floor_div(a: i32, b: i32) -> i32 {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((a ^ b) < 0) {
        q.wrapping_sub(1)
    } else {
        q
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction8 {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl Direction8 {
    pub const VALUES: [Direction8; 8] = [
        Direction8::North,
        Direction8::NorthEast,
        Direction8::East,
        Direction8::SouthEast,
        Direction8::South,
        Direction8::SouthWest,
        Direction8::West,
        Direction8::NorthWest,
    ];

    pub fn step_x(self) -> i32 {
        match self {
            Direction8::East | Direction8::NorthEast | Direction8::SouthEast => 1,
            Direction8::West | Direction8::NorthWest | Direction8::SouthWest => -1,
            _ => 0,
        }
    }

    pub fn step_z(self) -> i32 {
        match self {
            Direction8::South | Direction8::SouthEast | Direction8::SouthWest => 1,
            Direction8::North | Direction8::NorthEast | Direction8::NorthWest => -1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Direction8Set(u8);

impl Direction8Set {
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn insert(&mut self, d: Direction8) {
        self.0 |= 1u8 << (d as u8);
    }

    pub fn contains(&self, d: Direction8) -> bool {
        (self.0 & (1u8 << (d as u8))) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn move_down(&mut self) -> &mut Self {
        self.y = self.y.wrapping_sub(1);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelHeightAccessor {
    min_y: i32,
    height: i32,
}

impl LevelHeightAccessor {
    pub fn create(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    pub fn get_min_section_y(&self) -> i32 {
        block_to_section_coord(self.min_y)
    }

    pub fn get_max_section_y(&self) -> i32 {
        block_to_section_coord(self.min_y.wrapping_add(self.height).wrapping_sub(1))
    }

    pub fn get_max_y(&self) -> i32 {
        self.min_y.wrapping_add(self.height).wrapping_sub(1)
    }

    pub fn get_min_y(&self) -> i32 {
        self.min_y
    }

    pub fn get_sections_count(&self) -> i32 {
        self.get_max_section_y()
            .wrapping_sub(self.get_min_section_y())
            .wrapping_add(1)
    }
}

pub trait BlendingBlockState {
    fn is_air(&self) -> bool;
    fn is_leaves(&self) -> bool;
    fn is_logs(&self) -> bool;
    fn is_brown_mushroom_block(&self) -> bool;
    fn is_red_mushroom_block(&self) -> bool;
    fn has_empty_collision_shape(&self) -> bool;
    fn is_surface_blending_block(&self) -> bool;
}

pub trait BlendingChunkRead {
    type Biome;
    type BlockState: BlendingBlockState;

    fn highest_generated_status_is_before_biomes(&self) -> bool;
    fn has_primed_heightmap_world_surface_wg(&self) -> bool;
    fn get_height_world_surface_wg(&self, x: i32, z: i32) -> i32;
    fn get_block_state_at(&self, x: i32, y: i32, z: i32) -> Self::BlockState;
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Self::Biome;
}

pub trait ChunkOldNoiseQuery {
    fn is_old_noise_generation_at(&self, chunk_x: i32, chunk_z: i32) -> bool;
}

impl<F: Fn(i32, i32) -> bool> ChunkOldNoiseQuery for F {
    fn is_old_noise_generation_at(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self(chunk_x, chunk_z)
    }
}

pub trait BiomeConsumer<B> {
    fn consume(&mut self, x: i32, z: i32, biome: &B);
}

impl<B, F: FnMut(i32, i32, &B)> BiomeConsumer<B> for F {
    fn consume(&mut self, x: i32, z: i32, biome: &B) {
        self(x, z, biome);
    }
}

pub trait HeightConsumer {
    fn consume(&mut self, x: i32, z: i32, height: f64);
}

impl<F: FnMut(i32, i32, f64)> HeightConsumer for F {
    fn consume(&mut self, x: i32, z: i32, height: f64) {
        self(x, z, height);
    }
}

pub trait DensityConsumer {
    fn consume(&mut self, x: i32, y: i32, z: i32, density: f64);
}

impl<F: FnMut(i32, i32, i32, f64)> DensityConsumer for F {
    fn consume(&mut self, x: i32, y: i32, z: i32, density: f64) {
        self(x, y, z, density);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Packed {
    pub min_section: i32,
    pub max_section: i32,
    pub heights: Option<Vec<f64>>,
}

impl Packed {
    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData$Packed#validateArraySize
    pub fn validate_array_size(&self) -> Result<(), String> {
        if let Some(h) = &self.heights {
            if h.len() != CELL_COLUMN_COUNT {
                return Err(format!(
                    "heights has to be of length {}",
                    CELL_COLUMN_COUNT
                ));
            }
        }
        Ok(())
    }
}

pub struct BlendingData<B> {
    area_with_old_generation: LevelHeightAccessor,
    has_calculated_data: bool,
    heights: [f64; CELL_COLUMN_COUNT],
    biomes: Vec<Option<Vec<B>>>,
    densities: Vec<Option<Vec<f64>>>,
}

impl<B: fmt::Debug> fmt::Debug for BlendingData<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlendingData")
            .field("area_with_old_generation", &self.area_with_old_generation)
            .field("has_calculated_data", &self.has_calculated_data)
            .field("heights", &self.heights)
            .finish()
    }
}

impl<B> BlendingData<B> {
    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#<init>
    pub fn new(min_section: i32, max_section: i32, heights_opt: Option<Vec<f64>>) -> Self {
        let heights = match heights_opt {
            Some(v) => {
                let mut arr = [NO_VALUE; CELL_COLUMN_COUNT];
                let n = v.len().min(CELL_COLUMN_COUNT);
                arr[..n].copy_from_slice(&v[..n]);
                arr
            }
            None => [NO_VALUE; CELL_COLUMN_COUNT],
        };
        let mut biomes: Vec<Option<Vec<B>>> = Vec::with_capacity(CELL_COLUMN_COUNT);
        biomes.resize_with(CELL_COLUMN_COUNT, || None);
        let mut densities: Vec<Option<Vec<f64>>> = Vec::with_capacity(CELL_COLUMN_COUNT);
        densities.resize_with(CELL_COLUMN_COUNT, || None);
        let min_y = section_to_block_coord(min_section);
        let height = section_to_block_coord(max_section).wrapping_sub(min_y);
        Self {
            area_with_old_generation: LevelHeightAccessor::create(min_y, height),
            has_calculated_data: false,
            heights,
            biomes,
            densities,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#unpack
    pub fn unpack(packed: Option<&Packed>) -> Option<Self> {
        packed.map(|p| Self::new(p.min_section, p.max_section, p.heights.clone()))
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#pack
    pub fn pack(&self) -> Packed {
        let mut any = false;
        for &h in &self.heights {
            if h != NO_VALUE {
                any = true;
                break;
            }
        }
        Packed {
            min_section: self.area_with_old_generation.get_min_section_y(),
            max_section: self
                .area_with_old_generation
                .get_max_section_y()
                .wrapping_add(1),
            heights: if any {
                Some(self.heights.to_vec())
            } else {
                None
            },
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getOrUpdateBlendingData
    pub fn get_or_update_blending_data<'a, R, Q>(
        blending_data: &'a mut Option<Self>,
        chunk: &R,
        level: &Q,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&'a Self>
    where
        R: BlendingChunkRead<Biome = B>,
        Q: ChunkOldNoiseQuery,
    {
        let data = blending_data.as_mut()?;
        if chunk.highest_generated_status_is_before_biomes() {
            return None;
        }
        let sides = side_by_generation_age(level, chunk_x, chunk_z, false);
        data.calculate_data(chunk, &sides);
        Some(data)
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#calculateData
    fn calculate_data<R: BlendingChunkRead<Biome = B>>(
        &mut self,
        chunk: &R,
        sides: &Direction8Set,
    ) {
        if !self.has_calculated_data {
            if sides.contains(Direction8::North)
                || sides.contains(Direction8::West)
                || sides.contains(Direction8::NorthWest)
            {
                self.add_values_for_column(get_inside_index(0, 0), chunk, 0, 0);
            }

            if sides.contains(Direction8::North) {
                let mut i = 1i32;
                while i < QUARTS_PER_SECTION {
                    self.add_values_for_column(
                        get_inside_index(i, 0),
                        chunk,
                        4i32.wrapping_mul(i),
                        0,
                    );
                    i = i.wrapping_add(1);
                }
            }

            if sides.contains(Direction8::West) {
                let mut i = 1i32;
                while i < QUARTS_PER_SECTION {
                    self.add_values_for_column(
                        get_inside_index(0, i),
                        chunk,
                        0,
                        4i32.wrapping_mul(i),
                    );
                    i = i.wrapping_add(1);
                }
            }

            if sides.contains(Direction8::East) {
                let mut i = 1i32;
                while i < QUARTS_PER_SECTION {
                    self.add_values_for_column(
                        get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, i),
                        chunk,
                        15,
                        4i32.wrapping_mul(i),
                    );
                    i = i.wrapping_add(1);
                }
            }

            if sides.contains(Direction8::South) {
                let mut i = 0i32;
                while i < QUARTS_PER_SECTION {
                    self.add_values_for_column(
                        get_outside_index(i, CELL_HORIZONTAL_MAX_INDEX_OUTSIDE),
                        chunk,
                        4i32.wrapping_mul(i),
                        15,
                    );
                    i = i.wrapping_add(1);
                }
            }

            if sides.contains(Direction8::East) && sides.contains(Direction8::NorthEast) {
                self.add_values_for_column(
                    get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, 0),
                    chunk,
                    15,
                    0,
                );
            }

            if sides.contains(Direction8::East)
                && sides.contains(Direction8::South)
                && sides.contains(Direction8::SouthEast)
            {
                self.add_values_for_column(
                    get_outside_index(
                        CELL_HORIZONTAL_MAX_INDEX_OUTSIDE,
                        CELL_HORIZONTAL_MAX_INDEX_OUTSIDE,
                    ),
                    chunk,
                    15,
                    15,
                );
            }

            self.has_calculated_data = true;
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#addValuesForColumn
    fn add_values_for_column<R: BlendingChunkRead<Biome = B>>(
        &mut self,
        index: i32,
        chunk: &R,
        x: i32,
        z: i32,
    ) {
        let idx = index as usize;
        if self.heights[idx] == NO_VALUE {
            self.heights[idx] = self.get_height_at_xz(chunk, x, z);
        }
        let h = mth_floor(self.heights[idx]);
        self.densities[idx] = Some(self.get_density_column(chunk, x, z, h));
        self.biomes[idx] = Some(self.get_biome_column(chunk, x, z));
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getHeightAtXZ
    fn get_height_at_xz<R: BlendingChunkRead<Biome = B>>(
        &self,
        chunk: &R,
        x: i32,
        z: i32,
    ) -> f64 {
        let max_y = self.area_with_old_generation.get_max_y();
        let min_y = self.area_with_old_generation.get_min_y();
        let y0 = if chunk.has_primed_heightmap_world_surface_wg() {
            cmp::min(chunk.get_height_world_surface_wg(x, z), max_y)
        } else {
            max_y
        };
        let mut pos = BlockPos::new(x, y0, z);
        while pos.get_y() > min_y {
            let state = chunk.get_block_state_at(pos.x, pos.y, pos.z);
            if state.is_surface_blending_block() {
                return pos.get_y() as f64;
            }
            pos.move_down();
        }
        min_y as f64
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getDensityColumn
    fn get_density_column<R: BlendingChunkRead<Biome = B>>(
        &self,
        chunk: &R,
        x: i32,
        z: i32,
        height: i32,
    ) -> Vec<f64> {
        let len = self.cell_count_per_column() as usize;
        let mut col = vec![AIR_DENSITY; len];
        let mut pos = BlockPos::new(
            x,
            self.area_with_old_generation.get_max_y().wrapping_add(1),
            z,
        );
        let mut d6 = read7(chunk, &mut pos);
        let mut i = (len as i32).wrapping_sub(2);
        while i >= 0 {
            let d8 = read1(chunk, &mut pos);
            let d9 = read7(chunk, &mut pos);
            col[i as usize] = (d6 + d8 + d9) / 15.0;
            d6 = d9;
            i = i.wrapping_sub(1);
        }
        let idx = self.get_cell_y_index(mth_floor_div(height, 8));
        if idx >= 0 && idx < (len as i32).wrapping_sub(1) {
            let d11 = (((height as f64) + 0.5) % 8.0) / 8.0;
            let d12 = (1.0 - d11) / d11;
            let d13 = d12.max(1.0) * 0.25;
            col[idx.wrapping_add(1) as usize] = -d12 / d13;
            col[idx as usize] = 1.0 / d13;
        }
        col
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getBiomeColumn
    fn get_biome_column<R: BlendingChunkRead<Biome = B>>(
        &self,
        chunk: &R,
        x: i32,
        z: i32,
    ) -> Vec<B> {
        let size = self.quart_count_per_column() as usize;
        let mut col: Vec<B> = Vec::with_capacity(size);
        let qx = quart_pos_from_block(x);
        let qz = quart_pos_from_block(z);
        let min_y_quart = quart_pos_from_block(self.area_with_old_generation.get_min_y());
        let mut i = 0i32;
        while i < size as i32 {
            let qy = i.wrapping_add(min_y_quart);
            col.push(chunk.get_noise_biome(qx, qy, qz));
            i = i.wrapping_add(1);
        }
        col
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getHeight
    pub fn get_height(&self, x: i32, _y: i32, z: i32) -> f64 {
        if x == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE || z == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            return self.heights[get_outside_index(x, z) as usize];
        }
        if x != 0 && z != 0 {
            return NO_VALUE;
        }
        self.heights[get_inside_index(x, z) as usize]
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getDensity
    fn get_density_at(&self, col: Option<&[f64]>, y: i32) -> f64 {
        let Some(col) = col else {
            return NO_VALUE;
        };
        let idx = self.get_cell_y_index(y);
        if idx >= 0 && idx < col.len() as i32 {
            col[idx as usize] * 0.1
        } else {
            NO_VALUE
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getDensity
    pub fn get_density(&self, x: i32, y: i32, z: i32) -> f64 {
        if y == self.get_min_y() {
            return 0.1;
        }
        if x == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE || z == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            return self.get_density_at(
                self.densities[get_outside_index(x, z) as usize].as_deref(),
                y,
            );
        }
        if x != 0 && z != 0 {
            return NO_VALUE;
        }
        self.get_density_at(
            self.densities[get_inside_index(x, z) as usize].as_deref(),
            y,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#iterateBiomes
    pub fn iterate_biomes<C: BiomeConsumer<B>>(&self, x: i32, y: i32, z: i32, consumer: &mut C) {
        let min_y_quart = quart_pos_from_block(self.area_with_old_generation.get_min_y());
        let max_y_quart = quart_pos_from_block(self.area_with_old_generation.get_max_y());
        if y >= min_y_quart && y <= max_y_quart {
            let idx_y = y.wrapping_sub(min_y_quart) as usize;
            let mut i = 0usize;
            while i < self.biomes.len() {
                if let Some(col) = &self.biomes[i] {
                    if idx_y < col.len() {
                        let biome = &col[idx_y];
                        consumer.consume(
                            x.wrapping_add(get_x(i as i32)),
                            z.wrapping_add(get_z(i as i32)),
                            biome,
                        );
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#iterateHeights
    pub fn iterate_heights<C: HeightConsumer>(&self, x: i32, z: i32, consumer: &mut C) {
        let mut i = 0usize;
        while i < self.heights.len() {
            let h = self.heights[i];
            if h != NO_VALUE {
                consumer.consume(
                    x.wrapping_add(get_x(i as i32)),
                    z.wrapping_add(get_z(i as i32)),
                    h,
                );
            }
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#iterateDensities
    pub fn iterate_densities<C: DensityConsumer>(
        &self,
        x: i32,
        z: i32,
        min_y: i32,
        max_y: i32,
        consumer: &mut C,
    ) {
        let col_min_y = self.get_column_min_y();
        let i_min = cmp::max(0, min_y.wrapping_sub(col_min_y));
        let i_max = cmp::min(self.cell_count_per_column(), max_y.wrapping_sub(col_min_y));
        let mut i = 0usize;
        while i < self.densities.len() {
            if let Some(col) = &self.densities[i] {
                let dx = x.wrapping_add(get_x(i as i32));
                let dz = z.wrapping_add(get_z(i as i32));
                let mut j = i_min;
                while j < i_max {
                    consumer.consume(dx, j.wrapping_add(col_min_y), dz, col[j as usize] * 0.1);
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#cellCountPerColumn
    fn cell_count_per_column(&self) -> i32 {
        self.area_with_old_generation
            .get_sections_count()
            .wrapping_mul(CELLS_PER_SECTION_Y)
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#quartCountPerColumn
    fn quart_count_per_column(&self) -> i32 {
        quart_pos_from_section(self.area_with_old_generation.get_sections_count())
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getColumnMinY
    fn get_column_min_y(&self) -> i32 {
        self.get_min_y().wrapping_add(1)
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getMinY
    fn get_min_y(&self) -> i32 {
        self.area_with_old_generation
            .get_min_section_y()
            .wrapping_mul(CELLS_PER_SECTION_Y)
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getCellYIndex
    fn get_cell_y_index(&self, y: i32) -> i32 {
        y.wrapping_sub(self.get_column_min_y())
    }

    // JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getAreaWithOldGeneration
    pub fn get_area_with_old_generation(&self) -> &LevelHeightAccessor {
        &self.area_with_old_generation
    }
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#sideByGenerationAge
pub fn side_by_generation_age<Q: ChunkOldNoiseQuery>(
    level: &Q,
    chunk_x: i32,
    chunk_z: i32,
    old_noise_generation: bool,
) -> Direction8Set {
    let mut set = Direction8Set::empty();
    for &d in &Direction8::VALUES {
        let nx = chunk_x.wrapping_add(d.step_x());
        let nz = chunk_z.wrapping_add(d.step_z());
        if level.is_old_noise_generation_at(nx, nz) == old_noise_generation {
            set.insert(d);
        }
    }
    set
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#read1
fn read1<R: BlendingChunkRead>(chunk: &R, pos: &mut BlockPos) -> f64 {
    pos.move_down();
    if is_ground(chunk, pos) {
        SOLID_DENSITY
    } else {
        AIR_DENSITY
    }
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#read7
fn read7<R: BlendingChunkRead>(chunk: &R, pos: &mut BlockPos) -> f64 {
    let mut d = 0.0f64;
    let mut i = 0i32;
    while i < 7 {
        d += read1(chunk, pos);
        i = i.wrapping_add(1);
    }
    d
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#isGround
fn is_ground<R: BlendingChunkRead>(chunk: &R, pos: &BlockPos) -> bool {
    let state = chunk.get_block_state_at(pos.x, pos.y, pos.z);
    if state.is_air() {
        return false;
    }
    if state.is_leaves() {
        return false;
    }
    if state.is_logs() {
        return false;
    }
    if state.is_brown_mushroom_block() || state.is_red_mushroom_block() {
        return false;
    }
    !state.has_empty_collision_shape()
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getInsideIndex
fn get_inside_index(x: i32, z: i32) -> i32 {
    CELL_HORIZONTAL_MAX_INDEX_INSIDE
        .wrapping_sub(x)
        .wrapping_add(z)
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getOutsideIndex
fn get_outside_index(x: i32, z: i32) -> i32 {
    CELL_COLUMN_INSIDE_COUNT
        .wrapping_add(x)
        .wrapping_add(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE)
        .wrapping_sub(z)
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getX
fn get_x(index: i32) -> i32 {
    if index < CELL_COLUMN_INSIDE_COUNT {
        zero_if_negative(CELL_HORIZONTAL_MAX_INDEX_INSIDE.wrapping_sub(index))
    } else {
        let i = index.wrapping_sub(CELL_COLUMN_INSIDE_COUNT);
        CELL_HORIZONTAL_MAX_INDEX_OUTSIDE.wrapping_sub(zero_if_negative(
            CELL_HORIZONTAL_MAX_INDEX_OUTSIDE.wrapping_sub(i),
        ))
    }
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#getZ
fn get_z(index: i32) -> i32 {
    if index < CELL_COLUMN_INSIDE_COUNT {
        zero_if_negative(index.wrapping_sub(CELL_HORIZONTAL_MAX_INDEX_INSIDE))
    } else {
        let i = index.wrapping_sub(CELL_COLUMN_INSIDE_COUNT);
        CELL_HORIZONTAL_MAX_INDEX_OUTSIDE.wrapping_sub(zero_if_negative(
            i.wrapping_sub(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE),
        ))
    }
}

// JAVA: net.minecraft.world.level.levelgen.blending.BlendingData#zeroIfNegative
fn zero_if_negative(v: i32) -> i32 {
    v & !(v >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MockState {
        solid: bool,
    }

    impl BlendingBlockState for MockState {
        fn is_air(&self) -> bool {
            !self.solid
        }
        fn is_leaves(&self) -> bool {
            false
        }
        fn is_logs(&self) -> bool {
            false
        }
        fn is_brown_mushroom_block(&self) -> bool {
            false
        }
        fn is_red_mushroom_block(&self) -> bool {
            false
        }
        fn has_empty_collision_shape(&self) -> bool {
            !self.solid
        }
        fn is_surface_blending_block(&self) -> bool {
            self.solid
        }
    }

    struct MockChunk {
        solid_below: i32,
        before_biomes: bool,
    }

    impl BlendingChunkRead for MockChunk {
        type Biome = u32;
        type BlockState = MockState;

        fn highest_generated_status_is_before_biomes(&self) -> bool {
            self.before_biomes
        }
        fn has_primed_heightmap_world_surface_wg(&self) -> bool {
            true
        }
        fn get_height_world_surface_wg(&self, _x: i32, _z: i32) -> i32 {
            self.solid_below
        }
        fn get_block_state_at(&self, _x: i32, y: i32, _z: i32) -> MockState {
            MockState {
                solid: y < self.solid_below,
            }
        }
        fn get_noise_biome(&self, _qx: i32, _qy: i32, _qz: i32) -> u32 {
            7
        }
    }

    #[test]
    fn test_zero_if_negative_and_indices() {
        assert_eq!(zero_if_negative(0), 0);
        assert_eq!(zero_if_negative(5), 5);
        assert_eq!(zero_if_negative(-1), 0);
        assert_eq!(zero_if_negative(i32::MIN), 0);
        assert_eq!(zero_if_negative(i32::MAX), i32::MAX);

        let idx = get_inside_index(0, 0);
        assert_eq!(get_x(idx), 0);
        assert_eq!(get_z(idx), 0);
        let idx2 = get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, 0);
        assert_eq!(get_x(idx2), CELL_HORIZONTAL_MAX_INDEX_OUTSIDE);
        assert_eq!(get_z(idx2), 0);
    }

    #[test]
    fn test_negative_inputs_and_floor_div() {
        assert_eq!(mth_floor(-0.1), -1);
        assert_eq!(mth_floor(-1.0), -1);
        assert_eq!(mth_floor(1.9), 1);
        assert_eq!(mth_floor_div(-1, 8), -1);
        assert_eq!(mth_floor_div(-8, 8), -1);
        assert_eq!(mth_floor_div(-9, 8), -2);
        assert_eq!(mth_floor_div(7, 8), 0);

        let data = BlendingData::<u32>::new(-1, 1, None);
        let idx = data.get_cell_y_index(data.get_column_min_y());
        assert_eq!(idx, 0);
        assert_eq!(data.get_min_y(), -1i32.wrapping_mul(CELLS_PER_SECTION_Y));
    }

    #[test]
    fn test_packed_validate_and_roundtrip() {
        let mut data = BlendingData::<u32>::new(0, 1, None);
        let packed = data.pack();
        assert!(packed.heights.is_none());
        assert!(packed.validate_array_size().is_ok());

        data.heights[0] = 10.0;
        let packed2 = data.pack();
        assert_eq!(packed2.heights.as_ref().unwrap().len(), CELL_COLUMN_COUNT);
        let unpacked = BlendingData::<u32>::unpack(Some(&packed2)).unwrap();
        assert_eq!(unpacked.heights[0], 10.0);

        let bad = Packed {
            min_section: 0,
            max_section: 1,
            heights: Some(vec![1.0, 2.0]),
        };
        assert!(bad.validate_array_size().is_err());
    }

    #[test]
    fn test_density_float_bits_and_height() {
        let mut data = BlendingData::<u32>::new(0, 1, None);
        let col_len = data.cell_count_per_column() as usize;
        let mut col = vec![-1.0f64; col_len];
        col[0] = 1.0;
        data.densities[get_inside_index(0, 0) as usize] = Some(col);

        let y = data.get_column_min_y();
        let d = data.get_density(0, y, 0);
        assert_eq!(d.to_bits(), (0.1f64).to_bits());
        assert_eq!(
            data.get_density(0, data.get_min_y(), 0).to_bits(),
            0.1f64.to_bits()
        );
        assert_eq!(data.get_density(1, y, 1), NO_VALUE);

        let chunk = MockChunk {
            solid_below: 8,
            before_biomes: false,
        };
        let h = data.get_height_at_xz(&chunk, 0, 0);
        assert!(h <= chunk.solid_below as f64);
    }

    #[test]
    fn test_side_by_generation_age_and_update() {
        let q = |cx: i32, cz: i32| -> bool { cx == 0 && cz == -1 };
        let sides = side_by_generation_age(&q, 0, 0, true);
        assert!(sides.contains(Direction8::North));
        assert!(!sides.contains(Direction8::South));

        let chunk = MockChunk {
            solid_below: 4,
            before_biomes: false,
        };
        let mut bd: Option<BlendingData<u32>> = Some(BlendingData::new(0, 1, None));
        let got = BlendingData::get_or_update_blending_data(&mut bd, &chunk, &q, 0, 0);
        assert!(got.is_some());

        let mut bd2: Option<BlendingData<u32>> = Some(BlendingData::new(0, 1, None));
        let chunk_before = MockChunk {
            solid_below: 4,
            before_biomes: true,
        };
        let got2 = BlendingData::get_or_update_blending_data(&mut bd2, &chunk_before, &q, 0, 0);
        assert!(got2.is_none());
    }
}
