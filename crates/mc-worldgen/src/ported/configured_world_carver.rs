use std::cell::Cell;

thread_local! {
    static DEBUG_VOID_TERRAIN: Cell<bool> = Cell::new(false);
}

fn debug_void_terrain(_pos: &ChunkPos) -> bool {
    DEBUG_VOID_TERRAIN.with(|c| c.get())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biome {
    pub id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CarvingContext;

#[derive(Debug, Clone, Default)]
pub struct Aquifer;

#[derive(Debug, Clone, Default)]
pub struct CarvingMask;

#[derive(Debug, Clone)]
pub struct ChunkAccess {
    pos: ChunkPos,
}

impl ChunkAccess {
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos }
    }

    pub fn get_pos(&self) -> ChunkPos {
        self.pos
    }
}

#[derive(Debug, Clone)]
pub struct RandomSource {
    state: u64,
}

impl RandomSource {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.state
    }
}

pub trait WorldCarver<WC> {
    fn is_start_chunk(&self, config: &WC, random: &mut RandomSource) -> bool;
    fn carve(
        &self,
        context: &CarvingContext,
        config: &WC,
        chunk: &mut ChunkAccess,
        biome_at: &dyn Fn(&BlockPos) -> Holder<Biome>,
        random: &mut RandomSource,
        aquifer: &Aquifer,
        chunk_pos: &ChunkPos,
        carving_mask: &mut CarvingMask,
    ) -> bool;
}

pub struct ConfiguredWorldCarver<WC, W: WorldCarver<WC>> {
    world_carver: W,
    config: WC,
}

impl<WC, W: WorldCarver<WC>> ConfiguredWorldCarver<WC, W> {
    // JAVA: net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver#<init>
    pub fn new(world_carver: W, config: WC) -> Self {
        Self {
            world_carver,
            config,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver#worldCarver
    pub fn world_carver(&self) -> &W {
        &self.world_carver
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver#config
    pub fn config(&self) -> &WC {
        &self.config
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver#isStartChunk
    pub fn is_start_chunk(&self, random: &mut RandomSource) -> bool {
        self.world_carver.is_start_chunk(&self.config, random)
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver#carve
    pub fn carve(
        &self,
        context: &CarvingContext,
        chunk: &mut ChunkAccess,
        biome_at: &dyn Fn(&BlockPos) -> Holder<Biome>,
        random: &mut RandomSource,
        aquifer: &Aquifer,
        chunk_pos: &ChunkPos,
        carving_mask: &mut CarvingMask,
    ) -> bool {
        if debug_void_terrain(&chunk.get_pos()) {
            return false;
        }
        self.world_carver.carve(
            context,
            &self.config,
            chunk,
            biome_at,
            random,
            aquifer,
            chunk_pos,
            carving_mask,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockCarver {
        start: bool,
        carve_result: bool,
        last_chunk_pos: RefCell<Option<ChunkPos>>,
    }

    impl WorldCarver<i32> for MockCarver {
        fn is_start_chunk(&self, config: &i32, random: &mut RandomSource) -> bool {
            let _ = random.next_u64();
            self.start && *config != 0
        }

        fn carve(
            &self,
            _context: &CarvingContext,
            _config: &i32,
            _chunk: &mut ChunkAccess,
            _biome_at: &dyn Fn(&BlockPos) -> Holder<Biome>,
            _random: &mut RandomSource,
            _aquifer: &Aquifer,
            chunk_pos: &ChunkPos,
            _carving_mask: &mut CarvingMask,
        ) -> bool {
            *self.last_chunk_pos.borrow_mut() = Some(*chunk_pos);
            self.carve_result
        }
    }

    fn with_debug_void_terrain<F: FnOnce()>(value: bool, f: F) {
        struct Guard(bool);
        impl Drop for Guard {
            fn drop(&mut self) {
                DEBUG_VOID_TERRAIN.with(|c| c.set(self.0));
            }
        }
        let prev = DEBUG_VOID_TERRAIN.with(|c| {
            let p = c.get();
            c.set(value);
            p
        });
        let _guard = Guard(prev);
        f();
    }

    #[test]
    fn is_start_chunk_delegates() {
        let carver = MockCarver {
            start: true,
            carve_result: false,
            last_chunk_pos: RefCell::new(None),
        };
        let cfg = ConfiguredWorldCarver::new(carver, 1_i32);
        let mut rng = RandomSource::new(0);
        assert!(cfg.is_start_chunk(&mut rng));

        let carver2 = MockCarver {
            start: true,
            carve_result: false,
            last_chunk_pos: RefCell::new(None),
        };
        let cfg2 = ConfiguredWorldCarver::new(carver2, 0_i32);
        let mut rng2 = RandomSource::new(0);
        assert!(!cfg2.is_start_chunk(&mut rng2));
    }

    #[test]
    fn carve_delegates_when_not_debug_void() {
        with_debug_void_terrain(false, || {
            let carver = MockCarver {
                start: false,
                carve_result: true,
                last_chunk_pos: RefCell::new(None),
            };
            let cfg = ConfiguredWorldCarver::new(carver, 7_i32);
            let mut chunk = ChunkAccess::new(ChunkPos { x: 1, z: -2 });
            let mut rng = RandomSource::new(1);
            let mut mask = CarvingMask;
            let ctx = CarvingContext;
            let aquifer = Aquifer;
            let cp = ChunkPos { x: 5, z: -9 };
            let ok = cfg.carve(
                &ctx,
                &mut chunk,
                &|_| Holder(Biome { id: 0 }),
                &mut rng,
                &aquifer,
                &cp,
                &mut mask,
            );
            assert!(ok);
            assert_eq!(*cfg.world_carver().last_chunk_pos.borrow(), Some(cp));
        });
    }

    #[test]
    fn carve_returns_false_when_debug_void_terrain() {
        with_debug_void_terrain(true, || {
            let carver = MockCarver {
                start: false,
                carve_result: true,
                last_chunk_pos: RefCell::new(None),
            };
            let cfg = ConfiguredWorldCarver::new(carver, 7_i32);
            let mut chunk = ChunkAccess::new(ChunkPos {
                x: i32::MIN,
                z: i32::MAX,
            });
            let mut rng = RandomSource::new(2);
            let mut mask = CarvingMask;
            let ctx = CarvingContext;
            let aquifer = Aquifer;
            let cp = ChunkPos { x: 0, z: 0 };
            let ok = cfg.carve(
                &ctx,
                &mut chunk,
                &|_| Holder(Biome { id: 0 }),
                &mut rng,
                &aquifer,
                &cp,
                &mut mask,
            );
            assert!(!ok);
            assert_eq!(*cfg.world_carver().last_chunk_pos.borrow(), None);
        });
    }

    #[test]
    fn accessors_return_components() {
        let carver = MockCarver {
            start: false,
            carve_result: false,
            last_chunk_pos: RefCell::new(None),
        };
        let cfg = ConfiguredWorldCarver::new(carver, -123_i32);
        assert_eq!(*cfg.config(), -123_i32);
        assert!(!cfg.world_carver().start);
    }
}
