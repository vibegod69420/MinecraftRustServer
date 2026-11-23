use std::sync::OnceLock;

/// Minimal self-contained block state representation for carver debug settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub name: String,
}

impl BlockState {
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.carver.CarverDebugSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarverDebugSettings {
    debug_mode: bool,
    air_state: BlockState,
    water_state: BlockState,
    lava_state: BlockState,
    barrier_state: BlockState,
}

fn default_settings_cell() -> &'static OnceLock<CarverDebugSettings> {
    static DEFAULT: OnceLock<CarverDebugSettings> = OnceLock::new();
    &DEFAULT
}

impl CarverDebugSettings {
    /// Returns the Java `DEFAULT` debug settings.
    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#DEFAULT
    pub fn default_settings() -> &'static Self {
        default_settings_cell().get_or_init(|| Self {
            debug_mode: false,
            air_state: BlockState::named("minecraft:acacia_button"),
            water_state: BlockState::named("minecraft:candle"),
            lava_state: BlockState::named("minecraft:orange_stained_glass"),
            barrier_state: BlockState::named("minecraft:glass"),
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#of(boolean, BlockState, BlockState, BlockState, BlockState)
    pub fn of(
        debug_mode: bool,
        air_state: BlockState,
        water_state: BlockState,
        lava_state: BlockState,
        barrier_state: BlockState,
    ) -> Self {
        Self {
            debug_mode,
            air_state,
            water_state,
            lava_state,
            barrier_state,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#of(BlockState, BlockState, BlockState, BlockState)
    pub fn of_states(
        air_state: BlockState,
        water_state: BlockState,
        lava_state: BlockState,
        barrier_state: BlockState,
    ) -> Self {
        Self::of(false, air_state, water_state, lava_state, barrier_state)
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#of(boolean, BlockState)
    pub fn of_debug_air(debug_mode: bool, air_state: BlockState) -> Self {
        let default = Self::default_settings();
        Self::of(
            debug_mode,
            air_state,
            default.water_state.clone(),
            default.lava_state.clone(),
            default.barrier_state.clone(),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#isDebugMode()
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#getAirState()
    pub fn get_air_state(&self) -> &BlockState {
        &self.air_state
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#getWaterState()
    pub fn get_water_state(&self) -> &BlockState {
        &self.water_state
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#getLavaState()
    pub fn get_lava_state(&self) -> &BlockState {
        &self.lava_state
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#getBarrierState()
    pub fn get_barrier_state(&self) -> &BlockState {
        &self.barrier_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(name: &str) -> BlockState {
        BlockState {
            name: name.to_string(),
        }
    }

    #[test]
    fn default_settings_match_java() {
        let d = CarverDebugSettings::default_settings();
        assert!(!d.is_debug_mode());
        assert_eq!(d.get_air_state().name, "minecraft:acacia_button");
        assert_eq!(d.get_water_state().name, "minecraft:candle");
        assert_eq!(d.get_lava_state().name, "minecraft:orange_stained_glass");
        assert_eq!(d.get_barrier_state().name, "minecraft:glass");
    }

    #[test]
    fn of_sets_all_fields() {
        let s = CarverDebugSettings::of(true, bs("air"), bs("water"), bs("lava"), bs("barrier"));
        assert!(s.is_debug_mode());
        assert_eq!(s.get_air_state().name, "air");
        assert_eq!(s.get_water_state().name, "water");
        assert_eq!(s.get_lava_state().name, "lava");
        assert_eq!(s.get_barrier_state().name, "barrier");
    }

    #[test]
    fn of_states_defaults_debug_false() {
        let s = CarverDebugSettings::of_states(bs("air"), bs("water"), bs("lava"), bs("barrier"));
        assert!(!s.is_debug_mode());
        assert_eq!(s.get_air_state().name, "air");
        assert_eq!(s.get_barrier_state().name, "barrier");
    }

    #[test]
    fn of_debug_air_uses_default_non_air_states() {
        let s = CarverDebugSettings::of_debug_air(true, bs("custom_air"));
        let d = CarverDebugSettings::default_settings();
        assert!(s.is_debug_mode());
        assert_eq!(s.get_air_state().name, "custom_air");
        assert_eq!(s.get_water_state(), d.get_water_state());
        assert_eq!(s.get_lava_state(), d.get_lava_state());
        assert_eq!(s.get_barrier_state(), d.get_barrier_state());
    }
}
