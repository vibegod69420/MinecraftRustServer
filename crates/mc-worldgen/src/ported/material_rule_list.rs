/// Minimal stub for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState(pub u32);

/// Minimal stub for `net.minecraft.world.level.levelgen.DensityFunction.FunctionContext`.
pub trait FunctionContext {
    fn block_x(&self) -> i32;
    fn block_y(&self) -> i32;
    fn block_z(&self) -> i32;
}

/// Minimal stub for `net.minecraft.world.level.levelgen.NoiseChunk.BlockStateFiller`.
pub trait BlockStateFiller {
    fn calculate(&self, ctx: &dyn FunctionContext) -> Option<BlockState>;
}

/// Port of `net.minecraft.world.level.levelgen.material.MaterialRuleList`.
pub struct MaterialRuleList {
    pub material_rule_list: Vec<Box<dyn BlockStateFiller>>,
}

impl BlockStateFiller for MaterialRuleList {
    // JAVA: net.minecraft.world.level.levelgen.material.MaterialRuleList#calculate
    fn calculate(&self, ctx: &dyn FunctionContext) -> Option<BlockState> {
        for filler in &self.material_rule_list {
            let state = filler.calculate(ctx);
            if state.is_some() {
                return state;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCtx {
        x: i32,
        y: i32,
        z: i32,
    }

    impl FunctionContext for TestCtx {
        fn block_x(&self) -> i32 {
            self.x
        }

        fn block_y(&self) -> i32 {
            self.y
        }

        fn block_z(&self) -> i32 {
            self.z
        }
    }

    struct ConstFiller(Option<BlockState>);

    impl BlockStateFiller for ConstFiller {
        fn calculate(&self, _ctx: &dyn FunctionContext) -> Option<BlockState> {
            self.0
        }
    }

    struct MatchYFiller {
        y: i32,
        state: BlockState,
    }

    impl BlockStateFiller for MatchYFiller {
        fn calculate(&self, ctx: &dyn FunctionContext) -> Option<BlockState> {
            if ctx.block_y() == self.y {
                Some(self.state)
            } else {
                None
            }
        }
    }

    #[test]
    fn empty_list_returns_none() {
        let list = MaterialRuleList {
            material_rule_list: Vec::new(),
        };
        let ctx = TestCtx { x: 0, y: 0, z: 0 };
        assert_eq!(list.calculate(&ctx), None);
    }

    #[test]
    fn returns_first_non_null() {
        let list = MaterialRuleList {
            material_rule_list: vec![
                Box::new(ConstFiller(None)),
                Box::new(ConstFiller(Some(BlockState(1)))),
                Box::new(ConstFiller(Some(BlockState(2)))),
            ],
        };
        let ctx = TestCtx { x: 1, y: 2, z: 3 };
        assert_eq!(list.calculate(&ctx), Some(BlockState(1)));
    }

    #[test]
    fn all_null_returns_none_with_negative_coords() {
        let list = MaterialRuleList {
            material_rule_list: vec![
                Box::new(ConstFiller(None)),
                Box::new(ConstFiller(None)),
            ],
        };
        let ctx = TestCtx {
            x: -1,
            y: -64,
            z: i32::MIN,
        };
        assert_eq!(list.calculate(&ctx), None);
    }

    #[test]
    fn passes_context_with_extreme_and_float_bit_coords() {
        let bits = f32::NAN.to_bits() as i32;
        let list = MaterialRuleList {
            material_rule_list: vec![Box::new(MatchYFiller {
                y: bits,
                state: BlockState(7),
            })],
        };
        let ctx = TestCtx {
            x: i32::MAX,
            y: bits,
            z: i32::MIN,
        };
        assert_eq!(list.calculate(&ctx), Some(BlockState(7)));

        let ctx2 = TestCtx {
            x: 0,
            y: i32::MAX,
            z: 0,
        };
        assert_eq!(list.calculate(&ctx2), None);
    }

    #[test]
    fn negative_coords_passed_through() {
        let list = MaterialRuleList {
            material_rule_list: vec![Box::new(MatchYFiller {
                y: -320,
                state: BlockState(9),
            })],
        };
        let ctx = TestCtx {
            x: -100,
            y: -320,
            z: -200,
        };
        assert_eq!(list.calculate(&ctx), Some(BlockState(9)));
    }
}
