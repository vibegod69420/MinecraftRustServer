use std::collections::{HashMap, HashSet};

/// Minimal context key identifier, standing in for `net.minecraft.util.context.ContextKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

/// Minimal entity target enum, standing in for `LootContext.EntityTarget`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityTarget {
    This,
    Killer,
    DirectKiller,
    KillerPlayer,
}

impl EntityTarget {
    // JAVA: net.minecraft.world.level.storage.loot.LootContext.EntityTarget#contextParam
    pub fn context_param(&self) -> ContextKey {
        match self {
            Self::This => ContextKey("this_entity"),
            Self::Killer => ContextKey("attacker"),
            Self::DirectKiller => ContextKey("direct_attacker"),
            Self::KillerPlayer => ContextKey("attacking_player"),
        }
    }
}

/// Minimal loot item condition type marker, standing in for `LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemConditionType(pub &'static str);

pub const ENTITY_SCORES: LootItemConditionType = LootItemConditionType("entity_scores");

/// Minimal read-only score info, standing in for `ReadOnlyScoreInfo`.
pub trait ReadOnlyScoreInfo {
    fn value(&self) -> i32;
}

/// Minimal scoreboard access, standing in for `Scoreboard`.
pub trait Scoreboard<E> {
    type Objective;
    type ScoreInfo: ReadOnlyScoreInfo;
    fn get_objective(&self, name: &str) -> Option<&Self::Objective>;
    fn get_player_score_info(
        &self,
        entity: &E,
        objective: &Self::Objective,
    ) -> Option<&Self::ScoreInfo>;
}

/// Minimal level access, standing in for `Level`.
pub trait Level {
    type Entity;
    type Scoreboard: Scoreboard<Self::Entity>;
    fn get_scoreboard(&self) -> &Self::Scoreboard;
}

/// Minimal loot context access, standing in for `LootContext`.
pub trait LootContext {
    type Entity;
    type Level: Level<Entity = Self::Entity>;
    fn get_optional_parameter(&self, key: ContextKey) -> Option<&Self::Entity>;
    fn get_level(&self) -> &Self::Level;
}

/// Minimal integer range, standing in for `IntRange`.
pub trait IntRange {
    fn get_referenced_context_params(&self) -> Vec<ContextKey>;
    fn test<C: LootContext>(&self, ctx: &C, value: i32) -> bool;
}

/// Faithful port of `net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition`.
#[derive(Debug, Clone)]
pub struct EntityHasScoreCondition<R> {
    pub scores: HashMap<String, R>,
    pub entity_target: EntityTarget,
}

