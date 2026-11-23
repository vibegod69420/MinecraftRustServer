use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityTarget {
    This,
    Killer,
    DirectKiller,
    KillerPlayer,
}

impl EntityTarget {
    pub fn context_param(&self) -> ContextKey {
        match self {
            Self::This => ContextKey("minecraft:this_entity"),
            Self::Killer => ContextKey("minecraft:attacker"),
            Self::DirectKiller => ContextKey("minecraft:direct_attacker"),
            Self::KillerPlayer => ContextKey("minecraft:killer_player"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameProfile {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Player {
    pub game_profile: GameProfile,
}

impl Player {
    pub fn get_game_profile(&self) -> GameProfile {
        self.game_profile.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootEntity {
    Player(Player),
    Other,
}

#[derive(Default)]
pub struct LootContext {
    entities: HashMap<&'static str, LootEntity>,
}

impl LootContext {
    pub fn with_entity(mut self, key: &ContextKey, entity: LootEntity) -> Self {
        self.entities.insert(key.0, entity);
        self
    }

    pub fn get_optional_parameter(&self, key: &ContextKey) -> Option<&LootEntity> {
        self.entities.get(key.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: &'static str,
}

pub struct Items;

impl Items {
    pub const PLAYER_HEAD: Item = Item {
        id: "minecraft:player_head",
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvableProfile {
    pub profile: GameProfile,
}

impl ResolvableProfile {
    pub fn create_resolved(profile: GameProfile) -> Self {
        Self { profile }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileComponent;

pub struct DataComponents;

impl DataComponents {
    pub const PROFILE: ProfileComponent = ProfileComponent;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub item: Option<Item>,
    pub profile: Option<ResolvableProfile>,
}

impl ItemStack {
    pub fn is(&self, item: &Item) -> bool {
        self.item.as_ref() == Some(item)
    }

    pub fn set(&mut self, _component: ProfileComponent, value: ResolvableProfile) {
        self.profile = Some(value);
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const FILL_PLAYER_HEAD: LootItemFunctionType =
        LootItemFunctionType("minecraft:fill_player_head");
}

pub struct LootItemConditionalFunctionBuilder<T> {
    build_fn: Box<dyn Fn(Vec<LootItemCondition>) -> T>,
    predicates: Vec<LootItemCondition>,
}

impl<T> LootItemConditionalFunctionBuilder<T> {
    pub fn build(self) -> T {
        (self.build_fn)(self.predicates)
    }
}

pub fn simple_builder<T, F>(f: F) -> LootItemConditionalFunctionBuilder<T>
where
    F: Fn(Vec<LootItemCondition>) -> T + 'static,
{
    LootItemConditionalFunctionBuilder {
        build_fn: Box::new(f),
        predicates: Vec::new(),
    }
}

pub struct FillPlayerHead {
    pub predicates: Vec<LootItemCondition>,
    pub entity_target: EntityTarget,
}

impl FillPlayerHead {
    // JAVA: net.minecraft.world.level.storage.loot.functions.FillPlayerHead#<init>
    pub fn new(predicates: Vec<LootItemCondition>, entity_target: EntityTarget) -> Self {
        Self {
            predicates,
            entity_target,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FillPlayerHead#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::FILL_PLAYER_HEAD
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FillPlayerHead#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::from([self.entity_target.context_param()])
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FillPlayerHead#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if stack.is(&Items::PLAYER_HEAD) {
            if let Some(LootEntity::Player(player)) =
                ctx.get_optional_parameter(&self.entity_target.context_param())
            {
                stack.set(
                    DataComponents::PROFILE,
                    ResolvableProfile::create_resolved(player.get_game_profile()),
                );
            }
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FillPlayerHead#fillPlayerHead
    pub fn fill_player_head(
        entity_target: EntityTarget,
    ) -> LootItemConditionalFunctionBuilder<FillPlayerHead> {
        simple_builder(move |predicates| FillPlayerHead::new(predicates, entity_target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_head_stack() -> ItemStack {
        ItemStack {
            item: Some(Items::PLAYER_HEAD),
            profile: None,
        }
    }

    fn player_entity(name: &str) -> LootEntity {
        LootEntity::Player(Player {
            game_profile: GameProfile {
                name: name.to_string(),
                id: "uuid-1".to_string(),
            },
        })
    }

    #[test]
    fn run_fills_player_head_profile() {
        let f = FillPlayerHead::new(vec![], EntityTarget::This);
        let ctx = LootContext::default().with_entity(
            &EntityTarget::This.context_param(),
            player_entity("Steve"),
        );
        let out = f.run(player_head_stack(), &ctx);
        assert_eq!(out.profile.unwrap().profile.name, "Steve");
    }

    #[test]
    fn run_ignores_non_player_head() {
        let f = FillPlayerHead::new(vec![], EntityTarget::This);
        let stack = ItemStack {
            item: Some(Item {
                id: "minecraft:stone",
            }),
            profile: None,
        };
        let ctx = LootContext::default().with_entity(
            &EntityTarget::This.context_param(),
            player_entity("Steve"),
        );
        let out = f.run(stack, &ctx);
        assert!(out.profile.is_none());
    }

    #[test]
    fn run_ignores_missing_entity() {
        let f = FillPlayerHead::new(vec![], EntityTarget::KillerPlayer);
        let out = f.run(player_head_stack(), &LootContext::default());
        assert!(out.profile.is_none());
    }

    #[test]
    fn run_ignores_non_player_entity() {
        let f = FillPlayerHead::new(vec![], EntityTarget::This);
        let ctx = LootContext::default()
            .with_entity(&EntityTarget::This.context_param(), LootEntity::Other);
        let out = f.run(player_head_stack(), &ctx);
        assert!(out.profile.is_none());
    }

    #[test]
    fn referenced_context_params_contains_target() {
        let f = FillPlayerHead::new(vec![], EntityTarget::KillerPlayer);
        let params = f.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&ContextKey("minecraft:killer_player")));
    }

    #[test]
    fn fill_player_head_builder_builds_function() {
        let f = FillPlayerHead::fill_player_head(EntityTarget::This).build();
        assert_eq!(f.entity_target, EntityTarget::This);
        assert_eq!(f.get_type(), LootItemFunctions::FILL_PLAYER_HEAD);
    }
}
