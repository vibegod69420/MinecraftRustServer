use std::fmt;

/// Minimal self-contained placeholder for `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.lighting.ChunkSkyLightSources`.
#[derive(Debug, Clone, Default)]
pub struct ChunkSkyLightSources {
    pub source_count: i32,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.BlockGetter`.
pub trait BlockGetter: fmt::Debug {}

/// Port of `net.minecraft.world.level.chunk.LightChunk`.
pub trait LightChunk: BlockGetter {
    // JAVA: net.minecraft.world.level.chunk.LightChunk#findBlockLightSources
    fn find_block_light_sources(&mut self, output: &mut dyn FnMut(BlockPos, BlockState));

    // JAVA: net.minecraft.world.level.chunk.LightChunk#getSkyLightSources
    fn get_sky_light_sources(&self) -> &ChunkSkyLightSources;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct MockLightChunk {
        sources: Vec<(BlockPos, BlockState)>,
        sky: ChunkSkyLightSources,
    }

    impl BlockGetter for MockLightChunk {}

    impl LightChunk for MockLightChunk {
        fn find_block_light_sources(&mut self, output: &mut dyn FnMut(BlockPos, BlockState)) {
            for (pos, state) in &self.sources {
                output(*pos, state.clone());
            }
        }

        fn get_sky_light_sources(&self) -> &ChunkSkyLightSources {
            &self.sky
        }
    }

    #[test]
    fn find_block_light_sources_collects_all() {
        let mut chunk = MockLightChunk {
            sources: vec![
                (BlockPos { x: 1, y: 2, z: 3 }, BlockState { id: 10 }),
                (BlockPos { x: -1, y: -64, z: 0 }, BlockState { id: 20 }),
            ],
            sky: ChunkSkyLightSources { source_count: 2 },
        };
        let mut found = Vec::new();
        chunk.find_block_light_sources(&mut |p, s| found.push((p, s)));
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, BlockPos { x: 1, y: 2, z: 3 });
        assert_eq!(found[1].0.y, -64);
        assert_eq!(found[1].1.id, 20);
    }

    #[test]
    fn get_sky_light_sources_returns_reference() {
        let chunk = MockLightChunk {
            sources: Vec::new(),
            sky: ChunkSkyLightSources { source_count: 42 },
        };
        assert_eq!(chunk.get_sky_light_sources().source_count, 42);
    }

    #[test]
    fn empty_chunk_produces_no_sources() {
        let mut chunk = MockLightChunk::default();
        let mut count = 0i32;
        chunk.find_block_light_sources(&mut |_, _| {
            count = count.wrapping_add(1);
        });
        assert_eq!(count, 0);
        assert_eq!(chunk.get_sky_light_sources().source_count, 0);
    }

    #[test]
    fn negative_and_extreme_positions() {
        let mut chunk = MockLightChunk {
            sources: vec![
                (
                    BlockPos {
                        x: i32::MIN,
                        y: i32::MIN,
                        z: i32::MIN,
                    },
                    BlockState { id: u32::MAX },
                ),
                (
                    BlockPos {
                        x: i32::MAX,
                        y: i32::MAX,
                        z: i32::MAX,
                    },
                    BlockState { id: 0 },
                ),
            ],
            sky: ChunkSkyLightSources { source_count: -1 },
        };
        let mut found = Vec::new();
        chunk.find_block_light_sources(&mut |p, s| found.push((p, s)));
        assert_eq!(found[0].0.x, i32::MIN);
        assert_eq!(found[1].0.z, i32::MAX);
        assert_eq!(found[0].1.id, u32::MAX);
        assert_eq!(chunk.get_sky_light_sources().source_count, -1);
    }
}
