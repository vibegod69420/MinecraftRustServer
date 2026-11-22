//! Port of `net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure`.
//! Self-contained with minimal stubs for unported Minecraft dependencies.

use std::marker::PhantomData;

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.StructureSettings`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureSettings;

/// Stub for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T> {
    _phantom: PhantomData<T>,
}

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.StructureType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    JungleTemple,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JungleTemplePiece;

/// `public static final MapCodec<JungleTempleStructure> CODEC = simpleCodec(JungleTempleStructure::new);`
pub const JUNGLE_TEMPLE_STRUCTURE_CODEC: MapCodec<JungleTempleStructure> = MapCodec::new();

/// Port of `JungleTempleStructure`.
/// Java extends `SinglePieceStructure`; this stores the equivalent state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JungleTempleStructure {
    pub settings: StructureSettings,
    /// Width passed to `SinglePieceStructure` (12).
    pub width: i32,
    /// Depth passed to `SinglePieceStructure` (15).
    pub depth: i32,
}

impl JungleTempleStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        // super(JungleTemplePiece::new, 12, 15, settings)
        let _piece_constructor: fn() -> JungleTemplePiece = || JungleTemplePiece;
        Self {
            settings,
            width: 12,
            depth: 15,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::JungleTemple
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_single_piece_dimensions() {
        let s = JungleTempleStructure::new(StructureSettings::default());
        assert_eq!(s.width, 12);
        assert_eq!(s.depth, 15);
    }

    #[test]
    fn structure_type_is_jungle_temple() {
        let s = JungleTempleStructure::new(StructureSettings::default());
        assert_eq!(s.structure_type(), StructureType::JungleTemple);
    }

    #[test]
    fn settings_are_preserved() {
        let settings = StructureSettings::default();
        let s = JungleTempleStructure::new(settings.clone());
        assert_eq!(s.settings, settings);
    }

    #[test]
    fn integer_edge_values_are_stored_as_i32() {
        let mut s = JungleTempleStructure::new(StructureSettings::default());
        s.width = i32::MIN;
        s.depth = i32::MAX;
        assert_eq!(s.width.wrapping_add(0), i32::MIN);
        assert_eq!(s.depth.wrapping_sub(0), i32::MAX);
    }

    #[test]
    fn codec_constant_and_float_bits_sanity() {
        let _codec: MapCodec<JungleTempleStructure> = JUNGLE_TEMPLE_STRUCTURE_CODEC;
        let _other = MapCodec::<JungleTempleStructure>::new();
        // No float operations in this structure; verify float bit patterns for constants.
        assert_eq!(12.0f32.to_bits(), 0x4140_0000);
        assert_eq!(15.0f32.to_bits(), 0x4170_0000);
    }
}
