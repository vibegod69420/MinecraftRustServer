use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub String);

#[derive(Debug, Default, Clone)]
pub struct LootContext;

pub trait NumberProvider {
    fn get_int(&self, ctx: &LootContext) -> i32;
    fn get_referenced_context_params(&self) -> Vec<ContextKey>;
}

pub trait LootItemCondition {
    fn test(&self, ctx: &LootContext) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: &'static str,
}

pub struct Items;

impl Items {
    pub fn book() -> Item {
        Item {
            id: "minecraft:book",
        }
    }

    pub fn enchanted_book() -> Item {
        Item {
            id: "minecraft:enchanted_book",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Enchantment {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T: Clone + Eq + Hash> {
    pub value: T,
}

impl<T: Clone + Eq + Hash> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

pub type EnchantmentHolder = Holder<Enchantment>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: Item,
    pub enchantments: HashMap<EnchantmentHolder, i32>,
}

impl ItemStack {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            enchantments: HashMap::new(),
        }
    }

    pub fn is(&self, item: &Item) -> bool {
        self.item == *item
    }

    pub fn transmute_copy(&self, item: Item) -> Self {
        Self {
            item,
            enchantments: self.enchantments.clone(),
        }
    }
}

pub struct EnchantmentMutable<'a> {
    enchantments: &'a mut HashMap<EnchantmentHolder, i32>,
}

impl<'a> EnchantmentMutable<'a> {
    pub fn get_level(&self, ench: &EnchantmentHolder) -> i32 {
        self.enchantments.get(ench).copied().unwrap_or(0)
    }

    pub fn set(&mut self, ench: EnchantmentHolder, level: i32) {
        if level <= 0 {
            self.enchantments.remove(&ench);
        } else {
            self.enchantments.insert(ench, level);
        }
    }
}

pub struct EnchantmentHelper;

impl EnchantmentHelper {
    pub fn update_enchantments<F: FnOnce(&mut EnchantmentMutable)>(
        stack: &mut ItemStack,
        f: F,
    ) {
        let mut mutable = EnchantmentMutable {
            enchantments: &mut stack.enchantments,
        };
        f(&mut mutable);
    }
}

pub struct Mth;

