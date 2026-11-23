use std::fmt;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

/// Stub for `net.minecraft.world.level.storage.loot.functions.LootItemFunctionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub u32);

/// Placeholder type id for `LootItemFunctions.ENCHANT_RANDOMLY`.
pub const ENCHANT_RANDOMLY: LootItemFunctionType = LootItemFunctionType(0);

/// Stub for `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
pub trait LootItemCondition: fmt::Debug {}

/// Stub for `net.minecraft.world.level.storage.loot.functions.LootItemFunction`.
pub trait LootItemFunction: fmt::Debug {}

/// Stub for `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Stub for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.0
    }
}

/// Stub for `net.minecraft.core.HolderSet<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    pub entries: Vec<Holder<T>>,
}

impl<T: Clone> HolderSet<T> {
    pub fn stream(&self) -> impl Iterator<Item = Holder<T>> + '_ {
        self.entries.iter().cloned()
    }

    pub fn direct(entry: Holder<T>) -> Self {
        Self {
            entries: vec![entry],
        }
    }
}

/// Minimal item kind discriminator used by this function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Book,
    EnchantedBook,
    Other,
}

/// Minimal `ItemStack` stub with the behavior required by `EnchantRandomlyFunction`.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack<E: Clone> {
    pub kind: ItemKind,
    pub enchantments: Vec<(Holder<E>, i32)>,
}

impl<E: Clone> ItemStack<E> {
    pub fn new(kind: ItemKind) -> Self {
        Self {
            kind,
            enchantments: Vec::new(),
        }
    }

    pub fn is_book(&self) -> bool {
        self.kind == ItemKind::Book
    }

    pub fn enchant(&mut self, enchantment: Holder<E>, level: i32) {
        self.enchantments.push((enchantment, level));
    }
}

/// Minimal enchantment contract required by `EnchantRandomlyFunction`.
pub trait EnchantmentLike: Clone {
    fn can_enchant(&self, stack: &ItemStack<Self>) -> bool;
    fn get_min_level(&self) -> i32;
    fn get_max_level(&self) -> i32;
}

/// Minimal `LootContext` access required by `EnchantRandomlyFunction`.
pub trait LootContext<E: EnchantmentLike> {
    fn get_random(&mut self) -> &mut dyn RandomSource;
    fn registry_enchantments(&self) -> Vec<Holder<E>>;
}

// ---------------------------------------------------------------------------
// Helpers ported from other Minecraft classes used by this function
// ---------------------------------------------------------------------------

// JAVA: net.minecraft.util.Mth#nextInt
fn mth_next_int(random: &mut dyn RandomSource, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        random
            .next_int(max.wrapping_sub(min).wrapping_add(1))
            .wrapping_add(min)
    }
}

// JAVA: net.minecraft.util.Util#getRandomSafe
fn get_random_safe<'a, T>(list: &'a [T], random: &mut dyn RandomSource) -> Option<&'a T> {
    if list.is_empty() {
        None
    } else {
        let idx = random.next_int(list.len() as i32);
        Some(&list[idx as usize])
    }
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction
// ---------------------------------------------------------------------------

pub struct EnchantRandomlyFunction<E: EnchantmentLike> {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub options: Option<HolderSet<E>>,
    pub only_compatible: bool,
}

impl<E: EnchantmentLike + fmt::Debug> fmt::Debug for EnchantRandomlyFunction<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnchantRandomlyFunction")
            .field("options", &self.options)
            .field("only_compatible", &self.only_compatible)
            .finish()
    }
}

