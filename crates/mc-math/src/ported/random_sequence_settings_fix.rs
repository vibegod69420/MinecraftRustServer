use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

static DYNAMIC_NULL: Dynamic = Dynamic::Null;

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).unwrap_or(&DYNAMIC_NULL),
            _ => &DYNAMIC_NULL,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut next = m.clone();
                next.insert(key.to_string(), value);
                Dynamic::Map(next)
            }
            _ => self.clone(),
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Dynamic {
        let child = self.get(key).clone();
        self.set(key, f(child))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct TypeRewriteRule {
    name: String,
    apply_fn: Box<dyn Fn(Dynamic) -> Dynamic>,
}

impl TypeRewriteRule {
    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.apply_fn)(input)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct RandomSequenceSettingsFix {
    #[allow(dead_code)]
    schema: Schema,
    #[allow(dead_code)]
    changes_type: bool,
}

impl RandomSequenceSettingsFix {
    // JAVA: net.minecraft.util.datafix.fixes.RandomSequenceSettingsFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            changes_type: false,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RandomSequenceSettingsFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "RandomSequenceSettingsFix".to_string(),
            apply_fn: Box::new(|remainder: Dynamic| {
                remainder.update("data", |data| {
                    Dynamic::empty_map().set("sequences", data)
                })
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn wraps_existing_data_under_sequences() {
        let fix = RandomSequenceSettingsFix::new(Schema);
        let data = map(vec![("foo", Dynamic::String("bar".to_string()))]);
        let input = map(vec![("data", data.clone())]);
        let out = fix.make_rule().apply(input);
        let expected_data = Dynamic::empty_map().set("sequences", data);
        assert_eq!(out.get("data"), &expected_data);
    }

    #[test]
    fn preserves_other_top_level_fields() {
        let fix = RandomSequenceSettingsFix::new(Schema);
        let data = map(vec![("seq1", Dynamic::String("v".to_string()))]);
        let input = map(vec![
            ("data", data.clone()),
            ("other", Dynamic::Bool(true)),
        ]);
        let out = fix.make_rule().apply(input);
        assert_eq!(out.get("other"), &Dynamic::Bool(true));
        assert_eq!(
            out.get("data"),
            &Dynamic::empty_map().set("sequences", data)
        );
    }

    #[test]
    fn missing_data_becomes_sequences_null() {
        let fix = RandomSequenceSettingsFix::new(Schema);
        let input = Dynamic::empty_map();
        let out = fix.make_rule().apply(input);
        let expected_data = Dynamic::empty_map().set("sequences", Dynamic::Null);
        assert_eq!(out.get("data"), &expected_data);
    }

    #[test]
    fn rule_name_matches_java() {
        let fix = RandomSequenceSettingsFix::new(Schema);
        assert_eq!(fix.make_rule().name(), "RandomSequenceSettingsFix");
    }
}