impl<R> EntityHasScoreCondition<R> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        ENTITY_SCORES
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey>
    where
        R: IntRange,
    {
        let mut params = HashSet::new();
        params.insert(self.entity_target.context_param());
        for range in self.scores.values() {
            for p in range.get_referenced_context_params() {
                params.insert(p);
            }
        }
        params
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition#test
    pub fn test<C: LootContext>(&self, ctx: &C) -> bool
    where
        R: IntRange,
    {
        let Some(entity) = ctx.get_optional_parameter(self.entity_target.context_param()) else {
            return false;
        };
        let scoreboard = ctx.get_level().get_scoreboard();
        for (name, range) in &self.scores {
            if !self.has_score(ctx, entity, scoreboard, name, range) {
                return false;
            }
        }
        true
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition#hasScore
    pub fn has_score<C: LootContext>(
        &self,
        ctx: &C,
        entity: &C::Entity,
        scoreboard: &<C::Level as Level>::Scoreboard,
        name: &str,
        range: &R,
    ) -> bool
    where
        R: IntRange,
    {
        let Some(objective) = scoreboard.get_objective(name) else {
            return false;
        };
        let Some(score_info) = scoreboard.get_player_score_info(entity, objective) else {
            return false;
        };
        range.test(ctx, score_info.value())
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition#hasScores
    pub fn has_scores(entity_target: EntityTarget) -> Builder<R> {
        Builder::new(entity_target)
    }
}

/// Builder for `EntityHasScoreCondition`.
#[derive(Debug, Clone)]
pub struct Builder<R> {
    scores: HashMap<String, R>,
    entity_target: EntityTarget,
}

impl<R> Builder<R> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition$Builder#<init>
    pub fn new(entity_target: EntityTarget) -> Self {
        Self {
            scores: HashMap::new(),
            entity_target,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition$Builder#withScore
    pub fn with_score(mut self, name: String, range: R) -> Self {
        self.scores.insert(name, range);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition$Builder#build
    pub fn build(self) -> EntityHasScoreCondition<R> {
        EntityHasScoreCondition {
            scores: self.scores,
            entity_target: self.entity_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Ent(u32);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Obj(String);

    #[derive(Debug, Clone)]
    struct ScoreInfo(i32);

    impl ReadOnlyScoreInfo for ScoreInfo {
        fn value(&self) -> i32 {
            self.0
        }
    }

    struct Board {
        objectives: HashMap<String, Obj>,
        player_scores: HashMap<(u32, String), ScoreInfo>,
    }

    impl Scoreboard<Ent> for Board {
        type Objective = Obj;
        type ScoreInfo = ScoreInfo;

        fn get_objective(&self, name: &str) -> Option<&Self::Objective> {
            self.objectives.get(name)
        }

        fn get_player_score_info(
            &self,
            entity: &Ent,
            objective: &Self::Objective,
        ) -> Option<&Self::ScoreInfo> {
            self.player_scores.get(&(entity.0, objective.0.clone()))
        }
    }

    struct Lvl {
        board: Board,
    }

    impl Level for Lvl {
        type Entity = Ent;
        type Scoreboard = Board;

        fn get_scoreboard(&self) -> &Self::Scoreboard {
            &self.board
        }
    }

    struct Ctx {
        params: HashMap<ContextKey, Ent>,
        level: Lvl,
    }

    impl LootContext for Ctx {
        type Entity = Ent;
        type Level = Lvl;

        fn get_optional_parameter(&self, key: ContextKey) -> Option<&Self::Entity> {
            self.params.get(&key)
        }

        fn get_level(&self) -> &Self::Level {
            &self.level
        }
    }

    #[derive(Clone, Debug)]
    struct ExactRange(i32);

    impl IntRange for ExactRange {
        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            vec![]
        }

        fn test<C: LootContext>(&self, _ctx: &C, value: i32) -> bool {
            value == self.0
        }
    }

    fn mk_ctx(score_name: &str, score: i32) -> Ctx {
        let mut objectives = HashMap::new();
        objectives.insert(score_name.to_string(), Obj(score_name.to_string()));
        let mut player_scores = HashMap::new();
        player_scores.insert((1, score_name.to_string()), ScoreInfo(score));
        Ctx {
            params: HashMap::from([(EntityTarget::This.context_param(), Ent(1))]),
            level: Lvl {
                board: Board {
                    objectives,
                    player_scores,
                },
            },
        }
    }

    #[test]
    fn test_missing_entity_returns_false() {
        let cond = EntityHasScoreCondition {
            scores: HashMap::from([("s".to_string(), ExactRange(1))]),
            entity_target: EntityTarget::This,
        };
        let ctx = Ctx {
            params: HashMap::new(),
            level: Lvl {
                board: Board {
                    objectives: HashMap::new(),
                    player_scores: HashMap::new(),
                },
            },
        };
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_missing_objective_or_score_returns_false() {
        let make_ctx = |has_obj: bool, has_score: bool| {
            let mut objectives = HashMap::new();
            let mut player_scores = HashMap::new();
            if has_obj {
                objectives.insert("s".to_string(), Obj("s".to_string()));
            }
            if has_score {
                player_scores.insert((1, "s".to_string()), ScoreInfo(5));
            }
            Ctx {
                params: HashMap::from([(EntityTarget::This.context_param(), Ent(1))]),
                level: Lvl {
                    board: Board {
                        objectives,
                        player_scores,
                    },
                },
            }
        };
        let cond = EntityHasScoreCondition {
            scores: HashMap::from([("s".to_string(), ExactRange(5))]),
            entity_target: EntityTarget::This,
        };
        assert!(!cond.test(&make_ctx(false, false)));
        assert!(!cond.test(&make_ctx(true, false)));
        assert!(cond.test(&make_ctx(true, true)));
    }

    #[test]
    fn test_negative_and_boundary_scores() {
        #[derive(Clone)]
        struct BoundaryRange;
        impl IntRange for BoundaryRange {
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                vec![]
            }
            fn test<C: LootContext>(&self, _ctx: &C, value: i32) -> bool {
                value == i32::MIN || value == i32::MAX || value == -1
            }
        }
        let cond = EntityHasScoreCondition {
            scores: HashMap::from([("s".to_string(), BoundaryRange)]),
            entity_target: EntityTarget::This,
        };
        assert!(cond.test(&mk_ctx("s", i32::MIN)));
        assert!(cond.test(&mk_ctx("s", i32::MAX)));
        assert!(cond.test(&mk_ctx("s", -1)));
        assert!(!cond.test(&mk_ctx("s", 0)));
    }

    #[test]
    fn test_multiple_scores_all_must_pass() {
        let mut objectives = HashMap::new();
        objectives.insert("a".to_string(), Obj("a".to_string()));
        objectives.insert("b".to_string(), Obj("b".to_string()));
        let mut player_scores = HashMap::new();
        player_scores.insert((1, "a".to_string()), ScoreInfo(10));
        player_scores.insert((1, "b".to_string()), ScoreInfo(20));
        let ctx = Ctx {
            params: HashMap::from([(EntityTarget::This.context_param(), Ent(1))]),
            level: Lvl {
                board: Board {
                    objectives,
                    player_scores,
                },
            },
        };
        let cond_ok = EntityHasScoreCondition {
            scores: HashMap::from([
                ("a".to_string(), ExactRange(10)),
                ("b".to_string(), ExactRange(20)),
            ]),
            entity_target: EntityTarget::This,
        };
        let cond_bad = EntityHasScoreCondition {
            scores: HashMap::from([
                ("a".to_string(), ExactRange(10)),
                ("b".to_string(), ExactRange(99)),
            ]),
            entity_target: EntityTarget::This,
        };
        assert!(cond_ok.test(&ctx));
        assert!(!cond_bad.test(&ctx));
    }

    #[test]
    fn test_referenced_context_params_builder_and_type() {
        #[derive(Clone)]
        struct RangeWithParam(ContextKey);
        impl IntRange for RangeWithParam {
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                vec![self.0]
            }
            fn test<C: LootContext>(&self, _ctx: &C, _value: i32) -> bool {
                true
            }
        }

        let extra = ContextKey("extra");
        let cond = EntityHasScoreCondition::has_scores(EntityTarget::Killer)
            .with_score("a".to_string(), RangeWithParam(extra))
            .build();

        let params = cond.get_referenced_context_params();
        assert!(params.contains(&EntityTarget::Killer.context_param()));
        assert!(params.contains(&extra));
        assert_eq!(cond.entity_target, EntityTarget::Killer);
        assert_eq!(cond.get_type(), ENTITY_SCORES);
    }

    #[test]
    fn test_float_bits_through_score_value() {
        #[derive(Clone)]
        struct FloatBitsRange(u32);
        impl IntRange for FloatBitsRange {
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                vec![]
            }
            fn test<C: LootContext>(&self, _ctx: &C, value: i32) -> bool {
                f32::from_bits(value as u32).to_bits() == self.0
            }
        }

        let nan_bits = f32::NAN.to_bits();
        let cond = EntityHasScoreCondition {
            scores: HashMap::from([("f".to_string(), FloatBitsRange(nan_bits))]),
            entity_target: EntityTarget::This,
        };
        let mut objectives = HashMap::new();
        objectives.insert("f".to_string(), Obj("f".to_string()));
        let mut player_scores = HashMap::new();
        player_scores.insert((1, "f".to_string()), ScoreInfo(nan_bits as i32));
        let ctx = Ctx {
            params: HashMap::from([(EntityTarget::This.context_param(), Ent(1))]),
            level: Lvl {
                board: Board {
                    objectives,
                    player_scores,
                },
            },
        };
        assert!(cond.test(&ctx));

        let neg_zero_cond = EntityHasScoreCondition {
            scores: HashMap::from([(
                "f".to_string(),
                FloatBitsRange((-0.0f32).to_bits()),
            )]),
            entity_target: EntityTarget::This,
        };
        let mut ctx2 = mk_ctx("f", 0);
        ctx2.level.board.player_scores.insert(
            (1, "f".to_string()),
            ScoreInfo((-0.0f32).to_bits() as i32),
        );
        assert!(neg_zero_cond.test(&ctx2));
    }
}
