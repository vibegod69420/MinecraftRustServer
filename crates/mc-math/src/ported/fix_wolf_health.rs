use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Dynamic> {
        match self {
            Dynamic::Map(m) => m.get_mut(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Dynamic::Int(n) => Some(*n as f64),
            Dynamic::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Dynamic::Int(n) => Some(*n as f32),
            Dynamic::Float(f) => Some(*f as f32),
            _ => None,
        }
    }
}

fn ensure_namespaced(s: &str) -> String {
    if s.contains(':') {
        s.to_string()
    } else {
        format!("minecraft:{}", s)
    }
}

pub fn ensure_namespaced_name(s: &str) -> String {
    ensure_namespaced(s)
}

pub fn fix_wolf_health(mut value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.FixWolfHealth#fix
    let mut changed = false;

    if let Dynamic::Map(obj) = &mut value {
        if let Some(Dynamic::List(attributes)) = obj.get_mut("Attributes") {
            for attribute in attributes.iter_mut() {
                if let Dynamic::Map(attr) = attribute {
                    let name = attr
                        .get("Name")
                        .and_then(Dynamic::as_str)
                        .unwrap_or("");
                    if ensure_namespaced(name) == "minecraft:generic.max_health" {
                        if let Some(base) = attr.get_mut("Base") {
                            if base.as_f64().unwrap_or(0.0) == 20.0 {
                                changed = true;
                                *base = Dynamic::Float(40.0);
                            }
                        }
                    }
                }
            }
        }
    }

    if changed {
        if let Dynamic::Map(obj) = &mut value {
            if let Some(health) = obj.get_mut("Health") {
                let f = health.as_f32().unwrap_or(0.0);
                *health = Dynamic::Float((f * 2.0) as f64);
            }
        }
    }

    value
}

pub fn fix(value: Dynamic) -> Dynamic {
    fix_wolf_health(value)
}

pub fn apply(value: Dynamic) -> Dynamic {
    fix_wolf_health(value)
}

pub fn apply_fix(value: Dynamic) -> Dynamic {
    fix_wolf_health(value)
}

pub fn fix_in_place(value: &mut Dynamic) {
    let old = std::mem::replace(value, Dynamic::Null);
    *value = fix_wolf_health(old);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn doubles_health_when_max_health_was_20() {
        let input = map(vec![
            (
                "Attributes",
                Dynamic::List(vec![map(vec![
                    ("Name", Dynamic::Str("generic.max_health".to_string())),
                    ("Base", Dynamic::Float(20.0)),
                ])]),
            ),
            ("Health", Dynamic::Float(10.0)),
        ]);
        let out = fix_wolf_health(input);
        match out {
            Dynamic::Map(m) => {
                assert_eq!(m.get("Health"), Some(&Dynamic::Float(20.0)));
                if let Some(Dynamic::List(attrs)) = m.get("Attributes") {
                    assert_eq!(attrs[0].get("Base"), Some(&Dynamic::Float(40.0)));
                } else {
                    panic!("attributes missing");
                }
            }
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn leaves_other_max_health_unchanged() {
        let input = map(vec![
            (
                "Attributes",
                Dynamic::List(vec![map(vec![
                    ("Name", Dynamic::Str("minecraft:generic.max_health".to_string())),
                    ("Base", Dynamic::Float(30.0)),
                ])]),
            ),
            ("Health", Dynamic::Float(10.0)),
        ]);
        let out = fix_wolf_health(input);
        match out {
            Dynamic::Map(m) => {
                assert_eq!(m.get("Health"), Some(&Dynamic::Float(10.0)));
            }
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn no_attributes_does_not_change_health() {
        let input = map(vec![("Health", Dynamic::Float(7.0))]);
        let out = fix(input);
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("Health"), Some(&Dynamic::Float(7.0))),
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn namespaced_name_is_handled() {
        let input = map(vec![
            (
                "Attributes",
                Dynamic::List(vec![map(vec![
                    ("Name", Dynamic::Str("minecraft:generic.max_health".to_string())),
                    ("Base", Dynamic::Int(20)),
                ])]),
            ),
            ("Health", Dynamic::Int(5)),
        ]);
        let out = apply_fix(input);
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("Health"), Some(&Dynamic::Float(10.0))),
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn in_place_update_works() {
        let mut value = map(vec![
            (
                "Attributes",
                Dynamic::List(vec![map(vec![
                    ("Name", Dynamic::Str("generic.max_health".to_string())),
                    ("Base", Dynamic::Float(20.0)),
                ])]),
            ),
            ("Health", Dynamic::Float(3.0)),
        ]);
        fix_in_place(&mut value);
        match value {
            Dynamic::Map(m) => assert_eq!(m.get("Health"), Some(&Dynamic::Float(6.0))),
            _ => panic!("expected map"),
        }
    }
}
