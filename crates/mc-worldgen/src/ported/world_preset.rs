//! Port of `net.minecraft.world.level.levelgen.presets.WorldPreset`.
//!
//! This module is self-contained and provides minimal local stand-ins for
//! Minecraft types referenced by `WorldPreset`.

use std::collections::HashMap;

/// Minimal stand-in for `net.minecraft.resources.ResourceKey<LevelStem>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LevelStemKey {
    pub location: String,
}

impl LevelStemKey {
    /// Creates a new level stem key from a namespaced location string.
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
        }
    }

    /// Key for the overworld dimension (`minecraft:overworld`).
    pub fn overworld() -> Self {
        Self::new("minecraft:overworld")
    }

    /// Key for the nether dimension (`minecraft:the_nether`).
    pub fn the_nether() -> Self {
        Self::new("minecraft:the_nether")
    }

    /// Key for the end dimension (`minecraft:the_end`).
    pub fn the_end() -> Self {
        Self::new("minecraft:the_end")
    }
}

/// Minimal stand-in for `net.minecraft.world.level.levelgen.WorldDimensions`.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldDimensions<S> {
    /// Dimensions in deterministic order.
    pub dimensions: Vec<(LevelStemKey, S)>,
}

impl<S> WorldDimensions<S> {
    /// Creates world dimensions from an ordered list of dimension entries.
    pub fn new(dimensions: Vec<(LevelStemKey, S)>) -> Self {
        Self { dimensions }
    }

    /// Mirrors `WorldDimensions.keysInOrder`: canonical dimensions first
    /// (`overworld`, `the_nether`, `the_end`), then remaining non-canonical keys
    /// in the order they appear in the input.
    pub fn keys_in_order<I>(keys: I) -> Vec<LevelStemKey>
    where
        I: IntoIterator<Item = LevelStemKey>,
    {
        let canonical = [
            LevelStemKey::overworld(),
            LevelStemKey::the_nether(),
            LevelStemKey::the_end(),
        ];
        let mut out: Vec<LevelStemKey> = canonical.to_vec();
        for key in keys {
            if !canonical.contains(&key) {
                out.push(key);
            }
        }
        out
    }
}

/// Port of `net.minecraft.world.level.levelgen.presets.WorldPreset`.
///
/// Generic over `S` (stem type) to remain self-contained without depending on an
/// unported `LevelStem` implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldPreset<S> {
    dimensions: HashMap<LevelStemKey, S>,
}

impl<S: Clone> WorldPreset<S> {
    // JAVA: net.minecraft.world.level.levelgen.presets.WorldPreset#<init>
    pub fn new(dimensions: HashMap<LevelStemKey, S>) -> Self {
        Self { dimensions }
    }

