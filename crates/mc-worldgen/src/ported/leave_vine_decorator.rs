#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#west
    pub fn west(&self) -> Self {
        Self {
            x: self.x.wrapping_sub(1),
            y: self.y,
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#east
    pub fn east(&self) -> Self {
        Self {
            x: self.x.wrapping_add(1),
            y: self.y,
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#north
    pub fn north(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_sub(1),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#south
    pub fn south(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_add(1),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VineFaceProperty {
    East,
    West,
    South,
    North,
}

/// Java: net.minecraft.world.level.block.VineBlock
pub struct VineBlock;

impl VineBlock {
    pub const EAST: VineFaceProperty = VineFaceProperty::East;
    pub const WEST: VineFaceProperty = VineFaceProperty::West;
    pub const SOUTH: VineFaceProperty = VineFaceProperty::South;
    pub const NORTH: VineFaceProperty = VineFaceProperty::North;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDecoratorType {
    LeaveVine,
}

pub trait LeaveVineContext {
    fn next_float(&mut self) -> f32;
    fn leaves(&self) -> Vec<BlockPos>;
    fn is_air(&self, pos: BlockPos) -> bool;
    fn place_vine(&mut self, pos: BlockPos, face: VineFaceProperty);
}

pub struct LeaveVineDecorator {
    pub probability: f32,
}

impl LeaveVineDecorator {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator#<init>
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator#type
    pub fn type_(&self) -> TreeDecoratorType {
        TreeDecoratorType::LeaveVine
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator#place
    pub fn place<C: LeaveVineContext>(&self, ctx: &mut C) {
        let leaves = ctx.leaves();
        for leaf in leaves {
            if ctx.next_float() < self.probability {
                let pos = leaf.west();
                if ctx.is_air(pos) {
                    Self::add_hanging_vine(pos, VineFaceProperty::East, ctx);
                }
            }

            if ctx.next_float() < self.probability {
                let pos = leaf.east();
                if ctx.is_air(pos) {
                    Self::add_hanging_vine(pos, VineFaceProperty::West, ctx);
                }
            }

            if ctx.next_float() < self.probability {
                let pos = leaf.north();
                if ctx.is_air(pos) {
                    Self::add_hanging_vine(pos, VineFaceProperty::South, ctx);
                }
            }

            if ctx.next_float() < self.probability {
                let pos = leaf.south();
                if ctx.is_air(pos) {
                    Self::add_hanging_vine(pos, VineFaceProperty::North, ctx);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator#addHangingVine
    fn add_hanging_vine<C: LeaveVineContext>(pos: BlockPos, face: VineFaceProperty, ctx: &mut C) {
        ctx.place_vine(pos, face);
        let mut remaining: i32 = 4;

        let mut var4 = pos.below();
        while ctx.is_air(var4) && remaining > 0 {
            ctx.place_vine(var4, face);
            var4 = var4.below();
            remaining = remaining.wrapping_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockCtx {
        floats: Vec<f32>,
        float_idx: usize,
        leaves: Vec<BlockPos>,
        air: HashSet<BlockPos>,
        placed: Vec<(BlockPos, VineFaceProperty)>,
    }

    impl MockCtx {
        fn new(floats: Vec<f32>, leaves: Vec<BlockPos>, air: HashSet<BlockPos>) -> Self {
            Self {
                floats,
                float_idx: 0,
                leaves,
                air,
                placed: Vec::new(),
            }
        }
    }

    impl LeaveVineContext for MockCtx {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx % self.floats.len()];
            self.float_idx = self.float_idx.wrapping_add(1);
            v
        }

        fn leaves(&self) -> Vec<BlockPos> {
            self.leaves.clone()
        }

        fn is_air(&self, pos: BlockPos) -> bool {
            self.air.contains(&pos)
        }

        fn place_vine(&mut self, pos: BlockPos, face: VineFaceProperty) {
            self.placed.push((pos, face));
        }
    }

    #[test]
    fn test_no_vines_when_probability_zero() {
        let leaf = BlockPos::new(0, 10, 0);
        let mut air = HashSet::new();
        air.insert(leaf.west());
        air.insert(leaf.east());
        air.insert(leaf.north());
        air.insert(leaf.south());
        let mut ctx = MockCtx::new(vec![0.0], vec![leaf], air);
        let dec = LeaveVineDecorator::new(0.0);
        dec.place(&mut ctx);
        assert!(ctx.placed.is_empty());
        assert_eq!(dec.type_(), TreeDecoratorType::LeaveVine);
    }

    #[test]
    fn test_places_vines_on_all_four_sides_with_correct_faces() {
        let leaf = BlockPos::new(5, 20, -3);
        let mut air = HashSet::new();
        for p in [leaf.west(), leaf.east(), leaf.north(), leaf.south()] {
            air.insert(p);
        }
        let mut ctx = MockCtx::new(vec![0.0], vec![leaf], air);
        let dec = LeaveVineDecorator::new(1.0);
        dec.place(&mut ctx);

        assert_eq!(ctx.placed.len(), 4);
        assert_eq!(ctx.placed[0], (leaf.west(), VineBlock::EAST));
        assert_eq!(ctx.placed[1], (leaf.east(), VineBlock::WEST));
        assert_eq!(ctx.placed[2], (leaf.north(), VineBlock::SOUTH));
        assert_eq!(ctx.placed[3], (leaf.south(), VineBlock::NORTH));
    }

    #[test]
    fn test_hanging_vine_places_up_to_four_below() {
        let leaf = BlockPos::new(0, 10, 0);
        let west = leaf.west();
        let mut air = HashSet::new();
        // Make west and many blocks below it air
        for dy in 0..=10 {
            air.insert(BlockPos::new(west.x, west.y.wrapping_sub(dy), west.z));
        }
        // Only west side triggers: floats [0.0, 1.0, 1.0, 1.0] for 4 checks
        let mut ctx = MockCtx::new(vec![0.0, 1.0, 1.0, 1.0], vec![leaf], air);
        let dec = LeaveVineDecorator::new(0.5);
        dec.place(&mut ctx);

        // pos + 4 below = 5 placements total
        assert_eq!(ctx.placed.len(), 5);
        for (i, (p, face)) in ctx.placed.iter().enumerate() {
            assert_eq!(*face, VineFaceProperty::East);
            assert_eq!(p.x, west.x);
            assert_eq!(p.z, west.z);
            assert_eq!(p.y, west.y.wrapping_sub(i as i32));
        }
    }

    #[test]
    fn test_hanging_vine_stops_when_not_air() {
        let leaf = BlockPos::new(0, 10, 0);
        let west = leaf.west();
        let mut air = HashSet::new();
        air.insert(west);
        air.insert(west.below());
        // west.below().below() is not air -> stop after placing at below
        let mut ctx = MockCtx::new(vec![0.0, 1.0, 1.0, 1.0], vec![leaf], air);
        let dec = LeaveVineDecorator::new(0.5);
        dec.place(&mut ctx);
        assert_eq!(ctx.placed.len(), 2);
        assert_eq!(ctx.placed[0].0, west);
        assert_eq!(ctx.placed[1].0, west.below());
    }

    #[test]
    fn test_negative_coordinates_and_wrapping() {
        let leaf = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);
        let west = leaf.west();
        assert_eq!(west.x, i32::MAX); // wrapping_sub on MIN
        assert_eq!(west.y, i32::MIN);

        let below = leaf.below();
        assert_eq!(below.y, i32::MAX);

        let mut air = HashSet::new();
        air.insert(west);
        let mut ctx = MockCtx::new(vec![0.0, 1.0, 1.0, 1.0], vec![leaf], air);
        let dec = LeaveVineDecorator::new(1.0);
        dec.place(&mut ctx);
        assert_eq!(ctx.placed[0].0, west);
    }

    #[test]
    fn test_float_bits_and_nan_probability() {
        let bits: u32 = 0x3F00_0000; // 0.5f32
        let prob = f32::from_bits(bits);
        assert_eq!(prob.to_bits(), bits);

        let leaf = BlockPos::new(0, 0, 0);
        let mut ctx = MockCtx::new(vec![0.5], vec![leaf], HashSet::new());
        let dec = LeaveVineDecorator::new(prob);
        dec.place(&mut ctx);
        // 0.5 < 0.5 is false
        assert!(ctx.placed.is_empty());

        let mut ctx2 = MockCtx::new(vec![0.0], vec![BlockPos::new(0, 0, 0)], HashSet::new());
        let dec_nan = LeaveVineDecorator::new(f32::NAN);
        dec_nan.place(&mut ctx2);
        // anything < NaN is false
        assert!(ctx2.placed.is_empty());
    }
}