impl Mth {
    pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction
// ---------------------------------------------------------------------------

pub struct SetEnchantmentsFunction {
    conditions: Vec<Box<dyn LootItemCondition>>,
    enchantments: Vec<(EnchantmentHolder, Box<dyn NumberProvider>)>,
    add: bool,
}

impl SetEnchantmentsFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction#<init>
    pub fn new(
        conditions: Vec<Box<dyn LootItemCondition>>,
        enchantments: Vec<(EnchantmentHolder, Box<dyn NumberProvider>)>,
        add: bool,
    ) -> Self {
        Self {
            conditions,
            enchantments,
            add,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.enchantments
            .iter()
            .flat_map(|(_, provider)| provider.get_referenced_context_params())
            .collect()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if stack.is(&Items::book()) {
            stack = stack.transmute_copy(Items::enchanted_book());
        }

        EnchantmentHelper::update_enchantments(&mut stack, |enchants| {
            if self.add {
                for (ench, provider) in &self.enchantments {
                    let current: i32 = enchants.get_level(ench);
                    let delta: i32 = provider.get_int(ctx);
                    let value: i32 = Mth::clamp_i32(current.wrapping_add(delta), 0, 255);
                    enchants.set(ench.clone(), value);
                }
            } else {
                for (ench, provider) in &self.enchantments {
                    let value: i32 = Mth::clamp_i32(provider.get_int(ctx), 0, 255);
                    enchants.set(ench.clone(), value);
                }
            }
        });

        stack
    }

    /// Convenience constructor for a builder in set mode (not a direct Java method).
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Convenience constructor for a builder with explicit add mode (not a direct Java method).
    pub fn builder_with_add(add: bool) -> Builder {
        Builder::with_add(add)
    }
}

pub struct Builder {
    conditions: Vec<Box<dyn LootItemCondition>>,
    enchantments: Vec<(EnchantmentHolder, Box<dyn NumberProvider>)>,
    add: bool,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction$Builder#<init>
    pub fn new() -> Self {
        Self::with_add(false)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction$Builder#<init>
    pub fn with_add(add: bool) -> Self {
        Self {
            conditions: Vec::new(),
            enchantments: Vec::new(),
            add,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction$Builder#getThis
    pub fn get_this(self) -> Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction$Builder#withEnchantment
    pub fn with_enchantment(
        mut self,
        enchantment: EnchantmentHolder,
        provider: Box<dyn NumberProvider>,
    ) -> Self {
        if self
            .enchantments
            .iter()
            .any(|(existing, _)| existing == &enchantment)
        {
            panic!("duplicate enchantment key");
        }
        self.enchantments.push((enchantment, provider));
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction$Builder#build
    pub fn build(self) -> SetEnchantmentsFunction {
        SetEnchantmentsFunction::new(self.conditions, self.enchantments, self.add)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holder(id: &str) -> EnchantmentHolder {
        Holder::new(Enchantment { id: id.to_string() })
    }

    struct ConstantProvider(i32);
    impl NumberProvider for ConstantProvider {
        fn get_int(&self, _ctx: &LootContext) -> i32 {
            self.0
        }
        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            Vec::new()
        }
    }

    #[test]
    fn set_mode_transmutes_book_and_clamps_upper_bound() {
        let ench = holder("minecraft:sharpness");
        let func = SetEnchantmentsFunction::builder()
            .with_enchantment(ench.clone(), Box::new(ConstantProvider(300)))
            .build();

        let out = func.run(ItemStack::new(Items::book()), &LootContext::default());
        assert_eq!(out.item, Items::enchanted_book());
        assert_eq!(out.enchantments.get(&ench), Some(&255));
    }

    #[test]
    fn add_mode_uses_wrapping_add_on_overflow() {
        let ench = holder("minecraft:power");
        let func = SetEnchantmentsFunction::builder_with_add(true)
            .with_enchantment(ench.clone(), Box::new(ConstantProvider(i32::MAX)))
            .build();

        let mut stack = ItemStack::new(Item {
            id: "minecraft:bow",
        });
        stack.enchantments.insert(ench.clone(), 10);

        let out = func.run(stack, &LootContext::default());
        let wrapped = 10i32.wrapping_add(i32::MAX);
        assert_eq!(wrapped, -2_147_483_639i32);
        let expected = Mth::clamp_i32(wrapped, 0, 255);
        assert_eq!(expected, 0);
        // level 0 is removed by EnchantmentMutable#set
        assert_eq!(out.enchantments.get(&ench), None);
    }

    #[test]
    fn negative_inputs_and_negative_addition_are_clamped() {
        let ench = holder("minecraft:unbreaking");

        let set_func = SetEnchantmentsFunction::builder()
            .with_enchantment(ench.clone(), Box::new(ConstantProvider(-42)))
            .build();
        let out_set = set_func.run(
            ItemStack::new(Item {
                id: "minecraft:diamond_pickaxe",
            }),
            &LootContext::default(),
        );
        assert_eq!(out_set.enchantments.get(&ench), None);

        let add_func = SetEnchantmentsFunction::builder_with_add(true)
            .with_enchantment(ench.clone(), Box::new(ConstantProvider(-10)))
            .build();
        let mut stack = ItemStack::new(Item {
            id: "minecraft:diamond_pickaxe",
        });
        stack.enchantments.insert(ench.clone(), 5);
        let out_add = add_func.run(stack, &LootContext::default());
        let expected = Mth::clamp_i32(5i32.wrapping_add(-10i32), 0, 255);
        assert_eq!(expected, 0);
        assert_eq!(out_add.enchantments.get(&ench), None);
    }

    #[test]
    fn float_provider_preserves_f32_operation_order_and_bits() {
        struct F32Provider {
            a: f32,
            b: f32,
            c: f32,
        }
        impl NumberProvider for F32Provider {
            fn get_int(&self, _ctx: &LootContext) -> i32 {
                // Preserve Java-like evaluation order: (a * b) + c
                let v = (self.a * self.b) + self.c;
                (v * 1000.0f32) as i32
            }
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                Vec::new()
            }
        }

        let a: f32 = 1_677_7216.0f32; // 2^24, exercises f32 precision boundary
        let b: f32 = 1.0f32;
        let c: f32 = 1.0f32;
        let mul_then_add = (a * b) + c;
        let add_then_mul = a * (b + c);
        assert_eq!(mul_then_add.to_bits(), 0x4b80_0000u32); // 16777216.0f32
        assert_ne!(mul_then_add.to_bits(), add_then_mul.to_bits());

        let ench = holder("minecraft:luck_of_the_sea");
        let func = SetEnchantmentsFunction::builder()
            .with_enchantment(ench.clone(), Box::new(F32Provider { a, b, c }))
            .build();
        let out = func.run(
            ItemStack::new(Item {
                id: "minecraft:fishing_rod",
            }),
            &LootContext::default(),
        );

        let expected_i32 =
            Mth::clamp_i32((((a * b) + c) * 1000.0f32) as i32, 0, 255);
        assert_eq!(out.enchantments.get(&ench), Some(&expected_i32));
    }

    #[test]
    fn collects_referenced_context_params_from_all_providers() {
        struct ParamProvider(Vec<ContextKey>);
        impl NumberProvider for ParamProvider {
            fn get_int(&self, _ctx: &LootContext) -> i32 {
                1
            }
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                self.0.clone()
            }
        }

        let func = SetEnchantmentsFunction::builder()
            .with_enchantment(
                holder("minecraft:a"),
                Box::new(ParamProvider(vec![ContextKey("foo".into())])),
            )
            .with_enchantment(
                holder("minecraft:b"),
                Box::new(ParamProvider(vec![
                    ContextKey("bar".into()),
                    ContextKey("foo".into()),
                ])),
            )
            .build();

        let params = func.get_referenced_context_params();
        let mut names: Vec<String> = params.into_iter().map(|k| k.0).collect();
        names.sort();
        assert_eq!(names, vec!["bar".to_string(), "foo".to_string()]);
        assert_eq!(func.get_type(), LootItemFunctionType);
    }
}
