use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation(pub String);

impl ResourceLocation {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    pub location: ResourceLocation,
    _marker: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: ResourceLocation::new(location),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: ResourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T: Clone> {
    pub value: Rc<T>,
}

impl<T: Clone> Holder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: Holder<Item>,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item: Holder<Item>) -> Self {
        Self { item, count: 1 }
    }
}

pub trait LootContextAccess {
    fn get_item_tag_or_empty(&self, tag: &TagKey<Item>) -> Vec<Holder<Item>>;
}

#[derive(Clone)]
pub struct LootItemCondition {
    predicate: Rc<dyn Fn(&dyn LootContextAccess) -> bool>,
}

impl fmt::Debug for LootItemCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LootItemCondition")
    }
}

impl LootItemCondition {
    pub fn new<F: Fn(&dyn LootContextAccess) -> bool + 'static>(f: F) -> Self {
        Self {
            predicate: Rc::new(f),
        }
    }

    pub fn test(&self, ctx: &dyn LootContextAccess) -> bool {
        (self.predicate)(ctx)
    }
}

#[derive(Clone)]
pub struct LootItemFunction {
    func: Rc<dyn Fn(ItemStack, &dyn LootContextAccess) -> ItemStack>,
}

impl fmt::Debug for LootItemFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LootItemFunction")
    }
}

