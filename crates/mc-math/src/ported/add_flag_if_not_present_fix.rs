use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Map(Vec<(String, DynamicValue)>),
    List(Vec<DynamicValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return Dynamic { value: v.clone() };
                    }
                }
                Dynamic {
                    value: DynamicValue::Null,
                }
            }
            _ => Dynamic {
                value: DynamicValue::Null,
            },
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        if self.value == DynamicValue::Null {
            None
        } else {
            Some(self.clone())
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynamicValue::Map(entries) => {
                let mut new_entries: Vec<(String, DynamicValue)> =
                    Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), value.value));
                }
                Dynamic {
                    value: DynamicValue::Map(new_entries),
                }
            }
            _ => Dynamic {
                value: DynamicValue::Map(vec![(key.to_string(), value.value)]),
            },
        }
    }

    pub fn create_boolean(&self, b: bool) -> Dynamic {
        Dynamic {
            value: DynamicValue::Bool(b),
        }
    }
}

pub struct DataFixUtils;

impl DataFixUtils {
    pub fn or_else_get(opt: Option<Dynamic>, f: impl FnOnce() -> Dynamic) -> Dynamic {
        match opt {
            Some(v) => v,
            None => f(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Schema {
    version_key: String,
}

impl Schema {
    pub fn new(version_key: impl Into<String>) -> Self {
        Self {
            version_key: version_key.into(),
        }
    }

    pub fn get_version_key(&self) -> &str {
        &self.version_key
    }

    pub fn get_type(&self, _type_reference: &TypeReference) -> Type {
        Type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeReference {
    pub type_name: String,
}

impl TypeReference {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRewriteRule;

pub struct AddFlagIfNotPresentFix {
    name: String,
    flag_value: bool,
    flag_key: String,
    type_reference: TypeReference,
    schema: Schema,
}

impl AddFlagIfNotPresentFix {
    // JAVA: net.minecraft.util.datafix.fixes.AddFlagIfNotPresentFix#<init>
    pub fn new(
        schema: Schema,
        type_reference: TypeReference,
        flag_key: String,
        flag_value: bool,
    ) -> Self {
        let name = format!(
            "AddFlagIfNotPresentFix_{}={} for {}",
            flag_key,
            flag_value,
            schema.get_version_key()
        );
        Self {
            name,
            flag_value,
            flag_key,
            type_reference,
            schema,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AddFlagIfNotPresentFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let _input_type = self.schema.get_type(&self.type_reference);
        // Faithful rule structure: fixTypeEverywhereTyped updates remainder by setting
        // flagKey to existing value or the configured boolean default.
        TypeRewriteRule
    }

    /// Applies the core fix logic represented by makeRule's typed update lambda.
    pub fn apply_to_dynamic(&self, input: &Dynamic) -> Dynamic {
        let existing = input.get(&self.flag_key).result();
        let value = DataFixUtils::or_else_get(existing, || input.create_boolean(self.flag_value));
        input.set(&self.flag_key, value)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn flag_key(&self) -> &str {
        &self.flag_key
    }

    pub fn flag_value(&self) -> bool {
        self.flag_value
    }

    pub fn type_reference(&self) -> &TypeReference {
        &self.type_reference
    }
}

impl fmt::Debug for AddFlagIfNotPresentFix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddFlagIfNotPresentFix")
            .field("name", &self.name)
            .field("flag_value", &self.flag_value)
            .field("flag_key", &self.flag_key)
            .field("type_reference", &self.type_reference)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, DynamicValue)>) -> Dynamic {
        Dynamic {
            value: DynamicValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            ),
        }
    }

    #[test]
    fn adds_flag_when_missing() {
        let fix = AddFlagIfNotPresentFix::new(
            Schema::new("v1"),
            TypeReference::new("test:type"),
            "myFlag".to_string(),
            true,
        );
        let input = map(vec![("other", DynamicValue::Int(1))]);
        let out = fix.apply_to_dynamic(&input);
        assert_eq!(out.get("myFlag").result().unwrap().value, DynamicValue::Bool(true));
        assert_eq!(out.get("other").result().unwrap().value, DynamicValue::Int(1));
    }

    #[test]
    fn preserves_existing_flag_even_if_different() {
        let fix = AddFlagIfNotPresentFix::new(
            Schema::new("v1"),
            TypeReference::new("test:type"),
            "myFlag".to_string(),
            true,
        );
        let input = map(vec![("myFlag", DynamicValue::Bool(false))]);
        let out = fix.apply_to_dynamic(&input);
        assert_eq!(out.get("myFlag").result().unwrap().value, DynamicValue::Bool(false));
    }

    #[test]
    fn name_format_matches_java() {
        let fix = AddFlagIfNotPresentFix::new(
            Schema::new("1.20.5"),
            TypeReference::new("test:type"),
            "has_seen".to_string(),
            false,
        );
        assert_eq!(
            fix.name(),
            "AddFlagIfNotPresentFix_has_seen=false for 1.20.5"
        );
    }

    #[test]
    fn handles_negative_and_float_values_in_map() {
        let fix = AddFlagIfNotPresentFix::new(
            Schema::new("v2"),
            TypeReference::new("t"),
            "flag".to_string(),
            true,
        );
        let bits = f64::to_bits(-0.0f64);
        let input = map(vec![
            ("neg", DynamicValue::Int(i32::MIN)),
            ("long", DynamicValue::Long(i64::MIN)),
            ("dbl_bits", DynamicValue::Long(bits as i64)),
        ]);
        let out = fix.apply_to_dynamic(&input);
        assert_eq!(out.get("neg").result().unwrap().value, DynamicValue::Int(i32::MIN));
        assert_eq!(out.get("long").result().unwrap().value, DynamicValue::Long(i64::MIN));
        let got_bits = match out.get("dbl_bits").result().unwrap().value {
            DynamicValue::Long(v) => v as u64,
            _ => panic!("expected long"),
        };
        assert_eq!(got_bits, bits);
        assert_eq!(out.get("flag").result().unwrap().value, DynamicValue::Bool(true));
    }

    #[test]
    fn make_rule_returns_placeholder_and_type_reference_preserved() {
        let tr = TypeReference::new("minecraft:entity");
        let fix = AddFlagIfNotPresentFix::new(Schema::new("v3"), tr.clone(), "f".to_string(), false);
        let _rule = fix.make_rule();
        assert_eq!(fix.type_reference(), &tr);
        assert!(!fix.flag_value());
        assert_eq!(fix.flag_key(), "f");
    }
}