    // JAVA: net.minecraft.world.level.levelgen.presets.WorldPreset#dimensionsInOrder
    fn dimensions_in_order(&self) -> Vec<(LevelStemKey, S)> {
        let mut out = Vec::new();
        for key in WorldDimensions::<S>::keys_in_order(self.dimensions.keys().cloned()) {
            if let Some(stem) = self.dimensions.get(&key) {
                out.push((key, stem.clone()));
            }
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.presets.WorldPreset#createWorldDimensions
    pub fn create_world_dimensions(&self) -> WorldDimensions<S> {
        WorldDimensions::new(self.dimensions_in_order())
    }

    // JAVA: net.minecraft.world.level.levelgen.presets.WorldPreset#overworld
    pub fn overworld(&self) -> Option<&S> {
        self.dimensions.get(&LevelStemKey::overworld())
    }

    // JAVA: net.minecraft.world.level.levelgen.presets.WorldPreset#requireOverworld
    pub fn require_overworld(preset: &WorldPreset<S>) -> Result<&WorldPreset<S>, String> {
        if preset.overworld().is_none() {
            Err("Missing overworld dimension".to_string())
        } else {
            Ok(preset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn key(location: &str) -> LevelStemKey {
        LevelStemKey::new(location)
    }

    #[test]
    fn overworld_present_and_missing() {
        let mut dims = HashMap::new();
        dims.insert(LevelStemKey::overworld(), 42_i32);
        let preset = WorldPreset::new(dims);
        assert_eq!(preset.overworld(), Some(&42_i32));

        let empty: WorldPreset<i32> = WorldPreset::new(HashMap::new());
        assert_eq!(empty.overworld(), None);
    }

    #[test]
    fn require_overworld_validation() {
        let mut dims = HashMap::new();
        dims.insert(LevelStemKey::overworld(), "overworld_stem");
        let preset = WorldPreset::new(dims);
        assert!(WorldPreset::require_overworld(&preset).is_ok());

        let no_overworld: WorldPreset<&str> =
            WorldPreset::new(HashMap::from([(LevelStemKey::the_nether(), "nether_stem")]));
        let err = WorldPreset::require_overworld(&no_overworld).unwrap_err();
        assert_eq!(err, "Missing overworld dimension");
    }

    #[test]
    fn dimensions_in_canonical_order_with_custom() {
        let mut dims = HashMap::new();
        dims.insert(key("minecraft:custom"), 3_i32);
        dims.insert(LevelStemKey::the_end(), 2_i32);
        dims.insert(LevelStemKey::overworld(), 0_i32);
        dims.insert(LevelStemKey::the_nether(), 1_i32);

        let preset = WorldPreset::new(dims);
        let wd = preset.create_world_dimensions();
        let ordered_keys: Vec<String> = wd
            .dimensions
            .iter()
            .map(|(k, _)| k.location.clone())
            .collect();
        let ordered_vals: Vec<i32> = wd.dimensions.iter().map(|(_, v)| *v).collect();

        assert_eq!(
            ordered_keys,
            vec![
                "minecraft:overworld".to_string(),
                "minecraft:the_nether".to_string(),
                "minecraft:the_end".to_string(),
                "minecraft:custom".to_string(),
            ]
        );
        assert_eq!(ordered_vals, vec![0_i32, 1_i32, 2_i32, 3_i32]);
    }

    #[test]
    fn skips_missing_canonical_dimensions() {
        let mut dims = HashMap::new();
        dims.insert(LevelStemKey::overworld(), 10_i32);
        dims.insert(LevelStemKey::the_end(), 30_i32);

        let preset = WorldPreset::new(dims);
        let wd = preset.create_world_dimensions();

        assert_eq!(wd.dimensions.len(), 2);
        assert_eq!(wd.dimensions[0], (LevelStemKey::overworld(), 10_i32));
        assert_eq!(wd.dimensions[1], (LevelStemKey::the_end(), 30_i32));
    }

    #[test]
    fn preserves_negative_overflow_and_float_bits() {
        let neg: i32 = -7_i32;
        let overflowed: i32 = i32::MAX.wrapping_add(1); // i32::MIN
        let nan_bits: u64 = 0x7FF8_0000_0000_0001_u64;
        let neg_zero_bits: u64 = 0x8000_0000_0000_0000_u64;
        let nan = f64::from_bits(nan_bits);
        let neg_zero = f64::from_bits(neg_zero_bits);

        let mut dims = HashMap::new();
        dims.insert(
            LevelStemKey::overworld(),
            (neg, overflowed, nan.to_bits(), neg_zero.to_bits()),
        );

        let preset = WorldPreset::new(dims);
        let stem = preset.overworld().expect("overworld should exist");

        assert_eq!(stem.0, -7_i32);
        assert_eq!(stem.1, i32::MIN);
        assert_eq!(stem.2, nan_bits);
        assert_eq!(stem.3, neg_zero_bits);
        assert!(f64::from_bits(stem.2).is_nan());
        assert!(f64::from_bits(stem.3).is_sign_negative());
        assert_eq!(f64::from_bits(stem.3), 0.0_f64);
    }

    #[test]
    fn empty_preset_has_no_dimensions() {
        let preset: WorldPreset<i32> = WorldPreset::new(HashMap::new());
        let wd = preset.create_world_dimensions();
        assert!(wd.dimensions.is_empty());
        assert!(preset.overworld().is_none());
        assert_eq!(
            WorldPreset::require_overworld(&preset).unwrap_err(),
            "Missing overworld dimension"
        );
    }
}
