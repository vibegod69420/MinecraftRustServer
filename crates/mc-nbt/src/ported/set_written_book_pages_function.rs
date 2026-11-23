// Self-contained faithful port of
// net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filterable<T> {
    pub raw: T,
    pub filtered: Option<T>,
}

impl<T: Clone> Filterable<T> {
    pub fn pass_through(value: T) -> Self {
        Self {
            raw: value.clone(),
            filtered: Some(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenBookContent {
    pub pages: Vec<Filterable<Component>>,
}

impl WrittenBookContent {
    // JAVA: net.minecraft.world.item.component.WrittenBookContent#EMPTY
    pub const fn empty() -> Self {
        Self { pages: Vec::new() }
    }

    // JAVA: net.minecraft.world.item.component.WrittenBookContent#withReplacedPages
    pub fn with_replaced_pages(&self, pages: Vec<Filterable<Component>>) -> Self {
        Self { pages }
    }
}

impl Default for WrittenBookContent {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListOperation {
    ReplaceAll,
    Append,
    Insert { index: i32 },
    // other modes stubbed
}

impl ListOperation {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#apply
    pub fn apply<T: Clone>(&self, current: &[T], additions: &[T]) -> Vec<T> {
        match self {
            ListOperation::ReplaceAll => additions.to_vec(),
            ListOperation::Append => {
                let mut out = current.to_vec();
                out.extend_from_slice(additions);
                out
            }
            ListOperation::Insert { index } => {
                let mut out = current.to_vec();
                let len = out.len() as i32;
                // Clamp index like Java list insertion bounds in typical list ops; keep deterministic.
                let idx = if *index < 0 { 0 } else if *index > len { len } else { *index } as usize;
                let mut tail = out.split_off(idx);
                out.extend_from_slice(additions);
                out.append(&mut tail);
                out
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub written_book_content: Option<WrittenBookContent>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#update (DataComponents.WRITTEN_BOOK_CONTENT specialization)
    pub fn update_written_book_content<F>(&mut self, default: WrittenBookContent, f: F)
    where
        F: FnOnce(WrittenBookContent) -> WrittenBookContent,
    {
        let current = self.written_book_content.take().unwrap_or(default);
        self.written_book_content = Some(f(current));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;
#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    SetWrittenBookPages,
}

#[derive(Debug, Clone)]
pub struct SetWrittenBookPagesFunction {
    pub conditions: Vec<LootItemCondition>,
    pub pages: Vec<Filterable<Component>>,
    pub page_operation: ListOperation,
}

impl SetWrittenBookPagesFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction#CODEC
    // Codec placeholder: RecordCodecBuilder.mapCodec(commonFields(...).and(group(PAGES_CODEC, ListOperation.UNLIMITED_CODEC)...))
    pub const CODEC_NAME: &'static str = "minecraft:set_written_book_pages";

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction#<init>
    pub fn new(
        conditions: Vec<LootItemCondition>,
        pages: Vec<Filterable<Component>>,
        page_operation: ListOperation,
    ) -> Self {
        Self {
            conditions,
            pages,
            page_operation,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.update_written_book_content(WrittenBookContent::empty(), |c| self.apply(c));
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction#apply
    pub fn apply(&self, content: WrittenBookContent) -> WrittenBookContent {
        let new_pages = self.page_operation.apply(&content.pages, &self.pages);
        content.with_replaced_pages(new_pages)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetWrittenBookPages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(s: &str) -> Filterable<Component> {
        Filterable::pass_through(Component::literal(s))
    }

    #[test]
    fn replace_all_pages_on_empty_book() {
        let f = SetWrittenBookPagesFunction::new(vec![], vec![page("a"), page("b")], ListOperation::ReplaceAll);
        let out = f.run(ItemStack::default(), &LootContext::default());
        let pages = out.written_book_content.unwrap().pages;
        assert_eq!(pages, vec![page("a"), page("b")]);
        assert_eq!(f.get_type(), LootItemFunctionType::SetWrittenBookPages);
    }

    #[test]
    fn append_pages_preserves_existing() {
        let mut stack = ItemStack::default();
        stack.written_book_content = Some(WrittenBookContent { pages: vec![page("old")] });
        let f = SetWrittenBookPagesFunction::new(vec![], vec![page("new")], ListOperation::Append);
        let out = f.run(stack, &LootContext::default());
        assert_eq!(out.written_book_content.unwrap().pages, vec![page("old"), page("new")]);
    }

    #[test]
    fn insert_with_negative_index_clamps_to_zero() {
        let stack = ItemStack {
            written_book_content: Some(WrittenBookContent { pages: vec![page("a"), page("b")] }),
        };
        let f = SetWrittenBookPagesFunction::new(vec![], vec![page("x")], ListOperation::Insert { index: -10 });
        let out = f.run(stack, &LootContext::default());
        assert_eq!(out.written_book_content.unwrap().pages, vec![page("x"), page("a"), page("b")]);
    }

    #[test]
    fn insert_with_large_index_clamps_to_end() {
        let stack = ItemStack {
            written_book_content: Some(WrittenBookContent { pages: vec![page("a")] }),
        };
        let f = SetWrittenBookPagesFunction::new(vec![], vec![page("z")], ListOperation::Insert { index: i32::MAX });
        let out = f.run(stack, &LootContext::default());
        assert_eq!(out.written_book_content.unwrap().pages, vec![page("a"), page("z")]);
    }

    #[test]
    fn apply_replaces_pages_without_touching_other_state() {
        let content = WrittenBookContent { pages: vec![page("old")] };
        let f = SetWrittenBookPagesFunction::new(vec![], vec![page("new")], ListOperation::ReplaceAll);
        let out = f.apply(content);
        assert_eq!(out.pages, vec![page("new")]);
    }
}
