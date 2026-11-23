/// Minimal view over a JFR `RecordedEvent` for `ChunkIdentification` extraction.
pub trait ChunkIdentificationEvent {
    fn get_string(&self, name: &str) -> String;
    fn get_int(&self, name: &str) -> i32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkIdentification {
    pub level: String,
    pub dimension: String,
    pub x: i32,
    pub z: i32,
}

impl ChunkIdentification {
    // JAVA: net.minecraft.util.profiling.jfr.stats.ChunkIdentification#from
    pub fn from<E: ChunkIdentificationEvent + ?Sized>(event: &E) -> Self {
        Self {
            level: event.get_string("level"),
            dimension: event.get_string("dimension"),
            x: event.get_int("chunkPosX"),
            z: event.get_int("chunkPosZ"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvent {
        level: String,
        dimension: String,
        x: i32,
        z: i32,
    }

    impl ChunkIdentificationEvent for MockEvent {
        fn get_string(&self, name: &str) -> String {
            match name {
                "level" => self.level.clone(),
                "dimension" => self.dimension.clone(),
                _ => panic!("unexpected field: {name}"),
            }
        }

        fn get_int(&self, name: &str) -> i32 {
            match name {
                "chunkPosX" => self.x,
                "chunkPosZ" => self.z,
                _ => panic!("unexpected field: {name}"),
            }
        }
    }

    #[test]
    fn from_basic_values() {
        let stat = ChunkIdentification::from(&MockEvent {
            level: "ServerLevel".into(),
            dimension: "minecraft:overworld".into(),
            x: 10,
            z: -20,
        });
        assert_eq!(stat.level, "ServerLevel");
        assert_eq!(stat.dimension, "minecraft:overworld");
        assert_eq!(stat.x, 10);
        assert_eq!(stat.z, -20);
    }

    #[test]
    fn from_negative_coords() {
        let stat = ChunkIdentification::from(&MockEvent {
            level: String::new(),
            dimension: String::new(),
            x: -1,
            z: -100_000,
        });
        assert_eq!(stat.x, -1);
        assert_eq!(stat.z, -100_000);
    }

    #[test]
    fn from_extreme_int_values() {
        let stat = ChunkIdentification::from(&MockEvent {
            level: "l".into(),
            dimension: "d".into(),
            x: i32::MIN,
            z: i32::MAX,
        });
        assert_eq!(stat.x, i32::MIN);
        assert_eq!(stat.z, i32::MAX);
    }

    #[test]
    fn from_zero_coords() {
        let stat = ChunkIdentification::from(&MockEvent {
            level: "l".into(),
            dimension: "d".into(),
            x: 0,
            z: 0,
        });
        assert_eq!(stat.x, 0);
        assert_eq!(stat.z, 0);
    }
}
