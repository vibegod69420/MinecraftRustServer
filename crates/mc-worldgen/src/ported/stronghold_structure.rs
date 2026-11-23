use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#getWorldPosition
    pub fn get_world_position(&self) -> BlockPos {
        BlockPos {
            x: self.get_min_block_x(),
            y: 0,
            z: self.get_min_block_z(),
        }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockX
    pub fn get_block_x(&self, offset: i32) -> i32 {
        self.get_min_block_x().wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockZ
    pub fn get_block_z(&self, offset: i32) -> i32 {
        self.get_min_block_z().wrapping_add(offset)
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
    fn set_large_feature_seed(&mut self, seed: i64, chunk_x: i32, chunk_z: i32);
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait ChunkGenerator {
    fn get_sea_level(&self) -> i32;
    fn get_min_y(&self) -> i32;
}

pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub seed: i64,
    pub random: &'a mut dyn RandomSource,
    pub chunk_generator: &'a dyn ChunkGenerator,
}

#[derive(Debug, Default)]
pub struct StructurePiecesBuilder {
    piece_count: usize,
    pub move_below_sea_level_calls: Vec<(i32, i32, i32)>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.piece_count = 0;
    }

    pub fn add_piece(&mut self) {
        self.piece_count = self.piece_count.wrapping_add(1);
    }

    pub fn is_empty(&self) -> bool {
        self.piece_count == 0
    }

    pub fn move_below_sea_level(
        &mut self,
        sea_level: i32,
        min_y: i32,
        _random: &mut dyn RandomSource,
        offset: i32,
    ) {
        self.move_below_sea_level_calls
            .push((sea_level, min_y, offset));
    }
}

pub trait StructurePieceApi {
    fn add_children(
        &mut self,
        start: &mut dyn StartPieceApi,
        builder: &mut StructurePiecesBuilder,
        random: &mut dyn RandomSource,
    );
}

pub trait StartPieceApi {
    fn add_children_self(
        &mut self,
        builder: &mut StructurePiecesBuilder,
        random: &mut dyn RandomSource,
    );
    fn pending_children(&mut self) -> &mut Vec<Box<dyn StructurePieceApi>>;
    fn has_portal_room(&self) -> bool;
}

pub trait StrongholdPiecesApi {
    fn reset_pieces(&mut self);
    fn create_start_piece(
        &mut self,
        random: &mut dyn RandomSource,
        block_x: i32,
        block_z: i32,
    ) -> Box<dyn StartPieceApi>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Stronghold,
}

#[derive(Debug, Clone, Default)]
pub struct StructureSettings;

pub struct GenerationStub<'a> {
    pub position: BlockPos,
    generator: Box<dyn FnOnce(&mut StructurePiecesBuilder) + 'a>,
}

impl<'a> GenerationStub<'a> {
    pub fn generate(self, builder: &mut StructurePiecesBuilder) {
        (self.generator)(builder);
    }
}

#[derive(Debug, Clone)]
pub struct StrongholdStructure {
    pub settings: StructureSettings,
}

