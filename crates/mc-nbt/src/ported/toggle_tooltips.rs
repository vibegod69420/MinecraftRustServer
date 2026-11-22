use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    ToggleTooltips,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataComponentType(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TooltipDisplay {
    pub hidden_components: BTreeSet<DataComponentType>,
}

impl TooltipDisplay {
    pub fn with_hidden(mut self, component: &DataComponentType, hidden: bool) -> Self {
        if hidden {
            self.hidden_components.insert(component.clone());
        } else {
            self.hidden_components.remove(component);
        }
        self
    }

    pub fn is_hidden(&self, component: &DataComponentType) -> bool {
        self.hidden_components.contains(component)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub tooltip_display: TooltipDisplay,
}

impl ItemStack {
    pub fn update_tooltip_display<F: FnOnce(TooltipDisplay) -> TooltipDisplay>(&mut self, f: F) {
        let current = std::mem::take(&mut self.tooltip_display);
        self.tooltip_display = f(current);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleTooltips {
    pub conditions: Vec<LootItemCondition>,
    pub values: Vec<(DataComponentType, bool)>,
}

impl ToggleTooltips {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ToggleTooltips#<init>
    fn new(conditions: Vec<LootItemCondition>, values: Vec<(DataComponentType, bool)>) -> Self {
        Self { conditions, values }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ToggleTooltips#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.update_tooltip_display(|mut display| {
            for (component, show) in &self.values {
                let hidden = !*show;
                display = display.with_hidden(component, hidden);
            }
            display
        });
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ToggleTooltips#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::ToggleTooltips
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ct(name: &str) -> DataComponentType {
        DataComponentType(name.to_string())
    }

    #[test]
    fn hides_and_shows_components() {
        let func = ToggleTooltips::new(
            vec![],
            vec![(ct("enchantments"), false), (ct("attributes"), true)],
        );
        let mut stack = ItemStack::default();
        stack.tooltip_display.hidden_components.insert(ct("attributes"));
        let out = func.run(stack, &LootContext);
        assert!(out.tooltip_display.is_hidden(&ct("enchantments")));
        assert!(!out.tooltip_display.is_hidden(&ct("attributes")));
        assert_eq!(func.get_type(), LootItemFunctionType::ToggleTooltips);
    }

    #[test]
    fn multiple_toggles_applied_in_order() {
        // Same component toggled twice: last write wins.
        let func = ToggleTooltips::new(
            vec![],
            vec![(ct("lore"), false), (ct("lore"), true), (ct("name"), false)],
        );
        let out = func.run(ItemStack::default(), &LootContext);
        assert!(!out.tooltip_display.is_hidden(&ct("lore")));
        assert!(out.tooltip_display.is_hidden(&ct("name")));
    }

    #[test]
    fn preserves_existing_hidden_components() {
        let mut stack = ItemStack::default();
        stack.tooltip_display.hidden_components.insert(ct("keep_hidden"));
        let func = ToggleTooltips::new(vec![], vec![(ct("other"), true)]);
        let out = func.run(stack, &LootContext);
        assert!(out.tooltip_display.is_hidden(&ct("keep_hidden")));
        assert!(!out.tooltip_display.is_hidden(&ct("other")));
    }

    #[test]
    fn empty_toggles_preserves_display() {
        let mut stack = ItemStack::default();
        stack.tooltip_display.hidden_components.insert(ct("x"));
        let func = ToggleTooltips::new(vec![LootItemCondition], vec![]);
        let out = func.run(stack.clone(), &LootContext);
        assert_eq!(out.tooltip_display, stack.tooltip_display);
        assert_eq!(func.conditions.len(), 1);
    }
}
