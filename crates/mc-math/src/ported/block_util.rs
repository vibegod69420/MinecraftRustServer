use crate::packing::BlockPos;
use std::cmp::{max, min};
use std::fmt;

/// A minimal block-level view required by [`BlockUtil::get_top_connected_block`].
///
/// This keeps the module self-contained while preserving the Java call shape.
pub trait BlockGetter {
    type BlockState: BlockStateView;

    fn get_block_state(&self, pos: &BlockPos) -> Self::BlockState;
}

/// Minimal block-state view required by [`BlockUtil::get_top_connected_block`].
pub trait BlockStateView {
    type Block: PartialEq;

    fn is_block(&self, block: &Self::Block) -> bool;
}

/// Cardinal/axis directions used by [`BlockUtil`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    /// Returns the positive/negative direction for an axis, matching
    /// `Direction.get(Direction.AxisDirection, Direction.Axis)` usage in Java.
    pub fn get_negative(axis: Axis) -> Self {
        match axis {
            Axis::X => Direction::West,
            Axis::Y => Direction::Down,
            Axis::Z => Direction::North,
        }
    }

    /// Returns the opposite direction.
    pub fn opposite(self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    fn step(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }
}

/// Coordinate axes used by [`BlockUtil`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Mutable block position helper mirroring Java's `BlockPos.MutableBlockPos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_block_pos(pos: &BlockPos) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }

    pub fn set(&mut self, pos: &BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    pub fn set_xyz(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.step();
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    pub fn move_dir_dist(&mut self, dir: Direction, dist: i32) -> &mut Self {
        let (dx, dy, dz) = dir.step();
        self.x = self.x.wrapping_add(dx.wrapping_mul(dist));
        self.y = self.y.wrapping_add(dy.wrapping_mul(dist));
        self.z = self.z.wrapping_add(dz.wrapping_mul(dist));
        self
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

fn relative(pos: &BlockPos, axis: Axis, amount: i32) -> BlockPos {
    match axis {
        Axis::X => BlockPos::new(pos.x.wrapping_add(amount), pos.y, pos.z),
        Axis::Y => BlockPos::new(pos.x, pos.y.wrapping_add(amount), pos.z),
        Axis::Z => BlockPos::new(pos.x, pos.y, pos.z.wrapping_add(amount)),
    }
}

/// Port of `net.minecraft.util.BlockUtil`.
pub struct BlockUtil;

impl BlockUtil {
    // JAVA: net.minecraft.util.BlockUtil#getLargestRectangleAround
    pub fn get_largest_rectangle_around<F>(
        pos: &BlockPos,
        axis1: Axis,
        max1: i32,
        axis2: Axis,
        max2: i32,
        predicate: F,
    ) -> FoundRectangle
    where
        F: Fn(&BlockPos) -> bool,
    {
        let mut cursor = MutableBlockPos::from_block_pos(pos);
        let dir_neg_1 = Direction::get_negative(axis1);
        let dir_pos_1 = dir_neg_1.opposite();
        let dir_neg_2 = Direction::get_negative(axis2);
        let dir_pos_2 = dir_neg_2.opposite();

        let limit_neg_1 = Self::get_limit(&predicate, cursor.set(pos), dir_neg_1, max1);
        let limit_pos_1 = Self::get_limit(&predicate, cursor.set(pos), dir_pos_1, max1);
        let center_index = limit_neg_1;

        let bounds_len = (center_index + 1 + limit_pos_1) as usize;
        let mut bounds: Vec<IntBounds> = Vec::with_capacity(bounds_len);
        // Fill with placeholders; we'll assign by index.
        for _ in 0..bounds_len {
            bounds.push(IntBounds { min: 0, max: 0 });
        }

        let center_min_2 = Self::get_limit(&predicate, cursor.set(pos), dir_neg_2, max2);
        let center_max_2 = Self::get_limit(&predicate, cursor.set(pos), dir_pos_2, max2);
        bounds[center_index as usize] = IntBounds {
            min: center_min_2,
            max: center_max_2,
        };
        let anchor_min_2 = bounds[center_index as usize].min;

        let mut i = 1i32;
        while i <= limit_neg_1 {
            let prev = bounds[(center_index - (i - 1)) as usize];
            let idx = (center_index - i) as usize;
            cursor.set(pos);
            cursor.move_dir_dist(dir_neg_1, i);
            let min_bound = Self::get_limit(&predicate, &mut cursor.clone(), dir_neg_2, prev.min);
            cursor.set(pos);
            cursor.move_dir_dist(dir_neg_1, i);
            let max_bound = Self::get_limit(&predicate, &mut cursor.clone(), dir_pos_2, prev.max);
            bounds[idx] = IntBounds {
                min: min_bound,
                max: max_bound,
            };
            i += 1;
        }

        let mut j = 1i32;
        while j <= limit_pos_1 {
            let prev = bounds[(center_index + j - 1) as usize];
            let idx = (center_index + j) as usize;
            cursor.set(pos);
            cursor.move_dir_dist(dir_pos_1, j);
            let min_bound = Self::get_limit(&predicate, &mut cursor.clone(), dir_neg_2, prev.min);
            cursor.set(pos);
            cursor.move_dir_dist(dir_pos_1, j);
            let max_bound = Self::get_limit(&predicate, &mut cursor.clone(), dir_pos_2, prev.max);
            bounds[idx] = IntBounds {
                min: min_bound,
                max: max_bound,
            };
            j += 1;
        }

        let mut best_x_offset: i32 = 0;
        let mut best_y_offset: i32 = 0;
        let mut best_width: i32 = 0;
        let mut best_height: i32 = 0;
        let mut heights: Vec<i32> = vec![0; bounds.len()];

        let mut row = anchor_min_2;
        while row >= 0 {
            for (idx, b) in bounds.iter().enumerate() {
                let top = anchor_min_2 - b.min;
                let bottom = anchor_min_2 + b.max;
                heights[idx] = if row >= top && row <= bottom {
                    bottom + 1 - row
                } else {
                    0
                };
            }

            let (rect_bounds, rect_height) = Self::get_max_rectangle_location(&heights);
            let rect_width = 1 + rect_bounds.max - rect_bounds.min;
            if rect_width * rect_height > best_width * best_height {
                best_x_offset = rect_bounds.min;
                best_y_offset = row;
                best_width = rect_width;
                best_height = rect_height;
            }

            row -= 1;
        }

        let min_corner = relative(
            &relative(pos, axis1, best_x_offset - center_index),
            axis2,
            best_y_offset - anchor_min_2,
        );
        FoundRectangle {
            min_corner,
            axis1_size: best_width,
            axis2_size: best_height,
        }
    }

    // JAVA: net.minecraft.util.BlockUtil#getLimit
    fn get_limit<F>(
        predicate: &F,
        cursor: &mut MutableBlockPos,
        direction: Direction,
        max_dist: i32,
    ) -> i32
    where
        F: Fn(&BlockPos) -> bool,
    {
        let mut count: i32 = 0;
        while count < max_dist {
            cursor.move_dir(direction);
            let pos = cursor.to_block_pos();
            if !predicate(&pos) {
                break;
            }
            count += 1;
        }
        count
    }

    // JAVA: net.minecraft.util.BlockUtil#getMaxRectangleLocation
    pub fn get_max_rectangle_location(heights: &[i32]) -> (IntBounds, i32) {
        let mut best_left: i32 = 0;
        let mut best_right: i32 = 0;
        let mut best_area_height: i32 = 0;
        let mut stack: Vec<i32> = Vec::new();
        stack.push(0);

        let mut i: i32 = 1;
        let len_i32 = heights.len() as i32;
        while i <= len_i32 {
            let h = if i == len_i32 {
                0
            } else {
                heights[i as usize]
            };

            while !stack.is_empty() {
                let top_idx = *stack.last().unwrap();
                let top_h = heights[top_idx as usize];
                if h >= top_h {
                    stack.push(i);
                    break;
                }

                stack.pop();
                let left = if stack.is_empty() {
                    0
                } else {
                    *stack.last().unwrap() + 1
                };
                let width = i - left;
                let current_area_h = best_area_height;
                let current_width = best_right - best_left;
                if top_h * width > current_area_h * current_width {
                    best_right = i;
                    best_left = left;
                    best_area_height = top_h;
                }
            }

            if stack.is_empty() {
                stack.push(i);
            }

            i += 1;
        }

        (
            IntBounds {
                min: best_left,
                max: best_right - 1,
            },
            best_area_height,
        )
    }

    // JAVA: net.minecraft.util.BlockUtil#getTopConnectedBlock
    pub fn get_top_connected_block<G>(
        level: &G,
        pos: &BlockPos,
        connect_block: &<G::BlockState as BlockStateView>::Block,
        direction: Direction,
        target_block: &<G::BlockState as BlockStateView>::Block,
    ) -> Option<BlockPos>
    where
        G: BlockGetter,
    {
        let mut cursor = MutableBlockPos::from_block_pos(pos);
        let mut state;
        loop {
            cursor.move_dir(direction);
            let cur_pos = cursor.to_block_pos();
            state = level.get_block_state(&cur_pos);
            if !state.is_block(connect_block) {
                break;
            }
        }

        if state.is_block(target_block) {
            Some(cursor.to_block_pos())
        } else {
            None
        }
    }
}

/// Port of `net.minecraft.util.BlockUtil.FoundRectangle`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundRectangle {
    pub min_corner: BlockPos,
    pub axis1_size: i32,
    pub axis2_size: i32,
}

impl FoundRectangle {
    pub fn new(min_corner: BlockPos, axis1_size: i32, axis2_size: i32) -> Self {
        Self {
            min_corner,
            axis1_size,
            axis2_size,
        }
    }
}

/// Port of `net.minecraft.util.BlockUtil.IntBounds`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntBounds {
    pub min: i32,
    pub max: i32,
}

impl IntBounds {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

impl fmt::Display for IntBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntBounds{{min={}, max={}}}", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestBlock(u8);

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestState(u8);

    impl BlockStateView for TestState {
        type Block = TestBlock;
        fn is_block(&self, block: &Self::Block) -> bool {
            self.0 == block.0
        }
    }

    struct TestLevel {
        states: HashMap<(i32, i32, i32), TestState>,
        default: TestState,
    }

    impl TestLevel {
        fn new(default: u8) -> Self {
            Self {
                states: HashMap::new(),
                default: TestState(default),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, id: u8) {
            self.states.insert((x, y, z), TestState(id));
        }
    }

    impl BlockGetter for TestLevel {
        type BlockState = TestState;
        fn get_block_state(&self, pos: &BlockPos) -> Self::BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    #[test]
    fn int_bounds_display_matches_java() {
        let b = IntBounds::new(-2, 5);
        assert_eq!(b.to_string(), "IntBounds{min=-2, max=5}");
    }

    #[test]
    fn get_max_rectangle_location_simple() {
        // Classic histogram example: heights [2,1,5,6,2,3]
        let heights = [2, 1, 5, 6, 2, 3];
        let (bounds, h) = BlockUtil::get_max_rectangle_location(&heights);
        // Max area is height 5 over width 2 at indices 2,3 -> area 10
        assert_eq!(h, 5);
        assert_eq!(bounds.min, 2);
        assert_eq!(bounds.max, 3);
    }

    #[test]
    fn get_max_rectangle_location_empty_and_zeros() {
        let (bounds, h) = BlockUtil::get_max_rectangle_location(&[]);
        assert_eq!(h, 0);
        assert_eq!(bounds.min, 0);
        assert_eq!(bounds.max, -1);

        let (bounds, h) = BlockUtil::get_max_rectangle_location(&[0, 0, 0]);
        assert_eq!(h, 0);
    }

    #[test]
    fn get_top_connected_block_finds_target() {
        // Vertical column: pos at y=0 is connect block, y=1,2 connect, y=3 target
        let mut level = TestLevel::new(0);
        level.set(0, 0, 0, 1);
        level.set(0, 1, 0, 1);
        level.set(0, 2, 0, 1);
        level.set(0, 3, 0, 2);

        let pos = BlockPos::new(0, 0, 0);
        let result = BlockUtil::get_top_connected_block(
            &level,
            &pos,
            &TestBlock(1),
            Direction::Up,
            &TestBlock(2),
        );
        assert_eq!(result, Some(BlockPos::new(0, 3, 0)));
    }

    #[test]
    fn get_top_connected_block_returns_none_when_not_target() {
        let mut level = TestLevel::new(0);
        level.set(0, 0, 0, 1);
        level.set(0, 1, 0, 1);
        level.set(0, 2, 0, 3); // different block, not target

        let pos = BlockPos::new(0, 0, 0);
        let result = BlockUtil::get_top_connected_block(
            &level,
            &pos,
            &TestBlock(1),
            Direction::Up,
            &TestBlock(2),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn get_largest_rectangle_around_square() {
        // 3x3 valid area centered at origin in X/Z plane (Y constant)
        let valid = |p: &BlockPos| -> bool { p.y == 0 && p.x >= -1 && p.x <= 1 && p.z >= -1 && p.z <= 1 };
        let origin = BlockPos::new(0, 0, 0);
        let rect = BlockUtil::get_largest_rectangle_around(&origin, Axis::X, 5, Axis::Z, 5, valid);
        assert_eq!(rect.axis1_size * rect.axis2_size, 9);
        assert_eq!(rect.axis1_size, 3);
        assert_eq!(rect.axis2_size, 3);
        // min corner should be (-1,0,-1)
        assert_eq!(rect.min_corner, BlockPos::new(-1, 0, -1));
    }

    #[test]
    fn mutable_block_pos_move_and_set() {
        let origin = BlockPos::new(10, 20, 30);
        let mut m = MutableBlockPos::from_block_pos(&origin);
        m.move_dir(Direction::East);
        assert_eq!(m.to_block_pos(), BlockPos::new(11, 20, 30));
        m.move_dir_dist(Direction::Up, 3);
        assert_eq!(m.to_block_pos(), BlockPos::new(11, 23, 30));
        m.set(&origin);
        assert_eq!(m.to_block_pos(), origin);
    }
}
