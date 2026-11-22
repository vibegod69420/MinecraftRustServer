#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn relative(self, dir: Direction) -> Self {
        Self {
            x: self.x.wrapping_add(dir.step_x()),
            y: self.y.wrapping_add(dir.step_y()),
            z: self.z.wrapping_add(dir.step_z()),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn step_x(self) -> i32 {
        match self {
            Self::East => 1,
            Self::West => -1,
            _ => 0,
        }
    }

    pub fn step_y(self) -> i32 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
            _ => 0,
        }
    }

    pub fn step_z(self) -> i32 {
        match self {
            Self::South => 1,
            Self::North => -1,
            _ => 0,
        }
    }

    pub fn to_y_rot(self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 90.0,
            Self::North => 180.0,
            Self::East => 270.0,
            Self::Up | Self::Down => 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#shrink
    pub fn shrink(&mut self, amount: i32) {
        self.count = self.count.wrapping_sub(amount);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoatType {
    pub width: f32,
}

pub trait BoatDispenseContext {
    fn facing(&self) -> Direction;
    fn center(&self) -> Vec3;
    fn pos(&self) -> BlockPos;
    fn is_water(&self, pos: BlockPos) -> bool;
    fn is_air(&self, pos: BlockPos) -> bool;
    fn create_boat(&mut self, boat_type: BoatType) -> Option<u64>;
    fn set_boat_initial_pos(&mut self, boat: u64, x: f64, y: f64, z: f64);
    fn apply_default_stack_config(&mut self, boat: u64, stack: &ItemStack);
    fn set_boat_y_rot(&mut self, boat: u64, y_rot: f32);
    fn add_fresh_entity(&mut self, boat: u64);
    fn default_dispense(&mut self, stack: &mut ItemStack) -> ItemStack;
    fn level_event(&mut self, event: i32, pos: BlockPos, data: i32);
}

pub struct BoatDispenseItemBehavior {
    boat_type: BoatType,
}

impl BoatDispenseItemBehavior {
    // JAVA: net.minecraft.core.dispenser.BoatDispenseItemBehavior#<init>
    pub fn new(boat_type: BoatType) -> Self {
        Self { boat_type }
    }

    // JAVA: net.minecraft.core.dispenser.BoatDispenseItemBehavior#execute
    pub fn execute<C: BoatDispenseContext>(
        &self,
        ctx: &mut C,
        stack: &mut ItemStack,
    ) -> ItemStack {
        let dir = ctx.facing();
        let center = ctx.center();
        let d5 = 0.5625_f64 + (self.boat_type.width as f64) / 2.0_f64;
        let x = center.x + (dir.step_x() as f64) * d5;
        let y_base = center.y + f64::from((dir.step_y() as f32) * 1.125_f32);
        let z = center.z + (dir.step_z() as f64) * d5;
        let target_pos = ctx.pos().relative(dir);
        let y_offset = if ctx.is_water(target_pos) {
            1.0_f64
        } else {
            if !ctx.is_air(target_pos) || !ctx.is_water(target_pos.below()) {
                return ctx.default_dispense(stack);
            }
            0.0_f64
        };

        if let Some(boat) = ctx.create_boat(self.boat_type) {
            ctx.set_boat_initial_pos(boat, x, y_base + y_offset, z);
            ctx.apply_default_stack_config(boat, stack);
            ctx.set_boat_y_rot(boat, dir.to_y_rot());
            ctx.add_fresh_entity(boat);
            stack.shrink(1);
        }

        stack.clone()
    }

    // JAVA: net.minecraft.core.dispenser.BoatDispenseItemBehavior#playSound
    pub fn play_sound<C: BoatDispenseContext>(&self, ctx: &mut C) {
        let pos = ctx.pos();
        ctx.level_event(1000, pos, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        facing: Direction,
        center: Vec3,
        pos: BlockPos,
        water: Vec<BlockPos>,
        air: Vec<BlockPos>,
        events: Vec<(i32, BlockPos, i32)>,
        boats: Vec<(u64, f64, f64, f64, f32)>,
        stack_configs: Vec<(u64, i32)>,
        added: Vec<u64>,
        next_boat: u64,
        create_none: bool,
        default_calls: i32,
    }

    impl MockCtx {
        fn new(facing: Direction, center: Vec3, pos: BlockPos) -> Self {
            Self {
                facing,
                center,
                pos,
                water: Vec::new(),
                air: Vec::new(),
                events: Vec::new(),
                boats: Vec::new(),
                stack_configs: Vec::new(),
                added: Vec::new(),
                next_boat: 1,
                create_none: false,
                default_calls: 0,
            }
        }
    }

    impl BoatDispenseContext for MockCtx {
        fn facing(&self) -> Direction {
            self.facing
        }
        fn center(&self) -> Vec3 {
            self.center
        }
        fn pos(&self) -> BlockPos {
            self.pos
        }
        fn is_water(&self, pos: BlockPos) -> bool {
            self.water.contains(&pos)
        }
        fn is_air(&self, pos: BlockPos) -> bool {
            self.air.contains(&pos)
        }
        fn create_boat(&mut self, _boat_type: BoatType) -> Option<u64> {
            if self.create_none {
                return None;
            }
            let id = self.next_boat;
            self.next_boat = self.next_boat.wrapping_add(1);
            Some(id)
        }
        fn set_boat_initial_pos(&mut self, boat: u64, x: f64, y: f64, z: f64) {
            if let Some(b) = self.boats.iter_mut().find(|b| b.0 == boat) {
                b.1 = x;
                b.2 = y;
                b.3 = z;
            } else {
                self.boats.push((boat, x, y, z, 0.0));
            }
        }
        fn apply_default_stack_config(&mut self, boat: u64, stack: &ItemStack) {
            self.stack_configs.push((boat, stack.count));
        }
        fn set_boat_y_rot(&mut self, boat: u64, y_rot: f32) {
            if let Some(b) = self.boats.iter_mut().find(|b| b.0 == boat) {
                b.4 = y_rot;
            } else {
                self.boats.push((boat, 0.0, 0.0, 0.0, y_rot));
            }
        }
        fn add_fresh_entity(&mut self, boat: u64) {
            self.added.push(boat);
        }
        fn default_dispense(&mut self, stack: &mut ItemStack) -> ItemStack {
            self.default_calls = self.default_calls.wrapping_add(1);
            stack.clone()
        }
        fn level_event(&mut self, event: i32, pos: BlockPos, data: i32) {
            self.events.push((event, pos, data));
        }
    }

    #[test]
    fn dispense_on_water_places_boat() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 1.375 });
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let mut ctx = MockCtx::new(
            Direction::South,
            Vec3 {
                x: 0.5,
                y: 64.0,
                z: 0.5,
            },
            pos,
        );
        let target = pos.relative(Direction::South);
        ctx.water.push(target);
        let mut stack = ItemStack { count: 2 };
        let out = behavior.execute(&mut ctx, &mut stack);
        assert_eq!(out.count, 1);
        assert_eq!(ctx.default_calls, 0);
        assert_eq!(ctx.added, vec![1]);
        let b = &ctx.boats[0];
        let d5 = 0.5625_f64 + (1.375_f64 / 2.0_f64);
        assert_eq!(b.1.to_bits(), (0.5_f64 + 0.0_f64 * d5).to_bits());
        assert_eq!(b.3.to_bits(), (0.5_f64 + 1.0_f64 * d5).to_bits());
        assert_eq!(b.2.to_bits(), (64.0_f64 + 0.0_f64 + 1.0_f64).to_bits());
        assert_eq!(b.4.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn dispense_on_air_above_water() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 1.0 });
        let pos = BlockPos { x: 1, y: 64, z: 2 };
        let mut ctx = MockCtx::new(
            Direction::East,
            Vec3 {
                x: 1.5,
                y: 64.0,
                z: 2.5,
            },
            pos,
        );
        let target = pos.relative(Direction::East);
        ctx.air.push(target);
        ctx.water.push(target.below());
        let mut stack = ItemStack { count: 1 };
        behavior.execute(&mut ctx, &mut stack);
        assert_eq!(stack.count, 0);
        assert_eq!(ctx.boats[0].2.to_bits(), 64.0f64.to_bits());
        assert_eq!(ctx.boats[0].4.to_bits(), 270.0f32.to_bits());
    }

    #[test]
    fn dispense_falls_back_to_default() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 1.0 });
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        let mut ctx = MockCtx::new(
            Direction::North,
            Vec3 {
                x: 0.5,
                y: 0.0,
                z: 0.5,
            },
            pos,
        );
        let mut stack = ItemStack { count: 3 };
        let out = behavior.execute(&mut ctx, &mut stack);
        assert_eq!(ctx.default_calls, 1);
        assert_eq!(out.count, 3);
        assert!(ctx.added.is_empty());
    }

    #[test]
    fn no_shrink_when_boat_creation_fails() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 1.0 });
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        let mut ctx = MockCtx::new(
            Direction::Up,
            Vec3 {
                x: 0.5,
                y: 0.0,
                z: 0.5,
            },
            pos,
        );
        ctx.water.push(pos.relative(Direction::Up));
        ctx.create_none = true;
        let mut stack = ItemStack { count: 5 };
        let out = behavior.execute(&mut ctx, &mut stack);
        assert_eq!(out.count, 5);
        assert!(ctx.added.is_empty());
    }

    #[test]
    fn play_sound_emits_event_1000() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 1.0 });
        let mut ctx = MockCtx::new(
            Direction::North,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            BlockPos { x: -3, y: 7, z: 2 },
        );
        behavior.play_sound(&mut ctx);
        assert_eq!(
            ctx.events,
            vec![(1000, BlockPos { x: -3, y: 7, z: 2 }, 0)]
        );
    }

    #[test]
    fn negative_coords_vertical_step_and_wrapping() {
        let behavior = BoatDispenseItemBehavior::new(BoatType { width: 0.0 });
        let pos = BlockPos {
            x: -10,
            y: -5,
            z: -20,
        };
        let mut ctx = MockCtx::new(
            Direction::Down,
            Vec3 {
                x: -9.5,
                y: -5.0,
                z: -19.5,
            },
            pos,
        );
        let target = pos.relative(Direction::Down);
        ctx.water.push(target);
        let mut stack = ItemStack { count: 1 };
        behavior.execute(&mut ctx, &mut stack);
        let b = &ctx.boats[0];
        let expected_y_base = -5.0_f64 + f64::from((-1_i32 as f32) * 1.125_f32);
        assert_eq!(b.2.to_bits(), (expected_y_base + 1.0_f64).to_bits());
        assert_eq!(b.4.to_bits(), 0.0f32.to_bits());

        let p = BlockPos {
            x: i32::MAX,
            y: i32::MIN,
            z: 0,
        };
        assert_eq!(p.relative(Direction::East).x, i32::MIN);
        assert_eq!(p.below().y, i32::MAX);
    }
}
