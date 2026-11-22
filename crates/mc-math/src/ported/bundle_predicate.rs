/// Minimal self-contained stand-in for `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub id: i32,
    pub count: i32,
}

impl ItemStack {
    pub fn new(id: i32, count: i32) -> Self {
        Self { id, count }
    }
}

/// Minimal self-contained stand-in for `net.minecraft.world.item.component.BundleContents`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BundleContents {
    items: Vec<ItemStack>,
}

impl BundleContents {
    pub fn new(items: Vec<ItemStack>) -> Self {
        Self { items }
    }

    // JAVA: net.minecraft.world.item.component.BundleContents#items
    pub fn items(&self) -> &[ItemStack] {
        &self.items
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataComponentType(pub &'static str);

/// Minimal self-contained stand-in for
/// `net.minecraft.advancements.criterion.CollectionPredicate`.
pub trait CollectionPredicate<T> {
    fn test(&self, items: &[T]) -> bool;
}

/// Port of `net.minecraft.core.component.predicates.BundlePredicate`.
#[derive(Debug, Clone)]
pub struct BundlePredicate<C> {
    pub items: Option<C>,
}

impl<C> BundlePredicate<C> {
    pub fn new(items: Option<C>) -> Self {
        Self { items }
    }

    // JAVA: net.minecraft.core.component.predicates.BundlePredicate#componentType
    pub fn component_type(&self) -> DataComponentType {
        DataComponentType("minecraft:bundle_contents")
    }
}

impl<C: CollectionPredicate<ItemStack>> BundlePredicate<C> {
    // JAVA: net.minecraft.core.component.predicates.BundlePredicate#matches
    pub fn matches(&self, contents: &BundleContents) -> bool {
        match &self.items {
            None => true,
            Some(predicate) => predicate.test(contents.items()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Always(bool);
    impl CollectionPredicate<ItemStack> for Always {
        fn test(&self, _items: &[ItemStack]) -> bool {
            self.0
        }
    }

    struct CountAtLeast(usize);
    impl CollectionPredicate<ItemStack> for CountAtLeast {
        fn test(&self, items: &[ItemStack]) -> bool {
            items.len() >= self.0
        }
    }

    struct HasItemId(i32);
    impl CollectionPredicate<ItemStack> for HasItemId {
        fn test(&self, items: &[ItemStack]) -> bool {
            items.iter().any(|s| s.id == self.0)
        }
    }

    #[test]
    fn no_predicate_matches_any_bundle() {
        let pred = BundlePredicate::<Always>::new(None);
        assert!(pred.matches(&BundleContents::new(vec![])));
        assert!(pred.matches(&BundleContents::new(vec![ItemStack::new(1, 1)])));
    }

    #[test]
    fn predicate_present_delegates_to_collection_test() {
        let pred = BundlePredicate::new(Some(Always(true)));
        assert!(pred.matches(&BundleContents::new(vec![])));
        let pred = BundlePredicate::new(Some(Always(false)));
        assert!(!pred.matches(&BundleContents::new(vec![ItemStack::new(1, 1)])));
    }

    #[test]
    fn tests_against_bundle_items() {
        let contents = BundleContents::new(vec![ItemStack::new(10, 1), ItemStack::new(20, 64)]);
        assert!(BundlePredicate::new(Some(CountAtLeast(2))).matches(&contents));
        assert!(!BundlePredicate::new(Some(CountAtLeast(3))).matches(&contents));
        assert!(BundlePredicate::new(Some(HasItemId(20))).matches(&contents));
        assert!(!BundlePredicate::new(Some(HasItemId(99))).matches(&contents));
    }

    #[test]
    fn component_type_is_bundle_contents() {
        let pred = BundlePredicate::<Always>::new(None);
        assert_eq!(
            pred.component_type(),
            DataComponentType("minecraft:bundle_contents")
        );
    }

    #[test]
    fn empty_bundle_with_item_requirement_fails() {
        let pred = BundlePredicate::new(Some(HasItemId(1)));
        assert!(!pred.matches(&BundleContents::new(vec![])));
    }
}