impl<E: EnchantmentLike> EnchantRandomlyFunction<E> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#<init>
    pub fn new(
        conditions: Vec<Box<dyn LootItemCondition>>,
        options: Option<HolderSet<E>>,
        only_compatible: bool,
    ) -> Self {
        Self {
            conditions,
            options,
            only_compatible,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        ENCHANT_RANDOMLY
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#run
    pub fn run(&self, stack: ItemStack<E>, ctx: &mut impl LootContext<E>) -> ItemStack<E> {
        let is_book = stack.is_book();
        let only_compatible = !is_book && self.only_compatible;

        let options: Vec<Holder<E>> = match &self.options {
            Some(set) => set.stream().collect(),
            None => ctx.registry_enchantments(),
        };

        let compatible: Vec<Holder<E>> = options
            .into_iter()
            .filter(|h| !only_compatible || h.value().can_enchant(&stack))
            .collect();

        let random = ctx.get_random();
        let picked = get_random_safe(&compatible, random);
        match picked {
            None => {
                eprintln!(
                    "Couldn't find a compatible enchantment for {:?}",
                    stack.kind
                );
                stack
            }
            Some(holder) => enchant_item(stack, holder.clone(), random),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#enchantItem
    fn enchant_item(
        mut stack: ItemStack<E>,
        enchantment: Holder<E>,
        random: &mut dyn RandomSource,
    ) -> ItemStack<E> {
        let level = mth_next_int(
            random,
            enchantment.value().get_min_level(),
            enchantment.value().get_max_level(),
        );
        if stack.is_book() {
            stack = ItemStack::new(ItemKind::EnchantedBook);
        }
        stack.enchant(enchantment, level);
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#randomEnchantment
    pub fn random_enchantment() -> Builder<E> {
        Builder::new()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#randomApplicableEnchantment
    pub fn random_applicable_enchantment(on_random_loot: HolderSet<E>) -> Builder<E> {
        Self::random_enchantment().with_one_of(on_random_loot)
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction#enchantItem
fn enchant_item<E: EnchantmentLike>(
    stack: ItemStack<E>,
    enchantment: Holder<E>,
    random: &mut dyn RandomSource,
) -> ItemStack<E> {
    EnchantRandomlyFunction::enchant_item(stack, enchantment, random)
}

pub struct Builder<E: EnchantmentLike> {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub options: Option<HolderSet<E>>,
    pub only_compatible: bool,
}

impl<E: EnchantmentLike + fmt::Debug> fmt::Debug for Builder<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("options", &self.options)
            .field("only_compatible", &self.only_compatible)
            .finish()
    }
}

impl<E: EnchantmentLike> Builder<E> {
    fn new() -> Self {
        Self {
            conditions: Vec::new(),
            options: None,
            only_compatible: true,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction$Builder#getThis
    fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction$Builder#withEnchantment
    pub fn with_enchantment(mut self, enchantment: Holder<E>) -> Self {
        let _ = self.get_this();
        self.options = Some(HolderSet::direct(enchantment));
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction$Builder#withOneOf
    pub fn with_one_of(mut self, options: HolderSet<E>) -> Self {
        let _ = self.get_this();
        self.options = Some(options);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction$Builder#allowingIncompatibleEnchantments
    pub fn allowing_incompatible_enchantments(mut self) -> Self {
        let _ = self.get_this();
        self.only_compatible = false;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction$Builder#build
    pub fn build(self) -> EnchantRandomlyFunction<E> {
        EnchantRandomlyFunction::new(self.conditions, self.options, self.only_compatible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestEnchant {
        id: i32,
        min_level: i32,
        max_level: i32,
        compatible: bool,
    }

    impl EnchantmentLike for TestEnchant {
        fn can_enchant(&self, _stack: &ItemStack<Self>) -> bool {
            self.compatible
        }

        fn get_min_level(&self) -> i32 {
            self.min_level
        }

        fn get_max_level(&self) -> i32 {
            self.max_level
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct TestContext {
        random: SeqRandom,
        registry: Vec<Holder<TestEnchant>>,
    }

    impl LootContext<TestEnchant> for TestContext {
        fn get_random(&mut self) -> &mut dyn RandomSource {
            &mut self.random
        }

        fn registry_enchantments(&self) -> Vec<Holder<TestEnchant>> {
            self.registry.clone()
        }
    }

    #[test]
    fn no_compatible_enchantment_returns_original() {
        let func = EnchantRandomlyFunction::<TestEnchant>::new(vec![], None, true);
        let stack = ItemStack::new(ItemKind::Other);
        let mut ctx = TestContext {
            random: SeqRandom::new(vec![0]),
            registry: vec![Holder(TestEnchant {
                id: 1,
                min_level: 1,
                max_level: 1,
                compatible: false,
            })],
        };
        let result = func.run(stack.clone(), &mut ctx);
        assert_eq!(result, stack);
    }

    #[test]
    fn book_converts_to_enchanted_book_and_applies_level() {
        let enchant = Holder(TestEnchant {
            id: 10,
            min_level: 2,
            max_level: 4,
            compatible: true,
        });
        let func = EnchantRandomlyFunction::new(
            vec![],
            Some(HolderSet {
                entries: vec![enchant.clone()],
            }),
            true,
        );
        let stack = ItemStack::new(ItemKind::Book);
        // bound = 4-2+1 = 3, next_int returns 1 => level = 1 + 2 = 3
        let mut ctx = TestContext {
            random: SeqRandom::new(vec![1]),
            registry: vec![],
        };
        let result = func.run(stack, &mut ctx);
        assert_eq!(result.kind, ItemKind::EnchantedBook);
        assert_eq!(result.enchantments.len(), 1);
        assert_eq!(result.enchantments[0].0, enchant);
        assert_eq!(result.enchantments[0].1, 3);
    }

    #[test]
    fn only_compatible_filters_non_book_and_builder_allows_incompatible() {
        let compatible = Holder(TestEnchant {
            id: 1,
            min_level: 1,
            max_level: 1,
            compatible: true,
        });
        let incompatible = Holder(TestEnchant {
            id: 2,
            min_level: 5,
            max_level: 5,
            compatible: false,
        });

        let filtered = EnchantRandomlyFunction::new(vec![], None, true);
        let mut ctx = TestContext {
            random: SeqRandom::new(vec![0]),
            registry: vec![incompatible.clone(), compatible.clone()],
        };
        let result = filtered.run(ItemStack::new(ItemKind::Other), &mut ctx);
        assert_eq!(result.enchantments, vec![(compatible, 1)]);

        let allowing = EnchantRandomlyFunction::random_enchantment()
            .with_enchantment(incompatible.clone())
            .allowing_incompatible_enchantments()
            .build();
        let mut ctx2 = TestContext {
            random: SeqRandom::new(vec![0]),
            registry: vec![],
        };
        let result2 = allowing.run(ItemStack::new(ItemKind::Other), &mut ctx2);
        assert_eq!(result2.enchantments, vec![(incompatible, 5)]);
    }

    #[test]
    fn mth_next_int_negative_and_overflow_edge_cases() {
        struct FixedRandom(i32);
        impl RandomSource for FixedRandom {
            fn next_int(&mut self, bound: i32) -> i32 {
                if bound <= 0 {
                    0
                } else {
                    self.0.rem_euclid(bound)
                }
            }
        }

        // min >= max returns min
        assert_eq!(mth_next_int(&mut FixedRandom(0), 7, 7), 7);
        assert_eq!(mth_next_int(&mut FixedRandom(0), 9, 3), 9);

        // negative range
        assert_eq!(mth_next_int(&mut FixedRandom(0), -10, -6), -10);
        assert_eq!(mth_next_int(&mut FixedRandom(4), -10, -6), -6);

        // overflow in range computation:
        // i32::MAX.wrapping_sub(i32::MIN).wrapping_add(1) == 0
        assert_eq!(
            mth_next_int(&mut FixedRandom(0), i32::MIN, i32::MAX),
            i32::MIN
        );
    }

    #[test]
    fn float_bit_identity_sanity() {
        let neg_zero_f32: f32 = -0.0;
        let pos_zero_f32: f32 = 0.0;
        assert_ne!(neg_zero_f32.to_bits(), pos_zero_f32.to_bits());
        assert_eq!(f32::NAN.to_bits(), f32::NAN.to_bits());

        let neg_zero_f64: f64 = -0.0;
        let pos_zero_f64: f64 = 0.0;
        assert_ne!(neg_zero_f64.to_bits(), pos_zero_f64.to_bits());
        assert_eq!(f64::INFINITY.to_bits(), 0x7ff0_0000_0000_0000u64);
    }
}
