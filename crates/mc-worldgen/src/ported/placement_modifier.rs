use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlacementContext;

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementModifierType {
    pub id: u32,
}

/// Placeholder for Mojang Codec; serialization framework not ported.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(pub PhantomData<T>);

impl<T> Codec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

pub trait PlacementModifier {
    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementModifier#getPositions
    fn get_positions(
        &self,
        ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos>;

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementModifier#type
    fn modifier_type(&self) -> PlacementModifierType;
}

/// Placeholder for the static CODEC field on PlacementModifier.
/// JAVA: net.minecraft.world.level.levelgen.placement.PlacementModifier#CODEC
pub const PLACEMENT_MODIFIER_CODEC: Codec<()> = Codec::placeholder();

#[cfg(test)]
mod tests {
    use super::*;

    struct IdentityModifier;
    struct MockRandom;
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    impl PlacementModifier for IdentityModifier {
        fn get_positions(
            &self,
            _ctx: &PlacementContext,
            _random: &mut dyn RandomSource,
            pos: BlockPos,
        ) -> Vec<BlockPos> {
            vec![pos]
        }

        fn modifier_type(&self) -> PlacementModifierType {
            PlacementModifierType { id: 42 }
        }
    }

    #[test]
    fn trait_get_positions_and_type() {
        let m = IdentityModifier;
        let mut rng = MockRandom;
        let pos = BlockPos::new(1, -2, 3);
        let out = m.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out, vec![pos]);
        assert_eq!(m.modifier_type(), PlacementModifierType { id: 42 });
    }

    #[test]
    fn codec_placeholder_exists() {
        let _c: Codec<()> = PLACEMENT_MODIFIER_CODEC;
    }

    #[test]
    fn block_pos_extreme_values() {
        let pos = BlockPos::new(i32::MIN, 0, i32::MAX);
        assert_eq!(pos.x, i32::MIN);
        assert_eq!(pos.z, i32::MAX);
    }
}
