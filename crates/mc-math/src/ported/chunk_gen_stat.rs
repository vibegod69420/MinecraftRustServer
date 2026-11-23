use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnPos {
    pub x: i32,
    pub z: i32,
}

impl ColumnPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStatus {
    pub name: String,
}

impl ChunkStatus {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#byName
    pub fn by_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

pub trait TimedStat {
    fn duration(&self) -> Duration;
}

/// Minimal self-contained view over a JFR recorded event.
pub trait RecordedEventView {
    fn get_duration(&self) -> Duration;
    fn get_int(&self, name: &str) -> i32;
    fn get_string(&self, name: &str) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGenStat {
    pub duration: Duration,
    pub chunk_pos: ChunkPos,
    pub world_pos: ColumnPos,
    pub status: ChunkStatus,
    pub level: String,
}

impl TimedStat for ChunkGenStat {
    fn duration(&self) -> Duration {
        self.duration
    }
}

impl ChunkGenStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.ChunkGenStat#from
    pub fn from(event: &impl RecordedEventView) -> Self {
        Self {
            duration: event.get_duration(),
            // Faithful to Java source: both arguments read chunkPosX.
            chunk_pos: ChunkPos::new(event.get_int("chunkPosX"), event.get_int("chunkPosX")),
            world_pos: ColumnPos::new(event.get_int("worldPosX"), event.get_int("worldPosZ")),
            status: ChunkStatus::by_name(&event.get_string("status")),
            level: event.get_string("level"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockEvent {
        duration: Duration,
        ints: HashMap<&'static str, i32>,
        strings: HashMap<&'static str, String>,
    }

    impl MockEvent {
        fn new(duration: Duration) -> Self {
            Self {
                duration,
                ints: HashMap::new(),
                strings: HashMap::new(),
            }
        }
    }

    impl RecordedEventView for MockEvent {
        fn get_duration(&self) -> Duration {
            self.duration
        }
        fn get_int(&self, name: &str) -> i32 {
            *self.ints.get(name).unwrap_or(&0)
        }
        fn get_string(&self, name: &str) -> String {
            self.strings.get(name).cloned().unwrap_or_default()
        }
    }

    #[test]
    fn from_reads_fields() {
        let mut ev = MockEvent::new(Duration::from_secs(2));
        ev.ints.insert("chunkPosX", 3);
        ev.ints.insert("worldPosX", 48);
        ev.ints.insert("worldPosZ", -16);
        ev.strings.insert("status", "minecraft:full".to_string());
        ev.strings.insert("level", "overworld".to_string());

        let stat = ChunkGenStat::from(&ev);
        assert_eq!(stat.duration, Duration::from_secs(2));
        assert_eq!(stat.chunk_pos, ChunkPos::new(3, 3));
        assert_eq!(stat.world_pos, ColumnPos::new(48, -16));
        assert_eq!(stat.status, ChunkStatus::by_name("minecraft:full"));
        assert_eq!(stat.level, "overworld");
    }

    #[test]
    fn from_preserves_chunk_pos_x_bug() {
        let mut ev = MockEvent::new(Duration::ZERO);
        ev.ints.insert("chunkPosX", -123);
        ev.ints.insert("chunkPosZ", 456);
        let stat = ChunkGenStat::from(&ev);
        assert_eq!(stat.chunk_pos.x, -123);
        assert_eq!(stat.chunk_pos.z, -123);
    }

    #[test]
    fn from_handles_negative_and_extreme_ints() {
        let mut ev = MockEvent::new(Duration::from_millis(1));
        ev.ints.insert("chunkPosX", i32::MAX);
        ev.ints.insert("worldPosX", i32::MIN);
        ev.ints.insert("worldPosZ", i32::MAX);
        ev.strings.insert("status", "".to_string());
        let stat = ChunkGenStat::from(&ev);
        assert_eq!(stat.chunk_pos, ChunkPos::new(i32::MAX, i32::MAX));
        assert_eq!(stat.world_pos, ColumnPos::new(i32::MIN, i32::MAX));
        assert_eq!(stat.status.name, "");
    }
}
