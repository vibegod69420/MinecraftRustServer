#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    pub fn east(&self, dx: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y,
            z: self.z,
        }
    }

    pub fn south(&self, dz: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub is_spectator: bool,
    pub block_position: BlockPos,
    pub time_since_rest: i32,
}

pub trait PhantomDifficulty {
    fn is_harder_than(&self, value: f32) -> bool;
    fn difficulty_id(&self) -> i32;
}

pub trait PhantomLevel {
    type Difficulty: PhantomDifficulty;
    type BlockState;
    type FluidState;
    type SpawnGroupData;

    fn spawn_phantoms_rule(&self) -> bool;
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
    fn sky_darken(&self) -> i32;
    fn dimension_has_sky_light(&self) -> bool;
    fn sea_level(&self) -> i32;
    fn player_snapshots(&self) -> Vec<PlayerSnapshot>;
    fn can_see_sky(&self, pos: BlockPos) -> bool;
    fn current_difficulty_at(&self, pos: BlockPos) -> Self::Difficulty;
    fn block_state(&self, pos: BlockPos) -> Self::BlockState;
    fn fluid_state(&self, pos: BlockPos) -> Self::FluidState;
    fn is_valid_empty_spawn_block(
        &self,
        pos: BlockPos,
        block_state: &Self::BlockState,
        fluid_state: &Self::FluidState,
    ) -> bool;
    fn try_spawn_phantom(
        &mut self,
        pos: BlockPos,
        difficulty: &Self::Difficulty,
        group_data: Option<Self::SpawnGroupData>,
    ) -> Option<Self::SpawnGroupData>;
}

#[derive(Debug, Default)]
pub struct PhantomSpawner {
    pub next_tick: i32,
}

