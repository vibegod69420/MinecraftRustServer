/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
/// Kept local so this module is self-contained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState(pub i32);

pub trait BlockColumn {
    // JAVA: net.minecraft.world.level.chunk.BlockColumn#getBlock
    fn get_block(&self, var1: i32) -> BlockState;

    // JAVA: net.minecraft.world.level.chunk.BlockColumn#setBlock
    fn set_block(&mut self, var1: i32, var2: BlockState);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockColumn {
        blocks: Vec<BlockState>,
    }

    impl MockColumn {
        fn new(size: usize) -> Self {
            Self {
                blocks: vec![BlockState(0); size],
            }
        }
    }

    impl BlockColumn for MockColumn {
        fn get_block(&self, var1: i32) -> BlockState {
            let idx = var1 as usize;
            self.blocks[idx]
        }

        fn set_block(&mut self, var1: i32, var2: BlockState) {
            let idx = var1 as usize;
            self.blocks[idx] = var2;
        }
    }

    #[test]
    fn set_and_get_block() {
        let mut col = MockColumn::new(16);
        col.set_block(0, BlockState(42));
        col.set_block(15, BlockState(-7));
        assert_eq!(col.get_block(0), BlockState(42));
        assert_eq!(col.get_block(15), BlockState(-7));
    }

    #[test]
    fn overwrite_block() {
        let mut col = MockColumn::new(4);
        col.set_block(2, BlockState(10));
        assert_eq!(col.get_block(2), BlockState(10));
        col.set_block(2, BlockState(20));
        assert_eq!(col.get_block(2), BlockState(20));
    }

    #[test]
    fn default_block_state() {
        let col = MockColumn::new(4);
        assert_eq!(col.get_block(3), BlockState(0));
        assert_eq!(BlockState::default(), BlockState(0));
    }

    #[test]
    fn extreme_block_state_values() {
        let mut col = MockColumn::new(2);
        col.set_block(0, BlockState(i32::MIN));
        col.set_block(1, BlockState(i32::MAX));
        assert_eq!(col.get_block(0), BlockState(i32::MIN));
        assert_eq!(col.get_block(1), BlockState(i32::MAX));
    }
}
