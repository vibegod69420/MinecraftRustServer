use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    // JAVA: com.mojang.datafixers.util.Pair#mapSecond
    pub fn map_second<C, F: FnOnce(B) -> C>(self, f: F) -> Pair<A, C> {
        Pair(self.0, f(self.1))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#getType
    pub fn get_type<T>(&self, _reference: &str) -> Type<T> {
        Type(PhantomData)
    }
}

pub struct References;
impl References {
    pub const TEXT_COMPONENT: &'static str = "minecraft:text_component";
    pub const ITEM_STACK: &'static str = "minecraft:item_stack";
}

#[derive(Clone, Debug)]
pub struct Type<T = ()>(PhantomData<T>);

impl<T> Type<T> {
    // JAVA: com.mojang.datafixers.types.Type#findField
    pub fn find_field<U>(&self, name: &str) -> OpticFinder<U> {
        OpticFinder::new(name)
    }
}

#[derive(Clone, Debug)]
pub struct OpticFinder<T = ()> {
    name: String,
    _phantom: PhantomData<T>,
}

impl<T> OpticFinder<T> {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            _phantom: PhantomData,
        }
    }

    // JAVA: com.mojang.datafixers.OpticFinder#type
    pub fn optic_type(&self) -> Type<()> {
        Type(PhantomData)
    }
}

pub struct DSL;
impl DSL {
    // JAVA: com.mojang.datafixers.DSL#typeFinder
    pub fn type_finder<T>(_type: Type<T>) -> OpticFinder<T> {
        OpticFinder::new("")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    List(Vec<Dynamic>),
    Compound(Vec<(String, Dynamic)>),
    Empty,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed<T = ()> {
    value: Dynamic,
    _phantom: PhantomData<T>,
}

impl<T> Typed<T> {
    pub fn new(value: Dynamic) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    pub fn value(&self) -> &Dynamic {
        &self.value
    }

    pub fn into_value(self) -> Dynamic {
        self.value
    }

    // JAVA: com.mojang.datafixers.Typed#updateTyped
    pub fn update_typed<U, F>(mut self, finder: &OpticFinder<U>, f: F) -> Self
    where
        F: FnOnce(Typed<U>) -> Typed<U>,
    {
        self.value = update_at_field(self.value, &finder.name, f);
        self
    }

    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update<U, F>(mut self, _finder: &OpticFinder<U>, mut f: F) -> Self
    where
        F: FnMut(Pair<String, String>) -> Pair<String, String>,
    {
        self.value = update_string_pairs(self.value, &mut f);
        self
    }
}

fn update_at_field<U, F: FnOnce(Typed<U>) -> Typed<U>>(
    value: Dynamic,
    name: &str,
    f: F,
) -> Dynamic {
    fn walk<U, F: FnOnce(Typed<U>) -> Typed<U>>(
        value: Dynamic,
        name: &str,
        f: &mut Option<F>,
    ) -> Dynamic {
        match value {
            Dynamic::Compound(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    if k == name {
                        if let Some(func) = f.take() {
                            let inner = Typed::<U>::new(v);
                            out.push((k, func(inner).value));
                        } else {
                            out.push((k, v));
                        }
                    } else {
                        out.push((k, walk(v, name, f)));
                    }
                }
                Dynamic::Compound(out)
            }
            Dynamic::List(items) => Dynamic::List(
                items
                    .into_iter()
                    .map(|v| walk(v, name, f))
                    .collect(),
            ),
            other => other,
        }
    }
    let mut f = Some(f);
    walk(value, name, &mut f)
}

fn update_string_pairs<F: FnMut(Pair<String, String>) -> Pair<String, String>>(
    value: Dynamic,
    f: &mut F,
) -> Dynamic {
    match value {
        Dynamic::List(items) => Dynamic::List(
            items
                .into_iter()
                .map(|item| match item {
                    Dynamic::String(s) => {
                        let p = f(Pair(String::new(), s));
                        Dynamic::String(p.1)
                    }
                    other => update_string_pairs(other, f),
                })
                .collect(),
        ),
        Dynamic::Compound(entries) => Dynamic::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k, update_string_pairs(v, f)))
                .collect(),
        ),
        other => other,
    }
}

pub mod legacy_component_data_fix_utils {
    // JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#rewriteFromLenient
    pub fn rewrite_from_lenient(s: String) -> String {
        let trimmed = s.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.starts_with('"') {
            s
        } else {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("{{\"text\":\"{}\"}}", escaped)
        }
    }
}

pub struct ItemStackTagFix {
    schema: Schema,
    name: String,
    item_predicate: fn(&str) -> bool,
}

