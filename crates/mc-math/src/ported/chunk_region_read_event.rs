/// Faithful port of `net.minecraft.util.profiling.jfr.event.ChunkRegionReadEvent`.
///
/// Base `ChunkRegionIoEvent` fields and referenced Minecraft types are
/// represented with minimal self-contained structs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType {
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionStorageInfo {
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionFileVersion {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkRegionReadEvent {
    pub region_storage_info: RegionStorageInfo,
    pub chunk_pos: ChunkPos,
    pub region_file_version: RegionFileVersion,
    pub bytes: i32,
}

impl ChunkRegionReadEvent {
    pub const EVENT_NAME: &'static str = "minecraft.ChunkRegionRead";
    pub const TYPE: EventType = EventType {
        name: Self::EVENT_NAME,
    };

    // JAVA: net.minecraft.util.profiling.jfr.event.ChunkRegionReadEvent#<init>
    pub fn new(
        region_storage_info: RegionStorageInfo,
        chunk_pos: ChunkPos,
        region_file_version: RegionFileVersion,
        bytes: i32,
    ) -> Self {
        Self {
            region_storage_info,
            chunk_pos,
            region_file_version,
            bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(bytes: i32) -> ChunkRegionReadEvent {
        ChunkRegionReadEvent::new(
            RegionStorageInfo {
                key: "region".to_string(),
            },
            ChunkPos::new(1, -2),
            RegionFileVersion {
                name: "anvil".to_string(),
            },
            bytes,
        )
    }

    #[test]
    fn event_name_and_type_match() {
        assert_eq!(ChunkRegionReadEvent::EVENT_NAME, "minecraft.ChunkRegionRead");
        assert_eq!(ChunkRegionReadEvent::TYPE.name, ChunkRegionReadEvent::EVENT_NAME);
    }

    #[test]
    fn constructor_stores_fields() {
        let event = sample_event(4096);
        assert_eq!(event.region_storage_info, RegionStorageInfo { key: "region".to_string() });
        assert_eq!(event.chunk_pos, ChunkPos { x: 1, z: -2 });
        assert_eq!(event.region_file_version, RegionFileVersion { name: "anvil".to_string() });
        assert_eq!(event.bytes, 4096);
    }

    #[test]
    fn supports_negative_chunk_coords_and_extreme_bytes() {
        let event = ChunkRegionReadEvent::new(
            RegionStorageInfo { key: String::new() },
            ChunkPos::new(i32::MIN, i32::MAX),
            RegionFileVersion { name: String::new() },
            i32::MIN,
        );
        assert_eq!(event.chunk_pos.x, i32::MIN);
        assert_eq!(event.chunk_pos.z, i32::MAX);
        assert_eq!(event.bytes, i32::MIN);

        let max = sample_event(i32::MAX);
        assert_eq!(max.bytes, i32::MAX);
    }

    #[test]
    fn clone_and_equality_preserve_data() {
        let event = sample_event(-1);
        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert_eq!(cloned.bytes, -1);
    }
}
