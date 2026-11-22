use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootContextParam {
    Tool,
}

impl LootContextParam {
    pub fn key(&self) -> ContextKey {
        match self {
            Self::Tool => ContextKey("minecraft:tool"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    MatchTool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item_id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item_id: impl Into<String>, count: i32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item_id.is_empty()
    }
}

#[derive(Clone)]
pub struct ItemPredicate {
    test_fn: Arc<dyn Fn(&ItemStack) -> bool + Send + Sync>,
}

impl ItemPredicate {
    pub fn new<F: Fn(&ItemStack) -> bool + Send + Sync + 'static>(f: F) -> Self {
        Self {
            test_fn: Arc::new(f),
        }
    }

    // JAVA: net.minecraft.advancements.criterion.ItemPredicate#test
    pub fn test(&self, stack: &ItemStack) -> bool {
        (self.test_fn)(stack)
    }
}

impl std::fmt::Debug for ItemPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ItemPredicate(..)")
    }
}

pub struct ItemPredicateBuilder {
    predicate: ItemPredicate,
}

impl ItemPredicateBuilder {
    pub fn from_predicate(predicate: ItemPredicate) -> Self {
        Self { predicate }
    }

    pub fn build(self) -> ItemPredicate {
        self.predicate
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext {
    tool: Option<ItemStack>,
}

impl LootContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tool(mut self, tool: ItemStack) -> Self {
        self.tool = Some(tool);
        self
    }

    pub fn get_optional_parameter_tool(&self) -> Option<&ItemStack> {
        self.tool.as_ref()
    }
}

pub trait LootItemCondition {
    fn get_type(&self) -> LootItemConditionType;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
    fn test(&self, ctx: &LootContext) -> bool;
}

#[derive(Clone, Debug)]
pub struct MatchTool {
    predicate: Option<ItemPredicate>,
}

impl MatchTool {
    pub fn new(predicate: Option<ItemPredicate>) -> Self {
        Self { predicate }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.MatchTool#predicate
    pub fn predicate(&self) -> Option<&ItemPredicate> {
        self.predicate.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.MatchTool#toolMatches
    pub fn tool_matches(builder: ItemPredicateBuilder) -> Box<dyn Fn() -> Self> {
        let predicate = builder.build();
        Box::new(move || MatchTool::new(Some(predicate.clone())))
    }
}

impl LootItemCondition for MatchTool {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.MatchTool#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::MatchTool
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.MatchTool#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParam::Tool.key());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.MatchTool#test
    fn test(&self, ctx: &LootContext) -> bool {
        let tool = ctx.get_optional_parameter_tool();
        match tool {
            None => false,
            Some(stack) => match &self.predicate {
                None => true,
                Some(pred) => pred.test(stack),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pickaxe() -> ItemStack {
        ItemStack::new("minecraft:diamond_pickaxe", 1)
    }

    #[test]
    fn test_no_tool_returns_false() {
        let cond = MatchTool::new(None);
        let ctx = LootContext::new();
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_tool_present_no_predicate_returns_true() {
        let cond = MatchTool::new(None);
        let ctx = LootContext::new().with_tool(pickaxe());
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_tool_present_predicate_matches() {
        let pred = ItemPredicate::new(|s| s.item_id == "minecraft:diamond_pickaxe");
        let cond = MatchTool::new(Some(pred));
        let ctx = LootContext::new().with_tool(pickaxe());
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_tool_present_predicate_rejects() {
        let pred = ItemPredicate::new(|s| s.item_id == "minecraft:diamond_axe");
        let cond = MatchTool::new(Some(pred));
        let ctx = LootContext::new().with_tool(pickaxe());
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_tool_matches_builder_and_metadata() {
        let builder = ItemPredicateBuilder::from_predicate(ItemPredicate::new(|s| s.count > 0));
        let cond = (MatchTool::tool_matches(builder))();
        assert!(cond.predicate().is_some());
        assert_eq!(cond.get_type(), LootItemConditionType::MatchTool);
        let params = cond.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&LootContextParam::Tool.key()));
    }

    #[test]
    fn test_empty_or_negative_count_stack_is_still_present() {
        let cond = MatchTool::new(None);
        let ctx = LootContext::new().with_tool(ItemStack::new("minecraft:air", -1));
        assert!(cond.test(&ctx));
        let ctx_empty = LootContext::new().with_tool(ItemStack::new("", 0));
        assert!(cond.test(&ctx_empty));
    }
}
