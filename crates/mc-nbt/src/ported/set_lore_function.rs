//! Faithful port of `net.minecraft.world.level.storage.loot.functions.SetLoreFunction` (1.21.11).

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityTarget {
    pub name: String,
}

impl EntityTarget {
    pub fn context_param(&self) -> ContextKey {
        ContextKey(self.name.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLore {
    lines: Vec<Component>,
}

impl ItemLore {
    pub const EMPTY: Self = Self { lines: Vec::new() };

    pub fn new(lines: Vec<Component>) -> Self {
        Self { lines }
    }

    pub fn lines(&self) -> &[Component] {
        &self.lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    lore: Option<ItemLore>,
}

impl ItemStack {
    pub fn lore(&self) -> Option<&ItemLore> {
        self.lore.as_ref()
    }

    pub fn set_lore(&mut self, lore: ItemLore) {
        self.lore = Some(lore);
    }

    /// Minimal equivalent of `ItemStack.update(DataComponents.LORE, default, op)`.
    pub fn update_lore_component<F>(&mut self, default: ItemLore, f: F)
    where
        F: FnOnce(ItemLore) -> ItemLore,
    {
        let current = self.lore.take().unwrap_or(default);
        self.lore = Some(f(current));
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    SetLore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListOperation {
    Append,
    ReplaceAll,
}

impl ListOperation {
    /// Applies this list operation with a hard size limit, matching Java list operation semantics.
    pub fn apply(
        &self,
        source: Vec<Component>,
        additions: Vec<Component>,
        limit: usize,
    ) -> Vec<Component> {
        match self {
            ListOperation::Append => {
                let mut out = source;
                out.extend(additions);
                if out.len() > limit {
                    out.truncate(limit);
                }
                out
            }
            ListOperation::ReplaceAll => {
                let mut out = additions;
                if out.len() > limit {
                    out.truncate(limit);
                }
                out
            }
        }
    }
}

/// Minimal stub for `SetNameFunction.createResolver`.
fn create_name_resolver(
    _ctx: &LootContext,
    _target: Option<&EntityTarget>,
) -> impl Fn(Component) -> Component {
    |c| c
}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.storage.loot.functions.SetLoreFunction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetLoreFunction {
    pub conditions: Vec<LootItemCondition>,
    pub lore: Vec<Component>,
    pub mode: ListOperation,
    pub resolution_context: Option<EntityTarget>,
}

impl SetLoreFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#<init>
    pub fn new(
        conditions: Vec<LootItemCondition>,
        lore: Vec<Component>,
        mode: ListOperation,
        resolution_context: Option<EntityTarget>,
    ) -> Self {
        Self {
            conditions,
            lore: lore.into_iter().collect(),
            mode,
            resolution_context,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetLore
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.resolution_context
            .as_ref()
            .map(|t| HashSet::from([t.context_param()]))
            .unwrap_or_else(HashSet::new)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#run
    pub fn run<'a>(&'a self, stack: &'a mut ItemStack, ctx: &LootContext) -> &'a mut ItemStack {
        stack.update_lore_component(ItemLore::EMPTY, |lore| {
            ItemLore::new(self.update_lore(Some(&lore), ctx))
        });
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#updateLore
    fn update_lore(&self, lore: Option<&ItemLore>, ctx: &LootContext) -> Vec<Component> {
        if lore.is_none() && self.lore.is_empty() {
            return Vec::new();
        }

        let resolver = create_name_resolver(ctx, self.resolution_context.as_ref());
        let additions: Vec<Component> = self.lore.iter().cloned().map(resolver).collect();
        let source = lore.map(|l| l.lines().to_vec()).unwrap_or_default();
        self.mode.apply(source, additions, 256)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction#setLore
    pub fn set_lore() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
    resolution_context: Option<EntityTarget>,
    lore: Vec<Component>,
    mode: ListOperation,
    conditions: Vec<LootItemCondition>,
}

impl Builder {
    fn new() -> Self {
        Self {
            resolution_context: None,
            lore: Vec::new(),
            mode: ListOperation::Append,
            conditions: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction$Builder#setMode
    pub fn set_mode(mut self, mode: ListOperation) -> Self {
        self.mode = mode;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction$Builder#setResolutionContext
    pub fn set_resolution_context(mut self, target: EntityTarget) -> Self {
        self.resolution_context = Some(target);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction$Builder#addLine
    pub fn add_line(mut self, component: Component) -> Self {
        self.lore.push(component);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction$Builder#getThis
    fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetLoreFunction$Builder#build
    pub fn build(self) -> SetLoreFunction {
        SetLoreFunction::new(self.conditions, self.lore, self.mode, self.resolution_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_lore_to_existing_stack() {
        let mut stack = ItemStack::default();
        stack.set_lore(ItemLore::new(vec![Component::new("old")]));

        let f = SetLoreFunction::new(
            vec![],
            vec![Component::new("new")],
            ListOperation::Append,
            None,
        );
        f.run(&mut stack, &LootContext::default());

        let lines = stack.lore().unwrap().lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "old");
        assert_eq!(lines[1].text, "new");
    }

    #[test]
    fn update_lore_none_and_empty_returns_empty() {
        let f = SetLoreFunction::new(vec![], vec![], ListOperation::Append, None);
        let result = f.update_lore(None, &LootContext::default());
        assert!(result.is_empty());
    }

    #[test]
    fn append_truncates_at_256_limit() {
        let source: Vec<Component> = (0..200).map(|i| Component::new(i.to_string())).collect();
        let additions: Vec<Component> = (200..300).map(|i| Component::new(i.to_string())).collect();

        let f = SetLoreFunction::new(vec![], additions, ListOperation::Append, None);
        let mut stack = ItemStack::default();
        stack.set_lore(ItemLore::new(source));
        f.run(&mut stack, &LootContext::default());

        let lines = stack.lore().unwrap().lines();
        assert_eq!(lines.len(), 256);
        assert_eq!(lines[0].text, "0");
        assert_eq!(lines[255].text, "255");
    }

    #[test]
    fn referenced_context_params_empty_and_present() {
        let none = SetLoreFunction::new(vec![], vec![], ListOperation::Append, None);
        assert!(none.get_referenced_context_params().is_empty());

        let target = EntityTarget {
            name: "killer".to_string(),
        };
        let some = SetLoreFunction::new(
            vec![],
            vec![],
            ListOperation::Append,
            Some(target.clone()),
        );
        let params = some.get_referenced_context_params();
        assert_eq!(params, HashSet::from([target.context_param()]));
    }

    #[test]
    fn negative_inputs_and_float_bits_preserved() {
        let neg_i32 = i32::MIN;
        let neg_i64: i64 = -1;
        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();

        let f = SetLoreFunction::set_lore()
            .set_mode(ListOperation::Append)
            .add_line(Component::new(neg_i32.to_string()))
            .add_line(Component::new(neg_i64.to_string()))
            .add_line(Component::new(format!("f32:0x{nan_bits:08x}")))
            .add_line(Component::new(format!("f64:0x{neg_zero_bits:016x}")))
            .build();

        let mut stack = ItemStack::default();
        f.run(&mut stack, &LootContext::default());

        let lines = stack.lore().unwrap().lines();
        assert_eq!(lines[0].text, neg_i32.to_string());
        assert_eq!(lines[1].text, "-1");
        assert_eq!(lines[2].text, format!("f32:0x{nan_bits:08x}"));
        assert_eq!(lines[3].text, format!("f64:0x{neg_zero_bits:016x}"));
        assert_eq!(f.get_type(), LootItemFunctionType::SetLore);
    }

    #[test]
    fn replace_all_mode_replaces_existing_lore() {
        let mut stack = ItemStack::default();
        stack.set_lore(ItemLore::new(vec![
            Component::new("old1"),
            Component::new("old2"),
        ]));

        let f = SetLoreFunction::new(
            vec![],
            vec![Component::new("new1")],
            ListOperation::ReplaceAll,
            None,
        );
        f.run(&mut stack, &LootContext::default());

        let lines = stack.lore().unwrap().lines();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "new1");
    }
}
