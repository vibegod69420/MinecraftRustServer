use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn map(entries: BTreeMap<String, Dynamic>) -> Self {
        Self {
            value: DynamicValue::Map(entries),
        }
    }

    pub fn string(s: impl Into<String>) -> Self {
        Self {
            value: DynamicValue::String(s.into()),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut next = m.clone();
                next.remove(key);
                Dynamic {
                    value: DynamicValue::Map(next),
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    type_: Type,
    value: Dynamic,
}

impl Typed {
    pub fn new(type_: Type, value: Dynamic) -> Self {
        Self { type_, value }
    }

    pub fn get_type(&self) -> &Type {
        &self.type_
    }

    pub fn value(&self) -> &Dynamic {
        &self.value
    }
}

fn write_and_read_typed_or_throw(
    typed: Typed,
    type_: Type,
    f: impl FnOnce(Dynamic) -> Dynamic,
) -> Typed {
    let new_value = f(typed.value);
    Typed {
        type_,
        value: new_value,
    }
}

pub struct FilteredBooksFix {
    _name: &'static str,
    _schema: Schema,
}

impl FilteredBooksFix {
    // JAVA: net.minecraft.util.datafix.fixes.FilteredBooksFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            _name: "Remove filtered text from books",
            _schema: schema,
        }
    }

    pub fn is_target_item(item_id: &str) -> bool {
        item_id == "minecraft:writable_book" || item_id == "minecraft:written_book"
    }

    // JAVA: net.minecraft.util.datafix.fixes.FilteredBooksFix#fixItemStackTag
    pub fn fix_item_stack_tag(&self, typed: Typed) -> Typed {
        let type_ = typed.get_type().clone();
        write_and_read_typed_or_throw(typed, type_, |d| {
            d.remove("filtered_title").remove("filtered_pages")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn book_tag() -> Typed {
        let mut m = BTreeMap::new();
        m.insert(
            "filtered_title".to_string(),
            Dynamic::string("secret"),
        );
        m.insert(
            "filtered_pages".to_string(),
            Dynamic::string("pages"),
        );
        m.insert("title".to_string(), Dynamic::string("visible"));
        Typed::new(Type, Dynamic::map(m))
    }

    #[test]
    fn removes_filtered_fields_preserves_others() {
        let fix = FilteredBooksFix::new(Schema);
        let out = fix.fix_item_stack_tag(book_tag());
        let DynamicValue::Map(m) = &out.value().value else {
            panic!("expected map");
        };
        assert!(!m.contains_key("filtered_title"));
        assert!(!m.contains_key("filtered_pages"));
        assert_eq!(m.get("title"), Some(&Dynamic::string("visible")));
    }

    #[test]
    fn missing_filtered_fields_is_noop() {
        let fix = FilteredBooksFix::new(Schema);
        let mut m = BTreeMap::new();
        m.insert("title".to_string(), Dynamic::string("ok"));
        let typed = Typed::new(Type, Dynamic::map(m.clone()));
        let out = fix.fix_item_stack_tag(typed);
        assert_eq!(out.value(), &Dynamic::map(m));
    }

    #[test]
    fn target_item_predicate_matches_java() {
        assert!(FilteredBooksFix::is_target_item("minecraft:writable_book"));
        assert!(FilteredBooksFix::is_target_item("minecraft:written_book"));
        assert!(!FilteredBooksFix::is_target_item("minecraft:book"));
        assert!(!FilteredBooksFix::is_target_item(""));
    }

    #[test]
    fn preserves_negative_and_float_bit_values() {
        let fix = FilteredBooksFix::new(Schema);
        let mut m = BTreeMap::new();
        m.insert("filtered_title".to_string(), Dynamic::string("x"));
        m.insert(
            "count".to_string(),
            Dynamic {
                value: DynamicValue::Int(i64::MIN),
            },
        );
        m.insert(
            "weight".to_string(),
            Dynamic {
                value: DynamicValue::Float(f64::from_bits(0xBFF8_0000_0000_0000)), // -1.5
            },
        );
        let out = fix.fix_item_stack_tag(Typed::new(Type, Dynamic::map(m)));
        let DynamicValue::Map(out_m) = &out.value().value else {
            panic!("expected map");
        };
        assert_eq!(
            out_m.get("count"),
            Some(&Dynamic {
                value: DynamicValue::Int(i64::MIN)
            })
        );
        assert_eq!(
            out_m.get("weight"),
            Some(&Dynamic {
                value: DynamicValue::Float(f64::from_bits(0xBFF8_0000_0000_0000))
            })
        );
        assert!(!out_m.contains_key("filtered_title"));
    }

    #[test]
    fn remove_on_non_map_is_identity() {
        let d = Dynamic {
            value: DynamicValue::Int(-42),
        };
        assert_eq!(d.remove("filtered_title"), d);
    }
}
