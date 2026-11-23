#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionStorageInfo {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionFileVersion {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkRegionIoEvent {
    pub storage_info: RegionStorageInfo,
    pub chunk_pos: ChunkPos,
    pub version: RegionFileVersion,
    pub bytes: i32,
}

impl ChunkRegionIoEvent {
    // JAVA: net.minecraft.util.profiling.jfr.event.ChunkRegionIoEvent#<init>
    pub fn new(
        storage_info: RegionStorageInfo,
        chunk_pos: ChunkPos,
        version: RegionFileVersion,
        bytes: i32,
    ) -> Self {
        Self {
            storage_info,
            chunk_pos,
            version,
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkRegionWriteEvent {
    pub base: ChunkRegionIoEvent,
}

impl ChunkRegionWriteEvent {
    pub const EVENT_NAME: &'static str = "minecraft.ChunkRegionWrite";

    // JAVA: net.minecraft.util.profiling.jfr.event.ChunkRegionWriteEvent#<init>
    pub fn new(
        storage_info: RegionStorageInfo,
        chunk_pos: ChunkPos,
        version: RegionFileVersion,
        bytes: i32,
    ) -> Self {
        Self {
            base: ChunkRegionIoEvent::new(storage_info, chunk_pos, version, bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(bytes: i32) -> ChunkRegionWriteEvent {
        ChunkRegionWriteEvent::new(
            RegionStorageInfo {
                name: "world".to_string(),
            },
            ChunkPos { x: 1, z: -2 },
            RegionFileVersion {
                name: "anvil".to_string(),
            },
            bytes,
        )
    }

    #[test]
    fn constructor_stores_fields() {
        let ev = sample_event(4096);
        assert_eq!(ev.base.storage_info.name, "world");
        assert_eq!(ev.base.chunk_pos, ChunkPos { x: 1, z: -2 });
        assert_eq!(ev.base.version.name, "anvil");
        assert_eq!(ev.base.bytes, 4096);
    }

    #[test]
    fn event_name_constant() {
        assert_eq!(
            ChunkRegionWriteEvent::EVENT_NAME,
            "minecraft.ChunkRegionWrite"
        );
    }

    #[test]
    fn bytes_edge_cases() {
        assert_eq!(sample_event(-1).base.bytes, -1);
        assert_eq!(sample_event(0).base.bytes, 0);
        assert_eq!(sample_event(i32::MAX).base.bytes, i32::MAX);
        assert_eq!(sample_event(i32::MIN).base.bytes, i32::MIN);
    }

    #[test]
    fn chunk_pos_negative_coordinates() {
        let ev = ChunkRegionWriteEvent::new(
            RegionStorageInfo {
                name: "s".to_string(),
            },
            ChunkPos {
                x: i32::MIN,
                z: i32::MAX,
            },
            RegionFileVersion {
                name: "v".to_string(),
            },
            0,
        );
        assert_eq!(ev.base.chunk_pos.x, i32::MIN);
        assert_eq!(ev.base.chunk_pos.z, i32::MAX);
    }
}
