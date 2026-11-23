// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#getRandom
    pub fn get_random(random: &mut dyn RandomSource) -> Self {
        match random.next_int(4).rem_euclid(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
    OceanFloorWg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn min_x(&self) -> i32 {
        self.min_x
    }

    pub fn min_z(&self) -> i32 {
        self.min_z
    }

    pub fn get_x_span(&self) -> i32 {
        self.max_x.wrapping_sub(self.min_x).wrapping_add(1)
    }

    pub fn get_z_span(&self) -> i32 {
        self.max_z.wrapping_sub(self.min_z).wrapping_add(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Shipwreck,
}

#[derive(Debug, Clone, Default)]
pub struct StructureSettings;

pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub random: &'a mut dyn RandomSource,
}

impl<'a> GenerationContext<'a> {
    pub fn new(chunk_pos: ChunkPos, random: &'a mut dyn RandomSource) -> Self {
        Self { chunk_pos, random }
    }

    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn random(&mut self) -> &mut dyn RandomSource {
        self.random
    }
}

pub struct StructurePiecesBuilder {
    pub pieces_added: usize,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self { pieces_added: 0 }
    }

    pub fn add_piece(&mut self) {
        self.pieces_added = self.pieces_added.wrapping_add(1);
    }
}

impl Default for StructurePiecesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GenerationStub {
    pub position: BlockPos,
    pub generator: Box<dyn FnOnce(&mut StructurePiecesBuilder, &mut dyn RandomSource)>,
}

// Stub for net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces.ShipwreckPiece
#[derive(Debug, Clone)]
pub struct ShipwreckPiece {
    bounding_box: BoundingBox,
    too_big: bool,
    adjusted_y: Option<i32>,
}

impl ShipwreckPiece {
    pub fn new(bounding_box: BoundingBox, too_big: bool) -> Self {
        Self {
            bounding_box,
            too_big,
            adjusted_y: None,
        }
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    pub fn is_too_big_to_fit_in_world_gen_region(&self) -> bool {
        self.too_big
    }

    pub fn calculate_beached_position(&self, ground_y: i32, _random: &mut dyn RandomSource) -> i32 {
        ground_y
    }

    pub fn adjust_position_height(&mut self, y: i32) {
        self.adjusted_y = Some(y);
    }

    pub fn adjusted_y(&self) -> Option<i32> {
        self.adjusted_y
    }
}

// Stub for net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces
pub struct ShipwreckPieces;

impl ShipwreckPieces {
    pub fn add_random_piece(
        _structure_template_manager: (),
        _pos: BlockPos,
        _rotation: Rotation,
        builder: &mut StructurePiecesBuilder,
        _random: &mut dyn RandomSource,
        is_beached: bool,
    ) -> ShipwreckPiece {
        builder.add_piece();
        // Default stub piece; tests can exercise generation logic through structure methods.
        ShipwreckPiece::new(BoundingBox::new(0, 0, 0, 0, 0, 0), is_beached)
    }
}

// Stub helpers mirroring Structure static methods used by ShipwreckStructure.
pub struct StructureHeightAccess;

impl StructureHeightAccess {
    // JAVA: net.minecraft.world.level.levelgen.structure.Structure#getLowestY
    pub fn get_lowest_y(
        _ctx: &GenerationContext<'_>,
        min_x: i32,
        x_span: i32,
        min_z: i32,
        z_span: i32,
    ) -> i32 {
        // Deterministic stub: combine inputs so tests can verify call path.
        min_x
            .wrapping_add(x_span)
            .wrapping_add(min_z)
            .wrapping_add(z_span)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure#getMeanFirstOccupiedHeight
    pub fn get_mean_first_occupied_height(
        _ctx: &GenerationContext<'_>,
        min_x: i32,
        x_span: i32,
        min_z: i32,
        z_span: i32,
    ) -> i32 {
        min_x
            .wrapping_add(x_span)
            .wrapping_add(min_z)
            .wrapping_add(z_span)
            .wrapping_add(1)
    }
}

fn on_top_of_chunk_center(
    ctx: &GenerationContext<'_>,
    _heightmap: HeightmapType,
    generator: impl FnOnce(&mut StructurePiecesBuilder, &mut GenerationContext<'_>) + 'static,
) -> Option<GenerationStub> {
    let chunk_pos = ctx.chunk_pos();
    let pos = BlockPos::new(
        chunk_pos.get_min_block_x().wrapping_add(8),
        0,
        chunk_pos.get_min_block_z().wrapping_add(8),
    );
    // We cannot capture &mut GenerationContext in a returned boxed closure safely here.
    // Encode the generation as a closure that takes (builder, random) and reconstructs minimal context.
    let cp = ctx.chunk_pos();
    Some(GenerationStub {
        position: pos,
        generator: Box::new(move |builder, random| {
            let mut local_ctx = GenerationContext::new(cp, random);
            generator(builder, &mut local_ctx);
        }),
    })
}

// ---------------------------------------------------------------------------
// Ported class: ShipwreckStructure
// ---------------------------------------------------------------------------

pub struct ShipwreckStructure {
    pub settings: StructureSettings,
    pub is_beached: bool,
}

impl ShipwreckStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#<init>
    pub fn new(settings: StructureSettings, is_beached: bool) -> Self {
        Self {
            settings,
            is_beached,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &GenerationContext<'_>) -> Option<GenerationStub> {
        let heightmap = if self.is_beached {
            HeightmapType::WorldSurfaceWg
        } else {
            HeightmapType::OceanFloorWg
        };
        let is_beached = self.is_beached;
        on_top_of_chunk_center(ctx, heightmap, move |builder, gen_ctx| {
            let s = ShipwreckStructure::new(StructureSettings, is_beached);
            s.generate_pieces(builder, gen_ctx);
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#generatePieces
    fn generate_pieces(&self, builder: &mut StructurePiecesBuilder, ctx: &mut GenerationContext<'_>) {
        let rotation = Rotation::get_random(ctx.random());
        let pos = BlockPos::new(ctx.chunk_pos().get_min_block_x(), 90, ctx.chunk_pos().get_min_block_z());
        let mut piece = ShipwreckPieces::add_random_piece((), pos, rotation, builder, ctx.random(), self.is_beached);
        if piece.is_too_big_to_fit_in_world_gen_region() {
            let bb = piece.get_bounding_box();
            let y = if self.is_beached {
                let ground_y = StructureHeightAccess::get_lowest_y(
                    ctx,
                    bb.min_x(),
                    bb.get_x_span(),
                    bb.min_z(),
                    bb.get_z_span(),
                );
                piece.calculate_beached_position(ground_y, ctx.random())
            } else {
                StructureHeightAccess::get_mean_first_occupied_height(
                    ctx,
                    bb.min_x(),
                    bb.get_x_span(),
                    bb.min_z(),
                    bb.get_z_span(),
                )
            };
            piece.adjust_position_height(y);
        }
        let _ = piece;
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Shipwreck
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        index: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.index < self.values.len() {
                let val = self.values[self.index];
                self.index += 1;
                val
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    #[test]
    fn rotation_get_random_maps_bounds() {
        let mut rng = SeqRandom::new(vec![0, 1, 2, 3, -1]);
        assert_eq!(Rotation::get_random(&mut rng), Rotation::None);
        assert_eq!(Rotation::get_random(&mut rng), Rotation::Clockwise90);
        assert_eq!(Rotation::get_random(&mut rng), Rotation::Clockwise180);
        assert_eq!(Rotation::get_random(&mut rng), Rotation::Counterclockwise90);
        assert_eq!(Rotation::get_random(&mut rng), Rotation::Counterclockwise90);
    }

    #[test]
    fn find_generation_point_uses_chunk_center() {
        let mut rng = SeqRandom::new(vec![0]);
        let ctx = GenerationContext::new(ChunkPos::new(3, -2), &mut rng);
        let structure = ShipwreckStructure::new(StructureSettings, false);
        let stub = structure.find_generation_point(&ctx).expect("stub");
        assert_eq!(stub.position, BlockPos::new(56, 0, -24));
    }

    #[test]
    fn beached_selection_affects_heightmap_path_flag() {
        let beached = ShipwreckStructure::new(StructureSettings, true);
        let ocean = ShipwreckStructure::new(StructureSettings, false);
        assert!(beached.is_beached);
        assert!(!ocean.is_beached);
        assert_eq!(beached.structure_type(), StructureType::Shipwreck);
    }

    #[test]
    fn generate_pieces_runs_and_adds_piece() {
        let mut rng = SeqRandom::new(vec![0]);
        let mut ctx = GenerationContext::new(ChunkPos::new(1, 2), &mut rng);
        let mut builder = StructurePiecesBuilder::new();
        let structure = ShipwreckStructure::new(StructureSettings, true);
        structure.generate_pieces(&mut builder, &mut ctx);
        assert_eq!(builder.pieces_added, 1);
    }

    #[test]
    fn bounding_box_spans_use_wrapping_arithmetic() {
        let bb = BoundingBox::new(i32::MAX, 0, -5, i32::MIN, 0, 5);
        assert_eq!(bb.get_x_span(), i32::MIN.wrapping_sub(i32::MAX).wrapping_add(1));
        assert_eq!(bb.get_z_span(), 11);
    }

    #[test]
    fn negative_chunk_center_wrapping() {
        let mut rng = SeqRandom::new(vec![]);
        let ctx = GenerationContext::new(ChunkPos::new(-1, -1), &mut rng);
        let structure = ShipwreckStructure::new(StructureSettings, false);
        let stub = structure.find_generation_point(&ctx).expect("stub");
        assert_eq!(stub.position, BlockPos::new(-8, 0, -8));
    }
}
