use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub u32);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_BANNER_PATTERN: LootItemFunctionType = LootItemFunctionType(1);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerPattern {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerPatternLayers {
    pub layers: Vec<(Holder<BannerPattern>, DyeColor)>,
}

impl BannerPatternLayers {
    pub const EMPTY: Self = Self { layers: Vec::new() };
}

impl Default for BannerPatternLayers {
    fn default() -> Self {
        Self::EMPTY.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct BannerPatternLayersBuilder {
    layers: Vec<(Holder<BannerPattern>, DyeColor)>,
}

impl BannerPatternLayersBuilder {
    // JAVA: net.minecraft.world.level.block.entity.BannerPatternLayers$Builder#<init>
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    // JAVA: net.minecraft.world.level.block.entity.BannerPatternLayers$Builder#add
    pub fn add(&mut self, pattern: Holder<BannerPattern>, color: DyeColor) -> &mut Self {
        self.layers.push((pattern, color));
        self
    }

    // JAVA: net.minecraft.world.level.block.entity.BannerPatternLayers$Builder#addAll
    pub fn add_all(&mut self, other: &BannerPatternLayers) -> &mut Self {
        self.layers.extend(other.layers.iter().cloned());
        self
    }

    // JAVA: net.minecraft.world.level.block.entity.BannerPatternLayers$Builder#build
    pub fn build(&self) -> BannerPatternLayers {
        BannerPatternLayers {
            layers: self.layers.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataComponents {
    BannerPatterns,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub banner_patterns: Option<BannerPatternLayers>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#set
    pub fn set(&mut self, component: DataComponents, value: BannerPatternLayers) {
        match component {
            DataComponents::BannerPatterns => {
                self.banner_patterns = Some(value);
            }
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#update
    pub fn update<F>(
        &mut self,
        component: DataComponents,
        default: BannerPatternLayers,
        value: BannerPatternLayers,
        combiner: F,
    ) where
        F: FnOnce(BannerPatternLayers, BannerPatternLayers) -> BannerPatternLayers,
    {
        match component {
            DataComponents::BannerPatterns => {
                let current = self.banner_patterns.clone().unwrap_or(default);
                self.banner_patterns = Some(combiner(current, value));
            }
        }
    }
}

pub trait LootItemFunction {
    fn get_type(&self) -> LootItemFunctionType;
    fn run(&self, stack: ItemStack, context: &LootContext) -> ItemStack;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBannerPatternFunction {
    pub conditions: Vec<LootItemCondition>,
    pub patterns: BannerPatternLayers,
    pub append: bool,
}

impl SetBannerPatternFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#<init>
    pub fn new(conditions: Vec<LootItemCondition>, patterns: BannerPatternLayers, append: bool) -> Self {
        Self {
            conditions,
            patterns,
            append,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#run
    pub fn run(&self, mut stack: ItemStack, _context: &LootContext) -> ItemStack {
        if self.append {
            let patterns = self.patterns.clone();
            stack.update(
                DataComponents::BannerPatterns,
                BannerPatternLayers::EMPTY.clone(),
                patterns,
                |current, value| {
                    let mut builder = BannerPatternLayersBuilder::new();
                    builder.add_all(&current);
                    builder.add_all(&value);
                    builder.build()
                },
            );
        } else {
            stack.set(DataComponents::BannerPatterns, self.patterns.clone());
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_BANNER_PATTERN
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#setBannerPattern
    pub fn set_banner_pattern(append: bool) -> Builder {
        Builder::new(append)
    }
}

impl LootItemFunction for SetBannerPatternFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#getType
    fn get_type(&self) -> LootItemFunctionType {
        Self::get_type(self)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction#run
    fn run(&self, stack: ItemStack, context: &LootContext) -> ItemStack {
        Self::run(self, stack, context)
    }
}

#[derive(Debug, Clone)]
pub struct Builder {
    patterns: BannerPatternLayersBuilder,
    append: bool,
    conditions: Vec<LootItemCondition>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction$Builder#<init>
    pub fn new(append: bool) -> Self {
        Self {
            patterns: BannerPatternLayersBuilder::new(),
            append,
            conditions: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction$Builder#getConditions
    pub fn get_conditions(&self) -> Vec<LootItemCondition> {
        self.conditions.clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction$Builder#build
    pub fn build(&self) -> SetBannerPatternFunction {
        SetBannerPatternFunction::new(self.get_conditions(), self.patterns.build(), self.append)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction$Builder#addPattern
    pub fn add_pattern(&mut self, pattern: Holder<BannerPattern>, color: DyeColor) -> &mut Self {
        self.patterns.add(pattern, color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pat(id: &str) -> Holder<BannerPattern> {
        Holder(BannerPattern {
            id: id.to_string(),
        })
    }

    #[test]
    fn set_replaces_patterns() {
        let mut b = SetBannerPatternFunction::set_banner_pattern(false);
        b.add_pattern(pat("stripe"), DyeColor::Red);
        let f = b.build();
        let mut stack = ItemStack::default();
        stack.set(
            DataComponents::BannerPatterns,
            BannerPatternLayers {
                layers: vec![(pat("base"), DyeColor::White)],
            },
        );
        let out = f.run(stack, &LootContext::default());
        let layers = out.banner_patterns.unwrap();
        assert_eq!(layers.layers.len(), 1);
        assert_eq!(layers.layers[0].0 .0.id, "stripe");
        assert_eq!(layers.layers[0].1, DyeColor::Red);
    }

    #[test]
    fn append_merges_patterns() {
        let mut b = SetBannerPatternFunction::set_banner_pattern(true);
        b.add_pattern(pat("stripe"), DyeColor::Red);
        let f = b.build();
        let mut stack = ItemStack::default();
        stack.set(
            DataComponents::BannerPatterns,
            BannerPatternLayers {
                layers: vec![(pat("base"), DyeColor::White)],
            },
        );
        let out = f.run(stack, &LootContext::default());
        let layers = out.banner_patterns.unwrap();
        assert_eq!(layers.layers.len(), 2);
        assert_eq!(layers.layers[0].0 .0.id, "base");
        assert_eq!(layers.layers[1].0 .0.id, "stripe");
    }

    #[test]
    fn append_to_empty_uses_default() {
        let mut b = SetBannerPatternFunction::set_banner_pattern(true);
        b.add_pattern(pat("stripe"), DyeColor::Blue);
        let f = b.build();
        let stack = ItemStack::default();
        let out = f.run(stack, &LootContext::default());
        let layers = out.banner_patterns.unwrap();
        assert_eq!(layers.layers.len(), 1);
        assert_eq!(layers.layers[0].1, DyeColor::Blue);
    }

    #[test]
    fn get_type_is_set_banner_pattern() {
        let f = SetBannerPatternFunction::set_banner_pattern(false).build();
        assert_eq!(f.get_type(), LootItemFunctions::SET_BANNER_PATTERN);
        assert_eq!(LootItemFunction::get_type(&f), LootItemFunctions::SET_BANNER_PATTERN);
    }

    #[test]
    fn builder_accumulates_multiple_patterns() {
        let mut b = SetBannerPatternFunction::set_banner_pattern(false);
        b.add_pattern(pat("a"), DyeColor::Black);
        b.add_pattern(pat("b"), DyeColor::White);
        let f = b.build();
        assert_eq!(f.patterns.layers.len(), 2);
        assert!(!f.append);
        assert_eq!(f.patterns.layers[0].0 .0.id, "a");
        assert_eq!(f.patterns.layers[1].0 .0.id, "b");
    }
}
