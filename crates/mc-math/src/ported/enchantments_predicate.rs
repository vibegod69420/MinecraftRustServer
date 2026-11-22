use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.world.item.enchantment.ItemEnchantments`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemEnchantments {
    entries: Vec<(String, i32)>,
}

impl ItemEnchantments {
    pub fn new(entries: Vec<(String, i32)>) -> Self {
        Self { entries }
    }

    pub fn get_level(&self, id: &str) -> i32 {
        self.entries
            .iter()
            .find(|(entry_id, _)| entry_id == id)
            .map(|(_, lvl)| *lvl)
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Minimal self-contained stub for
/// `net.minecraft.advancements.criterion.EnchantmentPredicate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentPredicate {
    pub enchantment: Option<String>,
    pub min_level: i32,
    pub max_level: i32,
}

impl EnchantmentPredicate {
    pub fn any() -> Self {
        Self {
            enchantment: None,
            min_level: i32::MIN,
            max_level: i32::MAX,
        }
    }

    pub fn of(enchantment: impl Into<String>) -> Self {
        Self {
            enchantment: Some(enchantment.into()),
            min_level: i32::MIN,
            max_level: i32::MAX,
        }
    }

    pub fn with_levels(enchantment: impl Into<String>, min_level: i32, max_level: i32) -> Self {
        Self {
            enchantment: Some(enchantment.into()),
            min_level,
            max_level,
        }
    }

    // JAVA: net.minecraft.advancements.criterion.EnchantmentPredicate#containedIn
    pub fn contained_in(&self, item_enchantments: &ItemEnchantments) -> bool {
        match &self.enchantment {
            None => !item_enchantments.is_empty(),
            Some(id) => {
                let lvl = item_enchantments.get_level(id);
                lvl > 0 && lvl >= self.min_level && lvl <= self.max_level
            }
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.core.component.DataComponentType<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T> {
    id: u32,
    _marker: PhantomData<fn() -> T>,
}

/// Minimal self-contained stub for `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;

impl DataComponents {
    pub fn enchantments() -> DataComponentType<ItemEnchantments> {
        DataComponentType {
            id: 0,
            _marker: PhantomData,
        }
    }

    pub fn stored_enchantments() -> DataComponentType<ItemEnchantments> {
        DataComponentType {
            id: 1,
            _marker: PhantomData,
        }
    }
}

fn matches_all(predicates: &[EnchantmentPredicate], item_enchantments: &ItemEnchantments) -> bool {
    for pred in predicates {
        if !pred.contained_in(item_enchantments) {
            return false;
        }
    }
    true
}

/// Port of `net.minecraft.core.component.predicates.EnchantmentsPredicate.Enchantments`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enchantments {
    predicates: Vec<EnchantmentPredicate>,
}

impl Enchantments {
    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate$Enchantments#<init>
    pub fn new(enchantments: Vec<EnchantmentPredicate>) -> Self {
        Self {
            predicates: enchantments,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#enchantments
    pub fn enchantments(&self) -> &[EnchantmentPredicate] {
        &self.predicates
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#matches
    pub fn matches(&self, item_enchantments: &ItemEnchantments) -> bool {
        matches_all(&self.predicates, item_enchantments)
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate$Enchantments#componentType
    pub fn component_type(&self) -> DataComponentType<ItemEnchantments> {
        DataComponents::enchantments()
    }
}

/// Port of `net.minecraft.core.component.predicates.EnchantmentsPredicate.StoredEnchantments`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEnchantments {
    predicates: Vec<EnchantmentPredicate>,
}

impl StoredEnchantments {
    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate$StoredEnchantments#<init>
    pub fn new(enchantments: Vec<EnchantmentPredicate>) -> Self {
        Self {
            predicates: enchantments,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#enchantments
    pub fn enchantments(&self) -> &[EnchantmentPredicate] {
        &self.predicates
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#matches
    pub fn matches(&self, item_enchantments: &ItemEnchantments) -> bool {
        matches_all(&self.predicates, item_enchantments)
    }

    // JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate$StoredEnchantments#componentType
    pub fn component_type(&self) -> DataComponentType<ItemEnchantments> {
        DataComponents::stored_enchantments()
    }
}

// JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#enchantments
pub fn enchantments(predicates: Vec<EnchantmentPredicate>) -> Enchantments {
    Enchantments::new(predicates)
}

// JAVA: net.minecraft.core.component.predicates.EnchantmentsPredicate#storedEnchantments
pub fn stored_enchantments(predicates: Vec<EnchantmentPredicate>) -> StoredEnchantments {
    StoredEnchantments::new(predicates)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ench(id: &str, lvl: i32) -> (String, i32) {
        (id.to_string(), lvl)
    }

    #[test]
    fn empty_predicates_matches_any() {
        let pred = Enchantments::new(vec![]);
        assert!(pred.matches(&ItemEnchantments::default()));
        assert!(pred.matches(&ItemEnchantments::new(vec![ench(
            "minecraft:sharpness",
            3
        )])));
    }

    #[test]
    fn single_predicate_matches_and_rejects() {
        let pred = Enchantments::new(vec![EnchantmentPredicate::of("minecraft:sharpness")]);
        let matching = ItemEnchantments::new(vec![ench("minecraft:sharpness", 2)]);
        let missing = ItemEnchantments::new(vec![ench("minecraft:unbreaking", 1)]);
        assert!(pred.matches(&matching));
        assert!(!pred.matches(&missing));
    }

    #[test]
    fn all_predicates_must_match() {
        let pred = enchantments(vec![
            EnchantmentPredicate::of("minecraft:sharpness"),
            EnchantmentPredicate::of("minecraft:unbreaking"),
        ]);
        let both = ItemEnchantments::new(vec![
            ench("minecraft:sharpness", 1),
            ench("minecraft:unbreaking", 2),
        ]);
        let only_one = ItemEnchantments::new(vec![ench("minecraft:sharpness", 1)]);
        assert!(pred.matches(&both));
        assert!(!pred.matches(&only_one));
    }

    #[test]
    fn level_bounds_are_respected() {
        let pred = Enchantments::new(vec![EnchantmentPredicate::with_levels(
            "minecraft:sharpness",
            2,
            4,
        )]);
        assert!(!pred.matches(&ItemEnchantments::new(vec![ench("minecraft:sharpness", 1)])));
        assert!(pred.matches(&ItemEnchantments::new(vec![ench("minecraft:sharpness", 2)])));
        assert!(pred.matches(&ItemEnchantments::new(vec![ench("minecraft:sharpness", 4)])));
        assert!(!pred.matches(&ItemEnchantments::new(vec![ench("minecraft:sharpness", 5)])));
    }

    #[test]
    fn stored_enchantments_uses_same_matching_logic() {
        let pred = stored_enchantments(vec![EnchantmentPredicate::of("minecraft:mending")]);
        assert!(pred.matches(&ItemEnchantments::new(vec![ench("minecraft:mending", 1)])));
        assert!(!pred.matches(&ItemEnchantments::default()));
        assert_eq!(pred.enchantments().len(), 1);
    }

    #[test]
    fn component_types_distinguish_variants() {
        let e = Enchantments::new(vec![]);
        let s = StoredEnchantments::new(vec![]);
        assert_eq!(e.component_type(), DataComponents::enchantments());
        assert_eq!(s.component_type(), DataComponents::stored_enchantments());
        assert_ne!(
            DataComponents::enchantments().id,
            DataComponents::stored_enchantments().id
        );
    }
}
