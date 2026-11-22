use crate::random::RandomSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getAxis
    pub fn get_axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepX
    pub fn get_step_x(self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepY
    pub fn get_step_y(self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepZ
    pub fn get_step_z(self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#get3DDataValue
    pub fn get_3d_data_value(self) -> i32 {
        match self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub trait DispenseRandom {
    fn next_double(&mut self) -> f64;

    // JAVA: net.minecraft.util.RandomSource#triangle
    fn triangle(&mut self, mean: f64, spread: f64) -> f64 {
        mean + spread * (self.next_double() - self.next_double())
    }
}

impl<T: RandomSource + ?Sized> DispenseRandom for T {
    fn next_double(&mut self) -> f64 {
        RandomSource::next_double(self)
    }
}

pub trait ItemStackLike: Clone {
    // JAVA: net.minecraft.world.item.ItemStack#split
    fn split(&mut self, amount: i32) -> Self;
    // JAVA: net.minecraft.world.item.ItemStack#shrink
    fn shrink(&mut self, amount: i32);
    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    fn is_empty(&self) -> bool;
}

pub trait LevelLike {
    type Random: DispenseRandom;
    type ItemStack: ItemStackLike;

    fn random(&mut self) -> &mut Self::Random;
    fn add_fresh_entity(&mut self, pos: Position, velocity: [f64; 3], item: Self::ItemStack);
    fn level_event(&mut self, event_id: i32, pos: BlockPos, data: i32);
}

pub trait BlockSource {
    type Level: LevelLike;

    fn state_facing(&self) -> Direction;
    fn level(&mut self) -> &mut Self::Level;
    fn pos(&self) -> BlockPos;
    fn block_entity_insert_item(
        &mut self,
        item: <Self::Level as LevelLike>::ItemStack,
    ) -> <Self::Level as LevelLike>::ItemStack;
    fn dispense_position(&self) -> Position;
}

pub const DEFAULT_ACCURACY: i32 = 6;

// JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#spawnItem
pub fn spawn_item<L: LevelLike>(
    level: &mut L,
    item: L::ItemStack,
    accuracy: i32,
    direction: Direction,
    pos: Position,
) {
    let mut x = pos.x;
    let mut y = pos.y;
    let z = pos.z;
    if direction.get_axis() == Axis::Y {
        y -= 0.125;
    } else {
        y -= 0.15625;
    }

    let (dx, dy, dz) = {
        let rng = level.random();
        let power = rng.next_double() * 0.1 + 0.2;
        let dx = rng.triangle(
            (direction.get_step_x() as f64) * power,
            0.0172275 * (accuracy as f64),
        );
        let dy = rng.triangle(0.2, 0.0172275 * (accuracy as f64));
        let dz = rng.triangle(
            (direction.get_step_z() as f64) * power,
            0.0172275 * (accuracy as f64),
        );
        (dx, dy, dz)
    };

    level.add_fresh_entity(Position { x, y, z }, [dx, dy, dz], item);
}

// JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#playDefaultSound
fn play_default_sound<S: BlockSource>(source: &mut S) {
    let pos = source.pos();
    source.level().level_event(1000, pos, 0);
}

// JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#playDefaultAnimation
fn play_default_animation<S: BlockSource>(source: &mut S, direction: Direction) {
    let pos = source.pos();
    let data = direction.get_3d_data_value();
    source.level().level_event(2000, pos, data);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDispenseItemBehavior;

impl DefaultDispenseItemBehavior {
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#dispense
    pub fn dispense<S: BlockSource>(
        &mut self,
        source: &mut S,
        stack: <S::Level as LevelLike>::ItemStack,
    ) -> <S::Level as LevelLike>::ItemStack {
        let result = self.execute(source, stack);
        self.play_sound(source);
        let facing = source.state_facing();
        self.play_animation(source, facing);
        result
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#execute
    pub fn execute<S: BlockSource>(
        &mut self,
        source: &mut S,
        mut stack: <S::Level as LevelLike>::ItemStack,
    ) -> <S::Level as LevelLike>::ItemStack {
        let direction = source.state_facing();
        let dispense_pos = source.dispense_position();
        let split_item = stack.split(1);
        spawn_item(source.level(), split_item, DEFAULT_ACCURACY, direction, dispense_pos);
        stack
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#playSound
    pub fn play_sound<S: BlockSource>(&mut self, source: &mut S) {
        play_default_sound(source);
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#playAnimation
    pub fn play_animation<S: BlockSource>(&mut self, source: &mut S, direction: Direction) {
        play_default_animation(source, direction);
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#consumeWithRemainder
    pub fn consume_with_remainder<S: BlockSource>(
        &mut self,
        source: &mut S,
        mut stack: <S::Level as LevelLike>::ItemStack,
        remainder: <S::Level as LevelLike>::ItemStack,
    ) -> <S::Level as LevelLike>::ItemStack {
        stack.shrink(1);
        if stack.is_empty() {
            return remainder;
        }
        self.add_to_inventory_or_dispense(source, remainder);
        stack
    }

    // JAVA: net.minecraft.core.dispenser.DefaultDispenseItemBehavior#addToInventoryOrDispense
    fn add_to_inventory_or_dispense<S: BlockSource>(
        &mut self,
        source: &mut S,
        item: <S::Level as LevelLike>::ItemStack,
    ) {
        let leftover = source.block_entity_insert_item(item);
        if !leftover.is_empty() {
            let direction = source.state_facing();
            let dispense_pos = source.dispense_position();
            spawn_item(
                source.level(),
                leftover,
                DEFAULT_ACCURACY,
                direction,
                dispense_pos,
            );
            play_default_sound(source);
            play_default_animation(source, direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestItem {
        id: i32,
        count: i32,
    }

    impl ItemStackLike for TestItem {
        fn split(&mut self, amount: i32) -> Self {
            let taken = self.count.min(amount);
            self.count -= taken;
            TestItem { id: self.id, count: taken }
        }
        fn shrink(&mut self, amount: i32) {
            self.count -= amount;
        }
        fn is_empty(&self) -> bool {
            self.count <= 0
        }
    }

    #[derive(Clone, Debug)]
    struct TestRandom {
        doubles: Vec<f64>,
        idx: usize,
    }

    impl TestRandom {
        fn new(doubles: Vec<f64>) -> Self {
            Self { doubles, idx: 0 }
        }
    }

    impl DispenseRandom for TestRandom {
        fn next_double(&mut self) -> f64 {
            let v = self.doubles.get(self.idx).copied().unwrap_or(0.0);
            self.idx += 1;
            v
        }
    }

    #[derive(Debug)]
    struct SpawnRecord {
        pos: Position,
        velocity: [f64; 3],
        item: TestItem,
    }

    struct TestLevel {
        rng: TestRandom,
        events: Vec<(i32, BlockPos, i32)>,
        spawns: Vec<SpawnRecord>,
    }

    impl TestLevel {
        fn new(rng: TestRandom) -> Self {
            Self { rng, events: Vec::new(), spawns: Vec::new() }
        }
    }

    impl LevelLike for TestLevel {
        type Random = TestRandom;
        type ItemStack = TestItem;
        fn random(&mut self) -> &mut Self::Random { &mut self.rng }
        fn add_fresh_entity(&mut self, pos: Position, velocity: [f64; 3], item: Self::ItemStack) {
            self.spawns.push(SpawnRecord { pos, velocity, item });
        }
        fn level_event(&mut self, event_id: i32, pos: BlockPos, data: i32) {
            self.events.push((event_id, pos, data));
        }
    }

    struct TestSource {
        level: TestLevel,
        facing: Direction,
        pos: BlockPos,
        dispense_pos: Position,
        insert_leftover: Option<TestItem>,
    }

    impl TestSource {
        fn new(facing: Direction) -> Self {
            Self {
                level: TestLevel::new(TestRandom::new(vec![0.0; 16])),
                facing,
                pos: BlockPos::new(1, 2, 3),
                dispense_pos: Position::new(1.5, 2.5, 3.5),
                insert_leftover: None,
            }
        }
    }

    impl BlockSource for TestSource {
        type Level = TestLevel;
        fn state_facing(&self) -> Direction { self.facing }
        fn level(&mut self) -> &mut Self::Level { &mut self.level }
        fn pos(&self) -> BlockPos { self.pos }
        fn block_entity_insert_item(&mut self, item: TestItem) -> TestItem {
            self.insert_leftover.take().unwrap_or(TestItem { id: item.id, count: 0 })
        }
        fn dispense_position(&self) -> Position { self.dispense_pos }
    }

    #[test]
    fn spawn_item_y_axis_offset_and_velocity_order() {
        // randoms: power next_double, then 2 per triangle (dx,dy,dz)
        let mut level = TestLevel::new(TestRandom::new(vec![
            0.5, // power = 0.5*0.1+0.2 = 0.25
            1.0, 0.0, // dx triangle: mean + spread*(1-0)
            0.0, 1.0, // dy triangle: mean + spread*(0-1)
            0.5, 0.5, // dz triangle: mean + spread*(0)
        ]));
        let item = TestItem { id: 1, count: 1 };
        spawn_item(&mut level, item, 6, Direction::Up, Position::new(0.0, 1.0, 0.0));
        assert_eq!(level.spawns.len(), 1);
        let s = &level.spawns[0];
        assert_eq!(s.pos.y.to_bits(), (1.0f64 - 0.125).to_bits());
        let spread = 0.0172275 * 6.0;
        let power = 0.5 * 0.1 + 0.2;
        assert_eq!(s.velocity[0].to_bits(), (((Direction::Up.get_step_x() as f64) * power) + spread * (1.0 - 0.0)).to_bits());
        assert_eq!(s.velocity[1].to_bits(), (0.2 + spread * (0.0 - 1.0)).to_bits());
        assert_eq!(s.velocity[2].to_bits(), (((Direction::Up.get_step_z() as f64) * power) + spread * (0.5 - 0.5)).to_bits());
    }

    #[test]
    fn spawn_item_horizontal_axis_offset() {
        let mut level = TestLevel::new(TestRandom::new(vec![0.0; 7]));
        spawn_item(&mut level, TestItem { id: 2, count: 1 }, 6, Direction::North, Position::new(0.0, 1.0, 0.0));
        let s = &level.spawns[0];
        assert_eq!(s.pos.y.to_bits(), (1.0f64 - 0.15625).to_bits());
    }

    #[test]
    fn execute_splits_one_and_dispenses() {
        let mut src = TestSource::new(Direction::East);
        let mut beh = DefaultDispenseItemBehavior::new();
        let remaining = beh.execute(&mut src, TestItem { id: 5, count: 3 });
        assert_eq!(remaining.count, 2);
        assert_eq!(src.level.spawns.len(), 1);
        assert_eq!(src.level.spawns[0].item.count, 1);
    }

    #[test]
    fn dispense_plays_sound_and_animation() {
        let mut src = TestSource::new(Direction::West);
        let mut beh = DefaultDispenseItemBehavior::new();
        let _ = beh.dispense(&mut src, TestItem { id: 7, count: 1 });
        assert_eq!(src.level.events, vec![
            (1000, BlockPos::new(1, 2, 3), 0),
            (2000, BlockPos::new(1, 2, 3), Direction::West.get_3d_data_value()),
        ]);
    }

    #[test]
    fn consume_with_remainder_inserts_or_dispenses() {
        let mut src = TestSource::new(Direction::South);
        src.insert_leftover = Some(TestItem { id: 9, count: 1 });
        let mut beh = DefaultDispenseItemBehavior::new();
        let out = beh.consume_with_remainder(&mut src, TestItem { id: 1, count: 2 }, TestItem { id: 9, count: 1 });
        assert_eq!(out.count, 1);
        assert_eq!(src.level.spawns.len(), 1); // leftover dispensed
        assert!(src.level.events.iter().any(|e| e.0 == 1000));
        assert!(src.level.events.iter().any(|e| e.0 == 2000));
    }

    #[test]
    fn consume_with_remainder_returns_remainder_when_empty() {
        let mut src = TestSource::new(Direction::North);
        let mut beh = DefaultDispenseItemBehavior::new();
        let out = beh.consume_with_remainder(&mut src, TestItem { id: 1, count: 1 }, TestItem { id: 9, count: 1 });
        assert_eq!(out, TestItem { id: 9, count: 1 });
        assert!(src.level.spawns.is_empty());
    }
}
