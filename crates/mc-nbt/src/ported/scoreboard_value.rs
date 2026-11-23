use std::collections::BTreeSet;
use std::fmt::Debug;

/// Minimal self-contained representation of a loot context parameter key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub String);

/// Minimal port of loot number provider type identity used by `ScoreboardValue#getType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootNumberProviderType {
    Score,
}

/// Minimal port of `net.minecraft.world.level.storage.loot.LootContext.EntityTarget`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityTarget {
    This,
    Killer,
    DirectKiller,
    KillerPlayer,
}

impl EntityTarget {
    /// Returns the context parameter key associated with this entity target.
    pub fn context_param(self) -> ContextKey {
        let name = match self {
            Self::This => "minecraft:this_entity",
            Self::Killer => "minecraft:attacker",
            Self::DirectKiller => "minecraft:direct_attacker",
            Self::KillerPlayer => "minecraft:attacking_player",
        };
        ContextKey(name.to_string())
    }
}

/// Minimal abstraction for `net.minecraft.world.scores.ScoreHolder`.
pub trait ScoreHolder: Debug {}

/// Minimal abstraction for `net.minecraft.world.scores.ReadOnlyScoreInfo`.
pub trait ReadOnlyScoreInfo {
    fn value(&self) -> i32;
}

/// Minimal abstraction for `net.minecraft.world.scores.Objective`.
pub trait Objective: Debug {}

/// Minimal abstraction for `net.minecraft.world.scores.Scoreboard`.
pub trait Scoreboard {
    type Obj: Objective + ?Sized;
    type Info: ReadOnlyScoreInfo + ?Sized;

    fn get_objective(&self, name: &str) -> Option<&Self::Obj>;
    fn get_player_score_info<H: ScoreHolder + ?Sized>(
        &self,
        holder: &H,
        objective: &Self::Obj,
    ) -> Option<&Self::Info>;
}

/// Minimal abstraction for a level exposing a scoreboard.
pub trait Level {
    type Sb: Scoreboard;

    fn get_scoreboard(&self) -> &Self::Sb;
}

/// Minimal abstraction for `net.minecraft.world.level.storage.loot.LootContext`.
pub trait LootContext {
    type Lvl: Level;
    type Holder: ScoreHolder;

    fn get_level(&self) -> &Self::Lvl;
    fn get_context_entity(&self, target: EntityTarget) -> Option<&Self::Holder>;
}

/// Minimal abstraction for `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider`.
pub trait ScoreboardNameProvider<C: LootContext + ?Sized> {
    type Holder: ScoreHolder + ?Sized;

    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey>;
    fn get_score_holder<'a>(&'a self, ctx: &'a C) -> Option<&'a Self::Holder>;
}

/// Minimal port of `net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextScoreboardNameProvider {
    pub target: EntityTarget,
}

impl ContextScoreboardNameProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#forTarget
    pub fn for_target(target: EntityTarget) -> Self {
        Self { target }
    }
}

impl<C: LootContext + ?Sized> ScoreboardNameProvider<C> for ContextScoreboardNameProvider {
    type Holder = C::Holder;

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = BTreeSet::new();
        set.insert(self.target.context_param());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#getScoreHolder
    fn get_score_holder<'a>(&'a self, ctx: &'a C) -> Option<&'a Self::Holder> {
        ctx.get_context_entity(self.target)
    }
}

/// Port of `net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue`.
#[derive(Debug, Clone)]
pub struct ScoreboardValue<P> {
    pub target: P,
    pub score: String,
    pub scale: f32,
}

impl<P> ScoreboardValue<P> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue#getType
    pub fn get_type(&self) -> LootNumberProviderType {
        LootNumberProviderType::Score
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue#getReferencedContextParams
    pub fn get_referenced_context_params<C>(&self) -> BTreeSet<ContextKey>
    where
        C: LootContext + ?Sized,
        P: ScoreboardNameProvider<C>,
    {
        self.target.get_referenced_context_params()
    }
}

impl ScoreboardValue<ContextScoreboardNameProvider> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue#fromScoreboard
    pub fn from_scoreboard(target: EntityTarget, score: impl Into<String>) -> Self {
        Self::from_scoreboard_with_scale(target, score, 1.0f32)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue#fromScoreboard
    pub fn from_scoreboard_with_scale(
        target: EntityTarget,
        score: impl Into<String>,
        scale: f32,
    ) -> Self {
        Self {
            target: ContextScoreboardNameProvider::for_target(target),
            score: score.into(),
            scale,
        }
    }
}

