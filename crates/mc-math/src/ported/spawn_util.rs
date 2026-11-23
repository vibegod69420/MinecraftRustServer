use crate::random::RandomSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn mutable(self) -> Self {
        self
    }

    pub fn set_with_offset(&mut self, base: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = base.x.wrapping_add(dx);
        self.y = base.y.wrapping_add(dy);
        self.z = base.z.wrapping_add(dz);
        self
    }

    pub fn set_with_offset_dir(&mut self, base: BlockPos, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.normal();
        self.set_with_offset(base, dx, dy, dz)
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.normal();
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    pub fn set(&mut self, other: BlockPos) -> &mut Self {
        *self = other;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct EntitySpawnReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpawnBlock {
    Cobweb,
    Cactus,
    GlassPane,
    Conduit,
    Ice,
    Tnt,
    Glowstone,
    Beacon,
    SeaLantern,
    FrostedIce,
    TintedGlass,
    Glass,
    PowderSnow,
}

pub trait SpawnBlockState {
    fn is_air(&self) -> bool;
    fn liquid(&self) -> bool;
    fn is_solid(&self) -> bool;
    fn is_block(&self, block: SpawnBlock) -> bool;
    fn is_stained_glass_pane_block(&self) -> bool;
    fn is_stained_glass_block(&self) -> bool;
    fn is_leaves_block(&self) -> bool;
    fn is_in_leaves_tag(&self) -> bool;
}

pub trait SpawnLevel {
    type State: SpawnBlockState;

    fn random_source(&mut self) -> &mut dyn RandomSource;
    fn world_border_is_within_bounds(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> Self::State;
    fn no_collision_for_aabb(&self, aabb: &Aabb) -> bool;
    fn collision_shape_is_empty(&self, pos: BlockPos, state: &Self::State) -> bool;
    fn is_face_full_up(&self, pos: BlockPos, state: &Self::State) -> bool;
    fn add_fresh_entity_with_passengers<M>(&mut self, mob: &mut M);
}

pub trait SpawnMob<L: SpawnLevel + ?Sized> {
    fn check_spawn_rules(&self, level: &L, reason: EntitySpawnReason) -> bool;
    fn check_spawn_obstruction(&self, level: &L) -> bool;
    fn play_ambient_sound(&mut self);
    fn discard(&mut self);
}

pub trait SpawnEntityType<L: SpawnLevel + ?Sized> {
    type Mob: SpawnMob<L>;

    fn get_spawn_aabb(&self, x: f64, y: f64, z: f64) -> Aabb;
    fn create(&self, level: &mut L, pos: BlockPos, reason: EntitySpawnReason) -> Option<Self::Mob>;
}

pub trait Strategy<L: SpawnLevel + ?Sized> {
    // JAVA: net.minecraft.util.SpawnUtil$Strategy#canSpawnOn
    fn can_spawn_on(
        &self,
        level: &L,
        pos: BlockPos,
        state: &L::State,
        above_pos: BlockPos,
        above_state: &L::State,
    ) -> bool;
}

#[deprecated]
#[derive(Clone, Copy, Debug, Default)]
pub struct LegacyIronGolem;

#[allow(deprecated)]
impl<L: SpawnLevel + ?Sized> Strategy<L> for LegacyIronGolem {
    // JAVA: net.minecraft.util.SpawnUtil$Strategy#LEGACY_IRON_GOLEM
    fn can_spawn_on(
        &self,
        _level: &L,
        _pos: BlockPos,
        state: &L::State,
        _above_pos: BlockPos,
        above_state: &L::State,
    ) -> bool {
        let blocked = state.is_block(SpawnBlock::Cobweb)
            || state.is_block(SpawnBlock::Cactus)
            || state.is_block(SpawnBlock::GlassPane)
            || state.is_stained_glass_pane_block()
            || state.is_stained_glass_block()
            || state.is_leaves_block()
            || state.is_block(SpawnBlock::Conduit)
            || state.is_block(SpawnBlock::Ice)
            || state.is_block(SpawnBlock::Tnt)
            || state.is_block(SpawnBlock::Glowstone)
            || state.is_block(SpawnBlock::Beacon)
            || state.is_block(SpawnBlock::SeaLantern)
            || state.is_block(SpawnBlock::FrostedIce)
            || state.is_block(SpawnBlock::TintedGlass)
            || state.is_block(SpawnBlock::Glass);
        if blocked {
            false
        } else {
            (above_state.is_air() || above_state.liquid())
                && (state.is_solid() || state.is_block(SpawnBlock::PowderSnow))
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OnTopOfCollider;

impl<L: SpawnLevel + ?Sized> Strategy<L> for OnTopOfCollider {
    // JAVA: net.minecraft.util.SpawnUtil$Strategy#ON_TOP_OF_COLLIDER
    fn can_spawn_on(
        &self,
        level: &L,
        pos: BlockPos,
        state: &L::State,
        above_pos: BlockPos,
        above_state: &L::State,
    ) -> bool {
        level.collision_shape_is_empty(above_pos, above_state) && level.is_face_full_up(pos, state)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OnTopOfColliderNoLeaves;

impl<L: SpawnLevel + ?Sized> Strategy<L> for OnTopOfColliderNoLeaves {
    // JAVA: net.minecraft.util.SpawnUtil$Strategy#ON_TOP_OF_COLLIDER_NO_LEAVES
    fn can_spawn_on(
        &self,
        level: &L,
        pos: BlockPos,
        state: &L::State,
        above_pos: BlockPos,
        above_state: &L::State,
    ) -> bool {
        level.collision_shape_is_empty(above_pos, above_state)
            && !state.is_in_leaves_tag()
            && level.is_face_full_up(pos, state)
    }
}

#[allow(deprecated)]
#[deprecated]
pub const LEGACY_IRON_GOLEM: LegacyIronGolem = LegacyIronGolem;
pub const ON_TOP_OF_COLLIDER: OnTopOfCollider = OnTopOfCollider;
pub const ON_TOP_OF_COLLIDER_NO_LEAVES: OnTopOfColliderNoLeaves = OnTopOfColliderNoLeaves;

fn random_between_inclusive(random: &mut dyn RandomSource, min: i32, max: i32) -> i32 {
    random
        .next_int_bound(max.wrapping_sub(min).wrapping_add(1))
        .wrapping_add(min)
}

// JAVA: net.minecraft.util.SpawnUtil#trySpawnMob
pub fn try_spawn_mob<L, E, S>(
    entity_type: &E,
    reason: EntitySpawnReason,
    level: &mut L,
    pos: BlockPos,
    attempts: i32,
    xz_spread: i32,
    y_spread: i32,
    strategy: &S,
    check_collision: bool,
) -> Option<E::Mob>
where
    L: SpawnLevel + ?Sized,
    E: SpawnEntityType<L> + ?Sized,
    S: Strategy<L> + ?Sized,
{
    let mut block_pos = pos;
    let mut i = 0i32;
    while i < attempts {
        let x_off = random_between_inclusive(level.random_source(), -xz_spread, xz_spread);
        let z_off = random_between_inclusive(level.random_source(), -xz_spread, xz_spread);
        block_pos.set_with_offset(pos, x_off, y_spread, z_off);
        if level.world_border_is_within_bounds(block_pos)
            && move_to_possible_spawn_position(level, y_spread, &mut block_pos, strategy)
            && (!check_collision
                || level.no_collision_for_aabb(&entity_type.get_spawn_aabb(
                    f64::from(block_pos.x) + 0.5,
                    f64::from(block_pos.y),
                    f64::from(block_pos.z) + 0.5,
                )))
        {
            if let Some(mut mob) = entity_type.create(level, block_pos, reason) {
                if mob.check_spawn_rules(level, reason) && mob.check_spawn_obstruction(level) {
                    level.add_fresh_entity_with_passengers(&mut mob);
                    mob.play_ambient_sound();
                    return Some(mob);
                }

                mob.discard();
            }
        }

        i = i.wrapping_add(1);
    }

    None
}

// JAVA: net.minecraft.util.SpawnUtil#moveToPossibleSpawnPosition
fn move_to_possible_spawn_position<L, S>(
    level: &L,
    y_spread: i32,
    pos: &mut BlockPos,
    strategy: &S,
) -> bool
where
    L: SpawnLevel + ?Sized,
    S: Strategy<L> + ?Sized,
{
    let mut above_pos = *pos;
    let mut above_state = level.get_block_state(above_pos);

    let mut i = y_spread;
    let limit = 0i32.wrapping_sub(y_spread);
    while i >= limit {
        pos.move_dir(Direction::Down);
        above_pos.set_with_offset_dir(*pos, Direction::Up);
        let state = level.get_block_state(*pos);
        if strategy.can_spawn_on(level, *pos, &state, above_pos, &above_state) {
            pos.move_dir(Direction::Up);
            return true;
        }

        above_state = state;
        i = i.wrapping_sub(1);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{PositionalRandomFactory, RandomSource};
    use std::cell::Cell;
    use std::collections::HashMap;

    struct SeqRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(ints: &[i32]) -> Self {
            Self {
                ints: ints.to_vec(),
                idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn fork(&mut self) -> Box<dyn RandomSource> {
            Box::new(SeqRandom {
                ints: self.ints.clone(),
                idx: self.idx,
            })
        }

        fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
            unimplemented!("not needed in spawn_util tests")
        }

        fn set_seed(&mut self, _seed: i64) {}

        fn next_int(&mut self) -> i32 {
            let v = self.ints.get(self.idx).copied().unwrap_or(0);
            self.idx += 1;
            v
        }

        fn next_int_bound(&mut self, bound: i32) -> i32 {
            let v = self.next_int();
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_long(&mut self) -> i64 {
            0
        }

        fn next_boolean(&mut self) -> bool {
            false
        }

        fn next_float(&mut self) -> f32 {
            0.0
        }

        fn next_double(&mut self) -> f64 {
            0.0
        }

        fn next_gaussian(&mut self) -> f64 {
            0.0
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockState {
        air: bool,
        liquid: bool,
        solid: bool,
        block: Option<SpawnBlock>,
        stained_glass_pane: bool,
        stained_glass: bool,
        leaves_block: bool,
        leaves_tag: bool,
    }

    impl MockState {
        fn air() -> Self {
            Self {
                air: true,
                liquid: false,
                solid: false,
                block: None,
                stained_glass_pane: false,
                stained_glass: false,
                leaves_block: false,
                leaves_tag: false,
            }
        }

        fn solid() -> Self {
            Self {
                air: false,
                liquid: false,
                solid: true,
                block: None,
                stained_glass_pane: false,
                stained_glass: false,
                leaves_block: false,
                leaves_tag: false,
            }
        }

        fn solid_block(block: SpawnBlock) -> Self {
            let mut s = Self::solid();
            s.block = Some(block);
            s
        }
    }

    impl SpawnBlockState for MockState {
        fn is_air(&self) -> bool {
            self.air
        }
        fn liquid(&self) -> bool {
            self.liquid
        }
        fn is_solid(&self) -> bool {
            self.solid
        }
        fn is_block(&self, block: SpawnBlock) -> bool {
            self.block == Some(block)
        }
        fn is_stained_glass_pane_block(&self) -> bool {
            self.stained_glass_pane
        }
        fn is_stained_glass_block(&self) -> bool {
            self.stained_glass
        }
        fn is_leaves_block(&self) -> bool {
            self.leaves_block
        }
        fn is_in_leaves_tag(&self) -> bool {
            self.leaves_tag
        }
    }

    struct MockLevel {
        random: SeqRandom,
        states: HashMap<(i32, i32, i32), MockState>,
        default_state: MockState,
        border_min: i32,
        border_max: i32,
        no_collision: bool,
        added: usize,
        collision_empty: bool,
        face_full_up: bool,
    }

    impl SpawnLevel for MockLevel {
        type State = MockState;
        fn random_source(&mut self) -> &mut dyn RandomSource {
            &mut self.random
        }
        fn world_border_is_within_bounds(&self, pos: BlockPos) -> bool {
            pos.x >= self.border_min
                && pos.x <= self.border_max
                && pos.z >= self.border_min
                && pos.z <= self.border_max
        }
        fn get_block_state(&self, pos: BlockPos) -> Self::State {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default_state.clone())
        }
        fn no_collision_for_aabb(&self, _aabb: &Aabb) -> bool {
            self.no_collision
        }
        fn collision_shape_is_empty(&self, _pos: BlockPos, _state: &Self::State) -> bool {
            self.collision_empty
        }
        fn is_face_full_up(&self, _pos: BlockPos, _state: &Self::State) -> bool {
            self.face_full_up
        }
        fn add_fresh_entity_with_passengers<M>(&mut self, _mob: &mut M) {
            self.added += 1;
        }
    }

    #[derive(Debug, Default)]
    struct MockMob {
        rules: bool,
        obstruction: bool,
        ambient_played: bool,
        discarded: bool,
    }

    impl SpawnMob<MockLevel> for MockMob {
        fn check_spawn_rules(&self, _level: &MockLevel, _reason: EntitySpawnReason) -> bool {
            self.rules
        }
        fn check_spawn_obstruction(&self, _level: &MockLevel) -> bool {
            self.obstruction
        }
        fn play_ambient_sound(&mut self) {
            self.ambient_played = true;
        }
        fn discard(&mut self) {
            self.discarded = true;
        }
    }

    struct MockEntityType {
        create_ok: bool,
        rules: bool,
        obstruction: bool,
    }

    impl SpawnEntityType<MockLevel> for MockEntityType {
        type Mob = MockMob;
        fn get_spawn_aabb(&self, x: f64, y: f64, z: f64) -> Aabb {
            Aabb {
                min_x: x,
                min_y: y,
                min_z: z,
                max_x: x + 1.0,
                max_y: y + 1.0,
                max_z: z + 1.0,
            }
        }
        fn create(
            &self,
            _level: &mut MockLevel,
            _pos: BlockPos,
            _reason: EntitySpawnReason,
        ) -> Option<Self::Mob> {
            if self.create_ok {
                Some(MockMob {
                    rules: self.rules,
                    obstruction: self.obstruction,
                    ambient_played: false,
                    discarded: false,
                })
            } else {
                None
            }
        }
    }

    struct AlwaysTrue;
    impl<L: SpawnLevel + ?Sized> Strategy<L> for AlwaysTrue {
        fn can_spawn_on(
            &self,
            _: &L,
            _: BlockPos,
            _: &L::State,
            _: BlockPos,
            _: &L::State,
        ) -> bool {
            true
        }
    }

    struct SolidBelowAirAbove;
    impl<L: SpawnLevel + ?Sized> Strategy<L> for SolidBelowAirAbove {
        fn can_spawn_on(
            &self,
            _: &L,
            _: BlockPos,
            state: &L::State,
            _: BlockPos,
            above_state: &L::State,
        ) -> bool {
            state.is_solid() && above_state.is_air()
        }
    }

    #[test]
    fn try_spawn_mob_success() {
        let mut level = MockLevel {
            random: SeqRandom::new(&[1, 2]),
            states: HashMap::new(),
            default_state: MockState::air(),
            border_min: -1000,
            border_max: 1000,
            no_collision: true,
            added: 0,
            collision_empty: true,
            face_full_up: true,
        };
        let et = MockEntityType {
            create_ok: true,
            rules: true,
            obstruction: true,
        };
        let result = try_spawn_mob(
            &et,
            EntitySpawnReason,
            &mut level,
            BlockPos::new(10, 64, 20),
            3,
            5,
            1,
            &AlwaysTrue,
            false,
        );
        let mob = result.expect("spawn should succeed");
        assert!(mob.ambient_played);
        assert!(!mob.discarded);
        assert_eq!(level.added, 1);
    }

    #[test]
    fn move_to_possible_spawn_position_scans_down() {
        let mut states = HashMap::new();
        states.insert((0, 60, 0), MockState::solid());
        states.insert((0, 61, 0), MockState::air());
        states.insert((0, 62, 0), MockState::air());
        states.insert((0, 63, 0), MockState::air());
        let level = MockLevel {
            random: SeqRandom::new(&[]),
            states,
            default_state: MockState::air(),
            border_min: -1000,
            border_max: 1000,
            no_collision: true,
            added: 0,
            collision_empty: true,
            face_full_up: true,
        };
        let mut pos = BlockPos::new(0, 63, 0);
        let ok = move_to_possible_spawn_position(&level, 3, &mut pos, &SolidBelowAirAbove);
        assert!(ok);
        assert_eq!(pos, BlockPos::new(0, 61, 0));
    }

    #[test]
    fn move_to_possible_spawn_position_fails_when_no_valid_spot() {
        let level = MockLevel {
            random: SeqRandom::new(&[]),
            states: HashMap::new(),
            default_state: MockState::air(),
            border_min: -1000,
            border_max: 1000,
            no_collision: true,
            added: 0,
            collision_empty: true,
            face_full_up: true,
        };
        let mut pos = BlockPos::new(0, 64, 0);
        let ok = move_to_possible_spawn_position(&level, 2, &mut pos, &SolidBelowAirAbove);
        assert!(!ok);
    }

    #[test]
    fn legacy_iron_golem_strategy_rules() {
        let level = MockLevel {
            random: SeqRandom::new(&[]),
            states: HashMap::new(),
            default_state: MockState::air(),
            border_min: -1000,
            border_max: 1000,
            no_collision: true,
            added: 0,
            collision_empty: true,
            face_full_up: true,
        };
        let pos = BlockPos::new(0, 0, 0);
        let above = BlockPos::new(0, 1, 0);
        let solid = MockState::solid();
        let air = MockState::air();
        #[allow(deprecated)]
        let strat = LegacyIronGolem;

        assert!(strat.can_spawn_on(&level, pos, &solid, above, &air));

        let cobweb = MockState::solid_block(SpawnBlock::Cobweb);
        assert!(!strat.can_spawn_on(&level, pos, &cobweb, above, &air));

        let mut leaves = MockState::solid();
        leaves.leaves_block = true;
        assert!(!strat.can_spawn_on(&level, pos, &leaves, above, &air));

        let mut stained_glass = MockState::solid();
        stained_glass.stained_glass = true;
        assert!(!strat.can_spawn_on(&level, pos, &stained_glass, above, &air));

        let powder = MockState {
            air: false,
            liquid: false,
            solid: false,
            block: Some(SpawnBlock::PowderSnow),
            stained_glass_pane: false,
            stained_glass: false,
            leaves_block: false,
            leaves_tag: false,
        };
        assert!(strat.can_spawn_on(&level, pos, &powder, above, &air));

        let mut liquid_above = MockState::air();
        liquid_above.air = false;
        liquid_above.liquid = true;
        assert!(strat.can_spawn_on(&level, pos, &solid, above, &liquid_above));

        assert!(!strat.can_spawn_on(&level, pos, &solid, above, &solid));
    }

    #[test]
    fn on_top_of_collider_strategies_rules() {
        let mut level = MockLevel {
            random: SeqRandom::new(&[]),
            states: HashMap::new(),
            default_state: MockState::air(),
            border_min: -1000,
            border_max: 1000,
            no_collision: true,
            added: 0,
            collision_empty: true,
            face_full_up: true,
        };
        let pos = BlockPos::new(0, 0, 0);
        let above = BlockPos::new(0, 1, 0);
        let state = MockState::solid();
        let above_state = MockState::air();

        assert!(OnTopOfCollider.can_spawn_on(&level, pos, &state, above, &above_state));

        level.collision_empty = false;
        assert!(!OnTopOfCollider.can_spawn_on(&level, pos, &state, above, &above_state));

        level.collision_empty = true;
        level.face_full_up = false;
        assert!(!OnTopOfCollider.can_spawn_on(&level, pos, &state, above, &above_state));

        level.face_full_up = true;
        let mut leaves = MockState::solid();
        leaves.leaves_tag = true;
        assert!(!OnTopOfColliderNoLeaves.can_spawn_on(&level, pos, &leaves, above, &above_state));
        assert!(OnTopOfCollider.can_spawn_on(&level, pos, &leaves, above, &above_state));
    }

    #[test]
    fn negative_spread_float_bits_and_wrapping() {
        #[derive(Debug)]
        struct CapMob;
        struct CapLevel {
            random: SeqRandom,
            aabb: Cell<Option<Aabb>>,
            added: bool,
        }
        impl SpawnMob<CapLevel> for CapMob {
            fn check_spawn_rules(&self, _: &CapLevel, _: EntitySpawnReason) -> bool {
                true
            }
            fn check_spawn_obstruction(&self, _: &CapLevel) -> bool {
                true
            }
            fn play_ambient_sound(&mut self) {}
            fn discard(&mut self) {}
        }
        impl SpawnLevel for CapLevel {
            type State = MockState;
            fn random_source(&mut self) -> &mut dyn RandomSource {
                &mut self.random
            }
            fn world_border_is_within_bounds(&self, _: BlockPos) -> bool {
                true
            }
            fn get_block_state(&self, _: BlockPos) -> Self::State {
                MockState::air()
            }
            fn no_collision_for_aabb(&self, aabb: &Aabb) -> bool {
                self.aabb.set(Some(*aabb));
                true
            }
            fn collision_shape_is_empty(&self, _: BlockPos, _: &Self::State) -> bool {
                true
            }
            fn is_face_full_up(&self, _: BlockPos, _: &Self::State) -> bool {
                true
            }
            fn add_fresh_entity_with_passengers<M>(&mut self, _: &mut M) {
                self.added = true;
            }
        }
        struct CapEntity;
        impl SpawnEntityType<CapLevel> for CapEntity {
            type Mob = CapMob;
            fn get_spawn_aabb(&self, x: f64, y: f64, z: f64) -> Aabb {
                Aabb {
                    min_x: x,
                    min_y: y,
                    min_z: z,
                    max_x: x + 1.0,
                    max_y: y + 1.0,
                    max_z: z + 1.0,
                }
            }
            fn create(
                &self,
                _: &mut CapLevel,
                _: BlockPos,
                _: EntitySpawnReason,
            ) -> Option<CapMob> {
                Some(CapMob)
            }
        }

        let mut level = CapLevel {
            random: SeqRandom::new(&[0, 0]),
            aabb: Cell::new(None),
            added: false,
        };
        let pos = BlockPos::new(-3, -10, 7);
        let r = try_spawn_mob(
            &CapEntity,
            EntitySpawnReason,
            &mut level,
            pos,
            1,
            0,
            0,
            &AlwaysTrue,
            true,
        );
        assert!(r.is_some());
        assert!(level.added);
        let aabb = level.aabb.get().unwrap();
        assert_eq!(aabb.min_x.to_bits(), (-2.5f64).to_bits());
        assert_eq!(aabb.min_y.to_bits(), (-10.0f64).to_bits());
        assert_eq!(aabb.min_z.to_bits(), (7.5f64).to_bits());

        let mut p = BlockPos::new(0, 0, 0);
        p.set_with_offset(BlockPos::new(i32::MAX, 10, i32::MIN), 1, -20, -1);
        assert_eq!(p.x, i32::MIN);
        assert_eq!(p.y, -10);
        assert_eq!(p.z, i32::MAX);
    }
}
