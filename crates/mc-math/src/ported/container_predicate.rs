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

    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }
}

/// Minimal self-contained stand-in for
/// `net.minecraft.world.item.component.ItemContainerContents`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemContainerContents {
    items: Vec<ItemStack>,
}

impl ItemContainerContents {
    pub fn new(items: Vec<ItemStack>) -> Self {
        Self { items }
    }

    // JAVA: net.minecraft.world.item.component.ItemContainerContents#nonEmptyItems
    pub fn non_empty_items(&self) -> Vec<ItemStack> {
        self.items
            .iter()
            .filter(|stack| !stack.is_empty())
            .cloned()
            .collect()
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

/// Port of `net.minecraft.core.component.predicates.ContainerPredicate`.
#[derive(Debug, Clone)]
pub struct ContainerPredicate<C> {
    pub items: Option<C>,
}

impl<C> ContainerPredicate<C> {
    pub fn new(items: Option<C>) -> Self {
        Self { items }
    }

    // JAVA: net.minecraft.core.component.predicates.ContainerPredicate#componentType
    pub fn component_type(&self) -> DataComponentType {
        DataComponentType("minecraft:container")
    }
}

impl<C: CollectionPredicate<ItemStack>> ContainerPredicate<C> {
    // JAVA: net.minecraft.core.component.predicates.ContainerPredicate#matches
    pub fn matches(&self, contents: &ItemContainerContents) -> bool {
        match &self.items {
            None => true,
            Some(predicate) => predicate.test(&contents.non_empty_items()),
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

    struct CountExact(usize);
    impl CollectionPredicate<ItemStack> for CountExact {
        fn test(&self, items: &[ItemStack]) -> bool {
            items.len() == self.0
        }
    }

    struct HasItemId(i32);
    impl CollectionPredicate<ItemStack> for HasItemId {
        fn test(&self, items: &[ItemStack]) -> bool {
            items.iter().any(|s| s.id == self.0)
        }
    }

    #[test]
    fn no_predicate_matches_any_container() {
        let pred = ContainerPredicate::<Always>::new(None);
        assert!(pred.matches(&ItemContainerContents::new(vec![])));
        assert!(pred.matches(&ItemContainerContents::new(vec![ItemStack::new(1, 1)])));
    }

    #[test]
    fn predicate_present_delegates_to_collection_test() {
        let pred = ContainerPredicate::new(Some(Always(true)));
        assert!(pred.matches(&ItemContainerContents::new(vec![])));
        let pred = ContainerPredicate::new(Some(Always(false)));
        assert!(!pred.matches(&ItemContainerContents::new(vec![ItemStack::new(1, 1)])));
    }

    #[test]
    fn matches_uses_non_empty_items_only() {
        let contents = ItemContainerContents::new(vec![
            ItemStack::new(1, 0),
            ItemStack::new(2, 5),
            ItemStack::new(3, -1),
            ItemStack::new(4, 1),
        ]);
        assert!(ContainerPredicate::new(Some(CountExact(2))).matches(&contents));
        assert!(ContainerPredicate::new(Some(HasItemId(2))).matches(&contents));
        assert!(!ContainerPredicate::new(Some(HasItemId(1))).matches(&contents));
    }

    #[test]
    fn component_type_is_container() {
        let pred = ContainerPredicate::<Always>::new(None);
        assert_eq!(
            pred.component_type(),
            DataComponentType("minecraft:container")
        );
    }

    #[test]
    fn empty_or_all_empty_container_fails_item_requirement() {
        let pred = ContainerPredicate::new(Some(HasItemId(1)));
        assert!(!pred.matches(&ItemContainerContents::new(vec![])));
        assert!(!pred.matches(&ItemContainerContents::new(vec![
            ItemStack::new(1, 0),
            ItemStack::new(2, 0),
        ])));
    }
}
