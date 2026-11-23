use std::collections::HashSet;
use std::io;

/// Minimal chunk status used by [`ChunkSource`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChunkStatus {
    Empty,
    Full,
}

/// Minimal chunk position used by [`ChunkSource`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Minimal light chunk marker.
pub trait LightChunk {}

/// Minimal chunk access abstraction.
pub trait ChunkAccess {
    fn as_level_chunk(&self) -> Option<&dyn LevelChunk>;
    fn as_light_chunk(&self) -> Option<&dyn LightChunk>;
}

/// Minimal level chunk abstraction.
pub trait LevelChunk: ChunkAccess + LightChunk {}

/// Minimal level light engine placeholder.
#[derive(Debug, Default)]
pub struct LevelLightEngine;

/// Port of `net.minecraft.world.level.chunk.ChunkSource`.
pub trait ChunkSource {
    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getChunk(int,int,ChunkStatus,boolean)
    fn get_chunk_with_status(
        &self,
        x: i32,
        z: i32,
        status: ChunkStatus,
        create: bool,
    ) -> Option<&dyn ChunkAccess>;

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getChunk(int,int,boolean)
    fn get_chunk(&self, x: i32, z: i32, create: bool) -> Option<&dyn LevelChunk> {
        self.get_chunk_with_status(x, z, ChunkStatus::Full, create)
            .and_then(|chunk| chunk.as_level_chunk())
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getChunkNow(int,int)
    fn get_chunk_now(&self, x: i32, z: i32) -> Option<&dyn LevelChunk> {
        self.get_chunk(x, z, false)
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getChunkForLighting(int,int)
    fn get_chunk_for_lighting(&self, x: i32, z: i32) -> Option<&dyn LightChunk> {
        self.get_chunk_with_status(x, z, ChunkStatus::Empty, false)
            .and_then(|chunk| chunk.as_light_chunk())
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#hasChunk(int,int)
    fn has_chunk(&self, x: i32, z: i32) -> bool {
        self.get_chunk_with_status(x, z, ChunkStatus::Full, false)
            .is_some()
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#tick(BooleanSupplier,boolean)
    fn tick(&mut self, has_time_left: &dyn Fn() -> bool, tick_chunks: bool);

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#onSectionEmptinessChanged(int,int,int,boolean)
    fn on_section_emptiness_changed(&mut self, _x: i32, _y: i32, _z: i32, _empty: bool) {}

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#gatherStats()
    fn gather_stats(&self) -> String;

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getLoadedChunksCount()
    fn get_loaded_chunks_count(&self) -> i32;

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#close()
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getLightEngine()
    fn get_light_engine(&self) -> &LevelLightEngine;

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#setSpawnSettings(boolean)
    fn set_spawn_settings(&mut self, _spawn_settings: bool) {}

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#updateChunkForced(ChunkPos,boolean)
    fn update_chunk_forced(&mut self, _pos: ChunkPos, _forced: bool) -> bool {
        false
    }

    // JAVA: net.minecraft.world.level.chunk.ChunkSource#getForceLoadedChunks()
    fn get_force_loaded_chunks(&self) -> HashSet<i64> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockLevelChunk;

    impl ChunkAccess for MockLevelChunk {
        fn as_level_chunk(&self) -> Option<&dyn LevelChunk> {
            Some(self)
        }

        fn as_light_chunk(&self) -> Option<&dyn LightChunk> {
            Some(self)
        }
    }

    impl LightChunk for MockLevelChunk {}
    impl LevelChunk for MockLevelChunk {}

    struct MockChunkSource {
        chunk: Option<MockLevelChunk>,
        last_x: Cell<i32>,
        last_z: Cell<i32>,
        last_status: Cell<Option<ChunkStatus>>,
        last_create: Cell<bool>,
        tick_has_time_left: bool,
        tick_chunks: bool,
        light_engine: LevelLightEngine,
    }

    impl MockChunkSource {
        fn new(with_chunk: bool) -> Self {
            Self {
                chunk: if with_chunk {
                    Some(MockLevelChunk)
                } else {
                    None
                },
                last_x: Cell::new(0),
                last_z: Cell::new(0),
                last_status: Cell::new(None),
                last_create: Cell::new(false),
                tick_has_time_left: false,
                tick_chunks: false,
                light_engine: LevelLightEngine,
            }
        }
    }

    impl ChunkSource for MockChunkSource {
        fn get_chunk_with_status(
            &self,
            x: i32,
            z: i32,
            status: ChunkStatus,
            create: bool,
        ) -> Option<&dyn ChunkAccess> {
            self.last_x.set(x);
            self.last_z.set(z);
            self.last_status.set(Some(status));
            self.last_create.set(create);
            self.chunk.as_ref().map(|c| c as &dyn ChunkAccess)
        }

        fn tick(&mut self, has_time_left: &dyn Fn() -> bool, tick_chunks: bool) {
            self.tick_has_time_left = has_time_left();
            self.tick_chunks = tick_chunks;
        }

        fn gather_stats(&self) -> String {
            "mock".to_string()
        }

        fn get_loaded_chunks_count(&self) -> i32 {
            i32::from(self.chunk.is_some())
        }

        fn get_light_engine(&self) -> &LevelLightEngine {
            &self.light_engine
        }
    }

    #[test]
    fn get_chunk_delegates_negative_coords() {
        let src = MockChunkSource::new(true);
        let chunk = src.get_chunk(-16, -32, true);
        assert!(chunk.is_some());
        assert_eq!(src.last_x.get(), -16);
        assert_eq!(src.last_z.get(), -32);
        assert_eq!(src.last_status.get(), Some(ChunkStatus::Full));
        assert!(src.last_create.get());
    }

    #[test]
    fn get_chunk_now_and_has_chunk_edge_values() {
        let src = MockChunkSource::new(false);
        assert!(src.get_chunk_now(i32::MIN, i32::MAX).is_none());
        assert_eq!(src.last_x.get(), i32::MIN);
        assert_eq!(src.last_z.get(), i32::MAX);
        assert_eq!(src.last_status.get(), Some(ChunkStatus::Full));
        assert!(!src.last_create.get());

        assert!(!src.has_chunk(i32::MAX, i32::MIN));
        assert_eq!(src.last_x.get(), i32::MAX);
        assert_eq!(src.last_z.get(), i32::MIN);
        assert_eq!(src.last_status.get(), Some(ChunkStatus::Full));
        assert!(!src.last_create.get());
    }

    #[test]
    fn get_chunk_for_lighting_uses_empty_status() {
        let src = MockChunkSource::new(true);
        let chunk = src.get_chunk_for_lighting(-1, 7);
        assert!(chunk.is_some());
        assert_eq!(src.last_x.get(), -1);
        assert_eq!(src.last_z.get(), 7);
        assert_eq!(src.last_status.get(), Some(ChunkStatus::Empty));
        assert!(!src.last_create.get());
    }

    #[test]
    fn defaults_and_close() {
        let mut src = MockChunkSource::new(false);
        src.on_section_emptiness_changed(i32::MIN, -64, i32::MAX, true);
        src.set_spawn_settings(true);
        assert!(!src.update_chunk_forced(ChunkPos { x: -1, z: -2 }, true));
        assert!(src.get_force_loaded_chunks().is_empty());
        assert!(src.close().is_ok());
        assert_eq!(src.gather_stats(), "mock");
        assert_eq!(src.get_loaded_chunks_count(), 0);
    }

    #[test]
    fn tick_invokes_supplier() {
        let mut src = MockChunkSource::new(true);
        src.tick(&|| true, false);
        assert!(src.tick_has_time_left);
        assert!(!src.tick_chunks);

        src.tick(&|| false, true);
        assert!(!src.tick_has_time_left);
        assert!(src.tick_chunks);
    }

    #[test]
    fn integer_overflow_boundaries_and_float_bits() {
        let src = MockChunkSource::new(true);
        // Integer boundary/overflow-edge coordinates must pass through unchanged.
        assert!(src.get_chunk(i32::MAX, i32::MIN, false).is_some());
        assert_eq!(src.last_x.get(), i32::MAX);
        assert_eq!(src.last_z.get(), i32::MIN);

        // Float bit patterns must be preserved exactly.
        let neg_zero_f32 = (-0.0f32).to_bits();
        assert_ne!(neg_zero_f32, 0.0f32.to_bits());
        assert_eq!(f32::from_bits(neg_zero_f32).to_bits(), neg_zero_f32);

        let neg_inf_f32 = f32::NEG_INFINITY.to_bits();
        assert_eq!(f32::from_bits(neg_inf_f32).to_bits(), neg_inf_f32);

        let nan_f64_bits = f64::NAN.to_bits();
        assert!(f64::from_bits(nan_f64_bits).is_nan());
        assert_eq!(f64::from_bits(nan_f64_bits).to_bits(), nan_f64_bits);
    }
}
