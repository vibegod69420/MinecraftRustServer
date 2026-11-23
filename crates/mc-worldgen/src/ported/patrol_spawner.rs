//! Port of `net.minecraft.world.level.levelgen.PatrolSpawner` (Minecraft Java 1.21.11).

use std::collections::VecDeque;

/// Minimal block position used by the patrol spawner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

/// Minimal mutable block position used by the patrol spawner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    #[inline]
    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    #[inline]
    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    #[inline]
    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    #[inline]
    pub fn set_z(&mut self, z: i32) {
        self.z = z;
    }

    #[inline]
    pub fn as_pos(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

/// Minimal interface for a patrolling monster entity (e.g. pillager).
pub trait PatrollingMonster {
    fn set_patrol_leader(&mut self, leader: bool);
    fn find_patrol_target(&mut self);
    fn set_pos(&mut self, x: f64, y: f64, z: f64);
}

/// Minimal self-contained level interface required by `PatrolSpawner`.
///
/// This flattens dependencies on Java classes such as `ServerLevel`,
/// `RandomSource`, `GameRules`, `EnvironmentAttributes`, `NaturalSpawner`,
/// `PatrollingMonster` static checks, and entity creation/finalization.
pub trait PatrolLevel {
    type Monster: PatrollingMonster;

    fn random_next_int(&mut self, bound: i32) -> i32;
    fn random_next_boolean(&mut self) -> bool;

    fn game_rules_spawn_patrols(&self) -> bool;
    fn is_bright_outside(&self) -> bool;

    fn player_count(&self) -> i32;
    fn player_is_spectator(&self, index: i32) -> bool;
    fn player_block_position(&self, index: i32) -> BlockPos;

    fn is_close_to_village(&self, pos: BlockPos, sections: i32) -> bool;
    fn has_chunks_at(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> bool;
    fn can_pillager_patrol_spawn_at(&self, pos: BlockPos) -> bool;

    /// Returns `DifficultyInstance.getEffectiveDifficulty()` as `f32`.
    fn current_difficulty_effective(&self, pos: BlockPos) -> f32;
    /// Returns `getHeightmapPos(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, pos)`.
    fn heightmap_pos_motion_blocking_no_leaves(&self, pos: BlockPos) -> BlockPos;

    /// Corresponds to `NaturalSpawner.isValidEmptySpawnBlock(...)` for pillagers.
    fn is_valid_empty_spawn_block_for_pillager(&self, pos: BlockPos) -> bool;
    /// Corresponds to `PatrollingMonster.checkPatrollingMonsterSpawnRules(...)`.
    fn check_pillager_patrol_spawn_rules(&mut self, pos: BlockPos) -> bool;

    /// Corresponds to `EntityType.PILLAGER.create(level, EntitySpawnReason.PATROL)`.
    fn create_pillager_patrol(&mut self) -> Option<Self::Monster>;
    /// Corresponds to `monster.finalizeSpawn(level, difficultyAt(pos), PATROL, null)`.
    fn finalize_pillager_spawn(&mut self, monster: &mut Self::Monster, pos: BlockPos);
    /// Corresponds to `level.addFreshEntityWithPassengers(monster)`.
    fn add_fresh_entity_with_passengers(&mut self, monster: Self::Monster);
}

/// Port of `net.minecraft.world.level.levelgen.PatrolSpawner`.
#[derive(Debug, Clone)]
pub struct PatrolSpawner {
    pub next_tick: i32,
}

impl Default for PatrolSpawner {
    fn default() -> Self {
        Self { next_tick: 0 }
    }
}

impl PatrolSpawner {
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.world.level.levelgen.PatrolSpawner#tick
    pub fn tick<L: PatrolLevel>(&mut self, level: &mut L, spawn_enemies: bool) {
        if spawn_enemies {
            if level.game_rules_spawn_patrols() {
                self.next_tick = self.next_tick.wrapping_sub(1);
                if self.next_tick <= 0 {
                    self.next_tick = self
                        .next_tick
                        .wrapping_add(12_000)
                        .wrapping_add(level.random_next_int(1_200));
                    if level.is_bright_outside() {
                        if level.random_next_int(5) == 0 {
                            let player_count = level.player_count();
                            if player_count >= 1 {
                                let idx = level.random_next_int(player_count);
                                let is_spectator = level.player_is_spectator(idx);
                                let player_pos = level.player_block_position(idx);
                                if !is_spectator {
                                    if !level.is_close_to_village(player_pos, 2) {
                                        let x_off = (24_i32
                                            .wrapping_add(level.random_next_int(24)))
                                            .wrapping_mul(if level.random_next_boolean() {
                                                -1
                                            } else {
                                                1
                                            });
                                        let z_off = (24_i32
                                            .wrapping_add(level.random_next_int(24)))
                                            .wrapping_mul(if level.random_next_boolean() {
                                                -1
                                            } else {
                                                1
                                            });
                                        let mut pos = player_pos.mutable();
                                        pos.move_by(x_off, 0, z_off);

                                        let min_x = pos.x.wrapping_sub(10);
                                        let min_z = pos.z.wrapping_sub(10);
                                        let max_x = pos.x.wrapping_add(10);
                                        let max_z = pos.z.wrapping_add(10);
                                        if level.has_chunks_at(min_x, min_z, max_x, max_z) {
                                            if level.can_pillager_patrol_spawn_at(pos.as_pos()) {
                                                let eff = level
                                                    .current_difficulty_effective(pos.as_pos())
                                                    as f64;
                                                let count =
                                                    (eff.ceil() as i32).wrapping_add(1);
                                                let mut i: i32 = 0;
                                                while i < count {
                                                    let hm_y = level
                                                        .heightmap_pos_motion_blocking_no_leaves(
                                                            pos.as_pos(),
                                                        )
                                                        .y;
                                                    pos.set_y(hm_y);
                                                    if i == 0 {
                                                        if !self.spawn_patrol_member(
                                                            level,
                                                            pos.as_pos(),
                                                            true,
                                                        ) {
                                                            break;
                                                        }
                                                    } else {
                                                        self.spawn_patrol_member(
                                                            level,
                                                            pos.as_pos(),
                                                            false,
                                                        );
                                                    }

                                                    let dx = level
                                                        .random_next_int(5)
                                                        .wrapping_sub(level.random_next_int(5));
                                                    let dz = level
                                                        .random_next_int(5)
                                                        .wrapping_sub(level.random_next_int(5));
                                                    pos.set_x(pos.x.wrapping_add(dx));
                                                    pos.set_z(pos.z.wrapping_add(dz));

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

    // JAVA: net.minecraft.world.level.levelgen.PatrolSpawner#spawnPatrolMember
    fn spawn_patrol_member<L: PatrolLevel>(
        &self,
        level: &mut L,
        pos: BlockPos,
        leader: bool,
    ) -> bool {
        if !level.is_valid_empty_spawn_block_for_pillager(pos) {
            return false;
        }
        if !level.check_pillager_patrol_spawn_rules(pos) {
            return false;
        }
        if let Some(mut monster) = level.create_pillager_patrol() {
            if leader {
                monster.set_patrol_leader(true);
                monster.find_patrol_target();
            }
            monster.set_pos(pos.x as f64, pos.y as f64, pos.z as f64);
            level.finalize_pillager_spawn(&mut monster, pos);
            level.add_fresh_entity_with_passengers(monster);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct MockMonster {
        leader: bool,
        target_found: bool,
        pos: (f64, f64, f64),
        finalized: bool,
    }

    impl PatrollingMonster for MockMonster {
        fn set_patrol_leader(&mut self, leader: bool) {
            self.leader = leader;
        }
        fn find_patrol_target(&mut self) {
            self.target_found = true;
        }
        fn set_pos(&mut self, x: f64, y: f64, z: f64) {
            self.pos = (x, y, z);
        }
    }

    struct MockLevel {
        random_ints: VecDeque<i32>,
        random_bools: VecDeque<bool>,
        spawn_patrols: bool,
        bright_outside: bool,
        players: Vec<(bool, BlockPos)>,
        close_to_village: bool,
        has_chunks: bool,
        can_spawn_at: bool,
        difficulty_eff: f32,
        heightmap_y: i32,
        valid_spawn_block: bool,
        spawn_rules_ok: bool,
        create_ok: bool,
        spawned: Vec<MockMonster>,
    }

    impl Default for MockLevel {
        fn default() -> Self {
            Self {
                random_ints: VecDeque::new(),
                random_bools: VecDeque::new(),
                spawn_patrols: false,
                bright_outside: false,
                players: Vec::new(),
                close_to_village: false,
                has_chunks: false,
                can_spawn_at: false,
                difficulty_eff: 0.0,
                heightmap_y: 0,
                valid_spawn_block: false,
                spawn_rules_ok: false,
                create_ok: false,
                spawned: Vec::new(),
            }
        }
    }

    impl PatrolLevel for MockLevel {
        type Monster = MockMonster;

        fn random_next_int(&mut self, bound: i32) -> i32 {
            let v = self.random_ints.pop_front().unwrap_or(0);
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
        fn random_next_boolean(&mut self) -> bool {
            self.random_bools.pop_front().unwrap_or(false)
        }
        fn game_rules_spawn_patrols(&self) -> bool {
            self.spawn_patrols
        }
        fn is_bright_outside(&self) -> bool {
            self.bright_outside
        }
        fn player_count(&self) -> i32 {
            self.players.len() as i32
        }
        fn player_is_spectator(&self, index: i32) -> bool {
            self.players[index as usize].0
        }
        fn player_block_position(&self, index: i32) -> BlockPos {
            self.players[index as usize].1
        }
        fn is_close_to_village(&self, _pos: BlockPos, _sections: i32) -> bool {
            self.close_to_village
        }
        fn has_chunks_at(&self, _min_x: i32, _min_z: i32, _max_x: i32, _max_z: i32) -> bool {
            self.has_chunks
        }
        fn can_pillager_patrol_spawn_at(&self, _pos: BlockPos) -> bool {
            self.can_spawn_at
        }
        fn current_difficulty_effective(&self, _pos: BlockPos) -> f32 {
            self.difficulty_eff
        }
        fn heightmap_pos_motion_blocking_no_leaves(&self, pos: BlockPos) -> BlockPos {
            BlockPos {
                x: pos.x,
                y: self.heightmap_y,
                z: pos.z,
            }
        }
        fn is_valid_empty_spawn_block_for_pillager(&self, _pos: BlockPos) -> bool {
            self.valid_spawn_block
        }
        fn check_pillager_patrol_spawn_rules(&mut self, _pos: BlockPos) -> bool {
            self.spawn_rules_ok
        }
        fn create_pillager_patrol(&mut self) -> Option<Self::Monster> {
            if self.create_ok {
                Some(MockMonster::default())
            } else {
                None
            }
        }
        fn finalize_pillager_spawn(&mut self, monster: &mut Self::Monster, _pos: BlockPos) {
            monster.finalized = true;
        }
        fn add_fresh_entity_with_passengers(&mut self, monster: Self::Monster) {
            self.spawned.push(monster);
        }
    }

    #[test]
    fn tick_countdown_and_reset() {
        let mut spawner = PatrolSpawner { next_tick: 1 };
        let mut level = MockLevel::default();
        level.spawn_patrols = true;
        level.bright_outside = false;
        level.random_ints.push_back(100); // nextInt(1200)
        spawner.tick(&mut level, true);
        // 1 -> 0, reset: 0 + 12000 + 100
        assert_eq!(spawner.next_tick, 12_100);

        let mut spawner2 = PatrolSpawner { next_tick: 0 };
        let mut level2 = MockLevel::default();
        level2.spawn_patrols = true;
        level2.bright_outside = false;
        level2.random_ints.push_back(7);
        spawner2.tick(&mut level2, true);
        // 0 -> -1, reset: -1 + 12000 + 7
        assert_eq!(spawner2.next_tick, 12_006);
    }

    #[test]
    fn tick_wrapping_integer_semantics() {
        let mut spawner = PatrolSpawner { next_tick: i32::MIN };
        let mut level = MockLevel::default();
        level.spawn_patrols = true;
        spawner.tick(&mut level, true);
        // MIN.wrapping_sub(1) == MAX, MAX <= 0 is false
        assert_eq!(spawner.next_tick, i32::MAX);

        let mut spawner2 = PatrolSpawner {
            next_tick: i32::MIN.wrapping_add(1),
        };
        let mut level2 = MockLevel::default();
        level2.spawn_patrols = true;
        level2.bright_outside = false;
        level2.random_ints.push_back(0);
        spawner2.tick(&mut level2, true);
        // becomes MIN, reset: MIN + 12000 + 0
        assert_eq!(spawner2.next_tick, i32::MIN.wrapping_add(12_000));
    }

    #[test]
    fn spawns_patrol_with_leader_difficulty_and_jitter() {
        let mut spawner = PatrolSpawner { next_tick: 0 };
        let mut level = MockLevel::default();
        level.spawn_patrols = true;
        level.bright_outside = true;
        level.players = vec![(false, BlockPos { x: 100, y: 64, z: 200 })];
        level.has_chunks = true;
        level.can_spawn_at = true;
        level.difficulty_eff = 1.5; // ceil(1.5)+1 = 3
        level.heightmap_y = 70;
        level.valid_spawn_block = true;
        level.spawn_rules_ok = true;
        level.create_ok = true;
        // reset(1200)=0, chance(5)=0, player(1)=0,
        // xOff: nextInt(24)=0, bool=false(+1), zOff: nextInt(24)=1, bool=true(-1)
        // jitter per member: 4x nextInt(5)=0
        level
            .random_ints
            .extend([0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        level.random_bools.extend([false, true]);

        spawner.tick(&mut level, true);

        assert_eq!(level.spawned.len(), 3);
        assert!(level.spawned[0].leader);
        assert!(level.spawned[0].target_found);
        assert!(level.spawned[0].finalized);
        assert_eq!(level.spawned[0].pos, (124.0, 70.0, 175.0));
        assert!(!level.spawned[1].leader);
        assert!(!level.spawned[2].leader);
    }

    #[test]
    fn leader_spawn_failure_breaks_and_non_leader_ignored() {
        let mut spawner = PatrolSpawner { next_tick: 0 };
        let mut level = MockLevel::default();
        level.spawn_patrols = true;
        level.bright_outside = true;
        level.players = vec![(false, BlockPos { x: 0, y: 64, z: 0 })];
        level.has_chunks = true;
        level.can_spawn_at = true;
        level.difficulty_eff = 5.0; // 6 members attempted
        level.heightmap_y = 64;
        level.valid_spawn_block = false; // leader fails
        level.spawn_rules_ok = true;
        level.create_ok = true;
        level.random_ints.extend([0, 0, 0, 0, 0]);
        level.random_bools.extend([false, false]);
        spawner.tick(&mut level, true);
        assert!(level.spawned.is_empty());
    }

    #[test]
    fn negative_coords_and_wrapping_offsets() {
        let mut spawner = PatrolSpawner { next_tick: 0 };
        let mut level = MockLevel::default();
        level.spawn_patrols = true;
        level.bright_outside = true;
        level.players = vec![(
            false,
            BlockPos {
                x: i32::MIN,
                y: 64,
                z: -10,
            },
        )];
        level.has_chunks = true;
        level.can_spawn_at = true;
        level.difficulty_eff = 0.0; // ceil(0)+1 = 1
        level.heightmap_y = -5;
        level.valid_spawn_block = true;
        level.spawn_rules_ok = true;
        level.create_ok = true;
        // reset, chance, player, xOff int=0 bool=false => +24, zOff int=0 bool=false => +24
        // jitter: 4 ints
        level
            .random_ints
            .extend([0, 0, 0, 0, 0, 0, 0, 0, 0]);
        level.random_bools.extend([false, false]);

        spawner.tick(&mut level, true);
        assert_eq!(level.spawned.len(), 1);
        let expected_x = i32::MIN.wrapping_add(24) as f64;
        let expected_z = (-10_i32).wrapping_add(24) as f64;
        assert_eq!(level.spawned[0].pos.0.to_bits(), expected_x.to_bits());
        assert_eq!(level.spawned[0].pos.1.to_bits(), (-5.0f64).to_bits());
        assert_eq!(level.spawned[0].pos.2.to_bits(), expected_z.to_bits());
    }

    #[test]
    fn difficulty_ceil_uses_f32_to_f64_promotion() {
        // JAVA: (int)Math.ceil(getEffectiveDifficulty()) + 1
        // getEffectiveDifficulty() is float, Math.ceil takes double.
        let eff: f32 = f32::from_bits(0x3f7fffff); // 0.99999994f32
        let count = ((eff as f64).ceil() as i32).wrapping_add(1);
        assert_eq!(count, 2);

        let eff_neg: f32 = -0.5;
        let count_neg = ((eff_neg as f64).ceil() as i32).wrapping_add(1);
        assert_eq!(count_neg, 1); // ceil(-0.5)= -0.0 -> 0 as i32, +1 = 1

        let eff_exact: f32 = 2.0;
        let count_exact = ((eff_exact as f64).ceil() as i32).wrapping_add(1);
        assert_eq!(count_exact, 3);
    }
}
