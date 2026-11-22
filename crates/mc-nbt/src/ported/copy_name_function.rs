use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    CopyName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextArg {
    pub key: ContextKey,
}

impl LootContextArg {
    pub fn context_param(&self) -> &ContextKey {
        &self.key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootSource {
    Nameable { custom_name: Option<String> },
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub params: std::collections::BTreeMap<ContextKey, LootSource>,
}

impl LootContext {
    pub fn get(&self, key: &ContextKey) -> Option<&LootSource> {
        self.params.get(key)
    }

    pub fn insert_nameable(&mut self, key: ContextKey, custom_name: Option<String>) {
        self.params.insert(key, LootSource::Nameable { custom_name });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyNameFunction {
    pub conditions: Vec<LootItemCondition>,
    pub source: LootContextArg,
}

impl CopyNameFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyNameFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, source: LootContextArg) -> Self {
        Self { conditions, source }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyNameFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::CopyName
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyNameFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = BTreeSet::new();
        set.insert(self.source.context_param().clone());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyNameFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if let Some(LootSource::Nameable { custom_name }) = ctx.get(&self.source.key) {
            stack.custom_name = custom_name.clone();
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyNameFunction#copyName
    pub fn copy_name(
        source: LootContextArg,
    ) -> impl FnOnce(Vec<LootItemCondition>) -> CopyNameFunction {
        move |conditions| CopyNameFunction::new(conditions, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str) -> LootContextArg {
        LootContextArg {
            key: ContextKey(name.to_string()),
        }
    }

    #[test]
    fn copies_custom_name_from_nameable_source() {
        let func = CopyNameFunction::new(vec![], arg("block_entity"));
        let mut ctx = LootContext::default();
        ctx.insert_nameable(ContextKey("block_entity".into()), Some("Chest".into()));
        let out = func.run(ItemStack::default(), &ctx);
        assert_eq!(out.custom_name.as_deref(), Some("Chest"));
        assert_eq!(func.get_type(), LootItemFunctionType::CopyName);
    }

    #[test]
    fn nameable_with_no_custom_name_clears_stack_name() {
        let func = CopyNameFunction::new(vec![], arg("block_entity"));
        let mut ctx = LootContext::default();
        ctx.insert_nameable(ContextKey("block_entity".into()), None);
        let mut stack = ItemStack::default();
        stack.custom_name = Some("Old".into());
        let out = func.run(stack, &ctx);
        assert_eq!(out.custom_name, None);
    }

    #[test]
    fn non_nameable_or_missing_source_unchanged() {
        let func = CopyNameFunction::new(vec![], arg("this"));
        let mut stack = ItemStack::default();
        stack.custom_name = Some("Keep".into());

        let mut ctx_other = LootContext::default();
        ctx_other
            .params
            .insert(ContextKey("this".into()), LootSource::Other);
        let out_other = func.run(stack.clone(), &ctx_other);
        assert_eq!(out_other.custom_name.as_deref(), Some("Keep"));

        let ctx_missing = LootContext::default();
        let out_missing = func.run(stack, &ctx_missing);
        assert_eq!(out_missing.custom_name.as_deref(), Some("Keep"));
    }

    #[test]
    fn referenced_context_params_contains_source_key() {
        let func = CopyNameFunction::new(vec![], arg("killer"));
        let refs = func.get_referenced_context_params();
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(&ContextKey("killer".into())));
    }

    #[test]
    fn copy_name_builder_creates_function() {
        let builder = CopyNameFunction::copy_name(arg("block_entity"));
        let func = builder(vec![LootItemCondition]);
        assert_eq!(func.source.key, ContextKey("block_entity".into()));
        assert_eq!(func.conditions.len(), 1);
    }
}
