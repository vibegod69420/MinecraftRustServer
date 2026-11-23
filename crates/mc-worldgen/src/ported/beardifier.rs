// === FILE: beardifier.rs ===

use std::sync::LazyLock;

pub const BEARD_KERNEL_RADIUS: i32 = 12;
const BEARD_KERNEL_SIZE: i32 = 24;

fn mth_length_squared(x: f64, y: f64, z: f64) -> f64 {
    x * x + y * y + z * z
}

fn mth_length(x: f64, y: f64, z: f64) -> f64 {
    (x * x + y * y + z * z).sqrt()
}

fn mth_clamped_map(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    let t = (value - in_min) / (in_max - in_min);
    if t < 0.0 {
        out_min
    } else if t > 1.0 {
        out_max
    } else {
        out_min + t * (out_max - out_min)
    }
}

fn mth_fast_inv_sqrt(x: f64) -> f64 {
    let d = 0.5 * x;
    let mut l = x.to_bits() as i64;
    l = 6_910_469_410_427_058_090_i64.wrapping_sub(l >> 1);
    let x = f64::from_bits(l as u64);
    x * (1.5 - d * x * x)
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn from_point(pos: BlockPos) -> Self {
        Self {
            min_x: pos.x,
            min_y: pos.y,
            min_z: pos.z,
            max_x: pos.x,
            max_y: pos.y,
            max_z: pos.z,
        }
    }

    pub fn is_inside(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min_x
            && x <= self.max_x
            && z >= self.min_z
            && z <= self.max_z
            && y >= self.min_y
            && y <= self.max_y
    }

    pub fn inflated_by(&self, value: i32) -> Self {
        Self {
            min_x: self.min_x.wrapping_sub(value),
            min_y: self.min_y.wrapping_sub(value),
            min_z: self.min_z.wrapping_sub(value),
            max_x: self.max_x.wrapping_add(value),
            max_y: self.max_y.wrapping_add(value),
            max_z: self.max_z.wrapping_add(value),
        }
    }

    pub fn encapsulating(a: Self, b: Self) -> Self {
        Self {
            min_x: a.min_x.min(b.min_x),
            min_y: a.min_y.min(b.min_y),
            min_z: a.min_z.min(b.min_z),
            max_x: a.max_x.max(b.max_x),
            max_y: a.max_y.max(b.max_y),
            max_z: a.max_z.max(b.max_z),
        }
    }

    pub fn intersects_xz(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> bool {
        self.max_x >= min_x && self.min_x <= max_x && self.max_z >= min_z && self.min_z <= max_z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JigsawJunction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerrainAdjustment {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureTemplateProjection {
    TerrainMatching,
    Rigid,
}

pub trait FunctionContext {
    fn block_x(&self) -> i32;
    fn block_y(&self) -> i32;
    fn block_z(&self) -> i32;
}

pub trait ContextProvider {
    type Ctx: FunctionContext;
    fn for_index(&mut self, index: i32) -> Self::Ctx;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockContext {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl FunctionContext for BlockContext {
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

pub struct FnContextProvider<F> {
    f: F,
}

impl<F> FnContextProvider<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F, C: FunctionContext> ContextProvider for FnContextProvider<F>
where
    F: FnMut(i32) -> C,
{
    type Ctx = C;

    fn for_index(&mut self, index: i32) -> Self::Ctx {
        (self.f)(index)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolElementPieceData {
    pub bounding_box: BoundingBox,
    pub projection: StructureTemplateProjection,
    pub ground_level_delta: i32,
    pub junctions: Vec<JigsawJunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceData {
    PoolElement(PoolElementPieceData),
    Other {
        bounding_box: BoundingBox,
    },
}

impl PieceData {
    pub fn bounding_box(&self) -> BoundingBox {
        match self {
            PieceData::PoolElement(pe) => pe.bounding_box,
            PieceData::Other { bounding_box } => *bounding_box,
        }
    }

    pub fn is_close_to_chunk(&self, chunk_pos: ChunkPos, distance: i32) -> bool {
        let min_x = chunk_pos.get_min_block_x();
        let min_z = chunk_pos.get_min_block_z();
        self.bounding_box().intersects_xz(
            min_x.wrapping_sub(distance),
            min_z.wrapping_sub(distance),
            min_x.wrapping_add(15).wrapping_add(distance),
            min_z.wrapping_add(15).wrapping_add(distance),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureStartData {
    pub terrain_adaptation: TerrainAdjustment,
    pub pieces: Vec<PieceData>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rigid {
    pub bounding_box: BoundingBox,
    pub terrain_adjustment: TerrainAdjustment,
    pub ground_level_delta: i32,
}

fn build_beard_kernel() -> [f32; 13824] {
    let mut kernel = [0.0f32; 13824];
    let mut z_idx: i32 = 0;
    while z_idx < BEARD_KERNEL_SIZE {
        let mut x_idx: i32 = 0;
        while x_idx < BEARD_KERNEL_SIZE {
            let mut y_idx: i32 = 0;
            while y_idx < BEARD_KERNEL_SIZE {
                let idx = z_idx
                    .wrapping_mul(BEARD_KERNEL_SIZE)
                    .wrapping_mul(BEARD_KERNEL_SIZE)
                    .wrapping_add(x_idx.wrapping_mul(BEARD_KERNEL_SIZE))
                    .wrapping_add(y_idx) as usize;
                kernel[idx] = compute_beard_contribution_int(
                    x_idx.wrapping_sub(BEARD_KERNEL_RADIUS),
                    y_idx.wrapping_sub(BEARD_KERNEL_RADIUS),
                    z_idx.wrapping_sub(BEARD_KERNEL_RADIUS),
                ) as f32;
                y_idx = y_idx.wrapping_add(1);
            }
            x_idx = x_idx.wrapping_add(1);
        }
        z_idx = z_idx.wrapping_add(1);
    }
    kernel
}

static BEARD_KERNEL: LazyLock<[f32; 13824]> = LazyLock::new(build_beard_kernel);

#[derive(Clone, Debug)]
pub struct Beardifier {
    pieces: Vec<Rigid>,
    junctions: Vec<JigsawJunction>,
    affected_box: Option<BoundingBox>,
}

impl Beardifier {
    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new(), None)
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#<init>
    pub fn new(
        pieces: Vec<Rigid>,
        junctions: Vec<JigsawJunction>,
        affected_box: Option<BoundingBox>,
    ) -> Self {
        Self {
            pieces,
            junctions,
            affected_box,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#forStructuresInChunk
    pub fn for_structures_in_chunk(chunk_pos: ChunkPos, starts: &[StructureStartData]) -> Self {
        let mut filtered: Vec<&StructureStartData> = Vec::new();
        for start in starts {
            if start.terrain_adaptation != TerrainAdjustment::None {
                filtered.push(start);
            }
        }
        if filtered.is_empty() {
            return Self::empty();
        }

        let chunk_min_x = chunk_pos.get_min_block_x();
        let chunk_min_z = chunk_pos.get_min_block_z();
        let mut pieces: Vec<Rigid> = Vec::new();
        let mut junctions: Vec<JigsawJunction> = Vec::new();
        let mut affected_box: Option<BoundingBox> = None;

        for start in filtered {
            let terrain_adaptation = start.terrain_adaptation;
            for piece in &start.pieces {
                if piece.is_close_to_chunk(chunk_pos, 12) {
                    match piece {
                        PieceData::PoolElement(pe) => {
                            if pe.projection == StructureTemplateProjection::Rigid {
                                pieces.push(Rigid {
                                    bounding_box: pe.bounding_box,
                                    terrain_adjustment: terrain_adaptation,
                                    ground_level_delta: pe.ground_level_delta,
                                });
                                affected_box =
                                    Some(include_bounding_box(affected_box, pe.bounding_box));
                            }

                            for junction in &pe.junctions {
                                let jx = junction.source_x;
                                let jz = junction.source_z;
                                if jx > chunk_min_x.wrapping_sub(12)
                                    && jz > chunk_min_z.wrapping_sub(12)
                                    && jx < chunk_min_x.wrapping_add(15).wrapping_add(12)
                                    && jz < chunk_min_z.wrapping_add(15).wrapping_add(12)
                                {
                                    junctions.push(*junction);
                                    let junction_box = BoundingBox::from_point(BlockPos::new(
                                        jx,
                                        junction.source_ground_y,
                                        jz,
                                    ));
                                    affected_box =
                                        Some(include_bounding_box(affected_box, junction_box));
                                }
                            }
                        }
                        PieceData::Other { bounding_box } => {
                            pieces.push(Rigid {
                                bounding_box: *bounding_box,
                                terrain_adjustment: terrain_adaptation,
                                ground_level_delta: 0,
                            });
                            affected_box = Some(include_bounding_box(affected_box, *bounding_box));
                        }
                    }
                }
            }
        }

        match affected_box {
            None => Self::empty(),
            Some(ab) => {
                let inflated = ab.inflated_by(24);
                Self::new(pieces, junctions, Some(inflated))
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#fillArray
    pub fn fill_array<P: ContextProvider>(&self, array: &mut [f64], provider: &mut P) {
        if self.affected_box.is_none() {
            array.fill(0.0);
        } else {
            let mut i: usize = 0;
            while i < array.len() {
                let ctx = provider.for_index(i as i32);
                array[i] = self.compute(&ctx);
                i = i.wrapping_add(1);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#compute
    pub fn compute(&self, ctx: &impl FunctionContext) -> f64 {
        let Some(affected_box) = self.affected_box else {
            return 0.0;
        };

        let x = ctx.block_x();
        let y = ctx.block_y();
        let z = ctx.block_z();
        if !affected_box.is_inside(x, y, z) {
            return 0.0;
        }

        let mut d: f64 = 0.0;

        for piece in &self.pieces {
            let bbox = piece.bounding_box;
            let ground_level_delta = piece.ground_level_delta;
            let dx = 0i32.max(
                bbox.min_x
                    .wrapping_sub(x)
                    .max(x.wrapping_sub(bbox.max_x)),
            );
            let dz = 0i32.max(
                bbox.min_z
                    .wrapping_sub(z)
                    .max(z.wrapping_sub(bbox.max_z)),
            );
            let adjusted_min_y = bbox.min_y.wrapping_add(ground_level_delta);
            let dy_from_adjusted = y.wrapping_sub(adjusted_min_y);

            let distance_y = match piece.terrain_adjustment {
                TerrainAdjustment::None => 0,
                TerrainAdjustment::Bury | TerrainAdjustment::BeardThin => dy_from_adjusted,
                TerrainAdjustment::BeardBox => 0i32.max(
                    adjusted_min_y
                        .wrapping_sub(y)
                        .max(y.wrapping_sub(bbox.max_y)),
                ),
                TerrainAdjustment::Encapsulate => 0i32.max(
                    bbox.min_y
                        .wrapping_sub(y)
                        .max(y.wrapping_sub(bbox.max_y)),
                ),
            };

            d += match piece.terrain_adjustment {
                TerrainAdjustment::None => 0.0,
                TerrainAdjustment::Bury => {
                    get_bury_contribution(dx as f64, (distance_y as f64) / 2.0, dz as f64)
                }
                TerrainAdjustment::BeardThin | TerrainAdjustment::BeardBox => {
                    get_beard_contribution(dx, distance_y, dz, dy_from_adjusted) * 0.8
                }
                TerrainAdjustment::Encapsulate => {
                    get_bury_contribution(
                        (dx as f64) / 2.0,
                        (distance_y as f64) / 2.0,
                        (dz as f64) / 2.0,
                    ) * 0.8
                }
            };
        }

        for junction in &self.junctions {
            let dx_j = x.wrapping_sub(junction.source_x);
            let dy_j = y.wrapping_sub(junction.source_ground_y);
            let dz_j = z.wrapping_sub(junction.source_z);
            d += get_beard_contribution(dx_j, dy_j, dz_j, dy_j) * 0.4;
        }

        d
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#minValue
    pub fn min_value(&self) -> f64 {
        f64::NEG_INFINITY
    }

    // JAVA: net.minecraft.world.level.levelgen.Beardifier#maxValue
    pub fn max_value(&self) -> f64 {
        f64::INFINITY
    }
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#includeBoundingBox
fn include_bounding_box(existing: Option<BoundingBox>, other: BoundingBox) -> BoundingBox {
    match existing {
        None => other,
        Some(e) => BoundingBox::encapsulating(e, other),
    }
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#getBuryContribution
fn get_bury_contribution(x: f64, y: f64, z: f64) -> f64 {
    let d = mth_length(x, y, z);
    mth_clamped_map(d, 0.0, 6.0, 1.0, 0.0)
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#getBeardContribution
fn get_beard_contribution(x: i32, y: i32, z: i32, y_offset: i32) -> f64 {
    let xi = x.wrapping_add(BEARD_KERNEL_RADIUS);
    let yi = y.wrapping_add(BEARD_KERNEL_RADIUS);
    let zi = z.wrapping_add(BEARD_KERNEL_RADIUS);
    if is_in_kernel_range(xi) && is_in_kernel_range(yi) && is_in_kernel_range(zi) {
        let dy = (y_offset as f64) + 0.5;
        let len_sq = mth_length_squared(x as f64, dy, z as f64);
        let factor = -dy * mth_fast_inv_sqrt(len_sq / 2.0) / 2.0;
        let idx = zi
            .wrapping_mul(BEARD_KERNEL_SIZE)
            .wrapping_mul(BEARD_KERNEL_SIZE)
            .wrapping_add(xi.wrapping_mul(BEARD_KERNEL_SIZE))
            .wrapping_add(yi) as usize;
        factor * (BEARD_KERNEL[idx] as f64)
    } else {
        0.0
    }
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#isInKernelRange
fn is_in_kernel_range(value: i32) -> bool {
    value >= 0 && value < BEARD_KERNEL_SIZE
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#computeBeardContribution
fn compute_beard_contribution_int(x: i32, y: i32, z: i32) -> f64 {
    compute_beard_contribution(x, (y as f64) + 0.5, z)
}

// JAVA: net.minecraft.world.level.levelgen.Beardifier#computeBeardContribution
fn compute_beard_contribution(x: i32, y: f64, z: i32) -> f64 {
    let d = mth_length_squared(x as f64, y, z as f64);
    std::f64::consts::E.powf(-d / 16.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_beardifier_returns_zero() {
        let b = Beardifier::empty();
        let ctx = BlockContext { x: 0, y: 64, z: 0 };
        assert_eq!(b.compute(&ctx), 0.0);
        assert_eq!(b.min_value(), f64::NEG_INFINITY);
        assert_eq!(b.max_value(), f64::INFINITY);

        let mut arr = [1.0, 2.0, 3.0];
        let mut provider = FnContextProvider::new(|i: i32| BlockContext { x: i, y: 0, z: 0 });
        b.fill_array(&mut arr, &mut provider);
        assert_eq!(arr, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn beard_contribution_boundaries_and_float_bits() {
        assert!(is_in_kernel_range(0));
        assert!(is_in_kernel_range(23));
        assert!(!is_in_kernel_range(-1));
        assert!(!is_in_kernel_range(24));

        assert_eq!(get_beard_contribution(13, 0, 0, 0), 0.0);
        assert_eq!(get_beard_contribution(-13, 0, 0, 0), 0.0);
        assert_eq!(get_beard_contribution(0, 13, 0, 0), 0.0);
        assert_eq!(get_beard_contribution(0, 0, 13, 0), 0.0);

        let v = compute_beard_contribution_int(0, 0, 0);
        let expected = std::f64::consts::E
            .powf(-(mth_length_squared(0.0, 0.5, 0.0)) / 16.0);
        assert_eq!(v.to_bits(), expected.to_bits());

        let v_neg = compute_beard_contribution_int(-1, -2, -3);
        let expected_neg = std::f64::consts::E
            .powf(-(mth_length_squared(-1.0, -1.5, -3.0)) / 16.0);
        assert_eq!(v_neg.to_bits(), expected_neg.to_bits());
    }

    #[test]
    fn compute_with_rigid_beard_box() {
        let bbox = BoundingBox {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 5,
            max_y: 70,
            max_z: 5,
        };
        let rigid = Rigid {
            bounding_box: bbox,
            terrain_adjustment: TerrainAdjustment::BeardBox,
            ground_level_delta: 0,
        };
        let affected = bbox.inflated_by(24);
        let b = Beardifier::new(vec![rigid], vec![], Some(affected));

        let inside = b.compute(&BlockContext { x: 0, y: 64, z: 0 });
        let expected = get_beard_contribution(0, 0, 0, 0) * 0.8;
        assert_eq!(inside.to_bits(), expected.to_bits());

        let outside = b.compute(&BlockContext {
            x: 1000,
            y: 64,
            z: 1000,
        });
        assert_eq!(outside, 0.0);
    }

    #[test]
    fn for_structures_in_chunk_builds_beardifier() {
        let chunk_pos = ChunkPos::new(0, 0);
        let bbox = BoundingBox {
            min_x: 0,
            min_y: 60,
            min_z: 0,
            max_x: 10,
            max_y: 80,
            max_z: 10,
        };
        let starts = vec![StructureStartData {
            terrain_adaptation: TerrainAdjustment::Bury,
            pieces: vec![PieceData::Other { bounding_box: bbox }],
        }];

        let b = Beardifier::for_structures_in_chunk(chunk_pos, &starts);
        let val = b.compute(&BlockContext { x: 0, y: 60, z: 0 });
        let expected = get_bury_contribution(0.0, 0.0, 0.0);
        assert_eq!(val.to_bits(), expected.to_bits());
    }

    #[test]
    fn for_structures_in_chunk_with_junction_and_negative() {
        let chunk_pos = ChunkPos::new(-1, -1);
        let junction = JigsawJunction {
            source_x: -10,
            source_ground_y: 64,
            source_z: -10,
        };
        let bbox = BoundingBox {
            min_x: -16,
            min_y: 60,
            min_z: -16,
            max_x: -5,
            max_y: 80,
            max_z: -5,
        };
        let starts = vec![StructureStartData {
            terrain_adaptation: TerrainAdjustment::BeardThin,
            pieces: vec![PieceData::PoolElement(PoolElementPieceData {
                bounding_box: bbox,
                projection: StructureTemplateProjection::Rigid,
                ground_level_delta: 1,
                junctions: vec![junction],
            })],
        }];

        let b = Beardifier::for_structures_in_chunk(chunk_pos, &starts);
        let val = b.compute(&BlockContext {
            x: -10,
            y: 64,
            z: -10,
        });
        let expected =
            get_beard_contribution(0, 3, 0, 3) * 0.8 + get_beard_contribution(0, 0, 0, 0) * 0.4;
        assert_eq!(val.to_bits(), expected.to_bits());
    }

    #[test]
    fn overflow_and_negative_coordinates() {
        let b = Beardifier::empty();
        let ctx = BlockContext {
            x: i32::MIN,
            y: i32::MAX,
            z: i32::MIN,
        };
        assert_eq!(b.compute(&ctx), 0.0);

        assert_eq!(get_beard_contribution(i32::MAX, 0, 0, 0), 0.0);
        assert_eq!(get_beard_contribution(i32::MIN, 0, 0, 0), 0.0);

        let v = get_bury_contribution(1e15, 1e15, 1e15);
        assert!(v.is_finite());
        assert_eq!(v, 0.0);
    }
}
