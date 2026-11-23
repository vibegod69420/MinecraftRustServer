use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Byte(i8),
    Short(i16),
    Map(BTreeMap<String, DynValue>),
    List(Vec<DynValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    value: DynValue,
}

pub struct OptionalDynamic {
    inner: Option<Dynamic>,
}

impl OptionalDynamic {
    pub fn result(self) -> Option<Dynamic> {
        self.inner
    }

    pub fn or_else_empty_map(self) -> Dynamic {
        self.inner.unwrap_or_else(Dynamic::empty_map)
    }
}

impl Dynamic {
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    pub fn empty_map() -> Self {
        Self {
            value: DynValue::Map(BTreeMap::new()),
        }
    }

    pub fn value(&self) -> &DynValue {
        &self.value
    }

    pub fn into_value(self) -> DynValue {
        self.value
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        let inner = match &self.value {
            DynValue::Map(map) => map.get(key).cloned().map(Dynamic::new),
            _ => None,
        };
        OptionalDynamic { inner }
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let DynValue::Map(ref mut map) = self.value {
            map.remove(key);
        }
        self
    }

    pub fn set(mut self, key: &str, value: Dynamic) -> Self {
        match self.value {
            DynValue::Map(ref mut map) => {
                map.insert(key.to_string(), value.value);
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert(key.to_string(), value.value);
                self.value = DynValue::Map(map);
            }
        }
        self
    }

    pub fn update(self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        match self.get(key).result() {
            Some(child) => self.set(key, f(child)),
            None => self,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct TippedArrowPotionToItemFix {
    _schema: Schema,
}

impl TippedArrowPotionToItemFix {
    // JAVA: net.minecraft.util.datafix.fixes.TippedArrowPotionToItemFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.TippedArrowPotionToItemFix#fix
    pub fn fix(input: Dynamic) -> Dynamic {
        let potion = input.get("Potion").result();
        let custom_potion_effects = input.get("custom_potion_effects").result();
        let color = input.get("Color").result();
        if potion.is_none() && custom_potion_effects.is_none() && color.is_none() {
            return input;
        }

        input
            .remove("Potion")
            .remove("custom_potion_effects")
            .remove("Color")
            .update("item", |item| {
                let mut tag = item.get("tag").or_else_empty_map();
                if let Some(p) = potion {
                    tag = tag.set("Potion", p);
                }
                if let Some(cpe) = custom_potion_effects {
                    tag = tag.set("custom_potion_effects", cpe);
                }
                if let Some(c) = color {
                    tag = tag.set("CustomPotionColor", c);
                }
                item.set("tag", tag)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, DynValue)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::new(DynValue::Map(m))
    }

    #[test]
    fn no_potion_fields_returns_unchanged() {
        let input = map(vec![(
            "item",
            DynValue::Map(BTreeMap::from([(
                "id".to_string(),
                DynValue::String("minecraft:arrow".to_string()),
            )])),
        )]);
        let out = TippedArrowPotionToItemFix::fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn moves_potion_to_item_tag() {
        let input = map(vec![
            ("Potion", DynValue::String("minecraft:healing".to_string())),
            (
                "item",
                DynValue::Map(BTreeMap::from([(
                    "id".to_string(),
                    DynValue::String("minecraft:tipped_arrow".to_string()),
                )])),
            ),
        ]);
        let out = TippedArrowPotionToItemFix::fix(input);
        assert_eq!(out.get("Potion").result(), None);
        let tag = out.get("item").result().unwrap().get("tag").result().unwrap();
        assert_eq!(
            tag.get("Potion").result().unwrap().value(),
            &DynValue::String("minecraft:healing".to_string())
        );
    }

    #[test]
    fn moves_all_three_fields_and_preserves_existing_tag() {
        let mut existing_tag = BTreeMap::new();
        existing_tag.insert("Damage".to_string(), DynValue::Int(0));
        let mut item_map = BTreeMap::new();
        item_map.insert("id".to_string(), DynValue::String("minecraft:tipped_arrow".to_string()));
        item_map.insert("tag".to_string(), DynValue::Map(existing_tag));

        let input = map(vec![
            ("Potion", DynValue::String("minecraft:poison".to_string())),
            ("custom_potion_effects", DynValue::List(vec![DynValue::Int(1)])),
            ("Color", DynValue::Int(-1)),
            ("item", DynValue::Map(item_map)),
        ]);

        let out = TippedArrowPotionToItemFix::fix(input);
        assert!(out.get("Potion").result().is_none());
        assert!(out.get("custom_potion_effects").result().is_none());
        assert!(out.get("Color").result().is_none());

        let tag = out.get("item").result().unwrap().get("tag").result().unwrap();
        assert_eq!(tag.get("Damage").result().unwrap().value(), &DynValue::Int(0));
        assert_eq!(
            tag.get("Potion").result().unwrap().value(),
            &DynValue::String("minecraft:poison".to_string())
        );
        assert_eq!(
            tag.get("custom_potion_effects").result().unwrap().value(),
            &DynValue::List(vec![DynValue::Int(1)])
        );
        assert_eq!(
            tag.get("CustomPotionColor").result().unwrap().value(),
            &DynValue::Int(-1)
        );
    }

    #[test]
    fn missing_item_field_leaves_input_without_potion_fields_removed_only_if_updated() {
        // Java update returns original when field is missing; our update follows that behavior.
        let input = map(vec![("Potion", DynValue::String("minecraft:healing".to_string()))]);
        let out = TippedArrowPotionToItemFix::fix(input);
        // Potion/custom_potion_effects/Color are removed, but item update is a no-op if item missing.
        assert!(out.get("Potion").result().is_none());
        assert!(out.get("item").result().is_none());
    }

    #[test]
    fn color_only_triggers_fix() {
        let input = map(vec![
            ("Color", DynValue::Int(0xFF00FF)),
            ("item", DynValue::Map(BTreeMap::new())),
        ]);
        let out = TippedArrowPotionToItemFix::fix(input);
        let tag = out.get("item").result().unwrap().get("tag").result().unwrap();
        assert_eq!(
            tag.get("CustomPotionColor").result().unwrap().value(),
            &DynValue::Int(0xFF00FF)
        );
        assert!(tag.get("Potion").result().is_none());
    }
}
