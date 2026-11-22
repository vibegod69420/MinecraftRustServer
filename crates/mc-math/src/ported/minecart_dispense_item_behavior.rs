use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn relative(self, dir: Direction) -> Self {
        Self {
            x: self.x.wrapping_add(dir.get_step_x()),
            y: self.y.wrapping_add(dir.get_step_y()),
            z: self.z.wrapping_add(dir.get_step_z()),
        }
    }

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
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
    pub const fn get_step_x(self) -> i32 {
        match self {
            Self::East => 1,
            Self::West => -1,
            _ => 0,
        }
    }

    pub const fn get_step_y(self) -> i32 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
            _ => 0,
        }
    }

    pub const fn get_step_z(self) -> i32 {
        match self {
            Self::South => 1,
            Self::North => -1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

impl RailShape {
    pub const fn is_slope(self) -> bool {
        matches!(
            self,
            Self::AscendingEast
                | Self::AscendingWest
                | Self::AscendingNorth
                | Self::AscendingSouth
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntitySpawnReason {
    Dispenser,
}

pub trait BlockStateLike {
    fn is_rails(&self) -> bool;
    fn is_air(&self) -> bool;
    fn base_rail_shape(&self) -> Option<RailShape>;
}

pub trait ItemStackLike {
    fn shrink(&mut self, amount: i32);
}

pub trait DispenseSource<E> {
    type BlockState: BlockStateLike;
    type Stack: ItemStackLike;
    type Minecart;

    fn facing(&self) -> Direction;
    fn center(&self) -> Vec3d;
    fn pos(&self) -> BlockPos;
    fn get_block_state(&self, pos: BlockPos) -> Self::BlockState;
    fn create_minecart(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        entity_type: &E,
        spawn_reason: EntitySpawnReason,
        stack: &Self::Stack,
    ) -> Option<Self::Minecart>;
    fn add_fresh_entity(&mut self, entity: Self::Minecart);
    fn level_event(&mut self, event: i32, pos: BlockPos, data: i32);
}

pub trait DefaultDispense<S: ?Sized, St> {
    fn dispense(&mut self, source: &mut S, stack: St) -> St;
}

pub struct MinecartDispenseItemBehavior<E, D> {
    pub default_dispense_item_behavior: D,
    pub entity_type: E,
}

impl<E, D> MinecartDispenseItemBehavior<E, D> {
    // JAVA: net.minecraft.core.dispenser.MinecartDispenseItemBehavior#<init>
    pub fn new(entity_type: E, default_dispense_item_behavior: D) -> Self {
        Self {
            default_dispense_item_behavior,
            entity_type,
        }
    }

    // JAVA: net.minecraft.core.dispenser.MinecartDispenseItemBehavior#execute
    pub fn execute<S>(&mut self, source: &mut S, mut stack: S::Stack) -> S::Stack
    where
        S: DispenseSource<E>,
        D: DefaultDispense<S, S::Stack>,
    {
        let facing = source.facing();
        let center = source.center();
        let pos = source.pos();
        let x5 = center.x + (facing.get_step_x() as f64) * 1.125f64;
        let y6 = center.y.floor() + (facing.get_step_y() as f64);
        let z7 = center.z + (facing.get_step_z() as f64) * 1.125f64;
        let pos8 = pos.relative(facing);
        let state9 = source.get_block_state(pos8);
        let y10: f64;
        if state9.is_rails() {
            if get_rail_shape(&state9).is_slope() {
                y10 = 0.6f64;
            } else {
                y10 = 0.1f64;
            }
        } else {
            if !state9.is_air() {
                return self
                    .default_dispense_item_behavior
                    .dispense(source, stack);
            }

            let state12 = source.get_block_state(pos8.below());
            if !state12.is_rails() {
                return self
                    .default_dispense_item_behavior
                    .dispense(source, stack);
            }

            if facing != Direction::Down && get_rail_shape(&state12).is_slope() {
                y10 = -0.4f64;
            } else {
                y10 = -0.9f64;
            }
        }

        let spawn_pos = Vec3d::new(x5, y6 + y10, z7);
        let minecart = source.create_minecart(
            spawn_pos.x,
            spawn_pos.y,
            spawn_pos.z,
            &self.entity_type,
            EntitySpawnReason::Dispenser,
            &stack,
        );
        if let Some(entity) = minecart {
            source.add_fresh_entity(entity);
            stack.shrink(1);
        }

        stack
    }

    // JAVA: net.minecraft.core.dispenser.MinecartDispenseItemBehavior#playSound
    pub fn play_sound<S>(&self, source: &mut S)
    where
        S: DispenseSource<E>,
    {
        let pos = source.pos();
        source.level_event(1000, pos, 0);
    }
}

// JAVA: net.minecraft.core.dispenser.MinecartDispenseItemBehavior#getRailShape
fn get_rail_shape<S: BlockStateLike + ?Sized>(state: &S) -> RailShape {
    state.base_rail_shape().unwrap_or(RailShape::NorthSouth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq)]
    struct MockStack {
        count: i32,
    }

    impl ItemStackLike for MockStack {
        fn shrink(&mut self, amount: i32) {
            self.count -= amount;
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct MockState {
        rails: bool,
        air: bool,
        shape: Option<RailShape>,
    }

    impl BlockStateLike for MockState {
        fn is_rails(&self) -> bool {
            self.rails
        }

        fn is_air(&self) -> bool {
            self.air
        }

        fn base_rail_shape(&self) -> Option<RailShape> {
            self.shape
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct MockMinecart {
        pos: Vec3d,
        entity_type: i32,
        spawn_reason: EntitySpawnReason,
    }

    struct MockSource {
        facing_dir: Direction,
        center_pos: Vec3d,
        block_pos: BlockPos,
        states: HashMap<(i32, i32, i32), MockState>,
        minecarts: Vec<MockMinecart>,
        events: Vec<(i32, BlockPos, i32)>,
        create_returns_none: bool,
    }

    impl DispenseSource<i32> for MockSource {
        type BlockState = MockState;
        type Stack = MockStack;
        type Minecart = MockMinecart;

        fn facing(&self) -> Direction {
            self.facing_dir
        }

        fn center(&self) -> Vec3d {
            self.center_pos
        }

        fn pos(&self) -> BlockPos {
            self.block_pos
        }

        fn get_block_state(&self, pos: BlockPos) -> Self::BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or(MockState {
                    rails: false,
                    air: false,
                    shape: None,
                })
        }

        fn create_minecart(
            &mut self,
            x: f64,
            y: f64,
            z: f64,
            entity_type: &i32,
            spawn_reason: EntitySpawnReason,
            _stack: &Self::Stack,
        ) -> Option<Self::Minecart> {
            if self.create_returns_none {
                return None;
            }
            Some(MockMinecart {
                pos: Vec3d::new(x, y, z),
                entity_type: *entity_type,
                spawn_reason,
            })
        }

        fn add_fresh_entity(&mut self, entity: Self::Minecart) {
            self.minecarts.push(entity);
        }

        fn level_event(&mut self, event: i32, pos: BlockPos, data: i32) {
            self.events.push((event, pos, data));
        }
    }

    struct MockDefaultDispense {
        dispensed: bool,
    }

    impl DefaultDispense<MockSource, MockStack> for MockDefaultDispense {
        fn dispense(&mut self, _source: &mut MockSource, stack: MockStack) -> MockStack {
            self.dispensed = true;
            stack
        }
    }

    fn solid_state() -> MockState {
        MockState {
            rails: false,
            air: false,
            shape: None,
        }
    }

    #[test]
    fn execute_on_flat_rail_places_minecart() {
        let mut source = MockSource {
            facing_dir: Direction::South,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source.states.insert(
            (0, 64, 1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::NorthSouth),
            },
        );
        let mut behavior =
            MinecartDispenseItemBehavior::new(42i32, MockDefaultDispense { dispensed: false });
        let result = behavior.execute(&mut source, MockStack { count: 3 });

        assert_eq!(result.count, 2);
        assert_eq!(source.minecarts.len(), 1);
        let m = &source.minecarts[0];
        let expected_x = 0.5f64 + (0i32 as f64) * 1.125f64;
        let expected_y = 64.0f64.floor() + (0i32 as f64) + 0.1f64;
        let expected_z = 0.5f64 + (1i32 as f64) * 1.125f64;
        assert_eq!(m.pos.x.to_bits(), expected_x.to_bits());
        assert_eq!(m.pos.y.to_bits(), expected_y.to_bits());
        assert_eq!(m.pos.z.to_bits(), expected_z.to_bits());
        assert_eq!(m.entity_type, 42);
        assert_eq!(m.spawn_reason, EntitySpawnReason::Dispenser);
    }

    #[test]
    fn execute_on_sloped_rail_uses_slope_offset() {
        let mut source = MockSource {
            facing_dir: Direction::North,
            center_pos: Vec3d::new(0.5f64, 70.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 70, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source.states.insert(
            (0, 70, -1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::AscendingNorth),
            },
        );
        let mut behavior =
            MinecartDispenseItemBehavior::new(1i32, MockDefaultDispense { dispensed: false });
        behavior.execute(&mut source, MockStack { count: 1 });

        let m = &source.minecarts[0];
        let expected_y = 70.0f64.floor() + (0i32 as f64) + 0.6f64;
        let expected_z = 0.5f64 + (-1i32 as f64) * 1.125f64;
        assert_eq!(m.pos.y.to_bits(), expected_y.to_bits());
        assert_eq!(m.pos.z.to_bits(), expected_z.to_bits());
    }

    #[test]
    fn execute_air_above_rail_below_offsets() {
        // Case A: not facing down, below rail sloped -> -0.4
        let mut source = MockSource {
            facing_dir: Direction::North,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source.states.insert(
            (0, 64, -1),
            MockState {
                rails: false,
                air: true,
                shape: None,
            },
        );
        source.states.insert(
            (0, 63, -1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::AscendingSouth),
            },
        );
        let mut behavior =
            MinecartDispenseItemBehavior::new(1i32, MockDefaultDispense { dispensed: false });
        behavior.execute(&mut source, MockStack { count: 1 });
        let expected_y_a = 64.0f64.floor() + (0i32 as f64) + (-0.4f64);
        assert_eq!(source.minecarts[0].pos.y.to_bits(), expected_y_a.to_bits());

        // Case B: facing down, below rail sloped -> still -0.9
        let mut source2 = MockSource {
            facing_dir: Direction::Down,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source2.states.insert(
            (0, 63, 0),
            MockState {
                rails: false,
                air: true,
                shape: None,
            },
        );
        source2.states.insert(
            (0, 62, 0),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::AscendingNorth),
            },
        );
        let mut behavior2 =
            MinecartDispenseItemBehavior::new(1i32, MockDefaultDispense { dispensed: false });
        behavior2.execute(&mut source2, MockStack { count: 1 });
        let expected_y_b = 64.0f64.floor() + (-1i32 as f64) + (-0.9f64);
        assert_eq!(
            source2.minecarts[0].pos.y.to_bits(),
            expected_y_b.to_bits()
        );

        // Case C: below rail not sloped -> -0.9
        let mut source3 = MockSource {
            facing_dir: Direction::North,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source3.states.insert(
            (0, 64, -1),
            MockState {
                rails: false,
                air: true,
                shape: None,
            },
        );
        source3.states.insert(
            (0, 63, -1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::NorthSouth),
            },
        );
        let mut behavior3 =
            MinecartDispenseItemBehavior::new(1i32, MockDefaultDispense { dispensed: false });
        behavior3.execute(&mut source3, MockStack { count: 1 });
        let expected_y_c = 64.0f64.floor() + (0i32 as f64) + (-0.9f64);
        assert_eq!(
            source3.minecarts[0].pos.y.to_bits(),
            expected_y_c.to_bits()
        );
    }

    #[test]
    fn execute_fallback_and_null_minecart() {
        // Non-rail, non-air target -> fallback
        let mut source = MockSource {
            facing_dir: Direction::East,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source.states.insert((1, 64, 0), solid_state());
        let mut behavior =
            MinecartDispenseItemBehavior::new(7i32, MockDefaultDispense { dispensed: false });
        let result = behavior.execute(&mut source, MockStack { count: 5 });
        assert_eq!(result.count, 5);
        assert!(behavior.default_dispense_item_behavior.dispensed);
        assert!(source.minecarts.is_empty());

        // Air target, below not rails -> fallback
        let mut source2 = MockSource {
            facing_dir: Direction::East,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source2.states.insert(
            (1, 64, 0),
            MockState {
                rails: false,
                air: true,
                shape: None,
            },
        );
        source2.states.insert((1, 63, 0), solid_state());
        let mut behavior2 =
            MinecartDispenseItemBehavior::new(7i32, MockDefaultDispense { dispensed: false });
        let result2 = behavior2.execute(&mut source2, MockStack { count: 2 });
        assert_eq!(result2.count, 2);
        assert!(behavior2.default_dispense_item_behavior.dispensed);

        // create_minecart returns None -> no entity, no shrink
        let mut source3 = MockSource {
            facing_dir: Direction::South,
            center_pos: Vec3d::new(0.5f64, 64.0f64, 0.5f64),
            block_pos: BlockPos::new(0, 64, 0),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: true,
        };
        source3.states.insert(
            (0, 64, 1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::EastWest),
            },
        );
        let mut behavior3 =
            MinecartDispenseItemBehavior::new(7i32, MockDefaultDispense { dispensed: false });
        let result3 = behavior3.execute(&mut source3, MockStack { count: 5 });
        assert_eq!(result3.count, 5);
        assert!(source3.minecarts.is_empty());
    }

    #[test]
    fn negative_coords_overflow_and_float_bits() {
        let mut source = MockSource {
            facing_dir: Direction::West,
            center_pos: Vec3d::new(-10.5f64, -3.2f64, -0.5f64),
            block_pos: BlockPos::new(-11, -4, -1),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        source.states.insert(
            (-12, -4, -1),
            MockState {
                rails: true,
                air: false,
                shape: Some(RailShape::NorthSouth),
            },
        );
        let mut behavior =
            MinecartDispenseItemBehavior::new(-1i32, MockDefaultDispense { dispensed: false });
        behavior.execute(&mut source, MockStack { count: 1 });

        let m = &source.minecarts[0];
        let expected_x = -10.5f64 + (-1i32 as f64) * 1.125f64;
        let expected_y = (-3.2f64).floor() + (0i32 as f64) + 0.1f64;
        let expected_z = -0.5f64 + (0i32 as f64) * 1.125f64;
        assert_eq!(m.pos.x.to_bits(), expected_x.to_bits());
        assert_eq!(m.pos.y.to_bits(), expected_y.to_bits());
        assert_eq!(m.pos.z.to_bits(), expected_z.to_bits());

        let p = BlockPos::new(i32::MAX, i32::MIN, 0);
        let r = p.relative(Direction::East);
        assert_eq!(r.x, i32::MIN);
        assert_eq!(r.y, i32::MIN);
        let b = p.below();
        assert_eq!(b.y, i32::MAX);

        let non_rail = MockState {
            rails: false,
            air: true,
            shape: None,
        };
        assert_eq!(get_rail_shape(&non_rail), RailShape::NorthSouth);
    }

    #[test]
    fn play_sound_emits_level_event_1000() {
        let mut source = MockSource {
            facing_dir: Direction::North,
            center_pos: Vec3d::new(0.0f64, 0.0f64, 0.0f64),
            block_pos: BlockPos::new(5, 64, -3),
            states: HashMap::new(),
            minecarts: Vec::new(),
            events: Vec::new(),
            create_returns_none: false,
        };
        let behavior =
            MinecartDispenseItemBehavior::new(0i32, MockDefaultDispense { dispensed: false });
        behavior.play_sound(&mut source);
        assert_eq!(source.events, vec![(1000, BlockPos::new(5, 64, -3), 0)]);
    }
}