impl<P> ScoreboardValue<P> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue#getFloat
    pub fn get_float<C>(&self, ctx: &C) -> f32
    where
        C: LootContext + ?Sized,
        P: ScoreboardNameProvider<C>,
    {
        let Some(holder) = self.target.get_score_holder(ctx) else {
            return 0.0f32;
        };

        let scoreboard = ctx.get_level().get_scoreboard();
        let Some(objective) = scoreboard.get_objective(&self.score) else {
            return 0.0f32;
        };

        let Some(info) = scoreboard.get_player_score_info(holder, objective) else {
            return 0.0f32;
        };

        (info.value() as f32) * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug)]
    struct MockHolder;
    impl ScoreHolder for MockHolder {}

    #[derive(Debug)]
    struct MockObjective(String);
    impl Objective for MockObjective {}

    #[derive(Debug)]
    struct MockInfo(i32);
    impl ReadOnlyScoreInfo for MockInfo {
        fn value(&self) -> i32 {
            self.0
        }
    }

    struct MockScoreboard {
        objectives: BTreeMap<String, MockObjective>,
        infos: BTreeMap<String, MockInfo>,
    }

    impl Scoreboard for MockScoreboard {
        type Obj = MockObjective;
        type Info = MockInfo;

        fn get_objective(&self, name: &str) -> Option<&Self::Obj> {
            self.objectives.get(name)
        }

        fn get_player_score_info<H: ScoreHolder + ?Sized>(
            &self,
            _holder: &H,
            objective: &Self::Obj,
        ) -> Option<&Self::Info> {
            self.infos.get(&objective.0)
        }
    }

    struct MockLevel {
        sb: MockScoreboard,
    }

    impl Level for MockLevel {
        type Sb = MockScoreboard;

        fn get_scoreboard(&self) -> &Self::Sb {
            &self.sb
        }
    }

    struct MockContext {
        level: MockLevel,
        holder: Option<MockHolder>,
    }

    impl LootContext for MockContext {
        type Lvl = MockLevel;
        type Holder = MockHolder;

        fn get_level(&self) -> &Self::Lvl {
            &self.level
        }

        fn get_context_entity(&self, _target: EntityTarget) -> Option<&Self::Holder> {
            self.holder.as_ref()
        }
    }

    fn make_ctx(has_holder: bool, has_objective: bool, score: Option<i32>) -> MockContext {
        let mut objectives = BTreeMap::new();
        let mut infos = BTreeMap::new();
        if has_objective {
            objectives.insert("obj".to_string(), MockObjective("obj".to_string()));
        }
        if let Some(v) = score {
            infos.insert("obj".to_string(), MockInfo(v));
        }
        MockContext {
            level: MockLevel {
                sb: MockScoreboard { objectives, infos },
            },
            holder: if has_holder { Some(MockHolder) } else { None },
        }
    }

    #[test]
    fn get_float_basic_and_scaled() {
        let ctx = make_ctx(true, true, Some(10));
        let sv =
            ScoreboardValue::from_scoreboard_with_scale(EntityTarget::This, "obj", 2.5f32);
        let v = sv.get_float(&ctx);
        let expected = (10i32 as f32) * 2.5f32;
        assert_eq!(v.to_bits(), expected.to_bits());
        assert_eq!(sv.get_type(), LootNumberProviderType::Score);
    }

    #[test]
    fn get_float_missing_returns_zero() {
        let sv = ScoreboardValue::from_scoreboard(EntityTarget::This, "obj");

        let ctx_no_holder = make_ctx(false, true, Some(5));
        assert_eq!(sv.get_float(&ctx_no_holder).to_bits(), 0.0f32.to_bits());

        let ctx_no_obj = make_ctx(true, false, Some(5));
        assert_eq!(sv.get_float(&ctx_no_obj).to_bits(), 0.0f32.to_bits());

        let ctx_no_info = make_ctx(true, true, None);
        assert_eq!(sv.get_float(&ctx_no_info).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn get_float_negative_and_float_bits() {
        let ctx = make_ctx(true, true, Some(-7));
        let sv =
            ScoreboardValue::from_scoreboard_with_scale(EntityTarget::This, "obj", -0.5f32);
        let v = sv.get_float(&ctx);
        let expected = (-7i32 as f32) * -0.5f32;
        assert_eq!(v.to_bits(), expected.to_bits());
        assert!(v.is_sign_positive());
    }

    #[test]
    fn get_float_large_int_and_special_float_bits() {
        let ctx = make_ctx(true, true, Some(i32::MAX));
        let sv = ScoreboardValue::from_scoreboard(EntityTarget::This, "obj");
        let v = sv.get_float(&ctx);
        let expected = (i32::MAX as f32) * 1.0f32;
        assert_eq!(v.to_bits(), expected.to_bits());

        let ctx_neg = make_ctx(true, true, Some(i32::MIN));
        let sv_inf =
            ScoreboardValue::from_scoreboard_with_scale(EntityTarget::This, "obj", f32::INFINITY);
        let v_inf = sv_inf.get_float(&ctx_neg);
        assert_eq!(v_inf.to_bits(), f32::NEG_INFINITY.to_bits());

        let sv_nan =
            ScoreboardValue::from_scoreboard_with_scale(EntityTarget::This, "obj", f32::NAN);
        assert!(sv_nan.get_float(&ctx).is_nan());
    }

    #[test]
    fn referenced_context_params_delegate_to_target() {
        let sv = ScoreboardValue::from_scoreboard(EntityTarget::Killer, "s");
        let params: BTreeSet<ContextKey> =
            sv.get_referenced_context_params::<MockContext>();
        let mut expected = BTreeSet::new();
        expected.insert(EntityTarget::Killer.context_param());
        assert_eq!(params, expected);
    }
}
