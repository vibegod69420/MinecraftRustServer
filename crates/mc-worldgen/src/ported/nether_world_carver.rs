use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn get_y(&self) -> i32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u32,
}

pub const CAVE_AIR: BlockState = BlockState { id: 0 };

#[derive(Debug, Clone, Copy)]
pub struct LavaFluid;

impl LavaFluid {
    pub fn create_legacy_block(&self) -> BlockState {
        BlockState { id: 10 }
    }
}

pub const LAVA: LavaFluid = LavaFluid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biome {
    pub id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CarvingMask;

#[derive(Debug, Clone, Default)]
pub struct Aquifer;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MutableBoolean {
    pub value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fluid {
    Lava,
    Water,
}

pub struct Codec<T>(pub PhantomData<T>);

impl<T> Codec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CaveCarverConfiguration {
    pub replaceable_blocks: HashSet<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct CarvingContext {
    pub min_gen_y: i32,
}

impl CarvingContext {
    pub fn get_min_gen_y(&self) -> i32 {
        self.min_gen_y
    }
}

pub struct RandomSource {
    floats: Vec<f32>,
    idx: usize,
}

impl RandomSource {
    pub fn from_floats(floats: Vec<f32>) -> Self {
        Self { floats, idx: 0 }
    }

    pub fn next_float(&mut self) -> f32 {
        let v = self.floats.get(self.idx).copied().unwrap_or(0.0_f32);
        self.idx = self.idx.wrapping_add(1);
        v
    }
}

pub struct ChunkAccess {
    states: HashMap<(i32, i32, i32), BlockState>,
    default_state: BlockState,
}

impl ChunkAccess {
    pub fn new(default_state: BlockState) -> Self {
        Self {
            states: HashMap::new(),
            default_state,
        }
    }

    pub fn get_block_state(&self, pos: &MutableBlockPos) -> BlockState {
        self.states
            .get(&(pos.x, pos.y, pos.z))
            .copied()
            .unwrap_or(self.default_state)
    }

    pub fn set_block_state(&mut self, pos: &MutableBlockPos, state: BlockState) {
        self.states.insert((pos.x, pos.y, pos.z), state);
    }
}

pub struct NetherWorldCarver {
    pub liquids: Vec<Fluid>,
}

impl NetherWorldCarver {
    // JAVA: net.minecraft.world.level.levelgen.carver.NetherWorldCarver#<init>
    pub fn new(_codec: Codec<CaveCarverConfiguration>) -> Self {
        Self {
            liquids: vec![Fluid::Lava, Fluid::Water],
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.NetherWorldCarver#getCaveBound
    pub fn get_cave_bound(&self) -> i32 {
        10
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.NetherWorldCarver#getThickness
    pub fn get_thickness(&self, random: &mut RandomSource) -> f32 {
        let f0 = random.next_float();
        let f1 = random.next_float();
        (f0 * 2.0_f32 + f1) * 2.0_f32
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.NetherWorldCarver#getYScale
    pub fn get_y_scale(&self) -> f64 {
        5.0_f64
    }

    fn can_replace_block(&self, config: &CaveCarverConfiguration, state: BlockState) -> bool {
        config.replaceable_blocks.contains(&state.id)
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.NetherWorldCarver#carveBlock
    pub fn carve_block(
        &self,
        context: &CarvingContext,
        config: &CaveCarverConfiguration,
        chunk: &mut ChunkAccess,
        _biome_at: &dyn Fn(&BlockPos) -> Holder<Biome>,
        _carving_mask: &mut CarvingMask,
        pos: &MutableBlockPos,
        _offset_pos: &MutableBlockPos,
        _aquifer: &Aquifer,
        _reached_surface: &mut MutableBoolean,
    ) -> bool {
        if self.can_replace_block(config, chunk.get_block_state(pos)) {
            let state = if pos.get_y() <= context.get_min_gen_y().wrapping_add(31) {
                LAVA.create_legacy_block()
            } else {
                CAVE_AIR
            };
            chunk.set_block_state(pos, state);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_carver() -> NetherWorldCarver {
        NetherWorldCarver::new(Codec::new())
    }

    #[test]
    fn cave_bound_y_scale_and_liquids() {
        let c = make_carver();
        assert_eq!(c.get_cave_bound(), 10);
        assert_eq!(c.get_y_scale(), 5.0_f64);
        assert_eq!(c.liquids, vec![Fluid::Lava, Fluid::Water]);
    }

    #[test]
    fn thickness_operation_order_and_float_bits() {
        let c = make_carver();
        let mut rng = RandomSource::from_floats(vec![0.5_f32, 0.25_f32]);
        let t = c.get_thickness(&mut rng);
        assert_eq!(t.to_bits(), 2.5_f32.to_bits());

        let weird = f32::from_bits(0x3F00_0001);
        let mut rng2 = RandomSource::from_floats(vec![weird, -0.0_f32]);
        let t2 = c.get_thickness(&mut rng2);
        let expected = (weird * 2.0_f32 + (-0.0_f32)) * 2.0_f32;
        assert_eq!(t2.to_bits(), expected.to_bits());
    }

    #[test]
    fn carve_block_replaces_above_lava_level() {
        let c = make_carver();
        let mut config = CaveCarverConfiguration::default();
        config.replaceable_blocks.insert(5);
        let mut chunk = ChunkAccess::new(BlockState { id: 5 });
        let mut mask = CarvingMask;
        let mut reached = MutableBoolean::default();
        let pos = MutableBlockPos { x: 0, y: 10, z: 0 };
        let offset = MutableBlockPos { x: 0, y: 0, z: 0 };
        let ctx = CarvingContext { min_gen_y: -64 };
        let ok = c.carve_block(
            &ctx,
            &config,
            &mut chunk,
            &|_| Holder(Biome { id: 0 }),
            &mut mask,
            &pos,
            &offset,
            &Aquifer,
            &mut reached,
        );
        assert!(ok);
        assert_eq!(chunk.get_block_state(&pos), CAVE_AIR);
    }

    #[test]
    fn carve_block_places_lava_at_or_below_threshold() {
        let c = make_carver();
        let mut config = CaveCarverConfiguration::default();
        config.replaceable_blocks.insert(5);
        let mut chunk = ChunkAccess::new(BlockState { id: 5 });
        let mut mask = CarvingMask;
        let mut reached = MutableBoolean::default();
        let ctx = CarvingContext { min_gen_y: -64 };
        let pos = MutableBlockPos { x: 1, y: -33, z: -2 };
        let offset = MutableBlockPos { x: 0, y: 0, z: 0 };
        let ok = c.carve_block(
            &ctx,
            &config,
            &mut chunk,
            &|_| Holder(Biome { id: 0 }),
            &mut mask,
            &pos,
            &offset,
            &Aquifer,
            &mut reached,
        );
        assert!(ok);
        assert_eq!(chunk.get_block_state(&pos), LAVA.create_legacy_block());
    }

    #[test]
    fn carve_block_not_replaceable_negative_coords() {
        let c = make_carver();
        let config = CaveCarverConfiguration::default();
        let mut chunk = ChunkAccess::new(BlockState { id: 99 });
        let mut mask = CarvingMask;
        let mut reached = MutableBoolean::default();
        let pos = MutableBlockPos {
            x: -100,
            y: -64,
            z: i32::MIN,
        };
        let offset = MutableBlockPos { x: 0, y: 0, z: 0 };
        let ctx = CarvingContext { min_gen_y: i32::MIN };
        let ok = c.carve_block(
            &ctx,
            &config,
            &mut chunk,
            &|_| Holder(Biome { id: 0 }),
            &mut mask,
            &pos,
            &offset,
            &Aquifer,
            &mut reached,
        );
        assert!(!ok);
        assert_eq!(chunk.get_block_state(&pos), BlockState { id: 99 });
    }

    #[test]
    fn carve_block_wrapping_min_gen_y_overflow() {
        let c = make_carver();
        let mut config = CaveCarverConfiguration::default();
        config.replaceable_blocks.insert(7);
        let mut chunk = ChunkAccess::new(BlockState { id: 7 });
        let mut mask = CarvingMask;
        let mut reached = MutableBoolean::default();
        let ctx = CarvingContext { min_gen_y: i32::MAX };
        let pos = MutableBlockPos { x: 0, y: 0, z: 0 };
        let offset = MutableBlockPos { x: 0, y: 0, z: 0 };
        let ok = c.carve_block(
            &ctx,
            &config,
            &mut chunk,
            &|_| Holder(Biome { id: 0 }),
            &mut mask,
            &pos,
            &offset,
            &Aquifer,
            &mut reached,
        );
        assert!(ok);
        assert_eq!(chunk.get_block_state(&pos), CAVE_AIR);
    }
}
