use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn string(s: impl Into<String>) -> Self {
        Dynamic::String(s.into())
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn create_string(&self, s: String) -> Self {
        Dynamic::String(s)
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(child) = m.remove(key) {
                    m.insert(key.to_string(), f(child));
                    Dynamic::Map(m)
                } else {
                    Dynamic::Map(m)
                }
            }
            other => other,
        }
    }

    pub fn update_map_values<F: FnMut(MapEntry) -> MapEntry>(self, mut f: F) -> Self {
        match self {
            Dynamic::Map(m) => {
                let mut new_m = BTreeMap::new();
                for (k, v) in m {
                    let entry = f(MapEntry {
                        key: Dynamic::String(k),
                        value: v,
                    });
                    let key_str = match entry.key {
                        Dynamic::String(s) => s,
                        other => format!("{other:?}"),
                    };
                    new_m.insert(key_str, entry.value);
                }
                Dynamic::Map(new_m)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapEntry {
    pub key: Dynamic,
    pub value: Dynamic,
}

impl MapEntry {
    pub fn map_first<F: FnOnce(Dynamic) -> Dynamic>(mut self, f: F) -> Self {
        self.key = f(self.key);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct CriteriaRenameFix {
    pub name: String,
    pub advancement_id: String,
    pub conversions: Box<dyn Fn(String) -> String>,
}

impl CriteriaRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.CriteriaRenameFix#<init>
    pub fn new(
        _schema: Schema,
        name: String,
        advancement_id: String,
        conversions: Box<dyn Fn(String) -> String>,
    ) -> Self {
        Self {
            name,
            advancement_id,
            conversions,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.CriteriaRenameFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + '_ {
        move |data| self.fix_advancements(data)
    }

    // JAVA: net.minecraft.util.datafix.fixes.CriteriaRenameFix#fixAdvancements
    fn fix_advancements(&self, data: Dynamic) -> Dynamic {
        data.update(&self.advancement_id, |adv| {
            adv.update("criteria", |criteria| {
                criteria.update_map_values(|entry| {
                    entry.map_first(|key_dyn| {
                        if let Some(s) = key_dyn.as_string() {
                            let converted = (self.conversions)(s);
                            key_dyn.create_string(converted)
                        } else {
                            key_dyn
                        }
                    })
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn get_path<'a>(d: &'a Dynamic, path: &[&str]) -> Option<&'a Dynamic> {
        let mut cur = d;
        for k in path {
            match cur {
                Dynamic::Map(m) => cur = m.get(*k)?,
                _ => return None,
            }
        }
        Some(cur)
    }

    #[test]
    fn renames_criteria_keys() {
        let fix = CriteriaRenameFix::new(
            Schema,
            "test".into(),
            "adv".into(),
            Box::new(|s| if s == "old" { "new".into() } else { s }),
        );
        let input = map(vec![(
            "adv",
            map(vec![(
                "criteria",
                map(vec![("old", Dynamic::Int(1)), ("keep", Dynamic::Int(2))]),
            )]),
        )]);
        let out = fix.make_rule()(input);
        let criteria = match get_path(&out, &["adv", "criteria"]) {
            Some(Dynamic::Map(m)) => m,
            _ => panic!("missing criteria"),
        };
        assert_eq!(criteria.get("new"), Some(&Dynamic::Int(1)));
        assert!(!criteria.contains_key("old"));
        assert_eq!(criteria.get("keep"), Some(&Dynamic::Int(2)));
    }

    #[test]
    fn missing_advancement_is_noop() {
        let fix = CriteriaRenameFix::new(Schema, "t".into(), "adv".into(), Box::new(|s| s + "_x"));
        let input = map(vec![("other", Dynamic::Bool(true))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_criteria_is_noop() {
        let fix = CriteriaRenameFix::new(
            Schema,
            "t".into(),
            "adv".into(),
            Box::new(|s| s + "_x"),
        );
        let input = map(vec![("adv", map(vec![("display", Dynamic::string("d"))]))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn empty_criteria_map_unchanged() {
        let fix = CriteriaRenameFix::new(
            Schema,
            "t".into(),
            "adv".into(),
            Box::new(|s| s.to_uppercase()),
        );
        let input = map(vec![("adv", map(vec![("criteria", Dynamic::empty_map())]))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_map_root_unchanged() {
        let fix = CriteriaRenameFix::new(Schema, "t".into(), "adv".into(), Box::new(|s| s));
        let input = Dynamic::Long(-123);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }
}
