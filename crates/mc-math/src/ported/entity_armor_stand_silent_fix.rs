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
    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).unwrap_or(&DYNAMIC_NULL),
            _ => &DYNAMIC_NULL,
        }
    }

    pub fn as_boolean(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(b) => *b,
            _ => default,
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut next = m.clone();
                next.remove(key);
                Dynamic::Map(next)
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn update_remainder<F: FnOnce(Dynamic) -> Dynamic>(self, f: F) -> Typed {
        Typed {
            remainder: f(self.remainder),
        }
    }
}

pub struct EntityArmorStandSilentFix {
    #[allow(dead_code)]
    schema: Schema,
    #[allow(dead_code)]
    changes_type: bool,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    entity_name: String,
}

impl EntityArmorStandSilentFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityArmorStandSilentFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            schema,
            changes_type,
            name: "EntityArmorStandSilentFix".to_string(),
            entity_name: "ArmorStand".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityArmorStandSilentFix#fixTag
    pub fn fix_tag(&self, d: Dynamic) -> Dynamic {
        if d.get("Silent").as_boolean(false) && !d.get("Marker").as_boolean(false) {
            d.remove("Silent")
        } else {
            d
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityArmorStandSilentFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update_remainder(|d| self.fix_tag(d))
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
    fn removes_silent_when_silent_true_and_marker_false() {
        let fix = EntityArmorStandSilentFix::new(Schema, false);
        let input = map(vec![
            ("Silent", Dynamic::Bool(true)),
            ("Marker", Dynamic::Bool(false)),
            ("id", Dynamic::String("ArmorStand".to_string())),
        ]);
        let out = fix.fix_tag(input);
        assert_eq!(out.get("Silent"), &Dynamic::Null);
        assert_eq!(out.get("Marker"), &Dynamic::Bool(false));
        assert_eq!(out.get("id"), &Dynamic::String("ArmorStand".to_string()));
    }

    #[test]
    fn keeps_silent_when_marker_true() {
        let fix = EntityArmorStandSilentFix::new(Schema, false);
        let input = map(vec![
            ("Silent", Dynamic::Bool(true)),
            ("Marker", Dynamic::Bool(true)),
        ]);
        let out = fix.fix_tag(input);
        assert_eq!(out.get("Silent"), &Dynamic::Bool(true));
    }

    #[test]
    fn keeps_when_silent_false() {
        let fix = EntityArmorStandSilentFix::new(Schema, false);
        let input = map(vec![
            ("Silent", Dynamic::Bool(false)),
            ("Marker", Dynamic::Bool(false)),
        ]);
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn removes_silent_when_marker_missing() {
        let fix = EntityArmorStandSilentFix::new(Schema, false);
        let input = map(vec![("Silent", Dynamic::Bool(true))]);
        let out = fix.fix_tag(input);
        assert_eq!(out.get("Silent"), &Dynamic::Null);
    }

    #[test]
    fn fix_updates_typed_remainder() {
        let fix = EntityArmorStandSilentFix::new(Schema, false);
        let input = Typed {
            remainder: map(vec![("Silent", Dynamic::Bool(true))]),
        };
        let out = fix.fix(input);
        assert_eq!(out.remainder.get("Silent"), &Dynamic::Null);
    }
}
