use crate::packing::BlockPos;

/// Minimal self-contained placeholder for `net.minecraft.core.Holder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

/// Minimal self-contained placeholder for `net.minecraft.world.entity.ai.village.poi.PoiType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiType {
    pub id: String,
}

/// Minimal self-contained placeholder for `net.minecraft.world.entity.ai.village.poi.PoiRecord`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiRecord {
    pos: BlockPos,
    poi_type: Holder<PoiType>,
    free_tickets: i32,
}

impl PoiRecord {
    pub fn new(pos: BlockPos, poi_type: Holder<PoiType>, free_tickets: i32) -> Self {
        Self {
            pos,
            poi_type,
            free_tickets,
        }
    }

    pub fn get_pos(&self) -> &BlockPos {
        &self.pos
    }

    pub fn get_poi_type(&self) -> &Holder<PoiType> {
        &self.poi_type
    }

    pub fn get_free_tickets(&self) -> i32 {
        self.free_tickets
    }
}

/// Faithful port of `net.minecraft.util.debug.DebugPoiInfo`.
/// STREAM_CODEC is omitted because the Minecraft networking codec framework is not ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugPoiInfo {
    pub pos: BlockPos,
    pub poi_type: Holder<PoiType>,
    pub free_ticket_count: i32,
}

impl DebugPoiInfo {
    // JAVA: net.minecraft.util.debug.DebugPoiInfo#<init>
    pub fn new(pos: BlockPos, poi_type: Holder<PoiType>, free_ticket_count: i32) -> Self {
        Self {
            pos,
            poi_type,
            free_ticket_count,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugPoiInfo#<init>
    pub fn from_poi_record(record: &PoiRecord) -> Self {
        Self {
            pos: record.get_pos().clone(),
            poi_type: record.get_poi_type().clone(),
            free_ticket_count: record.get_free_tickets(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packing::BlockPos;

    fn bp(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::of(BlockPos::as_long_coords(x, y, z))
    }

    fn poi(id: &str) -> Holder<PoiType> {
        Holder::new(PoiType { id: id.to_string() })
    }

    #[test]
    fn new_stores_fields() {
        let pos = bp(1, 2, 3);
        let info = DebugPoiInfo::new(pos.clone(), poi("minecraft:home"), 5);
        assert_eq!(info.pos, pos);
        assert_eq!(info.poi_type.value.id, "minecraft:home");
        assert_eq!(info.free_ticket_count, 5);
    }

    #[test]
    fn from_poi_record_copies_fields() {
        let pos = bp(-10, 64, 20);
        let record = PoiRecord::new(pos.clone(), poi("minecraft:meeting"), 3);
        let info = DebugPoiInfo::from_poi_record(&record);
        assert_eq!(info.pos, pos);
        assert_eq!(info.poi_type, poi("minecraft:meeting"));
        assert_eq!(info.free_ticket_count, 3);
    }

    #[test]
    fn from_poi_record_preserves_negative_and_zero_ticket_counts() {
        let record_zero = PoiRecord::new(bp(0, 0, 0), poi("a"), 0);
        let record_neg = PoiRecord::new(bp(0, 0, 0), poi("b"), -1);
        assert_eq!(DebugPoiInfo::from_poi_record(&record_zero).free_ticket_count, 0);
        assert_eq!(DebugPoiInfo::from_poi_record(&record_neg).free_ticket_count, -1);
    }

    #[test]
    fn holder_wraps_poi_type() {
        let holder = Holder::new(PoiType { id: "test".to_string() });
        assert_eq!(holder.value.id, "test");
        let cloned = holder.clone();
        assert_eq!(holder, cloned);
    }
}
