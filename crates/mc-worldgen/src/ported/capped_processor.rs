//! Port of `net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor`.
//!
//! This module is self-contained: dependencies that are not ported in this batch
//! are represented with minimal local stubs preserving the behavior used by
//! `CappedProcessor`.

use std::fmt::Debug;

/// Minimal block position stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    /// Creates a new block position.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal `StructureTemplate.StructureBlockInfo` stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    /// Placeholder for block state identity.
    pub state: i32,
    /// Placeholder for optional NBT presence.
    pub nbt_present: bool,
}

impl StructureBlockInfo {
    /// Creates a simple block info value for tests and local processing.
    pub fn new(pos: BlockPos, state: i32, nbt_present: bool) -> Self {
        Self {
            pos,
            state,
            nbt_present,
        }
    }
}

/// Minimal `StructurePlaceSettings` stub.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructurePlaceSettings;

/// Minimal level stub exposing a seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    seed: i64,
}

impl Level {
    /// Returns the level seed.
    pub fn get_seed(&self) -> i64 {
        self.seed
    }
}

/// Minimal `ServerLevelAccessor` stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerLevelAccessor {
    level: Level,
}

impl ServerLevelAccessor {
    /// Creates a server level accessor with the given seed.
    pub fn new(seed: i64) -> Self {
        Self {
            level: Level { seed },
        }
    }

    /// Returns the backing level.
    pub fn get_level(&self) -> &Level {
        &self.level
    }
}

/// Deterministic positional random factory stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionalRandomFactory {
    seed: u64,
}

impl PositionalRandomFactory {
    /// Creates a random source at a block position.
    pub fn at(&self, pos: BlockPos) -> RandomSource {
        let mut h = self.seed;
        h ^= (pos.x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= (pos.y as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= (pos.z as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
        if h == 0 {
            h = 0xD1B5_4A32_D192_ED03;
        }
        RandomSource { state: h }
    }
}

/// Deterministic random source stub.
///
/// This is not a bit-identical port of Minecraft's Xoroshiro positional random,
/// but it provides deterministic behavior for the control flow used by
/// `CappedProcessor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSource {
    state: u64,
}

impl RandomSource {
    /// Creates a random source from a seed.
    pub fn create(seed: i64) -> Self {
        let mut state = seed as u64;
        if state == 0 {
            state = 0x9E37_79B9_7F4A_7C15;
        }
        Self { state }
    }

    /// Creates a positional random factory.
    pub fn fork_positional(&self) -> PositionalRandomFactory {
        PositionalRandomFactory { seed: self.state }
    }

    fn next_u32(&mut self) -> u32 {
        // xorshift64* deterministic stub
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        ((x.wrapping_mul(0x2545_F491_4F6C_DD1D)) >> 32) as u32
    }

    /// Returns a random integer in `[0, bound)`, or `0` for non-positive bounds.
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        let b = bound as u32;
        (self.next_u32() % b) as i32
    }
}

/// Minimal `IntProvider` stub supporting constant and uniform providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Constant(i32),
    Uniform { min: i32, max: i32 },
}

impl IntProvider {
    /// Creates a constant int provider.
    pub fn constant(value: i32) -> Self {
        Self::Constant(value)
    }

    /// Creates a uniform int provider.
    pub fn uniform(min: i32, max: i32) -> Self {
        Self::Uniform { min, max }
    }

    /// Returns the maximum possible value.
    pub fn get_max_value(&self) -> i32 {
        match self {
            Self::Constant(v) => *v,
            Self::Uniform { max, .. } => *max,
        }
    }

    /// Samples a value from this provider.
    pub fn sample(&self, random: &mut RandomSource) -> i32 {
        match self {
            Self::Constant(v) => *v,
            Self::Uniform { min, max } => {
                if *max <= *min {
                    return *min;
                }
                let span = (*max).wrapping_sub(*min);
                let bound = span.wrapping_add(1);
                (*min).wrapping_add(random.next_int_bound(bound))
            }
        }
    }
}

/// Minimal utility stub for methods used by `CappedProcessor`.
pub struct Util;

impl Util {
    /// Logs a message; Java pauses in IDE, which is not applicable here.
    pub fn log_and_pause_if_in_ide(msg: &str) {
        eprintln!("{msg}");
    }

    /// Returns a shuffled copy of the input list using Fisher-Yates shuffle.
    pub fn to_shuffled_list(mut list: Vec<i32>, random: &mut RandomSource) -> Vec<i32> {
        let len = list.len();
        if len > 1 {
            let mut i = len - 1;
            loop {
                let bound = (i as i32).wrapping_add(1);
                let j = random.next_int_bound(bound) as usize;
                list.swap(i, j);
                if i == 0 {
                    break;
                }
                i -= 1;
            }
        }
        list
    }
}

