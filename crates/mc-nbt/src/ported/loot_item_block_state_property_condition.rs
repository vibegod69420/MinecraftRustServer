use std::collections::{BTreeMap, HashSet};

/// Placeholder for Mojang serialization `MapCodec`.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T: ?Sized = ()>(std::marker::PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Minimal `DataResult` placeholder for validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataResult<T> {
    Success(T),
    Error(String),
}

/// Faithful representation of a loot condition type id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemConditionType {
    pub namespace: &'static str,
    pub path: &'static str,
}

/// Minimal registry constants needed by this module.
pub struct LootItemConditions;

impl LootItemConditions {
    pub const BLOCK_STATE_PROPERTY: LootItemConditionType = LootItemConditionType {
        namespace: "minecraft",
        path: "block_state_property",
    };
}

/// Minimal context key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

/// Minimal loot context params.
pub struct LootContextParams;

impl LootContextParams {
    pub const BLOCK_STATE: ContextKey = ContextKey("minecraft:block_state");
}

/// Minimal block state definition for property validation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockStateDefinition {
    pub properties: Vec<String>,
}

/// Minimal block representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub state_definition_properties: Vec<String>,
}

impl Block {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state_definition_properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.state_definition_properties = properties;
        self
    }

    // JAVA: net.minecraft.world.level.block.Block#builtInRegistryHolder
    pub fn built_in_registry_holder(&self) -> Holder<Block> {
        Holder {
            value: self.clone(),
        }
    }

    // JAVA: net.minecraft.world.level.block.state.StateDefinition#getStateDefinition (simplified access)
    pub fn get_state_definition(&self) -> BlockStateDefinition {
        BlockStateDefinition {
            properties: self.state_definition_properties.clone(),
        }
    }
}

/// Minimal holder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Minimal block state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub properties: BTreeMap<String, String>,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            properties: BTreeMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#is
    pub fn is(&self, block: &Holder<Block>) -> bool {
        &self.block == block.value()
    }
}

/// Minimal loot context supporting block state lookup.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub block_state: Option<BlockState>,
}

impl LootContext {
    pub fn with_block_state(block_state: BlockState) -> Self {
        Self {
            block_state: Some(block_state),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getOptionalParameter
    pub fn get_optional_parameter(&self, key: ContextKey) -> Option<&BlockState> {
        if key == LootContextParams::BLOCK_STATE {
            self.block_state.as_ref()
        } else {
            None
        }
    }
}

/// Minimal state properties predicate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatePropertiesPredicate {
    pub properties: BTreeMap<String, String>,
}

impl StatePropertiesPredicate {
    // JAVA: net.minecraft.advancements.criterion.StatePropertiesPredicate#checkState
    pub fn check_state(&self, state_definition: &BlockStateDefinition) -> Option<String> {
        for key in self.properties.keys() {
            if !state_definition.properties.iter().any(|p| p == key) {
                return Some(key.clone());
            }
        }
        None
    }

    // JAVA: net.minecraft.advancements.criterion.StatePropertiesPredicate#matches
    pub fn matches(&self, state: &BlockState) -> bool {
        self.properties
            .iter()
            .all(|(k, v)| state.properties.get(k) == Some(v))
    }
}

/// Minimal builder for `StatePropertiesPredicate`.
#[derive(Debug, Clone, Default)]
pub struct StatePropertiesPredicateBuilder {
    properties: BTreeMap<String, String>,
}

impl StatePropertiesPredicateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    // JAVA: net.minecraft.advancements.criterion.StatePropertiesPredicate$Builder#build
    pub fn build(self) -> Option<StatePropertiesPredicate> {
        Some(StatePropertiesPredicate {
            properties: self.properties,
        })
    }
}

/// Minimal loot item condition trait.
pub trait LootItemCondition {
    fn get_type(&self) -> LootItemConditionType;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
    fn test(&self, ctx: &LootContext) -> bool;
}

