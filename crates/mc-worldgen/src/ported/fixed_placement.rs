#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPos;

impl SectionPos {
    pub fn block_to_section_coord(coord: i32) -> i32 {
        coord >> 4
    }
}

pub struct PlacementContext;

pub trait RandomSource {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    FixedPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedPlacement {
    positions: Vec<BlockPos>,
}

impl FixedPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.FixedPlacement#of
    pub fn of(positions: &[BlockPos]) -> Self {
        Self::new(positions.to_vec())
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.FixedPlacement#<init>
    fn new(positions: Vec<BlockPos>) -> Self {
        Self { positions }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.FixedPlacement#getPositions
    pub fn get_positions<R: RandomSource + ?Sized>(
        &self,
        _context: &PlacementContext,
        _random: &mut R,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        let mut any = false;
        for p in &self.positions {
            if Self::is_same_chunk(chunk_x, chunk_z, *p) {
                any = true;
                break;
            }
        }

        if !any {
            return Vec::new();
        }

        self.positions
            .iter()
            .copied()
            .filter(|p| Self::is_same_chunk(chunk_x, chunk_z, *p))
            .collect()
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.FixedPlacement#isSameChunk
    fn is_same_chunk(chunk_x: i32, chunk_z: i32, pos: BlockPos) -> bool {
        chunk_x == SectionPos::block_to_section_coord(pos.get_x())
            && chunk_z == SectionPos::block_to_section_coord(pos.get_z())
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.FixedPlacement#type
    pub fn modifier_type(&self) -> PlacementModifierType {
        PlacementModifierType::FixedPlacement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    #[test]
    fn empty_when_no_position_in_same_chunk() {
        let placement = FixedPlacement::of(&[BlockPos::new(0, 64, 0)]);
        let mut rng = DummyRandom;
        let result =
            placement.get_positions(&PlacementContext, &mut rng, BlockPos::new(16, 0, 0));
        assert!(result.is_empty());
    }

    #[test]
    fn returns_only_positions_in_same_chunk() {
        let p1 = BlockPos::new(1, 64, 2);
        let p2 = BlockPos::new(20, 64, 2); // different chunk
        let p3 = BlockPos::new(5, 10, 15);
        let placement = FixedPlacement::of(&[p1, p2, p3]);
        let mut rng = DummyRandom;
        let result = placement.get_positions(&PlacementContext, &mut rng, BlockPos::new(0, 0, 0));
        assert_eq!(result, vec![p1, p3]);
    }

    #[test]
    fn negative_chunk_boundary() {
        // -1 -> section -1, -16 -> section -1, -17 -> section -2
        let p = BlockPos::new(-1, 0, -16);
        let placement = FixedPlacement::of(&[p]);
        let mut rng = DummyRandom;
        let same =
            placement.get_positions(&PlacementContext, &mut rng, BlockPos::new(-5, 0, -10));
        assert_eq!(same, vec![p]);

        let diff = placement.get_positions(&PlacementContext, &mut rng, BlockPos::new(0, 0, 0));
        assert!(diff.is_empty());
    }

    #[test]
    fn of_and_type() {
        let placement = FixedPlacement::of(&[BlockPos::new(1, 2, 3)]);
        assert_eq!(
            placement.modifier_type(),
            PlacementModifierType::FixedPlacement
        );
    }

    #[test]
    fn is_same_chunk_checks_section_coords() {
        assert!(FixedPlacement::is_same_chunk(0, 0, BlockPos::new(15, 0, 15)));
        assert!(!FixedPlacement::is_same_chunk(0, 0, BlockPos::new(16, 0, 0)));
        assert!(FixedPlacement::is_same_chunk(-1, -1, BlockPos::new(-1, 0, -16)));
        assert!(!FixedPlacement::is_same_chunk(-1, -1, BlockPos::new(-17, 0, -16)));
    }
}
