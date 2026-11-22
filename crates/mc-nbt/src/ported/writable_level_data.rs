/// Minimal self-contained stub for `net.minecraft.world.level.storage.LevelData.RespawnData`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RespawnData;

/// Minimal self-contained stub for `net.minecraft.world.level.storage.LevelData`.
pub trait LevelData {}

/// Port of `net.minecraft.world.level.storage.WritableLevelData`.
pub trait WritableLevelData: LevelData {
    // JAVA: net.minecraft.world.level.storage.WritableLevelData#setSpawn
    fn set_spawn(&mut self, respawn_data: RespawnData);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct DummyLevelData {
        spawn: Option<RespawnData>,
        calls: i32,
    }

    impl LevelData for DummyLevelData {}

    impl WritableLevelData for DummyLevelData {
        // JAVA: net.minecraft.world.level.storage.WritableLevelData#setSpawn
        fn set_spawn(&mut self, respawn_data: RespawnData) {
            self.calls = self.calls.wrapping_add(1);
            self.spawn = Some(respawn_data);
        }
    }

    #[test]
    fn set_spawn_stores_respawn_data() {
        let mut data = DummyLevelData::default();
        data.set_spawn(RespawnData);
        assert_eq!(data.spawn, Some(RespawnData));
        assert_eq!(data.calls, 1);
    }

    #[test]
    fn set_spawn_multiple_calls_and_negative_counter() {
        let mut data = DummyLevelData {
            spawn: None,
            calls: -1,
        };
        data.set_spawn(RespawnData);
        data.set_spawn(RespawnData);
        assert_eq!(data.calls, 1);
        assert_eq!(data.spawn, Some(RespawnData));
    }

    #[test]
    fn set_spawn_wrapping_overflow_edge_case() {
        let mut data = DummyLevelData {
            spawn: None,
            calls: i32::MAX,
        };
        data.set_spawn(RespawnData);
        assert_eq!(data.calls, i32::MIN);
    }

    #[test]
    fn float_bits_edge_case_sanity() {
        let neg_zero = (-0.0f32).to_bits();
        let pos_zero = 0.0f32.to_bits();
        let nan_bits = f32::NAN.to_bits();
        assert_ne!(neg_zero, pos_zero);
        assert_ne!(nan_bits, pos_zero);
    }
}
