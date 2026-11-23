use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biome {
    pub id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryAccess;

#[derive(Debug, Clone, Default)]
pub struct NoiseChunk;

#[derive(Debug, Clone, Default)]
pub struct ChunkAccess;

#[derive(Debug, Clone, Default)]
pub struct NoiseBasedChunkGenerator;

#[derive(Debug, Clone, Default)]
pub struct LevelHeightAccessor;

#[derive(Debug, Clone, Default)]
pub struct SurfaceRuleSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopMaterialCall {
    pub pos: BlockPos,
    pub above_surface: bool,
}

#[derive(Debug, Default)]
pub struct SurfaceSystem {
    pub result: Option<BlockState>,
    pub last_call: RefCell<Option<TopMaterialCall>>,
}

impl SurfaceSystem {
    pub fn top_material<F>(
        &self,
        _surface_rule: &SurfaceRuleSource,
        _context: &CarvingContext,
        _biome_at: F,
        _chunk: &ChunkAccess,
        _noise_chunk: &NoiseChunk,
        pos: &BlockPos,
        above_surface: bool,
    ) -> Option<BlockState>
    where
        F: Fn(&BlockPos) -> Holder<Biome>,
    {
        *self.last_call.borrow_mut() = Some(TopMaterialCall {
            pos: pos.clone(),
            above_surface,
        });
        self.result.clone()
    }
}

#[derive(Debug, Default)]
pub struct RandomState {
    surface_system: SurfaceSystem,
}

impl RandomState {
    pub fn new(surface_system: SurfaceSystem) -> Self {
        Self { surface_system }
    }

    pub fn surface_system(&self) -> &SurfaceSystem {
        &self.surface_system
    }
}

pub struct WorldGenerationContext {
    _generator: NoiseBasedChunkGenerator,
    _height_accessor: LevelHeightAccessor,
}

impl WorldGenerationContext {
    pub fn new(generator: NoiseBasedChunkGenerator, height_accessor: LevelHeightAccessor) -> Self {
        Self {
            _generator: generator,
            _height_accessor: height_accessor,
        }
    }
}

pub struct CarvingContext {
    _world_generation_context: WorldGenerationContext,
    registry_access: RegistryAccess,
    noise_chunk: NoiseChunk,
    random_state: RandomState,
    surface_rule: SurfaceRuleSource,
}

impl CarvingContext {
    // JAVA: net.minecraft.world.level.levelgen.carver.CarvingContext#<init>
    pub fn new(
        generator: NoiseBasedChunkGenerator,
        registry_access: RegistryAccess,
        height_accessor: LevelHeightAccessor,
        noise_chunk: NoiseChunk,
        random_state: RandomState,
        surface_rule: SurfaceRuleSource,
    ) -> Self {
        Self {
            _world_generation_context: WorldGenerationContext::new(generator, height_accessor),
            registry_access,
            noise_chunk,
            random_state,
            surface_rule,
        }
    }

    #[deprecated(note = "Matches Java @Deprecated")]
    // JAVA: net.minecraft.world.level.levelgen.carver.CarvingContext#topMaterial
    pub fn top_material<F>(
        &self,
        biome_at: F,
        chunk: &ChunkAccess,
        pos: &BlockPos,
        above_surface: bool,
    ) -> Option<BlockState>
    where
        F: Fn(&BlockPos) -> Holder<Biome>,
    {
        self.random_state.surface_system().top_material(
            &self.surface_rule,
            self,
            biome_at,
            chunk,
            &self.noise_chunk,
            pos,
            above_surface,
        )
    }

    #[deprecated(note = "Matches Java @Deprecated")]
    // JAVA: net.minecraft.world.level.levelgen.carver.CarvingContext#registryAccess
    pub fn registry_access(&self) -> &RegistryAccess {
        &self.registry_access
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CarvingContext#randomState
    pub fn random_state(&self) -> &RandomState {
        &self.random_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(result: Option<BlockState>) -> CarvingContext {
        let ss = SurfaceSystem {
            result,
            last_call: RefCell::new(None),
        };
        CarvingContext::new(
            NoiseBasedChunkGenerator,
            RegistryAccess,
            LevelHeightAccessor,
            NoiseChunk,
            RandomState::new(ss),
            SurfaceRuleSource,
        )
    }

    #[test]
    #[allow(deprecated)]
    fn registry_access_and_random_state_accessors() {
        let ctx = make_context(None);
        let _ra: &RegistryAccess = ctx.registry_access();
        let _rs: &RandomState = ctx.random_state();
    }

    #[test]
    #[allow(deprecated)]
    fn top_material_delegates_none() {
        let ctx = make_context(None);
        let pos = BlockPos {
            x: 0,
            y: 64,
            z: 0,
        };
        let chunk = ChunkAccess;
        let out = ctx.top_material(|_| Holder(Biome { id: 1 }), &chunk, &pos, false);
        assert_eq!(out, None);
    }

    #[test]
    #[allow(deprecated)]
    fn top_material_delegates_some_and_captures_args() {
        let ctx = make_context(Some(BlockState { id: 42 }));
        let pos = BlockPos {
            x: -7,
            y: -64,
            z: 12_345,
        };
        let chunk = ChunkAccess;
        let out = ctx.top_material(|p| Holder(Biome { id: p.x as u32 }), &chunk, &pos, true);
        assert_eq!(out, Some(BlockState { id: 42 }));
        let call = ctx
            .random_state()
            .surface_system()
            .last_call
            .borrow()
            .clone();
        assert_eq!(
            call,
            Some(TopMaterialCall {
                pos: BlockPos {
                    x: -7,
                    y: -64,
                    z: 12_345,
                },
                above_surface: true,
            })
        );
    }

    #[test]
    #[allow(deprecated)]
    fn top_material_negative_and_extreme_coordinates() {
        let ctx = make_context(Some(BlockState { id: 7 }));
        let pos = BlockPos {
            x: i32::MIN,
            y: i32::MAX,
            z: -1,
        };
        let chunk = ChunkAccess;
        let out = ctx.top_material(|_| Holder(Biome { id: 0 }), &chunk, &pos, false);
        assert_eq!(out, Some(BlockState { id: 7 }));
        let call = ctx
            .random_state()
            .surface_system()
            .last_call
            .borrow()
            .clone()
            .unwrap();
        assert_eq!(call.pos.x, i32::MIN);
        assert_eq!(call.pos.y, i32::MAX);
        assert_eq!(call.pos.z, -1);
        assert!(!call.above_surface);
    }
}
