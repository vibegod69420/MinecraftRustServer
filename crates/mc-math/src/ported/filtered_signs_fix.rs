use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference;

pub struct References;

impl References {
    pub const BLOCK_ENTITY: Reference = Reference;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dynamic<T: Clone> {
    value: BTreeMap<String, T>,
}

impl<T: Clone> Dynamic<T> {
    pub fn new(value: BTreeMap<String, T>) -> Self {
        Self { value }
    }

    pub fn remove(mut self, key: &str) -> Self {
        self.value.remove(key);
        self
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.value.get(key)
    }

    pub fn into_map(self) -> BTreeMap<String, T> {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredSignsFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: Reference,
    pub entity_name: String,
}

impl FilteredSignsFix {
    // JAVA: net.minecraft.util.datafix.fixes.FilteredSignsFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            changes_type: false,
            name: "Remove filtered text from signs".to_string(),
            type_reference: References::BLOCK_ENTITY,
            entity_name: "minecraft:sign".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.FilteredSignsFix#fix
    pub fn fix<T: Clone>(&self, d: Dynamic<T>) -> Dynamic<T> {
        d.remove("FilteredText1")
            .remove("FilteredText2")
            .remove("FilteredText3")
            .remove("FilteredText4")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_dynamic(pairs: &[(&str, &str)]) -> Dynamic<String> {
        let mut m = BTreeMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), (*v).to_string());
        }
        Dynamic::new(m)
    }

    #[test]
    fn removes_all_filtered_text_fields() {
        let fix = FilteredSignsFix::new(Schema);
        let d = make_dynamic(&[
            ("Text1", "a"),
            ("FilteredText1", "b"),
            ("Text2", "c"),
            ("FilteredText2", "d"),
            ("Text3", "e"),
            ("FilteredText3", "f"),
            ("Text4", "g"),
            ("FilteredText4", "h"),
        ]);
        let out = fix.fix(d).into_map();
        assert_eq!(out.len(), 4);
        assert_eq!(out.get("Text1").map(String::as_str), Some("a"));
        assert_eq!(out.get("Text2").map(String::as_str), Some("c"));
        assert_eq!(out.get("Text3").map(String::as_str), Some("e"));
        assert_eq!(out.get("Text4").map(String::as_str), Some("g"));
        assert!(!out.contains_key("FilteredText1"));
        assert!(!out.contains_key("FilteredText2"));
        assert!(!out.contains_key("FilteredText3"));
        assert!(!out.contains_key("FilteredText4"));
    }

    #[test]
    fn handles_missing_filtered_fields() {
        let fix = FilteredSignsFix::new(Schema);
        let d = make_dynamic(&[("Text1", "hello"), ("Text2", "world")]);
        let out = fix.fix(d).into_map();
        assert_eq!(out.len(), 2);
        assert_eq!(out.get("Text1").map(String::as_str), Some("hello"));
        assert_eq!(out.get("Text2").map(String::as_str), Some("world"));
    }

    #[test]
    fn preserves_other_fields_and_empty_input() {
        let fix = FilteredSignsFix::new(Schema);
        let d = make_dynamic(&[
            ("id", "minecraft:sign"),
            ("x", "0"),
            ("FilteredText2", "secret"),
        ]);
        let out = fix.fix(d).into_map();
        assert_eq!(out.len(), 2);
        assert_eq!(out.get("id").map(String::as_str), Some("minecraft:sign"));
        assert_eq!(out.get("x").map(String::as_str), Some("0"));

        let empty_out = fix.fix(Dynamic::new(BTreeMap::<String, String>::new())).into_map();
        assert!(empty_out.is_empty());
    }

    #[test]
    fn constructor_sets_expected_fields() {
        let fix = FilteredSignsFix::new(Schema);
        assert!(!fix.changes_type);
        assert_eq!(fix.name, "Remove filtered text from signs");
        assert_eq!(fix.entity_name, "minecraft:sign");
        assert_eq!(fix.type_reference, References::BLOCK_ENTITY);
        assert_eq!(fix.schema, Schema);
    }
}