impl LootItemFunction {
    pub fn new<F: Fn(ItemStack, &dyn LootContextAccess) -> ItemStack + 'static>(f: F) -> Self {
        Self { func: Rc::new(f) }
    }

    pub fn apply(&self, stack: ItemStack, ctx: &dyn LootContextAccess) -> ItemStack {
        (self.func)(stack, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct LootPoolSingletonContainerBase {
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

impl LootPoolSingletonContainerBase {
    fn can_run(&self, ctx: &dyn LootContextAccess) -> bool {
        self.conditions.iter().all(|c| c.test(ctx))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootPoolEntryType;

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub const TAG: LootPoolEntryType = LootPoolEntryType;
}

#[derive(Debug, Clone)]
pub enum LootPoolEntry {
    Tag(TagEntry),
    Singleton(SingletonEntry),
}

impl LootPoolEntry {
    pub fn create_item_stack(
        &self,
        ctx: &dyn LootContextAccess,
        output: &mut dyn FnMut(ItemStack),
    ) {
        match self {
            LootPoolEntry::Tag(entry) => entry.create_item_stack(ctx, output),
            LootPoolEntry::Singleton(entry) => entry.create_item_stack(ctx, output),
        }
    }
}

/// Anonymous `LootPoolSingletonContainer.EntryBase` emitted by `TagEntry#expandTag`.
#[derive(Debug, Clone)]
pub struct SingletonEntry {
    pub base: LootPoolSingletonContainerBase,
    pub item: Holder<Item>,
}

impl SingletonEntry {
    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#createItemStack
    pub fn create_item_stack(&self, _ctx: &dyn LootContextAccess, output: &mut dyn FnMut(ItemStack)) {
        output(ItemStack::new(self.item.clone()));
    }
}

#[derive(Debug, Clone)]
pub struct TagEntry {
    pub base: LootPoolSingletonContainerBase,
    pub tag: TagKey<Item>,
    pub expand: bool,
}

impl TagEntry {
    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#<init>
    fn new(
        tag: TagKey<Item>,
        expand: bool,
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            base: LootPoolSingletonContainerBase {
                weight,
                quality,
                conditions,
                functions,
            },
            tag,
            expand,
        }
    }

    fn can_run(&self, ctx: &dyn LootContextAccess) -> bool {
        self.base.can_run(ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::TAG
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#createItemStack
    pub fn create_item_stack(&self, ctx: &dyn LootContextAccess, output: &mut dyn FnMut(ItemStack)) {
        for holder in ctx.get_item_tag_or_empty(&self.tag) {
            output(ItemStack::new(holder));
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#expandTag
    fn expand_tag_inner(
        &self,
        ctx: &dyn LootContextAccess,
        output: &mut dyn FnMut(LootPoolEntry),
    ) -> bool {
        if !self.can_run(ctx) {
            return false;
        }

        for holder in ctx.get_item_tag_or_empty(&self.tag) {
            output(LootPoolEntry::Singleton(SingletonEntry {
                base: LootPoolSingletonContainerBase {
                    weight: self.base.weight,
                    quality: self.base.quality,
                    conditions: self.base.conditions.clone(),
                    functions: self.base.functions.clone(),
                },
                item: holder,
            }));
        }

        true
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#expand
    pub fn expand(
        &self,
        ctx: &dyn LootContextAccess,
        output: &mut dyn FnMut(LootPoolEntry),
    ) -> bool {
        if self.expand {
            self.expand_tag_inner(ctx, output)
        } else if self.can_run(ctx) {
            output(LootPoolEntry::Tag(self.clone()));
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#tagContents
    pub fn tag_contents(tag: TagKey<Item>) -> TagEntryBuilder {
        TagEntryBuilder {
            tag,
            expand: false,
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.TagEntry#expandTag
    pub fn expand_tag(tag: TagKey<Item>) -> TagEntryBuilder {
        TagEntryBuilder {
            tag,
            expand: true,
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TagEntryBuilder {
    tag: TagKey<Item>,
    expand: bool,
    weight: i32,
    quality: i32,
    conditions: Vec<LootItemCondition>,
    functions: Vec<LootItemFunction>,
}

impl TagEntryBuilder {
    pub fn set_weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }

    pub fn set_quality(mut self, quality: i32) -> Self {
        self.quality = quality;
        self
    }

    pub fn when(mut self, condition: LootItemCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn apply(mut self, function: LootItemFunction) -> Self {
        self.functions.push(function);
        self
    }

    pub fn build(self) -> TagEntry {
        TagEntry::new(
            self.tag,
            self.expand,
            self.weight,
            self.quality,
            self.conditions,
            self.functions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockContext {
        tags: HashMap<String, Vec<Holder<Item>>>,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                tags: HashMap::new(),
            }
        }

        fn with_tag(mut self, name: &str, items: &[&str]) -> Self {
            let holders = items
                .iter()
                .map(|id| {
                    Holder::new(Item {
                        id: ResourceLocation::new(*id),
                    })
                })
                .collect();
            self.tags.insert(name.to_string(), holders);
            self
        }
    }

    impl LootContextAccess for MockContext {
        fn get_item_tag_or_empty(&self, tag: &TagKey<Item>) -> Vec<Holder<Item>> {
            self.tags.get(&tag.location.0).cloned().unwrap_or_default()
        }
    }

    #[test]
    fn test_create_item_stack_empty_and_multiple() {
        let ctx = MockContext::new().with_tag(
            "minecraft:logs",
            &["minecraft:oak_log", "minecraft:birch_log"],
        );
        let entry = TagEntry::tag_contents(TagKey::new("minecraft:logs")).build();

        let mut out = Vec::new();
        entry.create_item_stack(&ctx, &mut |stack| out.push(stack));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].item.value.id.0, "minecraft:oak_log");
        assert_eq!(out[1].item.value.id.0, "minecraft:birch_log");
        assert_eq!(out[0].count, 1);

        let missing = TagEntry::tag_contents(TagKey::new("minecraft:missing")).build();
        let mut empty_out = Vec::new();
        missing.create_item_stack(&ctx, &mut |stack| empty_out.push(stack));
        assert!(empty_out.is_empty());
    }

    #[test]
    fn test_expand_false_delegates_to_super() {
        let ctx = MockContext::new().with_tag("minecraft:logs", &["minecraft:oak_log"]);
        let entry = TagEntry::tag_contents(TagKey::new("minecraft:logs")).build();

        let mut out = Vec::new();
        let ok = entry.expand(&ctx, &mut |e| out.push(e));
        assert!(ok);
        assert_eq!(out.len(), 1);
        match &out[0] {
            LootPoolEntry::Tag(t) => {
                assert!(!t.expand);
                assert_eq!(t.get_type(), LootPoolEntries::TAG);
            }
            _ => panic!("expected Tag entry"),
        }

        let denied = TagEntry::tag_contents(TagKey::new("minecraft:logs"))
            .when(LootItemCondition::new(|_| false))
            .build();
        let mut denied_out = Vec::new();
        let denied_ok = denied.expand(&ctx, &mut |e| denied_out.push(e));
        assert!(!denied_ok);
        assert!(denied_out.is_empty());
    }

    #[test]
    fn test_expand_true_produces_singleton_entries() {
        let ctx = MockContext::new().with_tag(
            "minecraft:logs",
            &["minecraft:oak_log", "minecraft:birch_log"],
        );
        let entry = TagEntry::expand_tag(TagKey::new("minecraft:logs")).build();

        let mut out = Vec::new();
        let ok = entry.expand(&ctx, &mut |e| out.push(e));
        assert!(ok);
        assert_eq!(out.len(), 2);

        let expected = ["minecraft:oak_log", "minecraft:birch_log"];
        for (entry_out, expected_id) in out.iter().zip(expected.iter()) {
            match entry_out {
                LootPoolEntry::Singleton(singleton) => {
                    let mut stacks = Vec::new();
                    singleton.create_item_stack(&ctx, &mut |stack| stacks.push(stack));
                    assert_eq!(stacks.len(), 1);
                    assert_eq!(stacks[0].item.value.id.0, *expected_id);
                }
                _ => panic!("expected Singleton entry"),
            }
        }

        let denied = TagEntry::expand_tag(TagKey::new("minecraft:logs"))
            .when(LootItemCondition::new(|_| false))
            .build();
        let mut denied_out = Vec::new();
        let denied_ok = denied.expand(&ctx, &mut |e| denied_out.push(e));
        assert!(!denied_ok);
        assert!(denied_out.is_empty());
    }

    #[test]
    fn test_negative_and_overflow_weight_quality() {
        let entry = TagEntry::tag_contents(TagKey::new("minecraft:test"))
            .set_weight(i32::MAX)
            .set_quality(i32::MIN)
            .build();
        assert_eq!(entry.base.weight, i32::MAX);
        assert_eq!(entry.base.quality, i32::MIN);

        let negative = TagEntry::tag_contents(TagKey::new("minecraft:test"))
            .set_weight(-1)
            .set_quality(-100)
            .build();
        assert_eq!(negative.base.weight, -1);
        assert_eq!(negative.base.quality, -100);

        // Java int semantics: wrapping arithmetic.
        let wrapped_weight = entry.base.weight.wrapping_add(1);
        assert_eq!(wrapped_weight, i32::MIN);
        let wrapped_quality = negative.base.quality.wrapping_sub(i32::MAX);
        assert_eq!(wrapped_quality, (-100i32).wrapping_sub(i32::MAX));
    }

    #[test]
    fn test_float_bits_in_condition() {
        let nan_bits_f32 = f32::NAN.to_bits();
        let neg_zero_f32 = (-0.0f32).to_bits();
        let pos_zero_f32 = 0.0f32.to_bits();
        assert_ne!(nan_bits_f32, pos_zero_f32);
        assert_ne!(neg_zero_f32, pos_zero_f32);
        assert_eq!(neg_zero_f32, 0x8000_0000u32);

        let nan_bits_f64 = f64::NAN.to_bits();
        let neg_zero_f64 = (-0.0f64).to_bits();
        let pos_zero_f64 = 0.0f64.to_bits();
        assert_ne!(nan_bits_f64, pos_zero_f64);
        assert_eq!(neg_zero_f64, 0x8000_0000_0000_0000u64);

        // Condition uses exact float bit comparison (Java floatToRawIntBits semantics).
        let ctx = MockContext::new().with_tag("minecraft:logs", &["minecraft:oak_log"]);
        let fval = f32::from_bits(nan_bits_f32);
        let dval = f64::from_bits(nan_bits_f64);
        let entry = TagEntry::tag_contents(TagKey::new("minecraft:logs"))
            .when(LootItemCondition::new(move |_| {
                fval.to_bits() == nan_bits_f32 && dval.to_bits() == nan_bits_f64
            }))
            .build();

        let mut out = Vec::new();
        let ok = entry.expand(&ctx, &mut |e| out.push(e));
        assert!(ok);
        assert_eq!(out.len(), 1);
    }
}
