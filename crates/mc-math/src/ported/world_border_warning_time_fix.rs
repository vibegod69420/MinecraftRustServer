use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Compound(BTreeMap<String, Dynamic>),
    Missing,
}

impl Dynamic {
    pub fn compound(entries: Vec<(String, Dynamic)>) -> Self {
        Dynamic::Compound(entries.into_iter().collect())
    }

    pub fn get_field(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Compound(map) => map.get(key),
            _ => None,
        }
    }

    pub fn update<F>(&self, key: &str, f: F) -> Dynamic
    where
        F: FnOnce(&Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Compound(map) => {
                let mut new_map = map.clone();
                let child = new_map.get(key).cloned().unwrap_or(Dynamic::Missing);
                new_map.insert(key.to_string(), f(&child));
                Dynamic::Compound(new_map)
            }
            Dynamic::Missing => {
                let mut new_map = BTreeMap::new();
                new_map.insert(key.to_string(), f(&Dynamic::Missing));
                Dynamic::Compound(new_map)
            }
            _ => self.clone(),
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Byte(v) => i32::from(*v),
            Dynamic::Short(v) => i32::from(*v),
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            Dynamic::Float(v) => *v as i32,
            Dynamic::Double(v) => *v as i32,
            _ => default,
        }
    }

    pub fn create_int(&self, v: i32) -> Dynamic {
        Dynamic::Int(v)
    }
}

pub struct TypeRewriteRule {
    pub name: &'static str,
    transform: Box<dyn Fn(Dynamic) -> Dynamic>,
}

impl TypeRewriteRule {
    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.transform)(input)
    }
}

#[derive(Debug, Clone)]
pub struct WorldBorderWarningTimeFix {
    pub schema: Schema,
    pub changes_type: bool,
}

impl WorldBorderWarningTimeFix {
    // JAVA: net.minecraft.util.datafix.fixes.WorldBorderWarningTimeFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            changes_type: false,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WorldBorderWarningTimeFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "WorldBorderWarningTimeFix",
            transform: Box::new(|root: Dynamic| {
                root.update("data", |data| {
                    data.update("warning_time", |warning_time| {
                        let old = warning_time.as_int(15);
                        data.create_int(old.wrapping_mul(20))
                    })
                })
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_root(warning_time: Option<Dynamic>) -> Dynamic {
        let mut data_entries = vec![("size".to_string(), Dynamic::Double(1000.0))];
        if let Some(wt) = warning_time {
            data_entries.push(("warning_time".to_string(), wt));
        }
        Dynamic::compound(vec![("data".to_string(), Dynamic::compound(data_entries))])
    }

    #[test]
    fn missing_warning_time_uses_default_15() {
        let fix = WorldBorderWarningTimeFix::new(Schema);
        let rule = fix.make_rule();
        let out = rule.apply(make_root(None));
        let data = out.get_field("data").unwrap();
        assert_eq!(data.get_field("warning_time"), Some(&Dynamic::Int(300)));
    }

    #[test]
    fn existing_warning_time_is_multiplied_by_20() {
        let fix = WorldBorderWarningTimeFix::new(Schema);
        let rule = fix.make_rule();
        let out = rule.apply(make_root(Some(Dynamic::Int(15))));
        let data = out.get_field("data").unwrap();
        assert_eq!(data.get_field("warning_time"), Some(&Dynamic::Int(300)));

        let out2 = rule.apply(make_root(Some(Dynamic::Int(2))));
        let data2 = out2.get_field("data").unwrap();
        assert_eq!(data2.get_field("warning_time"), Some(&Dynamic::Int(40)));
    }

    #[test]
    fn warning_time_wrapping_mul_matches_java_int_semantics() {
        let fix = WorldBorderWarningTimeFix::new(Schema);
        let rule = fix.make_rule();
        let out = rule.apply(make_root(Some(Dynamic::Int(i32::MAX))));
        let data = out.get_field("data").unwrap();
        assert_eq!(
            data.get_field("warning_time"),
            Some(&Dynamic::Int(i32::MAX.wrapping_mul(20)))
        );
    }

    #[test]
    fn preserves_other_data_fields() {
        let fix = WorldBorderWarningTimeFix::new(Schema);
        let rule = fix.make_rule();
        let out = rule.apply(make_root(Some(Dynamic::Int(1))));
        let data = out.get_field("data").unwrap();
        assert_eq!(data.get_field("size"), Some(&Dynamic::Double(1000.0)));
        assert_eq!(data.get_field("warning_time"), Some(&Dynamic::Int(20)));
    }

    #[test]
    fn constructor_and_rule_metadata() {
        let fix = WorldBorderWarningTimeFix::new(Schema);
        assert!(!fix.changes_type);
        let rule = fix.make_rule();
        assert_eq!(rule.name, "WorldBorderWarningTimeFix");
    }
}