impl ItemStackTagFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#<init>
    pub fn new(schema: Schema, name: &str, item_predicate: fn(&str) -> bool) -> Self {
        Self {
            schema,
            name: name.to_string(),
            item_predicate,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#getInputSchema
    pub fn get_input_schema(&self) -> &Schema {
        &self.schema
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#fixItemStackTag
    pub fn fix_item_stack_tag(&self, typed: Typed) -> Typed {
        typed
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn matches_item(&self, item_id: &str) -> bool {
        (self.item_predicate)(item_id)
    }
}

pub struct WrittenBookPagesStrictJsonFix {
    base: ItemStackTagFix,
}

impl WrittenBookPagesStrictJsonFix {
    // JAVA: net.minecraft.util.datafix.fixes.WrittenBookPagesStrictJsonFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            base: ItemStackTagFix::new(schema, "WrittenBookPagesStrictJsonFix", |id| {
                id == "minecraft:written_book"
            }),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WrittenBookPagesStrictJsonFix#fixItemStackTag
    pub fn fix_item_stack_tag(&self, typed: Typed) -> Typed {
        let type_text: Type<Pair<String, String>> =
            self.base.get_input_schema().get_type(References::TEXT_COMPONENT);
        let type_stack: Type<()> = self.base.get_input_schema().get_type(References::ITEM_STACK);
        let tag_finder = type_stack.find_field::<()>("tag");
        let pages_finder = tag_finder.optic_type().find_field::<()>("pages");
        let pair_finder: OpticFinder<Pair<String, String>> = DSL::type_finder(type_text);
        typed.update_typed(&pages_finder, |t| {
            t.update(&pair_finder, |p| {
                p.map_second(legacy_component_data_fix_utils::rewrite_from_lenient)
            })
        })
    }

    pub fn name(&self) -> &str {
        self.base.name()
    }

    pub fn matches_item(&self, item_id: &str) -> bool {
        self.base.matches_item(item_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book_with_pages(pages: Vec<&str>) -> Typed {
        Typed::new(Dynamic::Compound(vec![
            ("id".to_string(), Dynamic::String("minecraft:written_book".to_string())),
            (
                "tag".to_string(),
                Dynamic::Compound(vec![(
                    "pages".to_string(),
                    Dynamic::List(pages.into_iter().map(|s| Dynamic::String(s.to_string())).collect()),
                )]),
            ),
        ]))
    }

    #[test]
    fn rewrites_plain_text_pages() {
        let fix = WrittenBookPagesStrictJsonFix::new(Schema);
        let out = fix.fix_item_stack_tag(book_with_pages(vec!["hello"])).into_value();
        let expected = Dynamic::Compound(vec![
            ("id".to_string(), Dynamic::String("minecraft:written_book".to_string())),
            (
                "tag".to_string(),
                Dynamic::Compound(vec![(
                    "pages".to_string(),
                    Dynamic::List(vec![Dynamic::String("{\"text\":\"hello\"}".to_string())]),
                )]),
            ),
        ]);
        assert_eq!(out, expected);
    }

    #[test]
    fn preserves_already_json_pages() {
        let fix = WrittenBookPagesStrictJsonFix::new(Schema);
        let json = "{\"text\":\"hi\"}";
        let out = fix.fix_item_stack_tag(book_with_pages(vec![json])).into_value();
        if let Dynamic::Compound(entries) = out {
            let tag = entries.iter().find(|(k, _)| k == "tag").unwrap().1.clone();
            if let Dynamic::Compound(tag_entries) = tag {
                let pages = tag_entries.iter().find(|(k, _)| k == "pages").unwrap().1.clone();
                assert_eq!(pages, Dynamic::List(vec![Dynamic::String(json.to_string())]));
                return;
            }
        }
        panic!("unexpected structure");
    }

    #[test]
    fn handles_empty_pages_and_missing_tag() {
        let fix = WrittenBookPagesStrictJsonFix::new(Schema);
        let empty_pages = fix.fix_item_stack_tag(book_with_pages(vec![])).into_value();
        if let Dynamic::Compound(entries) = &empty_pages {
            let tag = entries.iter().find(|(k, _)| k == "tag").unwrap().1.clone();
            if let Dynamic::Compound(tag_entries) = tag {
                let pages = tag_entries.iter().find(|(k, _)| k == "pages").unwrap().1.clone();
                assert_eq!(pages, Dynamic::List(vec![]));
            } else { panic!("bad tag"); }
        } else { panic!("bad root"); }

        let no_tag = Typed::new(Dynamic::Compound(vec![(
            "id".to_string(),
            Dynamic::String("minecraft:written_book".to_string()),
        )]));
        let out = fix.fix_item_stack_tag(no_tag.clone()).into_value();
        assert_eq!(out, no_tag.into_value());
    }

    #[test]
    fn matches_only_written_book() {
        let fix = WrittenBookPagesStrictJsonFix::new(Schema);
        assert!(fix.matches_item("minecraft:written_book"));
        assert!(!fix.matches_item("minecraft:writable_book"));
        assert_eq!(fix.name(), "WrittenBookPagesStrictJsonFix");
    }

    #[test]
    fn rewrite_from_lenient_edge_cases() {
        assert_eq!(
            legacy_component_data_fix_utils::rewrite_from_lenient("a\"b".to_string()),
            "{\"text\":\"a\\\"b\"}"
        );
        assert_eq!(
            legacy_component_data_fix_utils::rewrite_from_lenient("[\"x\"]".to_string()),
            "[\"x\"]"
        );
    }
}