impl PhantomSpawner {
    // JAVA: net.minecraft.world.level.levelgen.PhantomSpawner#tick
    pub fn tick<L: PhantomLevel>(&mut self, level: &mut L, spawn_mobs: bool) {
        if spawn_mobs {
            if level.spawn_phantoms_rule() {
                self.next_tick = self.next_tick.wrapping_sub(1);
                if self.next_tick <= 0 {
                    self.next_tick = self.next_tick.wrapping_add(
                        60i32
                            .wrapping_add(level.next_int(60))
                            .wrapping_mul(20),
                    );
                    if level.sky_darken() >= 5 || !level.dimension_has_sky_light() {
                        let players = level.player_snapshots();
                        for player in players {
                            if !player.is_spectator {
                                let pos = player.block_position;
                                if !level.dimension_has_sky_light()
                                    || (pos.get_y() >= level.sea_level()
                                        && level.can_see_sky(pos))
                                {
                                    let difficulty = level.current_difficulty_at(pos);
                                    if difficulty
                                        .is_harder_than(level.next_float() * 3.0f32)
                                    {
                                        let time_since_rest =
                                            player.time_since_rest.clamp(1, i32::MAX);
                                        if level.next_int(time_since_rest) >= 72000 {
                                            let spawn_pos = pos
                                                .above(20i32.wrapping_add(level.next_int(15)))
                                                .east((-10i32).wrapping_add(level.next_int(21)))
                                                .south((-10i32).wrapping_add(level.next_int(21)));
                                            let block_state = level.block_state(spawn_pos);
                                            let fluid_state = level.fluid_state(spawn_pos);
                                            if level.is_valid_empty_spawn_block(
                                                spawn_pos,
                                                &block_state,
                                                &fluid_state,
                                            ) {
                                                let mut group_data: Option<L::SpawnGroupData> =
                                                    None;
                                                let count = 1i32.wrapping_add(
                                                    level.next_int(
                                                        difficulty
                                                            .difficulty_id()
                                                            .wrapping_add(1),
                                                    ),
                                                );
                                                let mut i = 0i32;
                                                while i < count {
                                                    group_data = level.try_spawn_phantom(
                                                        spawn_pos,
                                                        &difficulty,
                                                        group_data,
                                                    );
                                                    i = i.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct MockBlock;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct MockFluid;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct MockGroupData(u32);

    #[derive(Clone, Debug)]
    struct MockDifficulty {
        id: i32,
        result: bool,
        captured: Option<Rc<Cell<f32>>>,
    }

    impl PhantomDifficulty for MockDifficulty {
        fn is_harder_than(&self, value: f32) -> bool {
            if let Some(c) = &self.captured {
                c.set(value);
            }
            self.result
        }

        fn difficulty_id(&self) -> i32 {
            self.id
        }
    }

    struct MockLevel {
        spawn_phantoms_rule: bool,
        sky_darken: i32,
        has_sky_light: bool,
        sea_level: i32,
        players: Vec<PlayerSnapshot>,
        difficulty: MockDifficulty,
        can_see_sky: bool,
        valid_spawn_block: bool,
        random_ints: VecDeque<i32>,
        random_floats: VecDeque<f32>,
        spawned: Vec<(BlockPos, Option<MockGroupData>)>,
        next_group_data_id: u32,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                spawn_phantoms_rule: false,
                sky_darken: 0,
                has_sky_light: true,
                sea_level: 63,
                players: Vec::new(),
                difficulty: MockDifficulty {
                    id: 0,
                    result: false,
                    captured: None,
                },
                can_see_sky: false,
                valid_spawn_block: false,
                random_ints: VecDeque::new(),
                random_floats: VecDeque::new(),
                spawned: Vec::new(),
                next_group_data_id: 0,
            }
        }
    }

    impl PhantomLevel for MockLevel {
        type Difficulty = MockDifficulty;
        type BlockState = MockBlock;
        type FluidState = MockFluid;
        type SpawnGroupData = MockGroupData;

        fn spawn_phantoms_rule(&self) -> bool {
            self.spawn_phantoms_rule
        }

        fn next_int(&mut self, _bound: i32) -> i32 {
            self.random_ints.pop_front().unwrap_or(0)
        }

        fn next_float(&mut self) -> f32 {
            self.random_floats.pop_front().unwrap_or(0.0)
        }

        fn sky_darken(&self) -> i32 {
            self.sky_darken
        }

        fn dimension_has_sky_light(&self) -> bool {
            self.has_sky_light
        }

        fn sea_level(&self) -> i32 {
            self.sea_level
        }

        fn player_snapshots(&self) -> Vec<PlayerSnapshot> {
            self.players.clone()
        }

        fn can_see_sky(&self, _pos: BlockPos) -> bool {
            self.can_see_sky
        }

        fn current_difficulty_at(&self, _pos: BlockPos) -> Self::Difficulty {
            self.difficulty.clone()
        }

        fn block_state(&self, _pos: BlockPos) -> Self::BlockState {
            MockBlock
        }

        fn fluid_state(&self, _pos: BlockPos) -> Self::FluidState {
            MockFluid
        }

        fn is_valid_empty_spawn_block(
            &self,
            _pos: BlockPos,
            _block_state: &Self::BlockState,
            _fluid_state: &Self::FluidState,
        ) -> bool {
            self.valid_spawn_block
        }

        fn try_spawn_phantom(
            &mut self,
            pos: BlockPos,
            _difficulty: &Self::Difficulty,
            group_data: Option<Self::SpawnGroupData>,
        ) -> Option<Self::SpawnGroupData> {
            self.spawned.push((pos, group_data));
            self.next_group_data_id = self.next_group_data_id.wrapping_add(1);
            Some(MockGroupData(self.next_group_data_id))
        }
    }

    #[test]
    fn gating_and_tick_reset() {
        let mut spawner = PhantomSpawner { next_tick: 1 };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        level.random_ints = VecDeque::from([0]); // nextInt(60)=0
        spawner.tick(&mut level, true);
        // 1 -> 0, then reset: 0 + (60+0)*20 = 1200
        assert_eq!(spawner.next_tick, 1200);

        let mut spawner2 = PhantomSpawner { next_tick: 5 };
        let mut level2 = MockLevel::new();
        level2.spawn_phantoms_rule = false;
        spawner2.tick(&mut level2, true);
        assert_eq!(spawner2.next_tick, 5); // rule disabled, no change

        let mut spawner3 = PhantomSpawner { next_tick: 5 };
        let mut level3 = MockLevel::new();
        level3.spawn_phantoms_rule = true;
        spawner3.tick(&mut level3, false);
        assert_eq!(spawner3.next_tick, 5); // spawn_mobs false, no change
    }

    #[test]
    fn next_tick_wrapping_and_negative() {
        let mut spawner = PhantomSpawner { next_tick: i32::MIN };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        spawner.tick(&mut level, true);
        // i32::MIN.wrapping_sub(1) == i32::MAX, which is > 0
        assert_eq!(spawner.next_tick, i32::MAX);

        let mut spawner2 = PhantomSpawner { next_tick: -10 };
        let mut level2 = MockLevel::new();
        level2.spawn_phantoms_rule = true;
        level2.random_ints = VecDeque::from([10]);
        spawner2.tick(&mut level2, true);
        // -11 + (60+10)*20 = 1389
        assert_eq!(spawner2.next_tick, 1389);

        let mut spawner3 = PhantomSpawner { next_tick: 0 };
        let mut level3 = MockLevel::new();
        level3.spawn_phantoms_rule = true;
        level3.random_ints = VecDeque::from([59]);
        spawner3.tick(&mut level3, true);
        // -1 + (60+59)*20 = 2379
        assert_eq!(spawner3.next_tick, 2379);
    }

    #[test]
    fn float_multiplication_uses_f32_bits() {
        let captured = Rc::new(Cell::new(0.0f32));
        let mut spawner = PhantomSpawner { next_tick: 1 };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        level.sky_darken = 10;
        level.has_sky_light = true;
        level.sea_level = 64;
        level.can_see_sky = true;
        level.players = vec![PlayerSnapshot {
            is_spectator: false,
            block_position: BlockPos::new(0, 64, 0),
            time_since_rest: 72001,
        }];
        level.difficulty = MockDifficulty {
            id: 2,
            result: false,
            captured: Some(captured.clone()),
        };
        level.random_ints = VecDeque::from([0]);
        level.random_floats = VecDeque::from([0.1f32]);
        spawner.tick(&mut level, true);

        let got = captured.get();
        let expected = 0.1f32 * 3.0f32;
        assert_eq!(got.to_bits(), expected.to_bits());
        assert!(level.spawned.is_empty());
    }

    #[test]
    fn time_since_rest_clamped_and_threshold() {
        let mut spawner = PhantomSpawner { next_tick: 1 };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        level.sky_darken = 10;
        level.has_sky_light = true;
        level.sea_level = 0;
        level.can_see_sky = true;
        level.valid_spawn_block = true;
        level.difficulty = MockDifficulty {
            id: 1,
            result: true,
            captured: None,
        };
        level.players = vec![PlayerSnapshot {
            is_spectator: false,
            block_position: BlockPos::new(0, 64, 0),
            time_since_rest: -100,
        }];
        // reset nextInt(60)=0, nextFloat consumed, nextInt(clamp(-100,1,MAX)=1)=0
        level.random_ints = VecDeque::from([0, 0]);
        level.random_floats = VecDeque::from([0.0]);
        spawner.tick(&mut level, true);
        assert!(level.spawned.is_empty());
    }

    #[test]
    fn spawn_count_group_data_and_position() {
        let mut spawner = PhantomSpawner { next_tick: 1 };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        level.sky_darken = 10;
        level.has_sky_light = true;
        level.sea_level = 0;
        level.can_see_sky = true;
        level.valid_spawn_block = true;
        level.difficulty = MockDifficulty {
            id: 2,
            result: true,
            captured: None,
        };
        level.players = vec![PlayerSnapshot {
            is_spectator: false,
            block_position: BlockPos::new(10, 64, 20),
            time_since_rest: 100_000,
        }];
        // nextInt(60)=0, nextInt(100000)=72000, nextInt(15)=5, nextInt(21)=10, nextInt(21)=20,
        // nextInt(difficulty_id+1=3)=2 => count=3
        level.random_ints = VecDeque::from([0, 72_000, 5, 10, 20, 2]);
        level.random_floats = VecDeque::from([0.0]);
        spawner.tick(&mut level, true);

        let expected_pos = BlockPos::new(10, 64, 20)
            .above(20 + 5)
            .east(-10 + 10)
            .south(-10 + 20);
        assert_eq!(expected_pos, BlockPos::new(10, 89, 30));
        assert_eq!(level.spawned.len(), 3);
        for (pos, _) in &level.spawned {
            assert_eq!(*pos, expected_pos);
        }
        assert_eq!(level.spawned[0].1, None);
        assert_eq!(level.spawned[1].1, Some(MockGroupData(1)));
        assert_eq!(level.spawned[2].1, Some(MockGroupData(2)));
    }

    #[test]
    fn negative_offsets_and_boundaries() {
        let mut spawner = PhantomSpawner { next_tick: 1 };
        let mut level = MockLevel::new();
        level.spawn_phantoms_rule = true;
        level.sky_darken = 5; // boundary
        level.has_sky_light = true;
        level.sea_level = 64;
        level.can_see_sky = true;
        level.valid_spawn_block = true;
        level.difficulty = MockDifficulty {
            id: 0,
            result: true,
            captured: None,
        };
        level.players = vec![PlayerSnapshot {
            is_spectator: false,
            block_position: BlockPos::new(0, 64, 0),
            time_since_rest: 72_001,
        }];
        // reset=0, nextInt(72001)=72000, nextInt(15)=0, nextInt(21)=0, nextInt(21)=0, nextInt(1)=0
        level.random_ints = VecDeque::from([0, 72_000, 0, 0, 0, 0]);
        level.random_floats = VecDeque::from([0.0]);
        spawner.tick(&mut level, true);

        assert_eq!(level.spawned.len(), 1);
        assert_eq!(level.spawned[0].0, BlockPos::new(-10, 84, -10));
    }
}