/// Minimal builder trait for loot item conditions.
pub trait LootItemConditionBuilder {
    fn build(self) -> LootItemBlockStatePropertyCondition;
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemBlockStatePropertyCondition {
    pub block: Holder<Block>,
    pub properties: Option<StatePropertiesPredicate>,
}

impl LootItemBlockStatePropertyCondition {
    pub const CODEC: MapCodec<LootItemBlockStatePropertyCondition> = MapCodec::new();

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition#validate
    fn validate(condition: Self) -> DataResult<Self> {
        let err = condition
            .properties
            .as_ref()
            .and_then(|p| p.check_state(&condition.block.value().get_state_definition()));
        match err {
            Some(missing) => DataResult::Error(format!(
                "Block {:?} has no property{}",
                condition.block.value().name,
                missing
            )),
            None => DataResult::Success(condition),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition#hasBlockStateProperties
    pub fn has_block_state_properties(block: Block) -> Builder {
        Builder::new(block)
    }
}

impl LootItemCondition for LootItemBlockStatePropertyCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditions::BLOCK_STATE_PROPERTY
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParams::BLOCK_STATE);
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition#test
    fn test(&self, ctx: &LootContext) -> bool {
        let Some(state) = ctx.get_optional_parameter(LootContextParams::BLOCK_STATE) else {
            return false;
        };
        state.is(&self.block)
            && self
                .properties
                .as_ref()
                .is_none_or(|p| p.matches(state))
    }
}

/// Port of `LootItemBlockStatePropertyCondition.Builder`.
#[derive(Debug, Clone)]
pub struct Builder {
    block: Holder<Block>,
    properties: Option<StatePropertiesPredicate>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition$Builder#<init>
    pub fn new(block: Block) -> Self {
        Self {
            block: block.built_in_registry_holder(),
            properties: None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition$Builder#setProperties
    pub fn set_properties(&mut self, properties: StatePropertiesPredicateBuilder) -> &mut Self {
        self.properties = properties.build();
        self
    }
}

impl LootItemConditionBuilder for Builder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition$Builder#build
    fn build(self) -> LootItemBlockStatePropertyCondition {
        LootItemBlockStatePropertyCondition {
            block: self.block,
            properties: self.properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_with_age() -> Block {
        Block::new("minecraft:wheat").with_properties(vec!["age".to_string()])
    }

    #[test]
    fn type_and_context_params() {
        let cond = LootItemBlockStatePropertyCondition {
            block: block_with_age().built_in_registry_holder(),
            properties: None,
        };
        let t = cond.get_type();
        assert_eq!(t.namespace, "minecraft");
        assert_eq!(t.path, "block_state_property");
        let params = cond.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&LootContextParams::BLOCK_STATE));
    }

    #[test]
    fn test_matches_block_without_properties() {
        let block = block_with_age();
        let cond = LootItemBlockStatePropertyCondition {
            block: block.clone().built_in_registry_holder(),
            properties: None,
        };
        let ctx = LootContext::with_block_state(BlockState::new(block));
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_fails_for_missing_or_different_block() {
        let block = block_with_age();
        let cond = LootItemBlockStatePropertyCondition {
            block: block.clone().built_in_registry_holder(),
            properties: None,
        };
        assert!(!cond.test(&LootContext::default()));

        let other = Block::new("minecraft:carrots").with_properties(vec!["age".to_string()]);
        let ctx = LootContext::with_block_state(BlockState::new(other));
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_property_matching() {
        let block = block_with_age();
        let pred = StatePropertiesPredicateBuilder::new()
            .has_property("age", "7")
            .build();
        let cond = LootItemBlockStatePropertyCondition {
            block: block.clone().built_in_registry_holder(),
            properties: pred,
        };
        let ok_ctx = LootContext::with_block_state(BlockState::new(block.clone()).with_property("age", "7"));
        let bad_ctx = LootContext::with_block_state(BlockState::new(block).with_property("age", "6"));
        assert!(cond.test(&ok_ctx));
        assert!(!cond.test(&bad_ctx));
    }

    #[test]
    fn builder_builds_condition() {
        let block = block_with_age();
        let mut builder = LootItemBlockStatePropertyCondition::has_block_state_properties(block.clone());
        builder.set_properties(StatePropertiesPredicateBuilder::new().has_property("age", "0"));
        let cond = builder.build();
        assert_eq!(cond.block.value().name, "minecraft:wheat");
        assert!(cond.properties.is_some());
        let ctx = LootContext::with_block_state(BlockState::new(block).with_property("age", "0"));
        assert!(cond.test(&ctx));
    }

    #[test]
    fn validate_reports_unknown_property() {
        let block = Block::new("minecraft:stone"); // no "age" property
        let pred = StatePropertiesPredicateBuilder::new()
            .has_property("age", "0")
            .build();
        let cond = LootItemBlockStatePropertyCondition {
            block: block.built_in_registry_holder(),
            properties: pred,
        };
        match LootItemBlockStatePropertyCondition::validate(cond) {
            DataResult::Error(msg) => {
                assert!(msg.contains("minecraft:stone"));
                assert!(msg.contains("has no property"));
                assert!(msg.contains("age"));
            }
            DataResult::Success(_) => panic!("expected validation error"),
        }

        let valid_block = block_with_age();
        let valid_cond = LootItemBlockStatePropertyCondition {
            block: valid_block.built_in_registry_holder(),
            properties: StatePropertiesPredicateBuilder::new()
                .has_property("age", "1")
                .build(),
        };
        assert!(matches!(
            LootItemBlockStatePropertyCondition::validate(valid_cond),
            DataResult::Success(_)
        ));
    }
}
