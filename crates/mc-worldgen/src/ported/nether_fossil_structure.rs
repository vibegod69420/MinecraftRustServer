/// Minimal block position.
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

/// Mutable block position used during generation checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }
}

/// Minimal block state for nether fossil placement checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockState {
    pub air: bool,
    pub soul_sand: bool,
    pub face_sturdy_up: bool,
}

impl BlockState {
    pub fn is_air(&self) -> bool {
        self.air
    }

    pub fn is_soul_sand(&self) -> bool {
        self.soul_sand
    }

    pub fn is_face_sturdy_up(&self) -> bool {
        self.face_sturdy_up
    }
}

/// Noise column access.
pub trait NoiseColumn {
    fn get_block(&self, y: i32) -> BlockState;
}

/// Random source used by worldgen.
pub trait WorldgenRandom {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal world generation context.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldGenerationContext {
    pub min_gen_y: i32,
    pub gen_depth: i32,
}

impl WorldGenerationContext {
    pub fn new(min_gen_y: i32, gen_depth: i32) -> Self {
        Self {
            min_gen_y,
            gen_depth,
        }
    }
}

/// Height provider.
pub trait HeightProvider {
    fn sample(&self, random: &mut dyn WorldgenRandom, context: &WorldGenerationContext) -> i32;
}

/// Minimal chunk position.
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
}

/// Minimal structure settings placeholder.
#[derive(Clone, Debug, Default)]
pub struct StructureSettings;

/// Minimal structure type marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureType {
    NetherFossil,
}

/// Generation stub result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationStub {
    pub position: BlockPos,
}

/// Generation context required by NetherFossilStructure.
pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub random: &'a mut dyn WorldgenRandom,
    pub sea_level: i32,
    pub height_context: WorldGenerationContext,
    pub column: &'a dyn NoiseColumn,
}

/// Nether fossil structure.
#[derive(Clone, Debug)]
pub struct NetherFossilStructure<H: HeightProvider> {
    pub settings: StructureSettings,
    pub height: H,
}

impl<H: HeightProvider> NetherFossilStructure<H> {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure#<init>
    pub fn new(settings: StructureSettings, height: H) -> Self {
        Self { settings, height }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &mut GenerationContext<'_>) -> Option<GenerationStub> {
        let x = ctx
            .chunk_pos
            .get_min_block_x()
            .wrapping_add(ctx.random.next_int(16));
        let z = ctx
            .chunk_pos
            .get_min_block_z()
            .wrapping_add(ctx.random.next_int(16));
        let sea_level = ctx.sea_level;
        let mut y = self.height.sample(ctx.random, &ctx.height_context);
        let mut mutable_pos = MutableBlockPos::new(x, y, z);

        while y > sea_level {
            let state_at_y = ctx.column.get_block(y);
            y = y.wrapping_sub(1);
            let state_below = ctx.column.get_block(y);
            mutable_pos.set_y(y);
            if state_at_y.is_air()
                && (state_below.is_soul_sand() || state_below.is_face_sturdy_up())
            {
                break;
            }
        }

        if y <= sea_level {
            return None;
        }

        let pos = BlockPos::new(x, y, z);
        Some(GenerationStub { position: pos })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::NetherFossil
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

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

    struct SeqRandom {
        vals: Vec<i32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(vals: Vec<i32>) -> Self {
            Self { vals, idx: 0 }
        }
    }
    impl WorldgenRandom for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.vals.len() {
                self.vals[self.idx]
            } else {
                0
            };
            self.idx += 1;
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    struct FixedHeight(i32);
    impl HeightProvider for FixedHeight {
        fn sample(
            &self,
            _random: &mut dyn WorldgenRandom,
            _context: &WorldGenerationContext,
        ) -> i32 {
            self.0
        }
    }

    struct FlatColumn(BlockState);
    impl NoiseColumn for FlatColumn {
        fn get_block(&self, _y: i32) -> BlockState {
            self.0
        }
    }

    struct MapColumn {
        states: BTreeMap<i32, BlockState>,
        default: BlockState,
    }
    impl MapColumn {
        fn new(default: BlockState) -> Self {
            Self {
                states: BTreeMap::new(),
                default,
            }
        }
        fn set(&mut self, y: i32, s: BlockState) {
            self.states.insert(y, s);
        }
    }
    impl NoiseColumn for MapColumn {
        fn get_block(&self, y: i32) -> BlockState {
            self.states.get(&y).copied().unwrap_or(self.default)
        }
    }

    #[test]
    fn finds_valid_position_on_soul_sand() {
        let mut col = MapColumn::new(BlockState {
            air: false,
            soul_sand: false,
            face_sturdy_up: true,
        });
        col.set(
            65,
            BlockState {
                air: true,
                soul_sand: false,
                face_sturdy_up: false,
            },
        );
        col.set(
            64,
            BlockState {
                air: false,
                soul_sand: true,
                face_sturdy_up: false,
            },
        );
        let mut rng = FixedRandom(0);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(0, 0),
            random: &mut rng,
            sea_level: 32,
            height_context: WorldGenerationContext::new(-64, 384),
            column: &col,
        };
        let s = NetherFossilStructure::new(StructureSettings, FixedHeight(65));
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(stub.position, BlockPos::new(0, 64, 0));
    }

