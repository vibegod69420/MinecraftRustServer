//! Faithful port of `net.minecraft.core.dispenser.BlockSource` (1.21.11).
//!
//! This module is self-contained. External Minecraft types are represented by
//! minimal local stubs so the record and its `center()` behavior can compile
//! and be tested independently.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#getCenter
    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            self.x as f64 + 0.5,
            self.y as f64 + 0.5,
            self.z as f64 + 0.5,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerLevel;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DispenserBlockEntity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSource {
    level: ServerLevel,
    pos: BlockPos,
    state: BlockState,
    block_entity: DispenserBlockEntity,
}

impl BlockSource {
    pub fn new(
        level: ServerLevel,
        pos: BlockPos,
        state: BlockState,
        block_entity: DispenserBlockEntity,
    ) -> Self {
        Self {
            level,
            pos,
            state,
            block_entity,
        }
    }

    // JAVA: net.minecraft.core.dispenser.BlockSource#center
    pub fn center(&self) -> Vec3 {
        self.pos.get_center()
    }

    // JAVA: net.minecraft.core.dispenser.BlockSource#level
    pub fn level(&self) -> &ServerLevel {
        &self.level
    }

    // JAVA: net.minecraft.core.dispenser.BlockSource#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.core.dispenser.BlockSource#state
    pub fn state(&self) -> &BlockState {
        &self.state
    }

    // JAVA: net.minecraft.core.dispenser.BlockSource#blockEntity
    pub fn block_entity(&self) -> &DispenserBlockEntity {
        &self.block_entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_source(pos: BlockPos) -> BlockSource {
        BlockSource::new(
            ServerLevel,
            pos,
            BlockState,
            DispenserBlockEntity,
        )
    }

    #[test]
    fn center_adds_half() {
        let src = sample_source(BlockPos::new(0, 0, 0));
        assert_eq!(src.center(), Vec3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn center_handles_negative_coordinates() {
        let src = sample_source(BlockPos::new(-1, -2, -3));
        assert_eq!(src.center(), Vec3::new(-0.5, -1.5, -2.5));
    }

    #[test]
    fn center_preserves_float_bits_for_representable_values() {
        let src = sample_source(BlockPos::new(10, 64, -7));
        let c = src.center();
        assert_eq!(c.x.to_bits(), 10.5_f64.to_bits());
        assert_eq!(c.y.to_bits(), 64.5_f64.to_bits());
        assert_eq!(c.z.to_bits(), (-6.5_f64).to_bits());
    }

    #[test]
    fn record_accessors_return_components() {
        let pos = BlockPos::new(1, 2, 3);
        let src = sample_source(pos);
        assert_eq!(src.pos(), pos);
        assert_eq!(src.level(), &ServerLevel);
        assert_eq!(src.state(), &BlockState);
        assert_eq!(src.block_entity(), &DispenserBlockEntity);
    }

    #[test]
    fn block_pos_get_center_extreme_ints() {
        let pos = BlockPos::new(i32::MAX, i32::MIN, 0);
        let c = pos.get_center();
        assert_eq!(c.x, i32::MAX as f64 + 0.5);
        assert_eq!(c.y, i32::MIN as f64 + 0.5);
        assert_eq!(c.z, 0.5);
    }
}
