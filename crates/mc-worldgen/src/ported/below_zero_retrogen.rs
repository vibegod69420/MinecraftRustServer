// === FILE: below_zero_retrogen.rs ===

use std::marker::PhantomData;

/// Minimal block position used by retrogen operations.
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

    /// Java-ordered inclusive iteration: x increments fastest, then y, then z.
    pub fn between_closed(
        x1: i32,
        y1: i32,
        z1: i32,
        x2: i32,
        y2: i32,
        z2: i32,
    ) -> Vec<BlockPos> {
        let mut out = Vec::new();
        let mut z = z1;
        loop {
            let mut y = y1;
            loop {
                let mut x = x1;
                loop {
                    out.push(BlockPos::new(x, y, z));
                    if x == x2 {
                        break;
                    }
                    x = x.wrapping_add(1);
                }
                if y == y2 {
                    break;
                }
                y = y.wrapping_add(1);
            }
            if z == z2 {
                break;
            }
            z = z.wrapping_add(1);
        }
        out
    }
}

/// Minimal block identity for retrogen block replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Bedrock,
    Deepslate,
    Other(u32),
}

impl Block {
    pub fn default_block_state(self) -> BlockState {
        BlockState { block: self }
    }
}

/// Minimal block state wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }
}

/// Minimal height accessor (Java: LevelHeightAccessor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightAccessor {
    pub min_y: i32,
    pub height: i32,
}

impl HeightAccessor {
    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_min_y(&self) -> i32 {
        self.min_y
    }

    pub fn get_max_y(&self) -> i32 {
        self.min_y.wrapping_add(self.height).wrapping_sub(1)
    }
}

// JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#UPGRADE_HEIGHT_ACCESSOR
pub const UPGRADE_HEIGHT_ACCESSOR: HeightAccessor = HeightAccessor {
    min_y: -64,
    height: 64,
};

/// Minimal chunk status identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkStatus(pub &'static str);

impl ChunkStatus {
    pub const EMPTY: Self = Self("minecraft:empty");
}

/// Simple BitSet backed by u64 words, matching Java BitSet bit indexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitSet {
    bits: Vec<u64>,
}

impl BitSet {
    pub fn new(nbits: usize) -> Self {
        let words = (nbits.wrapping_add(63)) / 64;
        Self {
            bits: vec![0u64; words],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&w| w == 0)
    }

    pub fn get(&self, bit_index: usize) -> bool {
        let word = bit_index / 64;
        if word >= self.bits.len() {
            return false;
        }
        (self.bits[word] & (1u64 << (bit_index % 64))) != 0
    }

    pub fn set(&mut self, bit_index: usize) {
        let word = bit_index / 64;
        if word >= self.bits.len() {
            self.bits.resize(word.wrapping_add(1), 0u64);
        }
        self.bits[word] |= 1u64 << (bit_index % 64);
    }
}

/// Minimal biome resource key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BiomeResourceKey(pub &'static str);

pub mod biomes {
    use super::BiomeResourceKey;

    pub const LUSH_CAVES: BiomeResourceKey = BiomeResourceKey("minecraft:lush_caves");
    pub const DRIPSTONE_CAVES: BiomeResourceKey = BiomeResourceKey("minecraft:dripstone_caves");
    pub const DEEP_DARK: BiomeResourceKey = BiomeResourceKey("minecraft:deep_dark");
}

// JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#RETAINED_RETROGEN_BIOMES
pub const RETAINED_RETROGEN_BIOMES: &[BiomeResourceKey] = &[
    biomes::LUSH_CAVES,
    biomes::DRIPSTONE_CAVES,
    biomes::DEEP_DARK,
];

fn is_retained_retrogen_biome(key: &BiomeResourceKey) -> bool {
    RETAINED_RETROGEN_BIOMES.contains(key)
}

/// Minimal biome holder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeHolder {
    pub key: BiomeResourceKey,
}

impl BiomeHolder {
    pub fn is<F: Fn(&BiomeResourceKey) -> bool>(&self, predicate: F) -> bool {
        predicate(&self.key)
    }
}

/// Minimal biome resolver (Java: BiomeResolver).
pub trait BiomeResolver<S> {
    fn get_noise_biome(&self, x: i32, y: i32, z: i32, sampler: &S) -> BiomeHolder;
}

/// Minimal chunk access (Java: ChunkAccess).
pub trait ChunkAccess {
    fn is_upgrading(&self) -> bool;
    fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> BiomeHolder;
}

/// Minimal proto chunk (Java: ProtoChunk).
pub trait ProtoChunk: ChunkAccess {
    fn get_height_accessor_for_generation(&self) -> HeightAccessor;
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
    fn set_block_state(&mut self, pos: &BlockPos, state: BlockState);
}

/// Returned by `get_biome_resolver`, preserving Java's "return original when not upgrading" behavior.
pub enum BiomeResolverOrRetrogen<'a, S, R: BiomeResolver<S>> {
    Original(&'a R),
    Upgrading {
        original: &'a R,
        chunk: &'a dyn ChunkAccess,
        _marker: PhantomData<fn(&S)>,
    },
}

