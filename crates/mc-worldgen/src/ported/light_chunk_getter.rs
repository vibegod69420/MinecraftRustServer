/// Placeholder for `net.minecraft.world.level.LightLayer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightLayer {
    Block,
    Sky,
}

/// Placeholder for `net.minecraft.core.SectionPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Placeholder for `net.minecraft.world.level.chunk.LightChunk`.
pub trait LightChunk {}

/// Placeholder for `net.minecraft.world.level.BlockGetter`.
pub trait BlockGetter {}

pub trait LightChunkGetter {
    type Chunk: LightChunk;
    type Level: BlockGetter;

    // JAVA: net.minecraft.world.level.chunk.LightChunkGetter#getChunkForLighting
    fn get_chunk_for_lighting(&self, var1: i32, var2: i32) -> Option<&Self::Chunk>;

    // JAVA: net.minecraft.world.level.chunk.LightChunkGetter#onLightUpdate
    fn on_light_update(&mut self, _layer: LightLayer, _pos: SectionPos) {}

    // JAVA: net.minecraft.world.level.chunk.LightChunkGetter#getLevel
    fn get_level(&self) -> &Self::Level;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChunk;
    impl LightChunk for MockChunk {}

    struct MockLevel;
    impl BlockGetter for MockLevel {}

    struct MockGetter {
        chunk: Option<MockChunk>,
        level: MockLevel,
        updates: Vec<(LightLayer, SectionPos)>,
    }

    impl MockGetter {
        fn new(has_chunk: bool) -> Self {
            Self {
                chunk: if has_chunk { Some(MockChunk) } else { None },
                level: MockLevel,
                updates: Vec::new(),
            }
        }
    }

    impl LightChunkGetter for MockGetter {
        type Chunk = MockChunk;
        type Level = MockLevel;

        fn get_chunk_for_lighting(&self, _var1: i32, _var2: i32) -> Option<&Self::Chunk> {
            self.chunk.as_ref()
        }

        fn on_light_update(&mut self, layer: LightLayer, pos: SectionPos) {
            self.updates.push((layer, pos));
        }

        fn get_level(&self) -> &Self::Level {
            &self.level
        }
    }

    #[test]
    fn get_chunk_for_lighting_some_and_none() {
        let with_chunk = MockGetter::new(true);
        let without_chunk = MockGetter::new(false);
        assert!(with_chunk.get_chunk_for_lighting(0, 0).is_some());
        assert!(without_chunk.get_chunk_for_lighting(0, 0).is_none());
    }

    #[test]
    fn get_chunk_for_lighting_negative_and_extreme_coords() {
        let g = MockGetter::new(true);
        assert!(g.get_chunk_for_lighting(-1, -100).is_some());
        assert!(g.get_chunk_for_lighting(i32::MIN, i32::MAX).is_some());
    }

    #[test]
    fn default_on_light_update_is_noop() {
        struct MinimalGetter {
            level: MockLevel,
        }

        impl LightChunkGetter for MinimalGetter {
            type Chunk = MockChunk;
            type Level = MockLevel;

            fn get_chunk_for_lighting(&self, _var1: i32, _var2: i32) -> Option<&Self::Chunk> {
                None
            }

            fn get_level(&self) -> &Self::Level {
                &self.level
            }
        }

        let mut g = MinimalGetter { level: MockLevel };
        g.on_light_update(LightLayer::Block, SectionPos::new(1, -2, 3));
        g.on_light_update(LightLayer::Sky, SectionPos::new(i32::MIN, 0, i32::MAX));
    }

    #[test]
    fn on_light_update_records_calls() {
        let mut g = MockGetter::new(false);
        let pos = SectionPos::new(10, 20, 30);
        g.on_light_update(LightLayer::Sky, pos);
        assert_eq!(g.updates.len(), 1);
        assert_eq!(g.updates[0], (LightLayer::Sky, pos));
    }

    #[test]
    fn get_level_returns_reference() {
        let g = MockGetter::new(false);
        let _level: &MockLevel = g.get_level();
    }
}
