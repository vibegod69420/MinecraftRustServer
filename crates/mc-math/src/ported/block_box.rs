use std::cmp::{max, min};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisDirection {
    Positive,
    Negative,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getAxisDirection
    pub fn get_axis_direction(self) -> AxisDirection {
        match self {
            Direction::Up | Direction::South | Direction::East => AxisDirection::Positive,
            Direction::Down | Direction::North | Direction::West => AxisDirection::Negative,
        }
    }

    fn step(self) -> Vec3i {
        match self {
            Direction::Down => Vec3i::new(0, -1, 0),
            Direction::Up => Vec3i::new(0, 1, 0),
            Direction::North => Vec3i::new(0, 0, -1),
            Direction::South => Vec3i::new(0, 0, 1),
            Direction::West => Vec3i::new(-1, 0, 0),
            Direction::East => Vec3i::new(1, 0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.BlockPos#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#min
    pub fn min_pos(a: BlockPos, b: BlockPos) -> BlockPos {
        BlockPos::new(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z))
    }

    // JAVA: net.minecraft.core.BlockPos#max
    pub fn max_pos(a: BlockPos, b: BlockPos) -> BlockPos {
        BlockPos::new(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z))
    }

    // JAVA: net.minecraft.core.BlockPos#relative
    pub fn relative(&self, dir: Direction, amount: i32) -> BlockPos {
        let s = dir.step();
        BlockPos::new(
            self.x.wrapping_add(s.x.wrapping_mul(amount)),
            self.y.wrapping_add(s.y.wrapping_mul(amount)),
            self.z.wrapping_add(s.z.wrapping_mul(amount)),
        )
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, v: Vec3i) -> BlockPos {
        BlockPos::new(
            self.x.wrapping_add(v.x),
            self.y.wrapping_add(v.y),
            self.z.wrapping_add(v.z),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    // JAVA: net.minecraft.world.phys.AABB#encapsulatingFullBlocks
    pub fn encapsulating_full_blocks(a: BlockPos, b: BlockPos) -> Self {
        let min_x = min(a.x, b.x) as f64;
        let min_y = min(a.y, b.y) as f64;
        let min_z = min(a.z, b.z) as f64;
        let max_x = (max(a.x, b.x) as f64) + 1.0;
        let max_y = (max(a.y, b.y) as f64) + 1.0;
        let max_z = (max(a.z, b.z) as f64) + 1.0;
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

pub struct BlockPosIter {
    min: BlockPos,
    max: BlockPos,
    x: i32,
    y: i32,
    z: i32,
    done: bool,
}

impl BlockPosIter {
    fn new(min: BlockPos, max: BlockPos) -> Self {
        Self {
            min,
            max,
            x: min.x,
            y: min.y,
            z: min.z,
            done: false,
        }
    }
}

impl Iterator for BlockPosIter {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let current = BlockPos::new(self.x, self.y, self.z);
        if self.x < self.max.x {
            self.x = self.x.wrapping_add(1);
        } else if self.y < self.max.y {
            self.x = self.min.x;
            self.y = self.y.wrapping_add(1);
        } else if self.z < self.max.z {
            self.x = self.min.x;
            self.y = self.min.y;
            self.z = self.z.wrapping_add(1);
        } else {
            self.done = true;
        }
        Some(current)
    }
}

// JAVA: net.minecraft.core.BlockPos#betweenClosed
pub fn between_closed(a: BlockPos, b: BlockPos) -> BlockPosIter {
    let min_p = BlockPos::min_pos(a, b);
    let max_p = BlockPos::max_pos(a, b);
    BlockPosIter::new(min_p, max_p)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockBox {
    pub min: BlockPos,
    pub max: BlockPos,
}

impl BlockBox {
    // JAVA: net.minecraft.core.BlockBox#<init>
    pub fn new(min: BlockPos, max: BlockPos) -> Self {
        Self {
            min: BlockPos::min_pos(min, max),
            max: BlockPos::max_pos(min, max),
        }
    }

    // JAVA: net.minecraft.core.BlockBox#of
    pub fn of_single(pos: BlockPos) -> Self {
        Self::new(pos, pos)
    }

    // JAVA: net.minecraft.core.BlockBox#of
    pub fn of(a: BlockPos, b: BlockPos) -> Self {
        Self::new(a, b)
    }

    // JAVA: net.minecraft.core.BlockBox#include
    pub fn include(&self, pos: BlockPos) -> Self {
        Self::new(
            BlockPos::min_pos(self.min, pos),
            BlockPos::max_pos(self.max, pos),
        )
    }

    // JAVA: net.minecraft.core.BlockBox#isBlock
    pub fn is_block(&self) -> bool {
        self.min == self.max
    }

    // JAVA: net.minecraft.core.BlockBox#contains
    pub fn contains(&self, pos: BlockPos) -> bool {
        pos.get_x() >= self.min.get_x()
            && pos.get_y() >= self.min.get_y()
            && pos.get_z() >= self.min.get_z()
            && pos.get_x() <= self.max.get_x()
            && pos.get_y() <= self.max.get_y()
            && pos.get_z() <= self.max.get_z()
    }

    // JAVA: net.minecraft.core.BlockBox#aabb
    pub fn aabb(&self) -> Aabb {
        Aabb::encapsulating_full_blocks(self.min, self.max)
    }

    // JAVA: net.minecraft.core.BlockBox#sizeX
    pub fn size_x(&self) -> i32 {
        self.max.get_x().wrapping_sub(self.min.get_x()).wrapping_add(1)
    }

    // JAVA: net.minecraft.core.BlockBox#sizeY
    pub fn size_y(&self) -> i32 {
        self.max.get_y().wrapping_sub(self.min.get_y()).wrapping_add(1)
    }

    // JAVA: net.minecraft.core.BlockBox#sizeZ
    pub fn size_z(&self) -> i32 {
        self.max.get_z().wrapping_sub(self.min.get_z()).wrapping_add(1)
    }

    // JAVA: net.minecraft.core.BlockBox#extend
    pub fn extend(&self, dir: Direction, amount: i32) -> Self {
        if amount == 0 {
            return *self;
        }
        if dir.get_axis_direction() == AxisDirection::Positive {
            Self::of(
                self.min,
                BlockPos::max_pos(self.min, self.max.relative(dir, amount)),
            )
        } else {
            Self::of(
                BlockPos::min_pos(self.min.relative(dir, amount), self.max),
                self.max,
            )
        }
    }

    // JAVA: net.minecraft.core.BlockBox#move
    pub fn move_dir(&self, dir: Direction, amount: i32) -> Self {
        if amount == 0 {
            return *self;
        }
        Self::new(
            self.min.relative(dir, amount),
            self.max.relative(dir, amount),
        )
    }

    // JAVA: net.minecraft.core.BlockBox#offset
    pub fn offset(&self, v: Vec3i) -> Self {
        Self::new(self.min.offset(v), self.max.offset(v))
    }

    // JAVA: net.minecraft.core.BlockBox#iterator
    pub fn iter(&self) -> BlockPosIter {
        between_closed(self.min, self.max)
    }
}

impl IntoIterator for BlockBox {
    type Item = BlockPos;
    type IntoIter = BlockPosIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_normalizes_min_max() {
        let b = BlockBox::of(BlockPos::new(5, 5, 5), BlockPos::new(1, 2, 3));
        assert_eq!(b.min, BlockPos::new(1, 2, 3));
        assert_eq!(b.max, BlockPos::new(5, 5, 5));
    }

    #[test]
    fn contains_and_is_block() {
        let b = BlockBox::of_single(BlockPos::new(1, 2, 3));
        assert!(b.is_block());
        assert!(b.contains(BlockPos::new(1, 2, 3)));
        assert!(!b.contains(BlockPos::new(1, 2, 4)));
    }

    #[test]
    fn extend_positive_and_negative() {
        let b = BlockBox::of_single(BlockPos::new(0, 0, 0));
        let up = b.extend(Direction::Up, 2);
        assert_eq!(up.min, BlockPos::new(0, 0, 0));
        assert_eq!(up.max, BlockPos::new(0, 2, 0));
        let down = b.extend(Direction::Down, 2);
        assert_eq!(down.min, BlockPos::new(0, -2, 0));
        assert_eq!(down.max, BlockPos::new(0, 0, 0));
    }

    #[test]
    fn iterator_covers_all_positions() {
        let b = BlockBox::of(BlockPos::new(0, 0, 0), BlockPos::new(1, 0, 1));
        let pts: Vec<_> = b.iter().collect();
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[0], BlockPos::new(0, 0, 0));
        assert_eq!(pts[3], BlockPos::new(1, 0, 1));
    }

    #[test]
    fn move_and_offset_negative() {
        let b = BlockBox::of_single(BlockPos::new(0, 0, 0));
        let m = b.move_dir(Direction::West, 3);
        assert_eq!(m.min, BlockPos::new(-3, 0, 0));
        let o = b.offset(Vec3i::new(-1, -2, -3));
        assert_eq!(o.min, BlockPos::new(-1, -2, -3));
    }

    #[test]
    fn size_with_negative_coords() {
        let b = BlockBox::of(BlockPos::new(-2, -2, -2), BlockPos::new(-1, 0, 1));
        assert_eq!(b.size_x(), 2);
        assert_eq!(b.size_y(), 3);
        assert_eq!(b.size_z(), 4);
    }
}