impl<'a, S, R: BiomeResolver<S>> BiomeResolver<S> for BiomeResolverOrRetrogen<'a, S, R> {
    fn get_noise_biome(&self, x: i32, y: i32, z: i32, sampler: &S) -> BiomeHolder {
        match self {
            Self::Original(r) => r.get_noise_biome(x, y, z, sampler),
            Self::Upgrading {
                original, chunk, ..
            } => {
                let holder = original.get_noise_biome(x, y, z, sampler);
                if holder.is(is_retained_retrogen_biome) {
                    holder
                } else {
                    chunk.get_noise_biome(x, 0, z)
                }
            }
        }
    }
}

/// Java: net.minecraft.world.level.levelgen.BelowZeroRetrogen
#[derive(Debug, Clone)]
pub struct BelowZeroRetrogen {
    target_status: ChunkStatus,
    missing_bedrock: BitSet,
}

impl BelowZeroRetrogen {
    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#<init>
    pub fn new(target_status: ChunkStatus, missing_bedrock: Option<BitSet>) -> Self {
        Self {
            target_status,
            missing_bedrock: missing_bedrock.unwrap_or_else(|| BitSet::new(0)),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#replaceOldBedrock
    pub fn replace_old_bedrock(chunk: &mut dyn ProtoChunk) {
        for pos in BlockPos::between_closed(0, 0, 0, 15, 4, 15) {
            if chunk.get_block_state(&pos).is(Block::Bedrock) {
                chunk.set_block_state(&pos, Block::Deepslate.default_block_state());
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#applyBedrockMask
    pub fn apply_bedrock_mask(&self, chunk: &mut dyn ProtoChunk) {
        let height_accessor = chunk.get_height_accessor_for_generation();
        let min_y = height_accessor.get_min_y();
        let max_y = height_accessor.get_max_y();

        let mut x = 0i32;
        while x < 16 {
            let mut z = 0i32;
            while z < 16 {
                if self.has_bedrock_hole(x, z) {
                    for pos in BlockPos::between_closed(x, min_y, z, x, max_y, z) {
                        chunk.set_block_state(&pos, Block::Air.default_block_state());
                    }
                }
                z = z.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#targetStatus
    pub fn target_status(&self) -> ChunkStatus {
        self.target_status
    }

    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#hasBedrockHoles
    pub fn has_bedrock_holes(&self) -> bool {
        !self.missing_bedrock.is_empty()
    }

    // JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#hasBedrockHole
    pub fn has_bedrock_hole(&self, x: i32, z: i32) -> bool {
        let index = ((z & 15).wrapping_mul(16)).wrapping_add(x & 15) as usize;
        self.missing_bedrock.get(index)
    }
}

// JAVA: net.minecraft.world.level.levelgen.BelowZeroRetrogen#getBiomeResolver
pub fn get_biome_resolver<'a, S, R: BiomeResolver<S>>(
    resolver: &'a R,
    chunk: &'a dyn ChunkAccess,
) -> BiomeResolverOrRetrogen<'a, S, R> {
    if !chunk.is_upgrading() {
        return BiomeResolverOrRetrogen::Original(resolver);
    }
    BiomeResolverOrRetrogen::Upgrading {
        original: resolver,
        chunk,
        _marker: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockChunk {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        min_y: i32,
        height: i32,
        upgrading: bool,
        noise_biomes: HashMap<(i32, i32, i32), BiomeHolder>,
    }

    impl MockChunk {
        fn new(min_y: i32, height: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                min_y,
                height,
                upgrading: false,
                noise_biomes: HashMap::new(),
            }
        }
    }

    impl ChunkAccess for MockChunk {
        fn is_upgrading(&self) -> bool {
            self.upgrading
        }

        fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> BiomeHolder {
            self.noise_biomes
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BiomeHolder {
                    key: BiomeResourceKey("minecraft:plains"),
                })
        }
    }

    impl ProtoChunk for MockChunk {
        fn get_height_accessor_for_generation(&self) -> HeightAccessor {
            HeightAccessor {
                min_y: self.min_y,
                height: self.height,
            }
        }

        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(Block::Air.default_block_state())
        }

        fn set_block_state(&mut self, pos: &BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    #[test]
    fn has_bedrock_hole_masks_negative_coordinates() {
        let mut bits = BitSet::new(256);
        let index = ((2i32 & 15).wrapping_mul(16)).wrapping_add(1i32 & 15) as usize; // x=1,z=2
        bits.set(index);
        let retrogen = BelowZeroRetrogen::new(ChunkStatus("minecraft:full"), Some(bits));

        assert!(retrogen.has_bedrock_hole(1, 2));
        assert!(retrogen.has_bedrock_hole(-15, -14)); // -15&15==1, -14&15==2
        assert!(!retrogen.has_bedrock_hole(0, 0));
        assert!(retrogen.has_bedrock_holes());
        assert!(!retrogen.has_bedrock_hole(i32::MIN, i32::MAX));
    }

    #[test]
    fn empty_missing_bedrock_has_no_holes() {
        let retrogen = BelowZeroRetrogen::new(ChunkStatus("minecraft:full"), None);
        assert!(!retrogen.has_bedrock_holes());
        assert!(!retrogen.has_bedrock_hole(0, 0));
        assert!(!retrogen.has_bedrock_hole(-1, -1));
        assert_eq!(retrogen.target_status(), ChunkStatus("minecraft:full"));
    }

    #[test]
    fn replace_old_bedrock_replaces_only_bedrock_in_region() {
        let mut chunk = MockChunk::new(-64, 384);
        chunk.set_block_state(&BlockPos::new(0, 0, 0), Block::Bedrock.default_block_state());
        chunk.set_block_state(&BlockPos::new(15, 4, 15), Block::Bedrock.default_block_state());
        chunk.set_block_state(
            &BlockPos::new(5, 2, 5),
            Block::Deepslate.default_block_state(),
        );
        chunk.set_block_state(&BlockPos::new(0, 5, 0), Block::Bedrock.default_block_state());

        BelowZeroRetrogen::replace_old_bedrock(&mut chunk);

        assert_eq!(
            chunk.get_block_state(&BlockPos::new(0, 0, 0)).block,
            Block::Deepslate
        );
        assert_eq!(
            chunk.get_block_state(&BlockPos::new(15, 4, 15)).block,
            Block::Deepslate
        );
        assert_eq!(
            chunk.get_block_state(&BlockPos::new(5, 2, 5)).block,
            Block::Deepslate
        );
        assert_eq!(
            chunk.get_block_state(&BlockPos::new(0, 5, 0)).block,
            Block::Bedrock
        );
    }

    #[test]
    fn apply_bedrock_mask_clears_only_hole_columns() {
        let mut bits = BitSet::new(256);
        let idx = ((3i32 & 15).wrapping_mul(16)).wrapping_add(1i32 & 15) as usize; // x=1,z=3
        bits.set(idx);
        let retrogen = BelowZeroRetrogen::new(ChunkStatus("minecraft:full"), Some(bits));

        let mut chunk = MockChunk::new(0, 4); // min_y=0, height=4 => max_y=3
        chunk.set_block_state(&BlockPos::new(1, 0, 3), Block::Bedrock.default_block_state());
        chunk.set_block_state(
            &BlockPos::new(1, 3, 3),
            Block::Deepslate.default_block_state(),
        );
        chunk.set_block_state(&BlockPos::new(2, 0, 3), Block::Bedrock.default_block_state());

        retrogen.apply_bedrock_mask(&mut chunk);

        assert_eq!(chunk.get_block_state(&BlockPos::new(1, 0, 3)).block, Block::Air);
        assert_eq!(chunk.get_block_state(&BlockPos::new(1, 3, 3)).block, Block::Air);
        assert_eq!(
            chunk.get_block_state(&BlockPos::new(2, 0, 3)).block,
            Block::Bedrock
        );
    }

    #[test]
    fn get_biome_resolver_handles_upgrading_and_retained_biomes() {
        struct FixedResolver(BiomeHolder);
        impl BiomeResolver<()> for FixedResolver {
            fn get_noise_biome(
                &self,
                _x: i32,
                _y: i32,
                _z: i32,
                _sampler: &(),
            ) -> BiomeHolder {
                self.0
            }
        }

        let mut chunk = MockChunk::new(-64, 384);
        chunk.upgrading = true;
        chunk.noise_biomes.insert(
            (10, 0, 20),
            BiomeHolder {
                key: BiomeResourceKey("minecraft:plains"),
            },
        );

        let plains_resolver = FixedResolver(BiomeHolder {
            key: BiomeResourceKey("minecraft:plains"),
        });
        let resolved = get_biome_resolver(&plains_resolver, &chunk);
        let holder = resolved.get_noise_biome(10, 5, 20, &());
        assert_eq!(holder.key, BiomeResourceKey("minecraft:plains"));

        let lush_resolver = FixedResolver(BiomeHolder {
            key: biomes::LUSH_CAVES,
        });
        let resolved_lush = get_biome_resolver(&lush_resolver, &chunk);
        let lush_holder = resolved_lush.get_noise_biome(10, 5, 20, &());
        assert_eq!(lush_holder.key, biomes::LUSH_CAVES);

        chunk.upgrading = false;
        let not_upgrading = get_biome_resolver(&plains_resolver, &chunk);
        let holder2 = not_upgrading.get_noise_biome(10, 5, 20, &());
        assert_eq!(holder2.key, BiomeResourceKey("minecraft:plains"));
    }

    #[test]
    fn upgrade_height_accessor_matches_java() {
        assert_eq!(UPGRADE_HEIGHT_ACCESSOR.get_height(), 64);
        assert_eq!(UPGRADE_HEIGHT_ACCESSOR.get_min_y(), -64);
        assert_eq!(UPGRADE_HEIGHT_ACCESSOR.get_max_y(), -1);
    }
}
