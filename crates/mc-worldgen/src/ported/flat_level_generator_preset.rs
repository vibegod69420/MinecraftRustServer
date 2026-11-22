//! Port of `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset`.
//!
//! Java source: net/minecraft/world/level/levelgen/flat/FlatLevelGeneratorPreset.java
//! Record: `FlatLevelGeneratorPreset(Holder<Item> displayItem, FlatLevelGeneratorSettings settings)`
//!
//! Static codec fields (`DIRECT_CODEC`, `CODEC`) from the Java class depend on
//! Mojang serialization/registry infrastructure and are intentionally omitted in
//! this self-contained port.

use std::fmt::Debug;

/// Minimal self-contained placeholder for `net.minecraft.world.item.Item`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: String,
}

impl Item {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Minimal self-contained placeholder for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T> {
    value: T,
}

impl<T> Holder<T> {
    pub fn direct(value: T) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Minimal self-contained placeholder for
/// `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLevelGeneratorSettings {
    pub raw_id: i32,
}

impl FlatLevelGeneratorSettings {
    pub fn new(raw_id: i32) -> Self {
        Self { raw_id }
    }
}

/// Port of `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset`.
///
/// Java record components:
/// - `displayItem: Holder<Item>`
/// - `settings: FlatLevelGeneratorSettings`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLevelGeneratorPreset {
    display_item: Holder<Item>,
    settings: FlatLevelGeneratorSettings,
}

impl FlatLevelGeneratorPreset {
    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset#FlatLevelGeneratorPreset
    pub fn new(display_item: Holder<Item>, settings: FlatLevelGeneratorSettings) -> Self {
        Self {
            display_item,
            settings,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset#displayItem
    pub fn display_item(&self) -> &Holder<Item> {
        &self.display_item
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset#settings
    pub fn settings(&self) -> &FlatLevelGeneratorSettings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_preset(display_id: &str, raw_id: i32) -> FlatLevelGeneratorPreset {
        FlatLevelGeneratorPreset::new(
            Holder::direct(Item::new(display_id)),
            FlatLevelGeneratorSettings::new(raw_id),
        )
    }

    #[test]
    fn new_and_accessors_round_trip() {
        let preset = sample_preset("minecraft:grass_block", 42);
        assert_eq!(preset.display_item().value().id, "minecraft:grass_block");
        assert_eq!(preset.settings().raw_id, 42);
    }

    #[test]
    fn stores_negative_and_boundary_integer_values() {
        let cases = [i32::MIN, -1, 0, 1, i32::MAX];
        for raw_id in cases {
            let preset = sample_preset("minecraft:stone", raw_id);
            assert_eq!(preset.settings().raw_id, raw_id);
        }
    }

    #[test]
    fn stores_float_bit_patterns_in_integer_field() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let preset_nan = sample_preset("minecraft:debug", nan_bits);
        assert_eq!(preset_nan.settings().raw_id, nan_bits);
        assert_eq!(preset_nan.settings().raw_id as u32, f32::NAN.to_bits());

        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let preset_neg_zero = sample_preset("minecraft:debug", neg_zero_bits);
        assert_eq!(preset_neg_zero.settings().raw_id as u32, (-0.0f32).to_bits());
        assert_ne!(
            (-0.0f32).to_bits(),
            0.0f32.to_bits(),
            "negative zero must have distinct float bits"
        );
    }

    #[test]
    fn equality_matches_record_components() {
        let a = sample_preset("minecraft:grass_block", 7);
        let b = sample_preset("minecraft:grass_block", 7);
        let c = sample_preset("minecraft:grass_block", 8);
        let d = sample_preset("minecraft:dirt", 7);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn clone_is_independent_and_preserves_values() {
        let original = sample_preset("minecraft:stone", i32::MIN);
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.display_item().value().id, "minecraft:stone");
        assert_eq!(cloned.settings().raw_id, i32::MIN);
    }
}
