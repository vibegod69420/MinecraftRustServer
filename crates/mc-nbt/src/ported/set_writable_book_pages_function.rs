use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filterable<T>(pub T);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WritableBookContent {
    pub pages: Vec<Filterable<String>>,
}

impl WritableBookContent {
    pub const EMPTY: Self = Self { pages: Vec::new() };

    pub fn pages(&self) -> &[Filterable<String>] {
        &self.pages
    }

    pub fn with_replaced_pages(&self, pages: Vec<Filterable<String>>) -> Self {
        Self { pages }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListOperation {
    ReplaceAll,
    Append,
}

impl ListOperation {
    pub fn apply<T: Clone>(&self, existing: &[T], new_values: &[T], limit: usize) -> Vec<T> {
        match self {
            Self::ReplaceAll => new_values.iter().take(limit).cloned().collect(),
            Self::Append => {
                let mut out = existing.to_vec();
                for v in new_values {
                    if out.len() >= limit {
                        break;
                    }
                    out.push(v.clone());
                }
                out
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_WRITABLE_BOOK_PAGES: LootItemFunctionType =
        LootItemFunctionType("minecraft:set_writable_book_pages");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WritableBookContentComponent;

pub struct DataComponents;

impl DataComponents {
    pub const WRITABLE_BOOK_CONTENT: WritableBookContentComponent = WritableBookContentComponent;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub writable_book_content: Option<WritableBookContent>,
}

impl ItemStack {
    pub fn update<F>(
        &mut self,
        _component: WritableBookContentComponent,
        default: WritableBookContent,
        f: F,
    ) where
        F: FnOnce(WritableBookContent) -> WritableBookContent,
    {
        let current = self.writable_book_content.take().unwrap_or(default);
        self.writable_book_content = Some(f(current));
    }
}

impl fmt::Display for ItemStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ItemStack")
    }
}

pub struct SetWritableBookPagesFunction {
    pub predicates: Vec<LootItemCondition>,
    pub pages: Vec<Filterable<String>>,
    pub page_operation: ListOperation,
}

impl SetWritableBookPagesFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWritableBookPagesFunction#<init>
    pub fn new(
        predicates: Vec<LootItemCondition>,
        pages: Vec<Filterable<String>>,
        page_operation: ListOperation,
    ) -> Self {
        Self {
            predicates,
            pages,
            page_operation,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWritableBookPagesFunction#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.update(
            DataComponents::WRITABLE_BOOK_CONTENT,
            WritableBookContent::EMPTY,
            |c| self.apply(c),
        );
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWritableBookPagesFunction#apply
    pub fn apply(&self, content: WritableBookContent) -> WritableBookContent {
        let new_pages = self.page_operation.apply(content.pages(), &self.pages, 100);
        content.with_replaced_pages(new_pages)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWritableBookPagesFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_WRITABLE_BOOK_PAGES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(s: &str) -> Filterable<String> {
        Filterable(s.to_string())
    }

    #[test]
    fn apply_replace_pages() {
        let f = SetWritableBookPagesFunction::new(
            vec![],
            vec![page("a"), page("b")],
            ListOperation::ReplaceAll,
        );
        let content = WritableBookContent {
            pages: vec![page("old")],
        };
        let out = f.apply(content);
        assert_eq!(out.pages, vec![page("a"), page("b")]);
    }

    #[test]
    fn apply_append_respects_limit_100() {
        let existing: Vec<_> = (0..99).map(|i| page(&i.to_string())).collect();
        let new_pages: Vec<_> = (0..5).map(|i| page(&format!("n{i}"))).collect();
        let f = SetWritableBookPagesFunction::new(vec![], new_pages, ListOperation::Append);
        let out = f.apply(WritableBookContent { pages: existing });
        assert_eq!(out.pages.len(), 100);
        assert_eq!(out.pages[98], page("98"));
        assert_eq!(out.pages[99], page("n0"));
    }

    #[test]
    fn run_updates_item_stack_with_default() {
        let f = SetWritableBookPagesFunction::new(
            vec![],
            vec![page("hello")],
            ListOperation::ReplaceAll,
        );
        let stack = ItemStack::default();
        let out = f.run(stack, &LootContext);
        assert_eq!(
            out.writable_book_content.unwrap().pages,
            vec![page("hello")]
        );
    }

    #[test]
    fn run_preserves_existing_content_base() {
        let f = SetWritableBookPagesFunction::new(vec![], vec![page("new")], ListOperation::Append);
        let stack = ItemStack {
            writable_book_content: Some(WritableBookContent {
                pages: vec![page("old")],
            }),
        };
        let out = f.run(stack, &LootContext);
        assert_eq!(
            out.writable_book_content.unwrap().pages,
            vec![page("old"), page("new")]
        );
    }

    #[test]
    fn replace_truncates_to_limit_100() {
        let new_pages: Vec<_> = (0..120).map(|i| page(&i.to_string())).collect();
        let f = SetWritableBookPagesFunction::new(vec![], new_pages, ListOperation::ReplaceAll);
        let out = f.apply(WritableBookContent::EMPTY);
        assert_eq!(out.pages.len(), 100);
        assert_eq!(out.pages[99], page("99"));
    }

    #[test]
    fn get_type_is_set_writable_book_pages() {
        let f = SetWritableBookPagesFunction::new(vec![], vec![], ListOperation::ReplaceAll);
        assert_eq!(
            f.get_type(),
            LootItemFunctions::SET_WRITABLE_BOOK_PAGES
        );
    }
}
