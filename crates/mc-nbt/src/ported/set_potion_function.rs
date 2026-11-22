#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Potion(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    // JAVA: net.minecraft.core.Holder#value
    pub fn value(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_POTION: LootItemFunctionType = LootItemFunctionType("set_potion");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType(pub &'static str);

pub struct DataComponents;

impl DataComponents {
    pub const POTION_CONTENTS: DataComponentType = DataComponentType("potion_contents");
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PotionContents {
    pub potion: Option<Holder<Potion>>,
}

impl PotionContents {
    // JAVA: net.minecraft.world.item.alchemy.PotionContents#empty
    pub fn empty() -> Self {
        Self { potion: None }
    }

    // JAVA: net.minecraft.world.item.alchemy.PotionContents#withPotion
    pub fn with_potion(_base: Self, potion: Holder<Potion>) -> Self {
        Self { potion: Some(potion) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub potion_contents: PotionContents,
}

pub trait ApplyComponent {
    fn apply_to(self, stack: &mut ItemStack, component: DataComponentType);
}

impl ApplyComponent for PotionContents {
    fn apply_to(self, stack: &mut ItemStack, component: DataComponentType) {
        if component == DataComponents::POTION_CONTENTS {
            stack.potion_contents = self;
        }
    }
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#update
    pub fn update<T: ApplyComponent, U>(
        &mut self,
        component: DataComponentType,
        base: T,
        value: U,
        f: fn(T, U) -> T,
    ) -> &mut Self {
        let result = f(base, value);
        result.apply_to(self, component);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPotionFunction {
    pub conditions: Vec<LootItemCondition>,
    pub potion: Holder<Potion>,
}

impl SetPotionFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetPotionFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, potion: Holder<Potion>) -> Self {
        Self { conditions, potion }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetPotionFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_POTION
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetPotionFunction#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.update(
            DataComponents::POTION_CONTENTS,
            PotionContents::empty(),
            self.potion.clone(),
            PotionContents::with_potion,
        );
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetPotionFunction#setPotion
    pub fn set_potion(potion: Holder<Potion>) -> impl FnOnce(Vec<LootItemCondition>) -> Self {
        move |conditions| Self::new(conditions, potion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LootContext {
        LootContext
    }

    #[test]
    fn run_sets_potion_contents() {
        let f = SetPotionFunction::new(vec![], Holder(Potion("minecraft:healing".into())));
        let out = f.run(ItemStack::default(), &ctx());
        assert_eq!(
            out.potion_contents.potion,
            Some(Holder(Potion("minecraft:healing".into())))
        );
    }

    #[test]
    fn get_type_is_set_potion() {
        let f = SetPotionFunction::new(vec![], Holder(Potion("x".into())));
        assert_eq!(f.get_type(), LootItemFunctions::SET_POTION);
    }

    #[test]
    fn set_potion_builder_creates_function() {
        let builder = SetPotionFunction::set_potion(Holder(Potion("minecraft:swiftness".into())));
        let f = builder(vec![LootItemCondition]);
        assert_eq!(f.conditions.len(), 1);
        let out = f.run(ItemStack::default(), &ctx());
        assert_eq!(out.potion_contents.potion.unwrap().value().0, "minecraft:swiftness");
    }

    #[test]
    fn run_overwrites_existing_potion() {
        let mut stack = ItemStack::default();
        stack.potion_contents = PotionContents {
            potion: Some(Holder(Potion("old".into()))),
        };
        let f = SetPotionFunction::new(vec![], Holder(Potion("new".into())));
        let out = f.run(stack, &ctx());
        assert_eq!(out.potion_contents.potion, Some(Holder(Potion("new".into()))));
    }
}
