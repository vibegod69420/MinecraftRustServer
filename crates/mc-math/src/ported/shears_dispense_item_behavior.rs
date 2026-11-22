// === FILE: shears_dispense_item_behavior.rs ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn relative(self, direction: Direction) -> Self {
        Self {
            x: self.x.wrapping_add(direction.step_x()),
            y: self.y.wrapping_add(direction.step_y()),
            z: self.z.wrapping_add(direction.step_z()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const fn step_x(self) -> i32 {
        match self {
            Self::West => -1,
            Self::East => 1,
            _ => 0,
        }
    }

    pub const fn step_y(self) -> i32 {
        match self {
            Self::Down => -1,
            Self::Up => 1,
            _ => 0,
        }
    }

    pub const fn step_z(self) -> i32 {
        match self {
            Self::North => -1,
            Self::South => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Blocks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeeReleaseStatus {
    BeeReleased,
}

pub trait DispenseItemStack {
    fn hurt_and_break(&mut self, amount: i32);
}

pub trait BlockStateLike {
    fn is_beehives_tag(&self) -> bool;
    fn has_property_honey_level(&self) -> bool;
    fn get_block_is_beehive_block(&self) -> bool;
    fn get_honey_level(&self) -> i32;
}

pub trait ServerLevelLike {
    type BlockState: BlockStateLike;
    type EntityRef: Copy + Eq + std::fmt::Debug;

    fn is_client_side(&self) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> Self::BlockState;
    fn play_beehive_shear_sound(&mut self, pos: BlockPos, volume: f32, pitch: f32);
    fn drop_honeycomb<S: DispenseItemStack + ?Sized>(
        &mut self,
        pos: BlockPos,
        stack: &mut S,
        state: &Self::BlockState,
    );
    fn release_bees_and_reset_honey_level(
        &mut self,
        pos: BlockPos,
        state: &Self::BlockState,
        status: BeeReleaseStatus,
    );
    fn game_event_shear(&mut self, pos: BlockPos);

    fn get_entities_at(&self, pos: BlockPos) -> Vec<Self::EntityRef>;
    fn shear_off_all_leash_connections(&mut self, entity: Self::EntityRef) -> bool;
    fn is_shearable_and_ready(&self, entity: Self::EntityRef) -> bool;
    fn shear_entity<S: DispenseItemStack + ?Sized>(
        &mut self,
        entity: Self::EntityRef,
        sound_source: SoundSource,
        stack: &mut S,
    );
}

pub trait BlockSourceLike {
    type Level: ServerLevelLike;

    fn level(&mut self) -> &mut Self::Level;
    fn pos(&self) -> BlockPos;
    fn facing(&self) -> Direction;
}

#[derive(Debug, Default)]
pub struct ShearsDispenseItemBehavior {
    success: bool,
}

impl ShearsDispenseItemBehavior {
    pub fn new() -> Self {
        Self { success: false }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    // JAVA: net.minecraft.core.dispenser.ShearsDispenseItemBehavior#execute
    pub fn execute<B: BlockSourceLike, S: DispenseItemStack>(
        &mut self,
        source: &mut B,
        stack: &mut S,
    ) {
        let is_client_side = source.level().is_client_side();
        if !is_client_side {
            let pos = source.pos().relative(source.facing());
            let level = source.level();
            let success =
                try_shear_beehive(level, stack, pos) || try_shear_entity(level, pos, stack);
            self.set_success(success);
            if self.is_success() {
                stack.hurt_and_break(1);
            }
        }
    }
}

// JAVA: net.minecraft.core.dispenser.ShearsDispenseItemBehavior#tryShearBeehive
fn try_shear_beehive<L: ServerLevelLike + ?Sized, S: DispenseItemStack + ?Sized>(
    level: &mut L,
    stack: &mut S,
    pos: BlockPos,
) -> bool {
    let state = level.get_block_state(pos);
    if state.is_beehives_tag()
        && state.has_property_honey_level()
        && state.get_block_is_beehive_block()
    {
        let honey_level: i32 = state.get_honey_level();
        if honey_level >= 5 {
            level.play_beehive_shear_sound(pos, 1.0_f32, 1.0_f32);
            level.drop_honeycomb(pos, stack, &state);
            level.release_bees_and_reset_honey_level(pos, &state, BeeReleaseStatus::BeeReleased);
            level.game_event_shear(pos);
            return true;
        }
    }
    false
}

// JAVA: net.minecraft.core.dispenser.ShearsDispenseItemBehavior#tryShearEntity
fn try_shear_entity<L: ServerLevelLike + ?Sized, S: DispenseItemStack + ?Sized>(
    level: &mut L,
    pos: BlockPos,
    stack: &mut S,
) -> bool {
    let entities = level.get_entities_at(pos);
    for entity in entities {
        if level.shear_off_all_leash_connections(entity) {
            return true;
        }
        if level.is_shearable_and_ready(entity) {
            level.shear_entity(entity, SoundSource::Blocks, stack);
            level.game_event_shear(pos);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct MockItemStack {
        hurt_calls: Vec<i32>,
    }

    impl DispenseItemStack for MockItemStack {
        fn hurt_and_break(&mut self, amount: i32) {
            self.hurt_calls.push(amount);
        }
    }

    #[derive(Debug, Clone)]
    struct MockBlockState {
        beehives_tag: bool,
        has_honey: bool,
        is_beehive: bool,
        honey_level: i32,
    }

    impl MockBlockState {
        fn beehive(honey_level: i32) -> Self {
            Self {
                beehives_tag: true,
                has_honey: true,
                is_beehive: true,
                honey_level,
            }
        }

        fn not_beehive() -> Self {
            Self {
                beehives_tag: false,
                has_honey: false,
                is_beehive: false,
                honey_level: 0,
            }
        }
    }

    impl BlockStateLike for MockBlockState {
        fn is_beehives_tag(&self) -> bool {
            self.beehives_tag
        }
        fn has_property_honey_level(&self) -> bool {
            self.has_honey
        }
        fn get_block_is_beehive_block(&self) -> bool {
            self.is_beehive
        }
        fn get_honey_level(&self) -> i32 {
            self.honey_level
        }
    }

    #[derive(Debug, Clone)]
    struct MockEntity {
        leash_sheared: bool,
        shearable: bool,
        ready: bool,
        shear_called: bool,
    }

    impl MockEntity {
        fn leash_only() -> Self {
            Self {
                leash_sheared: true,
                shearable: false,
                ready: false,
                shear_called: false,
            }
        }
        fn shearable_ready() -> Self {
            Self {
                leash_sheared: false,
                shearable: true,
                ready: true,
                shear_called: false,
            }
        }
        fn shearable_not_ready() -> Self {
            Self {
                leash_sheared: false,
                shearable: true,
                ready: false,
                shear_called: false,
            }
        }
        fn plain() -> Self {
            Self {
                leash_sheared: false,
                shearable: false,
                ready: false,
                shear_called: false,
            }
        }
    }

    #[derive(Debug)]
    struct MockLevel {
        client_side: bool,
        block_state: MockBlockState,
        entities: Vec<MockEntity>,
        play_sound_calls: Vec<(BlockPos, u32, u32)>,
        drop_honeycomb_calls: Vec<BlockPos>,
        release_bees_calls: Vec<(BlockPos, BeeReleaseStatus)>,
        game_event_shear_calls: Vec<BlockPos>,
        shear_entity_calls: Vec<(usize, SoundSource)>,
    }

    impl MockLevel {
        fn new(client_side: bool, block_state: MockBlockState, entities: Vec<MockEntity>) -> Self {
            Self {
                client_side,
                block_state,
                entities,
                play_sound_calls: Vec::new(),
                drop_honeycomb_calls: Vec::new(),
                release_bees_calls: Vec::new(),
                game_event_shear_calls: Vec::new(),
                shear_entity_calls: Vec::new(),
            }
        }
    }

    impl ServerLevelLike for MockLevel {
        type BlockState = MockBlockState;
        type EntityRef = usize;

        fn is_client_side(&self) -> bool {
            self.client_side
        }

        fn get_block_state(&self, _pos: BlockPos) -> Self::BlockState {
            self.block_state.clone()
        }

        fn play_beehive_shear_sound(&mut self, pos: BlockPos, volume: f32, pitch: f32) {
            self.play_sound_calls
                .push((pos, volume.to_bits(), pitch.to_bits()));
        }

        fn drop_honeycomb<S: DispenseItemStack + ?Sized>(
            &mut self,
            pos: BlockPos,
            _stack: &mut S,
            _state: &Self::BlockState,
        ) {
            self.drop_honeycomb_calls.push(pos);
        }

        fn release_bees_and_reset_honey_level(
            &mut self,
            pos: BlockPos,
            _state: &Self::BlockState,
            status: BeeReleaseStatus,
        ) {
            self.release_bees_calls.push((pos, status));
        }

        fn game_event_shear(&mut self, pos: BlockPos) {
            self.game_event_shear_calls.push(pos);
        }

        fn get_entities_at(&self, _pos: BlockPos) -> Vec<Self::EntityRef> {
            (0..self.entities.len()).collect()
        }

        fn shear_off_all_leash_connections(&mut self, entity: Self::EntityRef) -> bool {
            self.entities[entity].leash_sheared
        }

        fn is_shearable_and_ready(&self, entity: Self::EntityRef) -> bool {
            let e = &self.entities[entity];
            e.shearable && e.ready
        }

        fn shear_entity<S: DispenseItemStack + ?Sized>(
            &mut self,
            entity: Self::EntityRef,
            sound_source: SoundSource,
            _stack: &mut S,
        ) {
            self.entities[entity].shear_called = true;
            self.shear_entity_calls.push((entity, sound_source));
        }
    }

    struct MockSource {
        level: MockLevel,
        pos: BlockPos,
        facing: Direction,
    }

    impl BlockSourceLike for MockSource {
        type Level = MockLevel;
        fn level(&mut self) -> &mut Self::Level {
            &mut self.level
        }
        fn pos(&self) -> BlockPos {
            self.pos
        }
        fn facing(&self) -> Direction {
            self.facing
        }
    }

    #[test]
    fn execute_client_side_noop() {
        let mut source = MockSource {
            level: MockLevel::new(true, MockBlockState::beehive(5), vec![]),
            pos: BlockPos::new(0, 0, 0),
            facing: Direction::North,
        };
        let mut stack = MockItemStack::default();
        let mut behavior = ShearsDispenseItemBehavior::new();
        behavior.set_success(true);
        behavior.execute(&mut source, &mut stack);
        assert!(behavior.is_success());
        assert!(stack.hurt_calls.is_empty());
        assert!(source.level.play_sound_calls.is_empty());
    }

    #[test]
    fn beehive_shear_success() {
        let target = BlockPos::new(1, 2, 3);
        let mut source = MockSource {
            level: MockLevel::new(false, MockBlockState::beehive(5), vec![]),
            pos: BlockPos::new(1, 2, 2),
            facing: Direction::South,
        };
        let mut stack = MockItemStack::default();
        let mut behavior = ShearsDispenseItemBehavior::new();
        behavior.execute(&mut source, &mut stack);

        assert!(behavior.is_success());
        assert_eq!(stack.hurt_calls, vec![1]);
        assert_eq!(
            source.level.play_sound_calls,
            vec![(target, 1.0_f32.to_bits(), 1.0_f32.to_bits())]
        );
        assert_eq!(source.level.drop_honeycomb_calls, vec![target]);
        assert_eq!(
            source.level.release_bees_calls,
            vec![(target, BeeReleaseStatus::BeeReleased)]
        );
        assert_eq!(source.level.game_event_shear_calls, vec![target]);
    }

    #[test]
    fn beehive_insufficient_honey_then_entity_shear() {
        let target = BlockPos::new(5, 0, 0);
        let mut source = MockSource {
            level: MockLevel::new(
                false,
                MockBlockState::beehive(4),
                vec![MockEntity::shearable_not_ready(), MockEntity::shearable_ready()],
            ),
            pos: BlockPos::new(4, 0, 0),
            facing: Direction::East,
        };
        let mut stack = MockItemStack::default();
        let mut behavior = ShearsDispenseItemBehavior::new();
        behavior.execute(&mut source, &mut stack);

        assert!(behavior.is_success());
        assert_eq!(stack.hurt_calls, vec![1]);
        assert!(source.level.play_sound_calls.is_empty());
        assert_eq!(
            source.level.shear_entity_calls,
            vec![(1, SoundSource::Blocks)]
        );
        assert_eq!(source.level.game_event_shear_calls, vec![target]);
        assert!(!source.level.entities[0].shear_called);
        assert!(source.level.entities[1].shear_called);
    }

    #[test]
    fn leash_connection_shear_takes_priority() {
        let mut source = MockSource {
            level: MockLevel::new(
                false,
                MockBlockState::not_beehive(),
                vec![MockEntity::leash_only(), MockEntity::shearable_ready()],
            ),
            pos: BlockPos::new(0, 0, 0),
            facing: Direction::Up,
        };
        let mut stack = MockItemStack::default();
        let mut behavior = ShearsDispenseItemBehavior::new();
        behavior.execute(&mut source, &mut stack);

        assert!(behavior.is_success());
        assert_eq!(stack.hurt_calls, vec![1]);
        assert!(source.level.shear_entity_calls.is_empty());
        assert!(source.level.game_event_shear_calls.is_empty());
        assert!(!source.level.entities[1].shear_called);
    }

    #[test]
    fn nothing_to_shear_fails() {
        let mut source = MockSource {
            level: MockLevel::new(
                false,
                MockBlockState::not_beehive(),
                vec![MockEntity::plain(), MockEntity::shearable_not_ready()],
            ),
            pos: BlockPos::new(0, 0, 0),
            facing: Direction::Down,
        };
        let mut stack = MockItemStack::default();
        let mut behavior = ShearsDispenseItemBehavior::new();
        behavior.set_success(true);
        behavior.execute(&mut source, &mut stack);
        assert!(!behavior.is_success());
        assert!(stack.hurt_calls.is_empty());

        let mut source2 = MockSource {
            level: MockLevel::new(
                false,
                MockBlockState {
                    beehives_tag: true,
                    has_honey: false,
                    is_beehive: true,
                    honey_level: 5,
                },
                vec![],
            ),
            pos: BlockPos::new(0, 0, 0),
            facing: Direction::Up,
        };
        let mut stack2 = MockItemStack::default();
        let mut behavior2 = ShearsDispenseItemBehavior::new();
        behavior2.execute(&mut source2, &mut stack2);
        assert!(!behavior2.is_success());
        assert!(stack2.hurt_calls.is_empty());
    }

    #[test]
    fn block_pos_relative_negative_and_overflow() {
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        let up = pos.relative(Direction::Up);
        assert_eq!(up.x, i32::MIN);
        assert_eq!(up.y, 0);
        assert_eq!(up.z, i32::MAX);

        let down = pos.relative(Direction::Down);
        assert_eq!(down.y, -2);

        let east = pos.relative(Direction::East);
        assert_eq!(east.x, i32::MIN.wrapping_add(1));

        let west = BlockPos::new(i32::MIN, 0, 0).relative(Direction::West);
        assert_eq!(west.x, i32::MAX);

        let south = BlockPos::new(0, 0, i32::MAX).relative(Direction::South);
        assert_eq!(south.z, i32::MIN);
    }
}
