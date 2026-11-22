use std::marker::PhantomData;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Self-contained stubs for unported Minecraft types referenced by this class.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Codec<T>(pub PhantomData<fn() -> T>);

impl<T> Codec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component(pub String);

impl Component {
    pub fn literal(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filterable<T> {
    raw_value: T,
}

impl<T> Filterable<T> {
    pub fn new(raw_value: T) -> Self {
        Self { raw_value }
    }

    pub fn raw(&self) -> &T {
        &self.raw_value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookTitle {
    raw_value: String,
}

impl BookTitle {
    pub fn new(raw_value: impl Into<String>) -> Self {
        Self {
            raw_value: raw_value.into(),
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw_value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenBookContent {
    pages: Vec<Filterable<Component>>,
    author: String,
    title: BookTitle,
    generation: i32,
    resolved: bool,
}

impl WrittenBookContent {
    pub fn new(
        pages: Vec<Filterable<Component>>,
        author: impl Into<String>,
        title: BookTitle,
        generation: i32,
        resolved: bool,
    ) -> Self {
        Self {
            pages,
            author: author.into(),
            title,
            generation,
            resolved,
        }
    }

    pub fn pages(&self) -> &[Filterable<Component>] {
        &self.pages
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn title(&self) -> &BookTitle {
        &self.title
    }

    pub fn generation(&self) -> i32 {
        self.generation
    }

    pub fn resolved(&self) -> bool {
        self.resolved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T>(PhantomData<fn() -> T>);

impl<T> DataComponentType<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct DataComponents;

impl DataComponents {
    pub fn written_book_content() -> DataComponentType<WrittenBookContent> {
        DataComponentType::new()
    }
}

/// Stub for `net.minecraft.advancements.criterion.MinMaxBounds.Ints`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ints {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl Ints {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }
}

/// Stub for `net.minecraft.advancements.criterion.CollectionPredicate`.
/// Real semantics are more complex; this stub provides a deterministic test hook.
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionPredicate<T, P> {
    pub entries: Vec<P>,
    _marker: PhantomData<fn() -> T>,
}

impl<T, P> CollectionPredicate<T, P> {
    pub fn new(entries: Vec<P>) -> Self {
        Self {
            entries,
            _marker: PhantomData,
        }
    }
}

impl<T, P: Predicate<T>> CollectionPredicate<T, P> {
    pub fn test(&self, values: &[T]) -> bool {
        // Simplified collection matching: every predicate entry must match at least one value.
        self.entries.iter().all(|p| values.iter().any(|v| p.test(v)))
    }
}

pub trait Predicate<T> {
    fn test(&self, value: &T) -> bool;
}

pub trait SingleComponentItemPredicate<T> {
    fn component_type(&self) -> DataComponentType<T>;
    fn matches(&self, value: &T) -> bool;
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.core.component.predicates.WrittenBookPredicate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct WrittenBookPredicate {
    pub pages: Option<CollectionPredicate<Filterable<Component>, PagePredicate>>,
    pub author: Option<String>,
    pub title: Option<String>,
    pub generation: Ints,
    pub resolved: Option<bool>,
}

pub static WRITTEN_BOOK_PREDICATE_CODEC: LazyLock<Codec<WrittenBookPredicate>> =
    LazyLock::new(Codec::new);

impl WrittenBookPredicate {
    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<WrittenBookContent> {
        DataComponents::written_book_content()
    }

    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate#matches
    pub fn matches(&self, content: &WrittenBookContent) -> bool {
        if let Some(author) = &self.author {
            if author != content.author() {
                return false;
            }
        }
        if let Some(title) = &self.title {
            if title != content.title().raw() {
                return false;
            }
        }
        if !self.generation.matches(content.generation()) {
            return false;
        }
        if let Some(resolved) = self.resolved {
            if resolved != content.resolved() {
                return false;
            }
        }
        match &self.pages {
            Some(pages) => pages.test(content.pages()),
            None => true,
        }
    }
}

impl SingleComponentItemPredicate<WrittenBookContent> for WrittenBookPredicate {
    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate#componentType
    fn component_type(&self) -> DataComponentType<WrittenBookContent> {
        WrittenBookPredicate::component_type(self)
    }

    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate#matches
    fn matches(&self, value: &WrittenBookContent) -> bool {
        WrittenBookPredicate::matches(self, value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagePredicate {
    pub contents: Component,
}

pub static PAGE_PREDICATE_CODEC: LazyLock<Codec<PagePredicate>> = LazyLock::new(Codec::new);

impl Predicate<Filterable<Component>> for PagePredicate {
    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate$PagePredicate#test
    fn test(&self, value: &Filterable<Component>) -> bool {
        value.raw() == &self.contents
    }
}

impl PagePredicate {
    // JAVA: net.minecraft.core.component.predicates.WrittenBookPredicate$PagePredicate#test
    pub fn test(&self, value: &Filterable<Component>) -> bool {
        <Self as Predicate<Filterable<Component>>>::test(self, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_content() -> WrittenBookContent {
        WrittenBookContent::new(
            vec![
                Filterable::new(Component::literal("hello")),
                Filterable::new(Component::literal("world")),
            ],
            "Alice",
            BookTitle::new("Title"),
            1,
            true,
        )
    }

    fn any_predicate() -> WrittenBookPredicate {
        WrittenBookPredicate {
            pages: None,
            author: None,
            title: None,
            generation: Ints::ANY,
            resolved: None,
        }
    }

    #[test]
    fn matches_when_all_optional_fields_absent() {
        let pred = any_predicate();
        assert!(pred.matches(&sample_content()));
    }

    #[test]
    fn author_and_title_mismatch_fail() {
        let mut pred = any_predicate();
        pred.author = Some("Bob".to_string());
        assert!(!pred.matches(&sample_content()));

        let mut pred2 = any_predicate();
        pred2.title = Some("Other".to_string());
        assert!(!pred2.matches(&sample_content()));
    }

    #[test]
    fn generation_bounds_handle_edges_and_negative() {
        let content = sample_content();
        let mut pred = any_predicate();
        pred.generation = Ints { min: Some(i32::MIN), max: Some(i32::MAX) };
        assert!(pred.matches(&content));

        let mut neg_content = sample_content();
        neg_content = WrittenBookContent::new(
            neg_content.pages().to_vec(),
            neg_content.author().to_string(),
            BookTitle::new(neg_content.title().raw().to_string()),
            -1,
            neg_content.resolved(),
        );
        pred.generation = Ints { min: Some(-5), max: Some(-1) };
        assert!(pred.matches(&neg_content));
        pred.generation = Ints::exactly(0);
        assert!(!pred.matches(&neg_content));
    }

    #[test]
    fn resolved_mismatch_fails() {
        let mut pred = any_predicate();
        pred.resolved = Some(false);
        assert!(!pred.matches(&sample_content()));
    }

    #[test]
    fn page_predicate_and_collection_match() {
        let page = PagePredicate { contents: Component::literal("hello") };
        assert!(page.test(&Filterable::new(Component::literal("hello"))));
        assert!(!page.test(&Filterable::new(Component::literal("bye"))));

        let mut pred = any_predicate();
        pred.pages = Some(CollectionPredicate::new(vec![
            PagePredicate { contents: Component::literal("hello") },
        ]));
        assert!(pred.matches(&sample_content()));

        pred.pages = Some(CollectionPredicate::new(vec![
            PagePredicate { contents: Component::literal("missing") },
        ]));
        assert!(!pred.matches(&sample_content()));
    }

    #[test]
    fn component_type_is_written_book_content() {
        let pred = any_predicate();
        let _ty: DataComponentType<WrittenBookContent> = pred.component_type();
        let _ty2 = DataComponents::written_book_content();
        // Type equality is enforced at compile time; runtime check ensures call works.
        assert!(pred.matches(&sample_content()));
    }
}