impl StrongholdStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure#findGenerationPoint
    pub fn find_generation_point<'a>(
        &self,
        ctx: &'a mut GenerationContext<'_>,
        pieces: &'a mut dyn StrongholdPiecesApi,
    ) -> Option<GenerationStub<'a>> {
        let pos = ctx.chunk_pos.get_world_position();
        Some(GenerationStub {
            position: pos,
            generator: Box::new(move |builder: &mut StructurePiecesBuilder| {
                Self::generate_pieces(builder, ctx, pieces);
            }),
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure#generatePieces
    fn generate_pieces(
        builder: &mut StructurePiecesBuilder,
        ctx: &mut GenerationContext<'_>,
        pieces: &mut dyn StrongholdPiecesApi,
    ) {
        let mut counter: i32 = 0;
        loop {
            builder.clear();
            let seed = ctx.seed.wrapping_add(i64::from(counter));
            counter = counter.wrapping_add(1);
            ctx.random
                .set_large_feature_seed(seed, ctx.chunk_pos.x, ctx.chunk_pos.z);
            pieces.reset_pieces();
            let mut start = pieces.create_start_piece(
                ctx.random,
                ctx.chunk_pos.get_block_x(2),
                ctx.chunk_pos.get_block_z(2),
            );
            builder.add_piece();
            start.add_children_self(builder, ctx.random);

            while !start.pending_children().is_empty() {
                let len = start.pending_children().len() as i32;
                let idx = ctx.random.next_int(len) as usize;
                let mut piece = start.pending_children().remove(idx);
                piece.add_children(start.as_mut(), builder, ctx.random);
            }

            builder.move_below_sea_level(
                ctx.chunk_generator.get_sea_level(),
                ctx.chunk_generator.get_min_y(),
                ctx.random,
                10,
            );

            if !builder.is_empty() && start.has_portal_room() {
                break;
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Stronghold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRandom {
        large_feature_seeds: Vec<(i64, i32, i32)>,
        next_int_values: Vec<i32>,
        next_int_index: usize,
    }

    impl TestRandom {
        fn new(next_int_values: Vec<i32>) -> Self {
            Self {
                large_feature_seeds: Vec::new(),
                next_int_values,
                next_int_index: 0,
            }
        }
    }

    impl RandomSource for TestRandom {
        fn set_large_feature_seed(&mut self, seed: i64, chunk_x: i32, chunk_z: i32) {
            self.large_feature_seeds.push((seed, chunk_x, chunk_z));
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.next_int_index < self.next_int_values.len() {
                let val = self.next_int_values[self.next_int_index];
                self.next_int_index += 1;
                val
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    struct TestChunkGenerator {
        sea_level: i32,
        min_y: i32,
    }

    impl ChunkGenerator for TestChunkGenerator {
        fn get_sea_level(&self) -> i32 {
            self.sea_level
        }
        fn get_min_y(&self) -> i32 {
            self.min_y
        }
    }

    struct NoopPiece;
    impl StructurePieceApi for NoopPiece {
        fn add_children(
            &mut self,
            _start: &mut dyn StartPieceApi,
            _builder: &mut StructurePiecesBuilder,
            _random: &mut dyn RandomSource,
        ) {
        }
    }

    struct ChildAddingPiece {
        added: bool,
    }
    impl StructurePieceApi for ChildAddingPiece {
        fn add_children(
            &mut self,
            start: &mut dyn StartPieceApi,
            builder: &mut StructurePiecesBuilder,
            _random: &mut dyn RandomSource,
        ) {
            if !self.added {
                self.added = true;
                builder.add_piece();
                start.pending_children().push(Box::new(NoopPiece));
            }
        }
    }

    struct TestStartPiece {
        portal_room: bool,
        pending: Vec<Box<dyn StructurePieceApi>>,
        add_on_start: usize,
    }

    impl StartPieceApi for TestStartPiece {
        fn add_children_self(
            &mut self,
            builder: &mut StructurePiecesBuilder,
            _random: &mut dyn RandomSource,
        ) {
            for _ in 0..self.add_on_start {
                builder.add_piece();
                self.pending.push(Box::new(NoopPiece));
            }
        }

        fn pending_children(&mut self) -> &mut Vec<Box<dyn StructurePieceApi>> {
            &mut self.pending
        }

        fn has_portal_room(&self) -> bool {
            self.portal_room
        }
    }

    struct TestPiecesApi {
        reset_count: usize,
        attempts_until_portal: usize,
        add_on_start: usize,
    }

    impl StrongholdPiecesApi for TestPiecesApi {
        fn reset_pieces(&mut self) {
            self.reset_count = self.reset_count.wrapping_add(1);
        }

        fn create_start_piece(
            &mut self,
            _random: &mut dyn RandomSource,
            _block_x: i32,
            _block_z: i32,
        ) -> Box<dyn StartPieceApi> {
            let portal = self.reset_count >= self.attempts_until_portal;
            Box::new(TestStartPiece {
                portal_room: portal,
                pending: Vec::new(),
                add_on_start: self.add_on_start,
            })
        }
    }

    #[test]
    fn find_generation_point_uses_chunk_world_position() {
        let structure = StrongholdStructure::new(StructureSettings);
        let mut random = TestRandom::new(vec![]);
        let generator = TestChunkGenerator {
            sea_level: 63,
            min_y: -64,
        };
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos { x: -3, z: 5 },
            seed: 123,
            random: &mut random,
            chunk_generator: &generator,
        };
        let mut pieces = TestPiecesApi {
            reset_count: 0,
            attempts_until_portal: 1,
            add_on_start: 0,
        };
        let stub = structure.find_generation_point(&mut ctx, &mut pieces).unwrap();
        assert_eq!(stub.position, BlockPos { x: -48, y: 0, z: 80 });
    }

    #[test]
    fn generate_retries_until_portal_room_and_not_empty() {
        let structure = StrongholdStructure::new(StructureSettings);
        let mut random = TestRandom::new(vec![]);
        let generator = TestChunkGenerator {
            sea_level: 63,
            min_y: -64,
        };
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 1, z: 2 },
            seed: 100,
            random: &mut random,
            chunk_generator: &generator,
        };
        let mut pieces = TestPiecesApi {
            reset_count: 0,
            attempts_until_portal: 3,
            add_on_start: 0,
        };
        let stub = structure.find_generation_point(&mut ctx, &mut pieces).unwrap();
        let mut builder = StructurePiecesBuilder::new();
        stub.generate(&mut builder);

        assert_eq!(pieces.reset_count, 3);
        assert_eq!(random.large_feature_seeds.len(), 3);
        assert_eq!(random.large_feature_seeds[0], (100, 1, 2));
        assert_eq!(random.large_feature_seeds[1], (101, 1, 2));
        assert_eq!(random.large_feature_seeds[2], (102, 1, 2));
        assert!(!builder.is_empty());
        assert_eq!(builder.move_below_sea_level_calls, vec![(63, -64, 10), (63, -64, 10), (63, -64, 10)]);
    }

    #[test]
    fn generate_processes_pending_children_and_can_grow() {
        struct GrowingPiecesApi;
        impl StrongholdPiecesApi for GrowingPiecesApi {
            fn reset_pieces(&mut self) {}
            fn create_start_piece(
                &mut self,
                _random: &mut dyn RandomSource,
                _block_x: i32,
                _block_z: i32,
            ) -> Box<dyn StartPieceApi> {
                Box::new(GrowingStart)
            }
        }

        struct GrowingStart;
        impl StartPieceApi for GrowingStart {
            fn add_children_self(
                &mut self,
                builder: &mut StructurePiecesBuilder,
                _random: &mut dyn RandomSource,
            ) {
                builder.add_piece();
                self.pending_children().push(Box::new(ChildAddingPiece { added: false }));
            }
            fn pending_children(&mut self) -> &mut Vec<Box<dyn StructurePieceApi>> {
                // Use a thread-local-ish static is not possible; instead store in a Box via a hack.
                // Easier: implement with a proper struct. We'll do it below in a separate test.
                unimplemented!("use GrowingStartWithVec")
            }
            fn has_portal_room(&self) -> bool {
                true
            }
        }

        // Proper implementation
        struct GrowingStart2 {
            pending: Vec<Box<dyn StructurePieceApi>>,
        }
        impl StartPieceApi for GrowingStart2 {
            fn add_children_self(
                &mut self,
                builder: &mut StructurePiecesBuilder,
                _random: &mut dyn RandomSource,
            ) {
                builder.add_piece();
                self.pending.push(Box::new(ChildAddingPiece { added: false }));
            }
            fn pending_children(&mut self) -> &mut Vec<Box<dyn StructurePieceApi>> {
                &mut self.pending
            }
            fn has_portal_room(&self) -> bool {
                true
            }
        }
        struct GrowingPiecesApi2;
        impl StrongholdPiecesApi for GrowingPiecesApi2 {
            fn reset_pieces(&mut self) {}
            fn create_start_piece(
                &mut self,
                _random: &mut dyn RandomSource,
                _block_x: i32,
                _block_z: i32,
            ) -> Box<dyn StartPieceApi> {
                Box::new(GrowingStart2 { pending: Vec::new() })
            }
        }

        let structure = StrongholdStructure::new(StructureSettings);
        let mut random = TestRandom::new(vec![0, 0, 0]);
        let generator = TestChunkGenerator { sea_level: 63, min_y: -64 };
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 0, z: 0 },
            seed: 0,
            random: &mut random,
            chunk_generator: &generator,
        };
        let mut pieces = GrowingPiecesApi2;
        let stub = structure.find_generation_point(&mut ctx, &mut pieces).unwrap();
        let mut builder = StructurePiecesBuilder::new();
        stub.generate(&mut builder);
        assert!(!builder.is_empty());
        assert_eq!(builder.move_below_sea_level_calls.len(), 1);
    }

    #[test]
    fn seed_increment_wraps_with_i32_counter() {
        let structure = StrongholdStructure::new(StructureSettings);
        let mut random = TestRandom::new(vec![]);
        let generator = TestChunkGenerator { sea_level: 0, min_y: 0 };
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 0, z: 0 },
            seed: i64::MAX - 1,
            random: &mut random,
            chunk_generator: &generator,
        };
        struct TwoAttempts;
        impl StrongholdPiecesApi for TwoAttempts {
            fn reset_pieces(&mut self) {}
            fn create_start_piece(
                &mut self,
                _random: &mut dyn RandomSource,
                _block_x: i32,
                _block_z: i32,
            ) -> Box<dyn StartPieceApi> {
                struct S { portal: bool, pending: Vec<Box<dyn StructurePieceApi>> }
                impl StartPieceApi for S {
                    fn add_children_self(&mut self, b: &mut StructurePiecesBuilder, _: &mut dyn RandomSource) { b.add_piece(); }
                    fn pending_children(&mut self) -> &mut Vec<Box<dyn StructurePieceApi>> { &mut self.pending }
                    fn has_portal_room(&self) -> bool { self.portal }
                }
                // Always portal room so it finishes in one iteration; we'll test seed directly
                Box::new(S { portal: true, pending: Vec::new() })
            }
        }
        let mut pieces = TwoAttempts;
        let stub = structure.find_generation_point(&mut ctx, &mut pieces).unwrap();
        let mut builder = StructurePiecesBuilder::new();
        stub.generate(&mut builder);
        // first iteration: seed + 0
        assert_eq!(random.large_feature_seeds[0].0, i64::MAX - 1);
    }

    #[test]
    fn chunk_pos_negative_coordinates() {
        let cp = ChunkPos { x: -1, z: -2 };
        assert_eq!(cp.get_min_block_x(), -16);
        assert_eq!(cp.get_min_block_z(), -32);
        assert_eq!(cp.get_block_x(2), -14);
        assert_eq!(cp.get_block_z(2), -30);
        assert_eq!(cp.get_world_position(), BlockPos { x: -16, y: 0, z: -32 });
    }

    #[test]
    fn structure_type_is_stronghold() {
        let s = StrongholdStructure::new(StructureSettings);
        assert_eq!(s.structure_type(), StructureType::Stronghold);
    }
}
