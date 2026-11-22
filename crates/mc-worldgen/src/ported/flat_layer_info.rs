use std::fmt;
use std::marker::PhantomData;

/// Placeholder for `net.minecraft.world.level.dimension.DimensionType#Y_SIZE`.
pub const DIMENSION_TYPE_Y_SIZE: i32 = 384;

/// Placeholder for Mojang serialization `Codec<T>`.
/// The real codec initialization is not ported; this preserves the static field shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codec<T>(PhantomData<T>);

impl<T> Codec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    key: String,
}

impl Block {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState
    pub fn default_block_state(&self) -> BlockState {
        BlockState {
            block: self.clone(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    block: Block,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
}

/// Port of `net.minecraft.world.level.levelgen.flat.FlatLayerInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLayerInfo {
    block: Block,
    height: i32,
}

/// Placeholder for `FlatLayerInfo.CODEC`.
pub static CODEC: Codec<FlatLayerInfo> = Codec::placeholder();

impl FlatLayerInfo {
    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLayerInfo#FlatLayerInfo
    pub fn new(height: i32, block: Block) -> Self {
        Self { height, block }
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLayerInfo#getHeight
    pub fn get_height(&self) -> i32 {
        self.height
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLayerInfo#getBlockState
    pub fn get_block_state(&self) -> BlockState {
        self.block.default_block_state()
    }

    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLayerInfo#heightLimited
    pub fn height_limited(&self, max_height: i32) -> Self {
        if self.height > max_height {
            Self::new(max_height, self.block.clone())
        } else {
            self.clone()
        }
    }
}

impl fmt::Display for FlatLayerInfo {
    // JAVA: net.minecraft.world.level.levelgen.flat.FlatLayerInfo#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.height != 1 {
            write!(f, "{}*{}", self.height, self.block.key())
        } else {
            write!(f, "{}", self.block.key())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(name: &str) -> Block {
        Block::new(name)
    }

    #[test]
    fn new_and_getters() {
        let info = FlatLayerInfo::new(3, block("minecraft:stone"));
        assert_eq!(info.get_height(), 3);
        assert_eq!(
            info.get_block_state().get_block().key(),
            "minecraft:stone"
        );
    }

    #[test]
    fn height_limited_clamps_only_when_greater() {
        let info = FlatLayerInfo::new(5, block("minecraft:dirt"));
        let limited = info.height_limited(3);
        assert_eq!(limited.get_height(), 3);
        assert_eq!(limited.get_block_state().get_block().key(), "minecraft:dirt");

        let unchanged_at_boundary = info.height_limited(5);
        assert_eq!(unchanged_at_boundary.get_height(), 5);

        let unchanged_above = info.height_limited(10);
        assert_eq!(unchanged_above.get_height(), 5);
    }

    #[test]
    fn to_string_formatting() {
        assert_eq!(
            FlatLayerInfo::new(1, block("minecraft:grass_block")).to_string(),
            "minecraft:grass_block"
        );
        assert_eq!(
            FlatLayerInfo::new(3, block("minecraft:stone")).to_string(),
            "3*minecraft:stone"
        );
        assert_eq!(
            FlatLayerInfo::new(0, block("minecraft:air")).to_string(),
            "0*minecraft:air"
        );
    }

    #[test]
    fn negative_and_extreme_heights() {
        let neg = FlatLayerInfo::new(-1, block("minecraft:stone"));
        assert_eq!(neg.get_height(), -1);
        assert_eq!(neg.to_string(), "-1*minecraft:stone");
        // -1 > 0 is false, so no clamping
        assert_eq!(neg.height_limited(0).get_height(), -1);

        let max_info = FlatLayerInfo::new(i32::MAX, block("minecraft:bedrock"));
        assert_eq!(max_info.get_height(), i32::MAX);
        let clamped = max_info.height_limited(i32::MIN);
        assert_eq!(clamped.get_height(), i32::MIN);
    }

    #[test]
    fn height_preserves_exact_integer_and_float_bit_patterns() {
        let bits = 0xFF80_0001u32 as i32;
        let info = FlatLayerInfo::new(bits, block("minecraft:air"));
        assert_eq!(info.get_height(), bits);
        assert_eq!(info.get_height().to_be_bytes(), bits.to_be_bytes());

        let nan_bits = f32::NAN.to_bits() as i32;
        let info2 = FlatLayerInfo::new(nan_bits, block("minecraft:air"));
        assert_eq!(
            info2.get_height().to_be_bytes(),
            f32::NAN.to_bits().to_be_bytes()
        );
    }
}
