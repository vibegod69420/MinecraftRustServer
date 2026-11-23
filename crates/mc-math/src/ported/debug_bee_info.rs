use crate::packing::BlockPos;

/// Faithful port of `net.minecraft.util.debug.DebugBeeInfo`.
/// STREAM_CODEC is omitted because the Minecraft networking codec framework is not ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugBeeInfo {
    pub hive_pos: Option<BlockPos>,
    pub flower_pos: Option<BlockPos>,
    pub travel_ticks: i32,
    pub blacklisted_hives: Vec<BlockPos>,
}

impl DebugBeeInfo {
    // JAVA: net.minecraft.util.debug.DebugBeeInfo#<init>
    pub fn new(
        hive_pos: Option<BlockPos>,
        flower_pos: Option<BlockPos>,
        travel_ticks: i32,
        blacklisted_hives: Vec<BlockPos>,
    ) -> Self {
        Self {
            hive_pos,
            flower_pos,
            travel_ticks,
            blacklisted_hives,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugBeeInfo#hasHive
    pub fn has_hive(&self, pos: &BlockPos) -> bool {
        self.hive_pos.as_ref().is_some_and(|p| p == pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packing::BlockPos;

    fn bp(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::of(BlockPos::as_long_coords(x, y, z))
    }

    #[test]
    fn has_hive_true_when_present_and_equal() {
        let hive = bp(1, 64, -3);
        let info = DebugBeeInfo::new(Some(hive.clone()), None, 10, vec![]);
        assert!(info.has_hive(&hive));
    }

    #[test]
    fn has_hive_false_when_absent() {
        let info = DebugBeeInfo::new(None, Some(bp(0, 0, 0)), 0, vec![]);
        assert!(!info.has_hive(&bp(0, 0, 0)));
    }

    #[test]
    fn has_hive_false_when_different_position() {
        let info = DebugBeeInfo::new(Some(bp(1, 2, 3)), None, 5, vec![]);
        assert!(!info.has_hive(&bp(1, 2, 4)));
    }

    #[test]
    fn stores_fields_and_blacklisted_hives() {
        let hive = bp(10, 70, 10);
        let flower = bp(11, 70, 12);
        let blacklisted = vec![bp(1, 2, 3), bp(-4, -5, -6)];
        let info = DebugBeeInfo::new(Some(hive.clone()), Some(flower.clone()), 42, blacklisted.clone());
        assert_eq!(info.hive_pos, Some(hive));
        assert_eq!(info.flower_pos, Some(flower));
        assert_eq!(info.travel_ticks, 42);
        assert_eq!(info.blacklisted_hives, blacklisted);
    }

    #[test]
    fn travel_ticks_negative_edge_case() {
        let info = DebugBeeInfo::new(None, None, -1, vec![]);
        assert_eq!(info.travel_ticks, -1);
        assert!(!info.has_hive(&bp(0, 0, 0)));
    }
}
