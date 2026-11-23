use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    List(Vec<Dynamic>),
}

#[derive(Clone, Debug)]
pub struct DynamicOpt(Option<Dynamic>);

impl DynamicOpt {
    pub fn or_else_empty_map(self) -> Dynamic {
        self.0.unwrap_or_else(Dynamic::empty_map)
    }

    pub fn as_string(&self, default: &str) -> String {
        self.0
            .as_ref()
            .map(|d| d.as_string(default))
            .unwrap_or_else(|| default.to_string())
    }

    pub fn as_int(&self, default: i32) -> i32 {
        self.0.as_ref().map(|d| d.as_int(default)).unwrap_or(default)
    }
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> DynamicOpt {
        match self {
            Dynamic::Map(m) => DynamicOpt(m.get(key).cloned()),
            _ => DynamicOpt(None),
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            Dynamic::Float(v) => *v as i32,
            Dynamic::Double(v) => *v as i32,
            _ => default,
        }
    }

    pub fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
fn ensure_namespaced(s: String) -> String {
    if s.contains(':') {
        s
    } else {
        format!("minecraft:{s}")
    }
}

pub struct EmptyItemInVillagerTradeFix {
    _schema: Schema,
}

impl EmptyItemInVillagerTradeFix {
    // JAVA: net.minecraft.util.datafix.fixes.EmptyItemInVillagerTradeFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EmptyItemInVillagerTradeFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic {
        |trade: Dynamic| {
            let buy_b = trade.get("buyB").or_else_empty_map();
            let id = ensure_namespaced(buy_b.get("id").as_string("minecraft:air"));
            let count = buy_b.get("count").as_int(0);
            if id != "minecraft:air" && count != 0 {
                trade
            } else {
                trade.remove("buyB")
            }
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
    fn removes_empty_air_buy_b() {
        let fix = EmptyItemInVillagerTradeFix::new(Schema);
        let input = map(vec![
            ("buyB", map(vec![("id", Dynamic::String("minecraft:air".to_string())), ("count", Dynamic::Int(1))])),
            ("buyA", Dynamic::Int(1)),
        ]);
        let out = (fix.make_rule())(input);
        if let Dynamic::Map(m) = out {
            assert!(!m.contains_key("buyB"));
            assert!(m.contains_key("buyA"));
        } else {
            panic!("expected map");
        }
    }

    #[test]
    fn removes_missing_or_zero_count_buy_b() {
        let fix = EmptyItemInVillagerTradeFix::new(Schema);
        let input = map(vec![(
            "buyB",
            map(vec![("id", Dynamic::String("minecraft:emerald".to_string())), ("count", Dynamic::Int(0))]),
        )]);
        let out = (fix.make_rule())(input);
        if let Dynamic::Map(m) = out {
            assert!(!m.contains_key("buyB"));
        } else {
            panic!("expected map");
        }
    }

    #[test]
    fn keeps_valid_buy_b_and_namespaces_id() {
        let fix = EmptyItemInVillagerTradeFix::new(Schema);
        let input = map(vec![(
            "buyB",
            map(vec![("id", Dynamic::String("emerald".to_string())), ("count", Dynamic::Int(1))]),
        )]);
        let out = (fix.make_rule())(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn removes_when_buy_b_missing() {
        let fix = EmptyItemInVillagerTradeFix::new(Schema);
        let input = map(vec![("sell", Dynamic::Int(1))]);
        let out = (fix.make_rule())(input);
        if let Dynamic::Map(m) = out {
            assert!(!m.contains_key("buyB"));
            assert_eq!(m.get("sell"), Some(&Dynamic::Int(1)));
        } else {
            panic!("expected map");
        }
    }

    #[test]
    fn negative_count_is_treated_as_present_nonzero() {
        let fix = EmptyItemInVillagerTradeFix::new(Schema);
        let input = map(vec![(
            "buyB",
            map(vec![("id", Dynamic::String("minecraft:stone".to_string())), ("count", Dynamic::Int(-1))]),
        )]);
        let out = (fix.make_rule())(input.clone());
        // Java: count != 0 means -1 is kept
        assert_eq!(out, input);
    }
}
