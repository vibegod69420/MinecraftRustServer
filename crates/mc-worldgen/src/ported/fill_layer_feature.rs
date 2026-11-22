use std::marker::PhantomData;

/// Placeholder for `com.mojang.serialization.Codec<T>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Codec<T>(pub PhantomData<T>);

/// Placeholder for `net.minecraft.util.RandomSource`.
pub trait RandomSource {}

/// Minimal block state abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#isAir
    pub fn is_air(&self) -> bool {
        self.id == 0
    }
}

/// Minimal equivalent of `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#new(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

/// Minimal equivalent of `net.minecraft.core.BlockPos.MutableBlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#<init>
    pub fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#set(int,int,int)
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }
}

impl Default for MutableBlockPos {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MutableBlockPos> for BlockPos {
    fn from(value: MutableBlockPos) -> Self {
        BlockPos::new(value.x, value.y, value.z)
    }
}

/// Minimal equivalent of `net.minecraft.world.level.WorldGenLevel`.
pub trait WorldGenLevel {
    fn get_min_y(&self) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

/// Placeholder for `net.minecraft.world.level.chunk.ChunkGenerator`.
pub trait ChunkGenerator {}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.configurations.LayerConfiguration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerConfiguration {
    pub height: i32,
    pub state: BlockState,
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.FeaturePlaceContext`.
pub struct FeaturePlaceContext<'a, C, Level: ?Sized, Generator: ?Sized, R: ?Sized> {
    pub config: &'a C,
    pub level: &'a mut Level,
    pub chunk_generator: &'a mut Generator,
    pub random: &'a mut R,
    pub origin: BlockPos,
}

impl<'a, C, Level: ?Sized, Generator: ?Sized, R: ?Sized> FeaturePlaceContext<'a, C, Level, Generator, R> {
    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#origin
    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#config
    pub fn config(&self) -> &C {
        self.config
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#level
    pub fn level(&mut self) -> &mut Level {
        self.level
    }
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.Feature`.
pub trait Feature<C, Level: ?Sized, Generator: ?Sized, R: ?Sized> {
    fn place(&mut self, ctx: &mut FeaturePlaceContext<'_, C, Level, Generator, R>) -> bool;
}

/// Port of `net.minecraft.world.level.levelgen.feature.FillLayerFeature`.
pub struct FillLayerFeature<C> {
    _codec: Codec<C>,
}

impl<C> FillLayerFeature<C> {
    // JAVA: net.minecraft.world.level.levelgen.feature.FillLayerFeature#<init>
    pub fn new(codec: Codec<C>) -> Self {
        Self { _codec: codec }
    }
}

impl<Level, Generator, R> Feature<LayerConfiguration, Level, Generator, R> for FillLayerFeature<LayerConfiguration>
where
    Level: WorldGenLevel + ?Sized,
    Generator: ChunkGenerator + ?Sized,
    R: RandomSource + ?Sized,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.FillLayerFeature#place
    fn place(
        &mut self,
        ctx: &mut FeaturePlaceContext<'_, LayerConfiguration, Level, Generator, R>,
    ) -> bool {
        let origin = ctx.origin();
        let config = *ctx.config();
        let level = ctx.level();
        let mut mutable_pos = MutableBlockPos::new();

        let mut i: i32 = 0;
        while i < 16 {
            let mut j: i32 = 0;
            while j < 16 {
                let x = origin.get_x().wrapping_add(i);
                let z = origin.get_z().wrapping_add(j);
                let y = level.get_min_y().wrapping_add(config.height);
                mutable_pos.set(x, y, z);
                let pos = BlockPos::from(mutable_pos);
                if level.get_block_state(pos).is_air() {
                    level.set_block(pos, config.state, 2);
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestRandom;
    impl RandomSource for TestRandom {}

    #[derive(Default)]
    struct TestGenerator;
    impl ChunkGenerator for TestGenerator {}

    struct TestLevel {
        min_y: i32,
        blocks: HashMap<(i32, i32, i32), BlockState>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
    }

    impl TestLevel {
        fn new(min_y: i32) -> Self {
            Self {
                min_y,
                blocks: HashMap::new(),
                set_calls: Vec::new(),
            }
        }
    }

    impl WorldGenLevel for TestLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self.blocks.get(&(pos.x, pos.y, pos.z)).unwrap_or(&BlockState { id: 0 })
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self.set_calls.push((pos, state, flags));
            true
        }
    }

    #[test]
    fn fills_16x16_only_where_air() {
        let mut level = TestLevel::new(-64);
        // Block one position as non-air
        level.blocks.insert((5, -60, 7), BlockState { id: 1 });
        let config = LayerConfiguration {
            height: 4,
            state: BlockState { id: 42 },
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom;
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin: BlockPos::new(0, 100, 0),
        };
        let mut feature = FillLayerFeature::new(Codec(PhantomData));
        let ok = feature.place(&mut ctx);

        assert!(ok);
        assert_eq!(level.set_calls.len(), 255);
        assert_eq!(level.get_block_state(BlockPos::new(5, -60, 7)), BlockState { id: 1 });
        assert_eq!(level.get_block_state(BlockPos::new(0, -60, 0)), BlockState { id: 42 });
        assert!(level.set_calls.iter().all(|(_, _, flags)| *flags == 2));
    }

    #[test]
    fn uses_wrapping_add_for_coordinates() {
        let mut level = TestLevel::new(0);
        let config = LayerConfiguration {
            height: 1,
            state: BlockState { id: 7 },
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom;
        let origin = BlockPos::new(i32::MAX - 1, 0, i32::MIN);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin,
        };
        let mut feature = FillLayerFeature::new(Codec(PhantomData));
        let ok = feature.place(&mut ctx);

        assert!(ok);
        assert_eq!(level.set_calls.len(), 256);
        // Check wrapped X values exist
        assert!(level.blocks.contains_key(&(i32::MAX - 1, 1, i32::MIN)));
        assert!(level.blocks.contains_key(&(i32::MAX, 1, i32::MIN)));
        assert!(level.blocks.contains_key(&(i32::MIN, 1, i32::MIN)));
    }

    #[test]
    fn always_returns_true() {
        let mut level = TestLevel::new(0);
        let config = LayerConfiguration {
            height: 0,
            state: BlockState { id: 5 },
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom;
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin: BlockPos::new(0, 0, 0),
        };
        let mut feature = FillLayerFeature::new(Codec(PhantomData));
        assert!(feature.place(&mut ctx));
    }

    #[test]
    fn mutable_block_pos_set_updates_all_axes() {
        let mut pos = MutableBlockPos::new();
        pos.set(-10, 20, 30);
        assert_eq!(pos.x, -10);
        assert_eq!(pos.y, 20);
        assert_eq!(pos.z, 30);
        let bp = BlockPos::from(pos);
        assert_eq!(bp, BlockPos::new(-10, 20, 30));
    }
}
