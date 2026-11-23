/// Port of `net.minecraft.world.level.chunk.status.WorldGenContext`.
///
/// Java record ported as a generic struct for self-contained compilation.
/// Type parameters map to:
/// `L`  = ServerLevel
/// `G`  = ChunkGenerator
/// `S`  = StructureTemplateManager
/// `LE` = ThreadedLevelLightEngine
/// `E`  = Executor
/// `U`  = LevelChunk.UnsavedListener
#[derive(Debug, Clone)]
pub struct WorldGenContext<L, G, S, LE, E, U> {
    pub level: L,
    pub generator: G,
    pub structure_manager: S,
    pub light_engine: LE,
    pub main_thread_executor: E,
    pub unsaved_listener: U,
}

impl<L, G, S, LE, E, U> WorldGenContext<L, G, S, LE, E, U> {
    // JAVA: net.minecraft.world.level.chunk.status.WorldGenContext#<init>
    pub fn new(
        level: L,
        generator: G,
        structure_manager: S,
        light_engine: LE,
        main_thread_executor: E,
        unsaved_listener: U,
    ) -> Self {
        Self {
            level,
            generator,
            structure_manager,
            light_engine,
            main_thread_executor,
            unsaved_listener,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let ctx = WorldGenContext::new(1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32);
        assert_eq!(ctx.level, 1);
        assert_eq!(ctx.generator, 2);
        assert_eq!(ctx.structure_manager, 3);
        assert_eq!(ctx.light_engine, 4);
        assert_eq!(ctx.main_thread_executor, 5);
        assert_eq!(ctx.unsaved_listener, 6);
    }

    #[test]
    fn fields_are_publicly_mutable() {
        let mut ctx = WorldGenContext::new("level", "gen", "sm", "le", "exec", "ul");
        ctx.level = "level2";
        ctx.unsaved_listener = "ul2";
        assert_eq!(ctx.level, "level2");
        assert_eq!(ctx.generator, "gen");
        assert_eq!(ctx.unsaved_listener, "ul2");
    }

    #[test]
    fn clone_preserves_values() {
        let ctx = WorldGenContext::new(10_u64, 20_u64, 30_u64, 40_u64, 50_u64, 60_u64);
        let cloned = ctx.clone();
        assert_eq!(cloned.level, ctx.level);
        assert_eq!(cloned.generator, ctx.generator);
        assert_eq!(cloned.structure_manager, ctx.structure_manager);
        assert_eq!(cloned.light_engine, ctx.light_engine);
        assert_eq!(cloned.main_thread_executor, ctx.main_thread_executor);
        assert_eq!(cloned.unsaved_listener, ctx.unsaved_listener);
    }

    #[test]
    fn supports_mixed_types_and_negative_values() {
        let ctx = WorldGenContext::new(vec![1_i32], Some(-2_i32), "s", -4.5_f64, true, -1_i8);
        assert_eq!(ctx.level, vec![1_i32]);
        assert_eq!(ctx.generator, Some(-2_i32));
        assert_eq!(ctx.structure_manager, "s");
        assert_eq!(ctx.light_engine.to_bits(), (-4.5_f64).to_bits());
        assert!(ctx.main_thread_executor);
        assert_eq!(ctx.unsaved_listener, -1_i8);
    }
}
