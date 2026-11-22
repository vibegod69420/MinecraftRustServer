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
pub enum PlacementModifierType {
    RarityFilter,
    InSquare,
    Other(u32),
}

/// Placeholder for Mojang MapCodec; serialization framework not ported.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(pub PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InSquarePlacement;

static INSTANCE: InSquarePlacement = InSquarePlacement;

/// Placeholder for MapCodec.unit(() -> INSTANCE)
/// JAVA: net.minecraft.world.level.levelgen.placement.InSquarePlacement#CODEC
pub const IN_SQUARE_PLACEMENT_CODEC: MapCodec<InSquarePlacement> = MapCodec::placeholder();

impl InSquarePlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.InSquarePlacement#spread
    pub fn spread() -> InSquarePlacement {
        INSTANCE
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.InSquarePlacement#getPositions
    pub fn get_positions(
        &self,
        _ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let x = random.next_int(16).wrapping_add(pos.x);
        let z = random.next_int(16).wrapping_add(pos.z);
        vec![BlockPos::new(x, pos.y, z)]
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.InSquarePlacement#type
    pub fn modifier_type(&self) -> PlacementModifierType {
        PlacementModifierType::InSquare
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        ints: Vec<i32>,
        idx: usize,
    }
    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.ints[self.idx];
            self.idx += 1;
            v
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    #[test]
    fn spreads_within_chunk_offsets() {
        let p = InSquarePlacement::spread();
        let mut rng = SeqRandom { ints: vec![5, 12], idx: 0 };
        let pos = BlockPos::new(100, 64, 200);
        let out = p.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out, vec![BlockPos::new(105, 64, 212)]);
    }

    #[test]
    fn preserves_y_and_uses_wrapping_add() {
        let p = InSquarePlacement::spread();
        let mut rng = SeqRandom { ints: vec![15, 1], idx: 0 };
        let pos = BlockPos::new(i32::MAX, -16, i32::MAX);
        let out = p.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].x, i32::MAX.wrapping_add(15));
        assert_eq!(out[0].y, -16);
        assert_eq!(out[0].z, i32::MAX.wrapping_add(1));
    }

    #[test]
    fn negative_base_coords() {
        let p = InSquarePlacement::spread();
        let mut rng = SeqRandom { ints: vec![0, 15], idx: 0 };
        let pos = BlockPos::new(-32, i32::MIN, -1);
        let out = p.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out[0], BlockPos::new(-32, i32::MIN, 14));
    }

    #[test]
    fn type_is_in_square() {
        assert_eq!(InSquarePlacement::spread().modifier_type(), PlacementModifierType::InSquare);
    }

    #[test]
    fn spread_returns_instance() {
        let a = InSquarePlacement::spread();
        let b = InSquarePlacement::spread();
        let mut rng_a = SeqRandom { ints: vec![1, 2], idx: 0 };
        let mut rng_b = SeqRandom { ints: vec![1, 2], idx: 0 };
        let pos = BlockPos::new(0, 5, 0);
        assert_eq!(
            a.get_positions(&PlacementContext, &mut rng_a, pos),
            b.get_positions(&PlacementContext, &mut rng_b, pos)
        );
    }
}
