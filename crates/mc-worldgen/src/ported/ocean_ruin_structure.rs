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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn get_middle_block_x(&self) -> i32 {
        self.get_min_block_x().wrapping_add(8)
    }

    pub fn get_middle_block_z(&self) -> i32 {
        self.get_min_block_z().wrapping_add(8)
    }
}

pub trait WorldgenRandom {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#getRandom
    pub fn get_random(random: &mut dyn WorldgenRandom) -> Self {
        match random.next_int(4).rem_euclid(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloorWg,
}

pub trait HeightmapSampler {
    fn get_first_occupied_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
}

#[derive(Clone, Debug, Default)]
pub struct StructureSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureType {
    OceanRuin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OceanRuinType {
    Warm,
    Cold,
}

impl OceanRuinType {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure$Type#getName
    pub fn get_name(&self) -> &'static str {
        match self {
            OceanRuinType::Warm => "warm",
            OceanRuinType::Cold => "cold",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure$Type#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "warm" => Some(OceanRuinType::Warm),
            "cold" => Some(OceanRuinType::Cold),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OceanRuinPieceInfo {
    pub anchor: BlockPos,
    pub rotation: Rotation,
    pub biome_temp: OceanRuinType,
    pub large_probability: f32,
    pub cluster_probability: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenerationStub {
    pub position: BlockPos,
    pub piece_info: OceanRuinPieceInfo,
}

pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub random: &'a mut dyn WorldgenRandom,
    pub height_sampler: &'a dyn HeightmapSampler,
}

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter
pub fn on_top_of_chunk_center(
    ctx: &mut GenerationContext<'_>,
    heightmap: HeightmapType,
) -> BlockPos {
    let chunk_pos = ctx.chunk_pos;
    let x = chunk_pos.get_middle_block_x();
    let z = chunk_pos.get_middle_block_z();
    let y = ctx
        .height_sampler
        .get_first_occupied_height(heightmap, x, z);
    BlockPos::new(x, y, z)
}

#[derive(Clone, Debug)]
pub struct OceanRuinStructure {
    pub settings: StructureSettings,
    pub biome_temp: OceanRuinType,
    pub large_probability: f32,
    pub cluster_probability: f32,
}

impl OceanRuinStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#<init>
    pub fn new(
        settings: StructureSettings,
        biome_temp: OceanRuinType,
        large_probability: f32,
        cluster_probability: f32,
    ) -> Self {
        Self {
            settings,
            biome_temp,
            large_probability,
            cluster_probability,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &mut GenerationContext<'_>) -> Option<GenerationStub> {
        let pos = on_top_of_chunk_center(ctx, HeightmapType::OceanFloorWg);
        let piece_info = self.generate_pieces(ctx);
        Some(GenerationStub {
            position: pos,
            piece_info,
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#generatePieces
    fn generate_pieces(&self, ctx: &mut GenerationContext<'_>) -> OceanRuinPieceInfo {
        let anchor = BlockPos::new(
            ctx.chunk_pos.get_min_block_x(),
            90,
            ctx.chunk_pos.get_min_block_z(),
        );
        let rotation = Rotation::get_random(ctx.random);
        OceanRuinPieceInfo {
            anchor,
            rotation,
            biome_temp: self.biome_temp,
            large_probability: self.large_probability,
            cluster_probability: self.cluster_probability,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::OceanRuin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl WorldgenRandom for FixedRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.0.rem_euclid(bound)
            }
        }
    }

    struct FixedHeight(i32);
    impl HeightmapSampler for FixedHeight {
        fn get_first_occupied_height(
            &self,
            _heightmap: HeightmapType,
            _x: i32,
            _z: i32,
        ) -> i32 {
            self.0
        }
    }

    #[test]
    fn find_generation_point_uses_chunk_center_and_ocean_floor() {
        let mut rng = FixedRandom(0);
        let hs = FixedHeight(42);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(3, -2),
            random: &mut rng,
            height_sampler: &hs,
        };
        let s = OceanRuinStructure::new(StructureSettings, OceanRuinType::Cold, 0.3, 0.9);
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(
            stub.position,
            BlockPos::new(3 * 16 + 8, 42, -2 * 16 + 8)
        );
    }

    #[test]
    fn generate_pieces_anchor_and_rotation() {
        let mut rng = FixedRandom(1);
        let hs = FixedHeight(10);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(1, 2),
            random: &mut rng,
            height_sampler: &hs,
        };
        let s = OceanRuinStructure::new(StructureSettings, OceanRuinType::Warm, 0.5, 0.8);
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(stub.piece_info.anchor, BlockPos::new(16, 90, 32));
        assert_eq!(stub.piece_info.rotation, Rotation::Clockwise90);
        assert_eq!(stub.piece_info.biome_temp, OceanRuinType::Warm);
        assert_eq!(stub.piece_info.large_probability.to_bits(), 0.5f32.to_bits());
        assert_eq!(stub.piece_info.cluster_probability.to_bits(), 0.8f32.to_bits());
    }

    #[test]
    fn rotation_wraps_negative_random() {
        let mut rng = FixedRandom(-1);
        let hs = FixedHeight(0);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(0, 0),
            random: &mut rng,
            height_sampler: &hs,
        };
        let s = OceanRuinStructure::new(StructureSettings, OceanRuinType::Cold, 0.0, 1.0);
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(stub.piece_info.rotation, Rotation::Counterclockwise90);
    }

    #[test]
    fn type_enum_names() {
        assert_eq!(OceanRuinType::Warm.get_name(), "warm");
        assert_eq!(OceanRuinType::Cold.get_serialized_name(), "cold");
        assert_eq!(OceanRuinType::from_name("warm"), Some(OceanRuinType::Warm));
        assert_eq!(OceanRuinType::from_name("cold"), Some(OceanRuinType::Cold));
        assert_eq!(OceanRuinType::from_name("hot"), None);
    }

    #[test]
    fn structure_type_is_ocean_ruin() {
        let s = OceanRuinStructure::new(StructureSettings, OceanRuinType::Warm, 0.0, 0.0);
        assert_eq!(s.structure_type(), StructureType::OceanRuin);
    }
}
