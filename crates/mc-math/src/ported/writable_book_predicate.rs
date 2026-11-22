//! Port of `net.minecraft.core.component.predicates.WritableBookPredicate`.

use std::marker::PhantomData;

/// Minimal stand-in for `net.minecraft.server.network.Filterable`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filterable<T> {
    pub raw: T,
    pub filtered: Option<T>,
}

impl<T> Filterable<T> {
    // JAVA: net.minecraft.server.network.Filterable#raw
    pub fn raw(&self) -> &T {
        &self.raw
    }
}

/// Minimal stand-in for `net.minecraft.world.item.component.WritableBookContent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritableBookContent {
    pub pages: Vec<Filterable<String>>,
}

impl WritableBookContent {
    // JAVA: net.minecraft.world.item.component.WritableBookContent#pages
    pub fn pages(&self) -> &[Filterable<String>] {
        &self.pages
    }
}

/// Minimal stand-in for `net.minecraft.advancements.criterion.CollectionPredicate`.
pub trait CollectionPredicate<T> {
    fn test(&self, items: &[T]) -> bool;
}

/// Minimal stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataComponentType<T>(PhantomData<T>);

/// Minimal stand-in for `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;
impl DataComponents {
    pub const WRITABLE_BOOK_CONTENT: DataComponentType<WritableBookContent> =
        DataComponentType(PhantomData);
}

/// Port of `net.minecraft.core.component.predicates.WritableBookPredicate.PagePredicate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagePredicate {
    pub contents: String,
}

impl PagePredicate {
    // JAVA: net.minecraft.core.component.predicates.WritableBookPredicate.PagePredicate#contents
    pub fn contents(&self) -> &str {
        &self.contents
    }

    // JAVA: net.minecraft.core.component.predicates.WritableBookPredicate.PagePredicate#test
    pub fn test(&self, value: &Filterable<String>) -> bool {
        value.raw() == &self.contents
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritableBookPredicate<P> {
    pub pages: Option<P>,
}

impl<P> WritableBookPredicate<P> {
    // JAVA: net.minecraft.core.component.predicates.WritableBookPredicate#pages
    pub fn pages(&self) -> Option<&P> {
        self.pages.as_ref()
    }

    // JAVA: net.minecraft.core.component.predicates.WritableBookPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<WritableBookContent> {
        DataComponents::WRITABLE_BOOK_CONTENT
    }
}

impl<P: CollectionPredicate<Filterable<String>>> WritableBookPredicate<P> {
    // JAVA: net.minecraft.core.component.predicates.WritableBookPredicate#matches
    pub fn matches(&self, content: &WritableBookContent) -> bool {
        match &self.pages {
            Some(pages) => pages.test(content.pages()),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PagesPred(bool);
    impl CollectionPredicate<Filterable<String>> for PagesPred {
        fn test(&self, _items: &[Filterable<String>]) -> bool {
            self.0
        }
    }

    fn page(raw: &str) -> Filterable<String> {
        Filterable {
            raw: raw.to_string(),
            filtered: None,
        }
    }

    fn book(pages: Vec<&str>) -> WritableBookContent {
        WritableBookContent {
            pages: pages.into_iter().map(page).collect(),
        }
    }

    #[test]
    fn no_pages_predicate_matches_any_content() {
        let pred = WritableBookPredicate::<PagesPred> { pages: None };
        assert!(pred.matches(&book(vec!["hello"])));
        assert!(pred.matches(&book(vec![])));
    }

    #[test]
    fn pages_predicate_delegates_to_collection_test() {
        let pred = WritableBookPredicate {
            pages: Some(PagesPred(true)),
        };
        assert!(pred.matches(&book(vec!["a"])));

        let pred = WritableBookPredicate {
            pages: Some(PagesPred(false)),
        };
        assert!(!pred.matches(&book(vec!["a"])));
    }

    #[test]
    fn page_predicate_matches_raw_content() {
        let pred = PagePredicate {
            contents: "hello".to_string(),
        };
        assert!(pred.test(&page("hello")));
        assert!(!pred.test(&page("world")));
        assert!(!pred.test(&page("")));
    }

    #[test]
    fn page_predicate_compares_raw_not_filtered() {
        let pred = PagePredicate {
            contents: "raw".to_string(),
        };
        let filtered_differs = Filterable {
            raw: "raw".to_string(),
            filtered: Some("filtered".to_string()),
        };
        assert!(pred.test(&filtered_differs));

        let raw_differs = Filterable {
            raw: "filtered".to_string(),
            filtered: Some("raw".to_string()),
        };
        assert!(!pred.test(&raw_differs));
    }

    #[test]
    fn component_type_is_writable_book_content() {
        let pred = WritableBookPredicate::<PagesPred> { pages: None };
        assert_eq!(
            pred.component_type(),
            DataComponents::WRITABLE_BOOK_CONTENT
        );
    }
}
