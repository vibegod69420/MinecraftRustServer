//! Port of `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn above(&self) -> Self {
        self.above_n(1)
    }

    pub fn above_n(&self, n: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(n),
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockStateProperty {
    Waterlogged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    is_air: bool,
    can_be_replaced_by_trees: bool,
    waterlogged: Option<bool>,
}

impl BlockState {
    pub fn new(is_air: bool, can_be_replaced_by_trees: bool, waterlogged: Option<bool>) -> Self {
        Self {
            is_air,
            can_be_replaced_by_trees,
            waterlogged,
        }
    }

    pub fn air() -> Self {
        Self::new(true, false, None)
    }

    pub fn is_air(&self) -> bool {
        self.is_air
    }

    pub fn has_property(&self, prop: BlockStateProperty) -> bool {
        match prop {
            BlockStateProperty::Waterlogged => self.waterlogged.is_some(),
        }
    }

    pub fn set_value(&self, prop: BlockStateProperty, value: bool) -> Self {
        let mut out = self.clone();
        match prop {
            BlockStateProperty::Waterlogged => {
                if out.waterlogged.is_some() {
                    out.waterlogged = Some(value);
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FluidState {
    pub is_water: bool,
}

impl FluidState {
    pub fn is_water_tag(&self) -> bool {
        self.is_water
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

pub trait IntProvider {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

pub trait BlockStateProvider {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

pub struct AboveRootPlacement {
    pub above_root_provider: Box<dyn BlockStateProvider>,
    pub above_root_placement_chance: f32,
}

impl fmt::Debug for AboveRootPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AboveRootPlacement")
            .field("above_root_placement_chance", &self.above_root_placement_chance)
            .finish_non_exhaustive()
    }
}

pub trait LevelSimulatedReader {
    fn is_state_at_position(&self, pos: BlockPos, predicate: &dyn Fn(&BlockState) -> bool) -> bool;
    fn is_fluid_at_position(&self, pos: BlockPos, predicate: &dyn Fn(&FluidState) -> bool) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct TreeConfiguration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RootPlacerType(pub i32);

pub struct TreeFeature;

impl TreeFeature {
    /// Minimal self-contained equivalent used by `RootPlacer#canPlaceRoot`.
    pub fn valid_tree_pos(level: &dyn LevelSimulatedReader, pos: BlockPos) -> bool {
        level.is_state_at_position(pos, &|state: &BlockState| {
            state.is_air() || state.can_be_replaced_by_trees
        })
    }
}

pub struct RootPlacerBase {
    pub trunk_offset_y: Box<dyn IntProvider>,
    pub root_provider: Box<dyn BlockStateProvider>,
    pub above_root_placement: Option<AboveRootPlacement>,
}

impl RootPlacerBase {
    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#<init>
    pub fn new(
        trunk_offset_y: Box<dyn IntProvider>,
        root_provider: Box<dyn BlockStateProvider>,
        above_root_placement: Option<AboveRootPlacement>,
    ) -> Self {
        Self {
            trunk_offset_y,
            root_provider,
            above_root_placement,
        }
    }
}

pub trait RootPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#type
    fn placer_type(&self) -> RootPlacerType;

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#placeRoots
    fn place_roots(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        trunk_origin: BlockPos,
        tree_origin: BlockPos,
        config: &TreeConfiguration,
    ) -> bool;

    fn trunk_offset_y(&self) -> &dyn IntProvider;
    fn root_provider(&self) -> &dyn BlockStateProvider;
    fn above_root_placement(&self) -> Option<&AboveRootPlacement>;

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#canPlaceRoot
    fn can_place_root(&self, level: &dyn LevelSimulatedReader, pos: BlockPos) -> bool {
        TreeFeature::valid_tree_pos(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#placeRoot
    fn place_root(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        pos: BlockPos,
        _config: &TreeConfiguration,
    ) {
        if self.can_place_root(level, pos) {
            let root_state = self.root_provider().get_state(random, pos);
            let root_state = self.get_potentially_waterlogged_state(level, pos, root_state);
            block_setter(pos, root_state);

            if let Some(above) = self.above_root_placement() {
                let above_pos = pos.above();
                if random.next_float() < above.above_root_placement_chance
                    && level.is_state_at_position(above_pos, &|s: &BlockState| s.is_air())
                {
                    let above_state = above.above_root_provider.get_state(random, above_pos);
                    let above_state =
                        self.get_potentially_waterlogged_state(level, above_pos, above_state);
                    block_setter(above_pos, above_state);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#getPotentiallyWaterloggedState
    fn get_potentially_waterlogged_state(
        &self,
        level: &dyn LevelSimulatedReader,
        pos: BlockPos,
        state: BlockState,
    ) -> BlockState {
        if state.has_property(BlockStateProperty::Waterlogged) {
            let water = level.is_fluid_at_position(pos, &|f: &FluidState| f.is_water_tag());
            state.set_value(BlockStateProperty::Waterlogged, water)
        } else {
            state
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer#getTrunkOrigin
    fn get_trunk_origin(&self, pos: BlockPos, random: &mut dyn RandomSource) -> BlockPos {
        pos.above_n(self.trunk_offset_y().sample(random))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FixedInt(i32);
    impl IntProvider for FixedInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct FixedState(BlockState);
    impl BlockStateProvider for FixedState {
        fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
            self.0.clone()
        }
    }

    #[derive(Default)]
    struct SeqRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        fi: usize,
        ii: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>) -> Self {
            Self {
                floats,
                ints: Vec::new(),
                fi: 0,
                ii: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.ints.get(self.ii).copied().unwrap_or(0);
            self.ii = self.ii.wrapping_add(1);
            v
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.fi).copied().unwrap_or(0.0);
            self.fi = self.fi.wrapping_add(1);
            v
        }
    }

    #[derive(Default)]
    struct MockLevel {
        states: HashMap<BlockPos, BlockState>,
        fluids: HashMap<BlockPos, FluidState>,
    }

    impl LevelSimulatedReader for MockLevel {
        fn is_state_at_position(
            &self,
            pos: BlockPos,
            predicate: &dyn Fn(&BlockState) -> bool,
        ) -> bool {
            let state = self.states.get(&pos).cloned().unwrap_or_else(BlockState::air);
            predicate(&state)
        }

        fn is_fluid_at_position(
            &self,
            pos: BlockPos,
            predicate: &dyn Fn(&FluidState) -> bool,
        ) -> bool {
            let fluid = self.fluids.get(&pos).copied().unwrap_or_default();
            predicate(&fluid)
        }
    }

    struct TestPlacer {
        base: RootPlacerBase,
    }

    impl TestPlacer {
        fn new(offset: i32, above: Option<AboveRootPlacement>) -> Self {
            Self {
                base: RootPlacerBase::new(
                    Box::new(FixedInt(offset)),
                    Box::new(FixedState(BlockState::new(false, false, Some(false)))),
                    above,
                ),
            }
        }
    }

    impl RootPlacer for TestPlacer {
        fn placer_type(&self) -> RootPlacerType {
            RootPlacerType(0)
        }

        fn place_roots(
            &self,
            _level: &dyn LevelSimulatedReader,
            _block_setter: &mut dyn FnMut(BlockPos, BlockState),
            _random: &mut dyn RandomSource,
            _trunk_origin: BlockPos,
            _tree_origin: BlockPos,
            _config: &TreeConfiguration,
        ) -> bool {
            false
        }

        fn trunk_offset_y(&self) -> &dyn IntProvider {
            self.base.trunk_offset_y.as_ref()
        }

        fn root_provider(&self) -> &dyn BlockStateProvider {
            self.base.root_provider.as_ref()
        }

        fn above_root_placement(&self) -> Option<&AboveRootPlacement> {
            self.base.above_root_placement.as_ref()
        }
    }

    #[test]
    fn trunk_origin_wrapping_and_negative() {
        let placer = TestPlacer::new(1, None);
        let mut rng = SeqRandom::default();
        let pos = BlockPos::new(10, i32::MAX, -20);
        let out = placer.get_trunk_origin(pos, &mut rng);
        assert_eq!(out, BlockPos::new(10, i32::MIN, -20));

        let placer_neg = TestPlacer::new(-5, None);
        let out_neg = placer_neg.get_trunk_origin(BlockPos::new(0, 0, 0), &mut rng);
        assert_eq!(out_neg, BlockPos::new(0, -5, 0));
    }

    #[test]
    fn potentially_waterlogged_state() {
        let placer = TestPlacer::new(0, None);
        let mut level = MockLevel::default();
        let pos = BlockPos::new(1, 2, 3);
        level.fluids.insert(pos, FluidState { is_water: true });

        let waterloggable = BlockState::new(false, false, Some(false));
        let out = placer.get_potentially_waterlogged_state(&level, pos, waterloggable.clone());
        assert_eq!(out, BlockState::new(false, false, Some(true)));

        let no_water_pos = BlockPos::new(5, 5, 5);
        let out2 =
            placer.get_potentially_waterlogged_state(&level, no_water_pos, waterloggable);
        assert_eq!(out2, BlockState::new(false, false, Some(false)));

        let plain = BlockState::new(false, false, None);
        let out3 = placer.get_potentially_waterlogged_state(&level, pos, plain.clone());
        assert_eq!(out3, plain);
    }

    #[test]
    fn can_place_root_delegates_to_valid_tree_pos() {
        let placer = TestPlacer::new(0, None);
        let mut level = MockLevel::default();
        let air_pos = BlockPos::new(0, 0, 0);
        level.states.insert(air_pos, BlockState::air());
        assert!(placer.can_place_root(&level, air_pos));

        let replaceable_pos = BlockPos::new(1, 0, 0);
        level
            .states
            .insert(replaceable_pos, BlockState::new(false, true, None));
        assert!(placer.can_place_root(&level, replaceable_pos));

        let solid_pos = BlockPos::new(2, 0, 0);
        level
            .states
            .insert(solid_pos, BlockState::new(false, false, None));
        assert!(!placer.can_place_root(&level, solid_pos));
    }

    #[test]
    fn place_root_places_and_above_root_chance_boundary() {
        let above = AboveRootPlacement {
            above_root_provider: Box::new(FixedState(BlockState::new(false, false, Some(false)))),
            above_root_placement_chance: f32::from_bits(0x3f00_0000), // 0.5
        };
        let placer = TestPlacer::new(0, Some(above));
        let mut level = MockLevel::default();
        let root_pos = BlockPos::new(0, 0, 0);
        level.states.insert(root_pos, BlockState::air());
        level.states.insert(root_pos.above(), BlockState::air());

        // Exactly 0.5 < 0.5 is false: no above root placement.
        let mut rng = SeqRandom::new(vec![0.5_f32]);
        let mut placed = Vec::new();
        placer.place_root(
            &level,
            &mut |p, s| placed.push((p, s)),
            &mut rng,
            root_pos,
            &TreeConfiguration,
        );
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].0, root_pos);

        // Just below 0.5: above root should be placed.
        let mut rng2 = SeqRandom::new(vec![f32::from_bits(0x3eff_ffff)]);
        let mut placed2 = Vec::new();
        placer.place_root(
            &level,
            &mut |p, s| placed2.push((p, s)),
            &mut rng2,
            root_pos,
            &TreeConfiguration,
        );
        assert_eq!(placed2.len(), 2);
        assert_eq!(placed2[0].0, root_pos);
        assert_eq!(placed2[1].0, root_pos.above());
    }

    #[test]
    fn place_root_skips_invalid_position() {
        let above = AboveRootPlacement {
            above_root_provider: Box::new(FixedState(BlockState::air())),
            above_root_placement_chance: 1.0,
        };
        let placer = TestPlacer::new(0, Some(above));
        let mut level = MockLevel::default();
        let root_pos = BlockPos::new(0, 0, 0);
        level
            .states
            .insert(root_pos, BlockState::new(false, false, None)); // solid, invalid

        let mut rng = SeqRandom::new(vec![0.0]);
        let mut placed = Vec::new();
        placer.place_root(
            &level,
            &mut |p, s| placed.push((p, s)),
            &mut rng,
            root_pos,
            &TreeConfiguration,
        );
        assert!(placed.is_empty());
        assert_eq!(rng.fi, 0, "random should not be consumed for invalid root");
    }
}
