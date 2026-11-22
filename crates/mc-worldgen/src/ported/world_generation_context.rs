/// Minimal interface for `net.minecraft.world.level.chunk.ChunkGenerator`.
///
/// This local trait keeps the port self-contained without requiring the full
/// chunk generator implementation.
pub trait ChunkGenerator {
    /// Corresponds to Java `ChunkGenerator#getMinY()`.
    fn get_min_y(&self) -> i32;

    /// Corresponds to Java `ChunkGenerator#getGenDepth()`.
    fn get_gen_depth(&self) -> i32;
}

/// Minimal interface for `net.minecraft.world.level.LevelHeightAccessor`.
///
/// This local trait keeps the port self-contained without requiring the full
/// level height accessor implementation.
pub trait LevelHeightAccessor {
    /// Corresponds to Java `LevelHeightAccessor#getMinY()`.
    fn get_min_y(&self) -> i32;

    /// Corresponds to Java `LevelHeightAccessor#getHeight()`.
    fn get_height(&self) -> i32;
}

/// Port of `net.minecraft.world.level.levelgen.WorldGenerationContext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationContext {
    min_y: i32,
    height: i32,
}

impl WorldGenerationContext {
    // JAVA: net.minecraft.world.level.levelgen.WorldGenerationContext#<init>
    pub fn new(generator: &impl ChunkGenerator, level: &impl LevelHeightAccessor) -> Self {
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

    struct MockGenerator {
        min_y: i32,
        gen_depth: i32,
    }

    impl ChunkGenerator for MockGenerator {
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

    impl LevelHeightAccessor for MockLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_height(&self) -> i32 {
            self.height
        }
    }

    #[test]
    fn selects_max_min_y_and_min_height() {
        let generator = MockGenerator {
            min_y: -64,
            gen_depth: 384,
        };
        let level = MockLevel {
            min_y: 0,
            height: 256,
        };

        let ctx = WorldGenerationContext::new(&generator, &level);

        assert_eq!(ctx.get_min_gen_y(), 0);
        assert_eq!(ctx.get_gen_depth(), 256);
    }

    #[test]
    fn generator_can_constrain_bounds() {
        let generator = MockGenerator {
            min_y: -64,
            gen_depth: 128,
        };
        let level = MockLevel {
            min_y: -128,
            height: 384,
        };

        let ctx = WorldGenerationContext::new(&generator, &level);

        assert_eq!(ctx.get_min_gen_y(), -64);
        assert_eq!(ctx.get_gen_depth(), 128);
    }

    #[test]
    fn negative_coordinates_are_preserved() {
        let generator = MockGenerator {
            min_y: -2048,
            gen_depth: 4096,
        };
        let level = MockLevel {
            min_y: -1024,
            height: 2048,
        };

        let ctx = WorldGenerationContext::new(&generator, &level);

        assert_eq!(ctx.get_min_gen_y(), -1024);
        assert_eq!(ctx.get_gen_depth(), 2048);
    }

    #[test]
    fn extreme_i32_bounds_match_java_int_comparisons() {
        let generator = MockGenerator {
            min_y: i32::MIN,
            gen_depth: i32::MAX,
        };
        let level = MockLevel {
            min_y: i32::MAX,
            height: i32::MIN,
        };

        let ctx = WorldGenerationContext::new(&generator, &level);

        assert_eq!(ctx.get_min_gen_y(), i32::MAX);
        assert_eq!(ctx.get_gen_depth(), i32::MIN);
    }

    #[test]
    fn equal_bounds_are_kept() {
        let generator = MockGenerator {
            min_y: 16,
            gen_depth: 256,
        };
        let level = MockLevel {
            min_y: 16,
            height: 256,
        };

        let ctx = WorldGenerationContext::new(&generator, &level);

        assert_eq!(ctx.get_min_gen_y(), 16);
        assert_eq!(ctx.get_gen_depth(), 256);
    }
}
