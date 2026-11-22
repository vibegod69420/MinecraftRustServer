#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
    pub fn step_x(self) -> f64 {
        match self {
            Self::East => 1.0,
            Self::West => -1.0,
            _ => 0.0,
        }
    }

    pub fn step_y(self) -> f64 {
        match self {
            Self::Up => 1.0,
            Self::Down => -1.0,
            _ => 0.0,
        }
    }

    pub fn step_z(self) -> f64 {
        match self {
            Self::South => 1.0,
            Self::North => -1.0,
            _ => 0.0,
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

pub trait BlockSourceAccess {
    fn pos(&self) -> BlockPos;
    fn center(&self) -> Vec3;
}

pub trait ServerLevel {
    fn level_event(&mut self, event: i32, pos: BlockPos, data: i32);
    fn spawn_projectile_using_shoot(
        &mut self,
        projectile: i64,
        stack: &ItemStack,
        x: f64,
        y: f64,
        z: f64,
        power: f32,
        uncertainty: f32,
    );
}

pub trait BlockSource: BlockSourceAccess {
    type Level: ServerLevel;
    fn level(&mut self) -> &mut Self::Level;
    fn facing(&self) -> Direction;
}

#[derive(Clone, Copy, Debug)]
pub struct DispenseConfig {
    pub position_fn: fn(&dyn BlockSourceAccess, Direction) -> Vec3,
    pub power: f32,
    pub uncertainty: f32,
    pub override_dispense_event: Option<i32>,
}

pub trait ProjectileItem {
    fn create_dispense_config(&self) -> DispenseConfig;
    fn as_projectile(
        &self,
        level: &mut dyn ServerLevel,
        pos: Vec3,
        stack: &ItemStack,
        dir: Direction,
    ) -> i64;
}

pub struct ProjectileDispenseBehavior<P: ProjectileItem> {
    projectile_item: P,
    dispense_config: DispenseConfig,
}

impl<P: ProjectileItem> ProjectileDispenseBehavior<P> {
    // JAVA: net.minecraft.core.dispenser.ProjectileDispenseBehavior#<init>
    pub fn new(item: P) -> Self {
        let dispense_config = item.create_dispense_config();
        Self {
            projectile_item: item,
            dispense_config,
        }
    }

    // JAVA: net.minecraft.core.dispenser.ProjectileDispenseBehavior#execute
    pub fn execute<S: BlockSource>(&self, source: &mut S, stack: &mut ItemStack) -> ItemStack {
        let dir = source.facing();
        let pos = (self.dispense_config.position_fn)(source as &dyn BlockSourceAccess, dir);
        let power = self.dispense_config.power;
        let uncertainty = self.dispense_config.uncertainty;
        let projectile = {
            let level = source.level();
            self.projectile_item
                .as_projectile(level, pos, stack, dir)
        };
        let level = source.level();
        level.spawn_projectile_using_shoot(
            projectile,
            stack,
            dir.step_x(),
            dir.step_y(),
            dir.step_z(),
            power,
            uncertainty,
        );
        stack.shrink(1);
        stack.clone()
    }

    // JAVA: net.minecraft.core.dispenser.ProjectileDispenseBehavior#playSound
    pub fn play_sound<S: BlockSource>(&self, source: &mut S) {
        let event = self.dispense_config.override_dispense_event.unwrap_or(1002);
        let pos = source.pos();
        source.level().level_event(event, pos, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_pos(src: &dyn BlockSourceAccess, _dir: Direction) -> Vec3 {
        let c = src.center();
        Vec3 {
            x: c.x,
            y: c.y + 0.1,
            z: c.z,
        }
    }

    struct MockLevel {
        events: Vec<(i32, BlockPos, i32)>,
        spawns: Vec<(i64, i32, f64, f64, f64, u32, u32)>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                events: Vec::new(),
                spawns: Vec::new(),
            }
        }
    }

    impl ServerLevel for MockLevel {
        fn level_event(&mut self, event: i32, pos: BlockPos, data: i32) {
            self.events.push((event, pos, data));
        }

        fn spawn_projectile_using_shoot(
            &mut self,
            projectile: i64,
            stack: &ItemStack,
            x: f64,
            y: f64,
            z: f64,
            power: f32,
            uncertainty: f32,
        ) {
            self.spawns.push((
                projectile,
                stack.count,
                x,
                y,
                z,
                power.to_bits(),
                uncertainty.to_bits(),
            ));
        }
    }

    struct MockSource {
        level: MockLevel,
        facing: Direction,
        pos: BlockPos,
        center: Vec3,
    }

    impl BlockSourceAccess for MockSource {
        fn pos(&self) -> BlockPos {
            self.pos
        }
        fn center(&self) -> Vec3 {
            self.center
        }
    }

    impl BlockSource for MockSource {
        type Level = MockLevel;
        fn level(&mut self) -> &mut Self::Level {
            &mut self.level
        }
        fn facing(&self) -> Direction {
            self.facing
        }
    }

    #[derive(Clone, Copy)]
    struct MockProjectileItem {
        config: DispenseConfig,
        proj_id: i64,
    }

    impl ProjectileItem for MockProjectileItem {
        fn create_dispense_config(&self) -> DispenseConfig {
            self.config
        }

        fn as_projectile(
            &self,
            _level: &mut dyn ServerLevel,
            _pos: Vec3,
            _stack: &ItemStack,
            _dir: Direction,
        ) -> i64 {
            self.proj_id
        }
    }

    #[test]
    fn execute_spawns_shrinks_and_uses_direction_steps() {
        let item = MockProjectileItem {
            config: DispenseConfig {
                position_fn: default_pos,
                power: 1.1,
                uncertainty: 6.0,
                override_dispense_event: None,
            },
            proj_id: 42,
        };
        let behavior = ProjectileDispenseBehavior::new(item);
        let mut src = MockSource {
            level: MockLevel::new(),
            facing: Direction::North,
            pos: BlockPos { x: 1, y: 2, z: 3 },
            center: Vec3 {
                x: 1.5,
                y: 2.5,
                z: 3.5,
            },
        };
        let mut stack = ItemStack { count: 5 };
        let out = behavior.execute(&mut src, &mut stack);
        assert_eq!(out.count, 4);
        assert_eq!(stack.count, 4);
        assert_eq!(src.level.spawns.len(), 1);
        let s = &src.level.spawns[0];
        assert_eq!(s.0, 42);
        assert_eq!(s.2.to_bits(), 0.0f64.to_bits());
        assert_eq!(s.3.to_bits(), 0.0f64.to_bits());
        assert_eq!(s.4.to_bits(), (-1.0f64).to_bits());
        assert_eq!(s.5, 1.1f32.to_bits());
        assert_eq!(s.6, 6.0f32.to_bits());
    }

    #[test]
    fn play_sound_default_and_override() {
        let mk = |evt| MockProjectileItem {
            config: DispenseConfig {
                position_fn: default_pos,
                power: 0.0,
                uncertainty: 0.0,
                override_dispense_event: evt,
            },
            proj_id: 0,
        };
        let mut src = MockSource {
            level: MockLevel::new(),
            facing: Direction::Up,
            pos: BlockPos { x: -7, y: 0, z: 3 },
            center: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        ProjectileDispenseBehavior::new(mk(None)).play_sound(&mut src);
        assert_eq!(
            src.level.events,
            vec![(1002, BlockPos { x: -7, y: 0, z: 3 }, 0)]
        );

        let mut src2 = MockSource {
            level: MockLevel::new(),
            facing: Direction::Up,
            pos: BlockPos { x: 0, y: 0, z: 0 },
            center: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        ProjectileDispenseBehavior::new(mk(Some(1004))).play_sound(&mut src2);
        assert_eq!(src2.level.events[0].0, 1004);
    }

    #[test]
    fn direction_steps_and_negative_float_bits() {
        assert_eq!(Direction::West.step_x(), -1.0);
        assert_eq!(Direction::East.step_x(), 1.0);
        assert_eq!(Direction::Down.step_y(), -1.0);
        assert_eq!(Direction::Up.step_y(), 1.0);
        assert_eq!(Direction::North.step_z(), -1.0);
        assert_eq!(Direction::South.step_z(), 1.0);

        let item = MockProjectileItem {
            config: DispenseConfig {
                position_fn: |s, _| Vec3 {
                    x: s.center().x - 10.5,
                    y: -1.0,
                    z: s.center().z,
                },
                power: -0.0,
                uncertainty: f32::NAN,
                override_dispense_event: None,
            },
            proj_id: -1,
        };
        let behavior = ProjectileDispenseBehavior::new(item);
        let mut src = MockSource {
            level: MockLevel::new(),
            facing: Direction::West,
            pos: BlockPos { x: -5, y: -1, z: -2 },
            center: Vec3 {
                x: -4.5,
                y: -0.5,
                z: -1.5,
            },
        };
        let mut stack = ItemStack { count: 1 };
        behavior.execute(&mut src, &mut stack);
        assert_eq!(stack.count, 0);
        let s = &src.level.spawns[0];
        assert_eq!(s.2.to_bits(), (-1.0f64).to_bits());
        assert_eq!(s.5, (-0.0f32).to_bits());
        assert!(f32::from_bits(s.6).is_nan());
    }

    #[test]
    fn shrink_wraps_on_overflow() {
        let mut stack = ItemStack { count: i32::MIN };
        stack.shrink(1);
        assert_eq!(stack.count, i32::MAX);
        let mut stack2 = ItemStack { count: 0 };
        stack2.shrink(-1);
        assert_eq!(stack2.count, 1);
    }
}