/// Minimal `StructureProcessorType` stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureProcessorType {
    pub name: &'static str,
}

impl StructureProcessorType {
    pub const CAPPED: Self = Self { name: "capped" };
}

/// Minimal `StructureProcessor` base trait.
pub trait StructureProcessor: Debug {
    /// Returns the processor type.
    fn get_type(&self) -> StructureProcessorType;

    /// Processes a single structure block.
    fn process_block(
        &self,
        _level: &ServerLevelAccessor,
        _offset: BlockPos,
        _pos: BlockPos,
        _original: &StructureBlockInfo,
        processed: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        Some(processed.clone())
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor`.
#[derive(Debug)]
pub struct CappedProcessor {
    delegate: Box<dyn StructureProcessor>,
    limit: IntProvider,
}

impl CappedProcessor {
    /// Creates a new capped processor.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor#<init>
    pub fn new(delegate: Box<dyn StructureProcessor>, limit: IntProvider) -> Self {
        Self { delegate, limit }
    }

    /// Returns the delegate processor.
    pub fn delegate(&self) -> &dyn StructureProcessor {
        &*self.delegate
    }

    /// Returns the limit provider.
    pub fn limit(&self) -> IntProvider {
        self.limit
    }

    /// Returns the processor type.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::CAPPED
    }

    /// Finalizes processing by applying the delegate to at most `limit` randomly selected blocks.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor#finalizeProcessing
    pub fn finalize_processing(
        &self,
        level: &ServerLevelAccessor,
        offset: BlockPos,
        pos: BlockPos,
        original_list: &[StructureBlockInfo],
        mut processed_list: Vec<StructureBlockInfo>,
        settings: &StructurePlaceSettings,
    ) -> Vec<StructureBlockInfo> {
        if self.limit.get_max_value() != 0 && !processed_list.is_empty() {
            if original_list.len() != processed_list.len() {
                Util::log_and_pause_if_in_ide(&format!(
                    "Original block info list not in sync with processed list, skipping processing. Original size: {}, Processed size: {}",
                    original_list.len(),
                    processed_list.len()
                ));
                return processed_list;
            }

            let mut random = RandomSource::create(level.get_level().get_seed())
                .fork_positional()
                .at(offset);

            let sampled = self.limit.sample(&mut random);
            let processed_len_i32 = processed_list.len() as i32;
            let limit_count = if sampled < processed_len_i32 {
                sampled
            } else {
                processed_len_i32
            };
            if limit_count < 1 {
                return processed_list;
            }

            let indices: Vec<i32> = (0..processed_len_i32).collect();
            let shuffled = Util::to_shuffled_list(indices, &mut random);
            let mut applied: i32 = 0;

            for &idx in &shuffled {
                if !(applied < limit_count) {
                    break;
                }
                let i = idx as usize;
                let original = &original_list[i];
                let processed = processed_list[i].clone();
                let result = self.delegate.process_block(
                    level,
                    offset,
                    pos,
                    original,
                    &processed,
                    settings,
                );
                if let Some(new_block) = result {
                    if processed != new_block {
                        applied = applied.wrapping_add(1);
                        processed_list[i] = new_block;
                    }
                }
            }

            return processed_list;
        }

        processed_list
    }
}

impl StructureProcessor for CappedProcessor {
    fn get_type(&self) -> StructureProcessorType {
        CappedProcessor::get_type(self)
    }

    fn process_block(
        &self,
        _level: &ServerLevelAccessor,
        _offset: BlockPos,
        _pos: BlockPos,
        _original: &StructureBlockInfo,
        processed: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        Some(processed.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ReplaceDelegate {
        delta: i32,
    }

    impl StructureProcessor for ReplaceDelegate {
        fn get_type(&self) -> StructureProcessorType {
            StructureProcessorType { name: "replace" }
        }

        fn process_block(
            &self,
            _level: &ServerLevelAccessor,
            _offset: BlockPos,
            _pos: BlockPos,
            _original: &StructureBlockInfo,
            processed: &StructureBlockInfo,
            _settings: &StructurePlaceSettings,
        ) -> Option<StructureBlockInfo> {
            let mut out = processed.clone();
            out.state = out.state.wrapping_add(self.delta);
            Some(out)
        }
    }

    #[derive(Debug)]
    struct NoChangeDelegate;

    impl StructureProcessor for NoChangeDelegate {
        fn get_type(&self) -> StructureProcessorType {
            StructureProcessorType { name: "noop" }
        }

        fn process_block(
            &self,
            _level: &ServerLevelAccessor,
            _offset: BlockPos,
            _pos: BlockPos,
            _original: &StructureBlockInfo,
            processed: &StructureBlockInfo,
            _settings: &StructurePlaceSettings,
        ) -> Option<StructureBlockInfo> {
            Some(processed.clone())
        }
    }

    #[derive(Debug)]
    struct NullDelegate;

    impl StructureProcessor for NullDelegate {
        fn get_type(&self) -> StructureProcessorType {
            StructureProcessorType { name: "null" }
        }

        fn process_block(
            &self,
            _level: &ServerLevelAccessor,
            _offset: BlockPos,
            _pos: BlockPos,
            _original: &StructureBlockInfo,
            _processed: &StructureBlockInfo,
            _settings: &StructurePlaceSettings,
        ) -> Option<StructureBlockInfo> {
            None
        }
    }

    fn block(x: i32, y: i32, z: i32, state: i32) -> StructureBlockInfo {
        StructureBlockInfo::new(BlockPos::new(x, y, z), state, false)
    }

    #[test]
    fn limit_zero_returns_unchanged() {
        let proc = CappedProcessor::new(Box::new(ReplaceDelegate { delta: 10 }), IntProvider::constant(0));
        let level = ServerLevelAccessor::new(123);
        let pos = BlockPos::new(1, 2, 3);
        let original = vec![block(0, 0, 0, 1), block(1, 0, 0, 2)];
        let processed = original.clone();
        let out = proc.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out, processed);
        assert_eq!(proc.get_type(), StructureProcessorType::CAPPED);
    }

    #[test]
    fn size_mismatch_returns_unchanged() {
        let proc = CappedProcessor::new(Box::new(ReplaceDelegate { delta: 10 }), IntProvider::constant(5));
        let level = ServerLevelAccessor::new(123);
        let pos = BlockPos::new(0, 0, 0);
        let original = vec![block(0, 0, 0, 1)];
        let processed = vec![block(0, 0, 0, 1), block(1, 0, 0, 2)];
        let out = proc.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out, processed);
    }

    #[test]
    fn applies_delegate_up_to_limit() {
        let proc = CappedProcessor::new(Box::new(ReplaceDelegate { delta: 100 }), IntProvider::constant(2));
        let level = ServerLevelAccessor::new(42);
        let pos = BlockPos::new(7, 8, 9);
        let original: Vec<StructureBlockInfo> = (0..5).map(|i| block(i, 0, 0, i)).collect();
        let processed = original.clone();
        let out = proc.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);

        let changed = out.iter().zip(processed.iter()).filter(|(a, b)| a != b).count();
        assert_eq!(changed, 2);
        for (o, p) in out.iter().zip(processed.iter()) {
            assert!(o.state == p.state || o.state == p.state.wrapping_add(100));
        }
    }

    #[test]
    fn no_change_or_null_delegate_does_not_count_toward_limit() {
        let level = ServerLevelAccessor::new(99);
        let pos = BlockPos::new(0, 0, 0);
        let original: Vec<StructureBlockInfo> = (0..4).map(|i| block(i, 0, 0, i)).collect();
        let processed = original.clone();

        let no_change = CappedProcessor::new(Box::new(NoChangeDelegate), IntProvider::constant(3));
        let out1 = no_change.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out1, processed);

        let null_del = CappedProcessor::new(Box::new(NullDelegate), IntProvider::constant(3));
        let out2 = null_del.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out2, processed);
    }

    #[test]
    fn empty_or_negative_limit_and_overflow_safe_wrapping() {
        let level = ServerLevelAccessor::new(-1);
        let pos = BlockPos::new(i32::MIN, 0, i32::MAX);
        let original = vec![block(0, 0, 0, i32::MAX)];
        let processed = original.clone();

        let neg = CappedProcessor::new(Box::new(ReplaceDelegate { delta: 1 }), IntProvider::constant(-5));
        let out_neg = neg.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out_neg, processed);

        let wrap = CappedProcessor::new(Box::new(ReplaceDelegate { delta: 1 }), IntProvider::constant(1));
        let out_wrap = wrap.finalize_processing(&level, pos, pos, &original, processed.clone(), &StructurePlaceSettings);
        assert_eq!(out_wrap[0].state, i32::MIN); // wrapping_add(i32::MAX, 1)
    }

    #[test]
    fn shuffled_list_is_permutation() {
        let mut r1 = RandomSource::create(12345);
        let mut r2 = RandomSource::create(12345);
        let input: Vec<i32> = (0..10).collect();
        let s1 = Util::to_shuffled_list(input.clone(), &mut r1);
        let s2 = Util::to_shuffled_list(input.clone(), &mut r2);
        assert_eq!(s1, s2);
        let mut sorted = s1.clone();
        sorted.sort();
        assert_eq!(sorted, input);
    }
}
