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

pub trait TimedStat {
    fn duration(&self) -> Duration;
}

/// Minimal self-contained view over a JFR recorded event.
pub trait RecordedEventView {
    fn get_duration(&self) -> Duration;
    fn get_int(&self, name: &str) -> i32;
    fn get_string(&self, name: &str) -> String;
    fn get_bool(&self, name: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureGenStat {
    pub duration: Duration,
    pub chunk_pos: ChunkPos,
    pub structure_name: String,
    pub level: String,
    pub success: bool,
}

impl TimedStat for StructureGenStat {
    fn duration(&self) -> Duration {
        self.duration
    }
}

impl StructureGenStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.StructureGenStat#from
    pub fn from(event: &impl RecordedEventView) -> Self {
        Self {
            duration: event.get_duration(),
            // Faithful to Java source: both arguments read chunkPosX.
            chunk_pos: ChunkPos::new(event.get_int("chunkPosX"), event.get_int("chunkPosX")),
            structure_name: event.get_string("structure"),
            level: event.get_string("level"),
            success: event.get_bool("success"),
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
        bools: HashMap<&'static str, bool>,
    }

    impl MockEvent {
        fn new(duration: Duration) -> Self {
            Self {
                duration,
                ints: HashMap::new(),
                strings: HashMap::new(),
                bools: HashMap::new(),
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
        fn get_bool(&self, name: &str) -> bool {
            *self.bools.get(name).unwrap_or(&false)
        }
    }

    #[test]
    fn from_reads_fields() {
        let mut ev = MockEvent::new(Duration::from_millis(42));
        ev.ints.insert("chunkPosX", 7);
        ev.strings.insert("structure", "minecraft:village".to_string());
        ev.strings.insert("level", "overworld".to_string());
        ev.bools.insert("success", true);

        let stat = StructureGenStat::from(&ev);
        assert_eq!(stat.duration, Duration::from_millis(42));
        assert_eq!(stat.chunk_pos, ChunkPos::new(7, 7));
        assert_eq!(stat.structure_name, "minecraft:village");
        assert_eq!(stat.level, "overworld");
        assert!(stat.success);
        assert_eq!(TimedStat::duration(&stat), Duration::from_millis(42));
    }

    #[test]
    fn from_preserves_chunk_pos_x_bug() {
        let mut ev = MockEvent::new(Duration::ZERO);
        ev.ints.insert("chunkPosX", -5);
        ev.ints.insert("chunkPosZ", 99); // should be ignored by faithful port
        let stat = StructureGenStat::from(&ev);
        assert_eq!(stat.chunk_pos.x, -5);
        assert_eq!(stat.chunk_pos.z, -5);
    }

    #[test]
    fn from_handles_negative_and_missing_values() {
        let mut ev = MockEvent::new(Duration::from_nanos(u64::MAX));
        ev.ints.insert("chunkPosX", i32::MIN);
        ev.bools.insert("success", false);
        let stat = StructureGenStat::from(&ev);
        assert_eq!(stat.chunk_pos, ChunkPos::new(i32::MIN, i32::MIN));
        assert_eq!(stat.structure_name, "");
        assert_eq!(stat.level, "");
        assert!(!stat.success);
    }
}
