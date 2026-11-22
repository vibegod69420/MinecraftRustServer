use std::fmt;

/// Minimal self-contained stub for a Minecraft resource key (e.g. `ResourceKey<Level>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub value: String,
}

impl ResourceKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// Minimal self-contained stub for `net.minecraft.core.BlockPos`.
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

    // JAVA: net.minecraft.core.Vec3i#distChessboard
    pub fn dist_chessboard(&self, other: BlockPos) -> i32 {
        let dx = self.x.wrapping_sub(other.x).wrapping_abs();
        let dy = self.y.wrapping_sub(other.y).wrapping_abs();
        let dz = self.z.wrapping_sub(other.z).wrapping_abs();
        dx.max(dy).max(dz)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockPos{{x={}, y={}, z={}}}", self.x, self.y, self.z)
    }
}

/// Faithful port of `net.minecraft.core.GlobalPos`.
///
/// Codec/StreamCodec fields from Java are omitted for self-containment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalPos {
    pub dimension: ResourceKey,
    pub pos: BlockPos,
}

impl GlobalPos {
    // JAVA: net.minecraft.core.GlobalPos#<init>
    pub fn new(dimension: ResourceKey, pos: BlockPos) -> Self {
        Self { dimension, pos }
    }

    // JAVA: net.minecraft.core.GlobalPos#of
    pub fn of(dimension: ResourceKey, pos: BlockPos) -> Self {
        Self::new(dimension, pos)
    }

    // JAVA: net.minecraft.core.GlobalPos#dimension
    pub fn dimension(&self) -> &ResourceKey {
        &self.dimension
    }

    // JAVA: net.minecraft.core.GlobalPos#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.core.GlobalPos#isCloseEnough
    pub fn is_close_enough(&self, dimension: &ResourceKey, pos: BlockPos, max_distance: i32) -> bool {
        self.dimension == *dimension && self.pos.dist_chessboard(pos) <= max_distance
    }
}

impl fmt::Display for GlobalPos {
    // JAVA: net.minecraft.core.GlobalPos#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.dimension, self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockPos, GlobalPos, ResourceKey};

    #[test]
    fn of_and_accessors() {
        let dim = ResourceKey::new("minecraft:overworld");
        let pos = BlockPos::new(1, 64, -2);
        let gp = GlobalPos::of(dim.clone(), pos);
        assert_eq!(gp.dimension(), &dim);
        assert_eq!(gp.pos(), pos);
    }

    #[test]
    fn to_string_format() {
        let gp = GlobalPos::of(ResourceKey::new("minecraft:the_nether"), BlockPos::new(7, 8, 9));
        assert_eq!(gp.to_string(), "minecraft:the_nether BlockPos{x=7, y=8, z=9}");
    }

    #[test]
    fn is_close_enough_same_dimension_boundary() {
        let dim = ResourceKey::new("minecraft:overworld");
        let gp = GlobalPos::of(dim.clone(), BlockPos::new(0, 0, 0));
        assert!(gp.is_close_enough(&dim, BlockPos::new(1, 2, 3), 3));
        assert!(!gp.is_close_enough(&dim, BlockPos::new(1, 2, 4), 3));
    }

    #[test]
    fn is_close_enough_different_dimension() {
        let gp = GlobalPos::of(ResourceKey::new("minecraft:overworld"), BlockPos::new(0, 0, 0));
        let other_dim = ResourceKey::new("minecraft:the_end");
        assert!(!gp.is_close_enough(&other_dim, BlockPos::new(0, 0, 0), 100));
    }

    #[test]
    fn dist_chessboard_negative_and_abs_overflow_edge() {
        let a = BlockPos::new(-5, 10, -3);
        let b = BlockPos::new(5, -10, 3);
        assert_eq!(a.dist_chessboard(b), 20);

        // Java Math.abs(Integer.MIN_VALUE) returns Integer.MIN_VALUE; max with 0 yields 0.
        let min = BlockPos::new(i32::MIN, 0, 0);
        let zero = BlockPos::new(0, 0, 0);
        assert_eq!(min.dist_chessboard(zero), 0);
    }
}
