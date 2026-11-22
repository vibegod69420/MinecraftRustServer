#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationContext {
    min_y: i32,
    height: i32,
}

pub trait ChunkGeneratorLike {
    fn get_min_y(&self) -> i32;
    fn get_gen_depth(&self) -> i32;
}

pub trait LevelHeightAccessorLike {
    fn get_min_y(&self) -> i32;
    fn get_height(&self) -> i32;
}

impl WorldGenerationContext {
    // JAVA: net.minecraft.world.level.levelgen.WorldGenerationContext#<init>
    pub fn new<G: ChunkGeneratorLike, L: LevelHeightAccessorLike>(generator: &G, level: &L) -> Self {
        let min_y = level.get_min_y().max(generator.get_min_y());
        let height = level.get_height().min(generator.get_gen_depth());
        Self { min_y, height }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenerationContext#getMinGenY
    pub fn get_min_gen_y(&self) -> i32 {
        self.min_y
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenerationContext#getGenDepth
    pub fn get_gen_depth(&self) -> i32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGen {
        min_y: i32,
        gen_depth: i32,
    }

    impl ChunkGeneratorLike for MockGen {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_gen_depth(&self) -> i32 {
            self.gen_depth
        }
    }

    struct MockLevel {
        min_y: i32,
        height: i32,
    }

    impl LevelHeightAccessorLike for MockLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_height(&self) -> i32 {
            self.height
        }
    }

    #[test]
    fn uses_level_min_y_and_level_height_when_more_restrictive() {
        let gen = MockGen {
            min_y: -64,
            gen_depth: 384,
        };
        let level = MockLevel {
            min_y: 0,
            height: 256,
        };
        let ctx = WorldGenerationContext::new(&gen, &level);
        assert_eq!(ctx.get_min_gen_y(), 0);
        assert_eq!(ctx.get_gen_depth(), 256);
    }

    #[test]
    fn uses_generator_min_y_and_gen_depth_when_more_restrictive() {
        let gen = MockGen {
            min_y: -32,
            gen_depth: 128,
        };
        let level = MockLevel {
            min_y: -64,
            height: 384,
        };
        let ctx = WorldGenerationContext::new(&gen, &level);
        assert_eq!(ctx.get_min_gen_y(), -32);
        assert_eq!(ctx.get_gen_depth(), 128);
    }

    #[test]
    fn handles_negative_values() {
        let gen = MockGen {
            min_y: -128,
            gen_depth: -16,
        };
        let level = MockLevel {
            min_y: -64,
            height: -32,
        };
        let ctx = WorldGenerationContext::new(&gen, &level);
        assert_eq!(ctx.get_min_gen_y(), -64);
        assert_eq!(ctx.get_gen_depth(), -32);
    }

    #[test]
    fn handles_equal_values() {
        let gen = MockGen {
            min_y: 16,
            gen_depth: 256,
        };
        let level = MockLevel {
            min_y: 16,
            height: 256,
        };
        let ctx = WorldGenerationContext::new(&gen, &level);
        assert_eq!(ctx.get_min_gen_y(), 16);
        assert_eq!(ctx.get_gen_depth(), 256);
    }

    #[test]
    fn handles_extreme_i32_values() {
        let gen = MockGen {
            min_y: i32::MIN,
            gen_depth: i32::MAX,
        };
        let level = MockLevel {
            min_y: i32::MIN + 1,
            height: i32::MAX - 1,
        };
        let ctx = WorldGenerationContext::new(&gen, &level);
        assert_eq!(ctx.get_min_gen_y(), i32::MIN + 1);
        assert_eq!(ctx.get_gen_depth(), i32::MAX - 1);
    }
}