    #[test]
    fn returns_none_when_at_or_below_sea_level() {
        let col = FlatColumn(BlockState {
            air: true,
            soul_sand: false,
            face_sturdy_up: false,
        });
        let mut rng = FixedRandom(0);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(1, -2),
            random: &mut rng,
            sea_level: 32,
            height_context: WorldGenerationContext::new(-64, 384),
            column: &col,
        };
        let s = NetherFossilStructure::new(StructureSettings, FixedHeight(32));
        assert!(s.find_generation_point(&mut ctx).is_none());
    }

    #[test]
    fn chunk_offset_uses_wrapping_add() {
        let mut col = MapColumn::new(BlockState {
            air: false,
            soul_sand: false,
            face_sturdy_up: true,
        });
        col.set(
            40,
            BlockState {
                air: true,
                soul_sand: false,
                face_sturdy_up: false,
            },
        );
        col.set(
            39,
            BlockState {
                air: false,
                soul_sand: false,
                face_sturdy_up: true,
            },
        );
        let mut rng = SeqRandom::new(vec![15, 7]);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(i32::MAX >> 4, 0),
            random: &mut rng,
            sea_level: 10,
            height_context: WorldGenerationContext::new(0, 128),
            column: &col,
        };
        let s = NetherFossilStructure::new(StructureSettings, FixedHeight(40));
        let stub = s.find_generation_point(&mut ctx).unwrap();
        let expected_x = (i32::MAX >> 4).wrapping_shl(4).wrapping_add(15);
        assert_eq!(stub.position.x, expected_x);
        assert_eq!(stub.position.z, 7);
        assert_eq!(stub.position.y, 39);
    }

    #[test]
    fn negative_coordinates_and_height() {
        let mut col = MapColumn::new(BlockState {
            air: false,
            soul_sand: false,
            face_sturdy_up: false,
        });
        col.set(
            -10,
            BlockState {
                air: true,
                soul_sand: false,
                face_sturdy_up: false,
            },
        );
        col.set(
            -11,
            BlockState {
                air: false,
                soul_sand: false,
                face_sturdy_up: true,
            },
        );
        let mut rng = SeqRandom::new(vec![3, 4]);
        let mut ctx = GenerationContext {
            chunk_pos: ChunkPos::new(-1, -1),
            random: &mut rng,
            sea_level: -20,
            height_context: WorldGenerationContext::new(-64, 384),
            column: &col,
        };
        let s = NetherFossilStructure::new(StructureSettings, FixedHeight(-10));
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(stub.position, BlockPos::new(-16 + 3, -11, -16 + 4));
    }

    #[test]
    fn structure_type_is_nether_fossil() {
        let s = NetherFossilStructure::new(StructureSettings, FixedHeight(0));
        assert_eq!(s.structure_type(), StructureType::NetherFossil);
    }
}
