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

    pub fn bool(value: bool) -> Self {
        Self {
            value: DynamicValue::Bool(value),
        }
    }

    pub fn null() -> Self {
        Self {
            value: DynamicValue::Null,
        }
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => m.get(key).cloned().unwrap_or_else(Dynamic::null),
            _ => Dynamic::null(),
        }
    }

    pub fn as_boolean(&self, default: bool) -> bool {
        match &self.value {
            DynamicValue::Bool(b) => *b,
            _ => default,
        }
    }
}

pub struct EntityElderGuardianSplitFix {
    _name: &'static str,
    _schema: Schema,
    _changes_type: bool,
}

impl EntityElderGuardianSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _name: "EntityElderGuardianSplitFix",
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix#getNewNameAndTag
    pub fn get_new_name_and_tag(&self, name: &str, tag: Dynamic) -> (String, Dynamic) {
        let new_name = if name == "Guardian" && tag.get("Elder").as_boolean(false) {
            "ElderGuardian".to_string()
        } else {
            name.to_string()
        };
        (new_name, tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn guardian_tag(elder: bool) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("Elder".to_string(), Dynamic::bool(elder));
        Dynamic::map(m)
    }

    #[test]
    fn guardian_elder_true_becomes_elder_guardian() {
        let fix = EntityElderGuardianSplitFix::new(Schema, true);
        let tag = guardian_tag(true);
        let (name, out_tag) = fix.get_new_name_and_tag("Guardian", tag.clone());
        assert_eq!(name, "ElderGuardian");
        assert_eq!(out_tag, tag);
    }

    #[test]
    fn guardian_elder_false_or_missing_stays_guardian() {
        let fix = EntityElderGuardianSplitFix::new(Schema, true);
        let (name_false, _) = fix.get_new_name_and_tag("Guardian", guardian_tag(false));
        assert_eq!(name_false, "Guardian");

        let (name_missing, _) =
            fix.get_new_name_and_tag("Guardian", Dynamic::map(BTreeMap::new()));
        assert_eq!(name_missing, "Guardian");
    }

    #[test]
    fn non_guardian_with_elder_true_unchanged() {
        let fix = EntityElderGuardianSplitFix::new(Schema, false);
        let tag = guardian_tag(true);
        let (name, out_tag) = fix.get_new_name_and_tag("Zombie", tag.clone());
        assert_eq!(name, "Zombie");
        assert_eq!(out_tag, tag);
    }

    #[test]
    fn elder_non_boolean_defaults_false() {
        let fix = EntityElderGuardianSplitFix::new(Schema, true);
        let mut m = BTreeMap::new();
        m.insert(
            "Elder".to_string(),
            Dynamic {
                value: DynamicValue::Int(-1),
            },
        );
        let (name_int, _) = fix.get_new_name_and_tag("Guardian", Dynamic::map(m));
        assert_eq!(name_int, "Guardian");

        let mut m2 = BTreeMap::new();
        m2.insert(
            "Elder".to_string(),
            Dynamic {
                value: DynamicValue::Float(f64::from_bits(0xBFF0_0000_0000_0000)), // -1.0
            },
        );
        let (name_float, _) = fix.get_new_name_and_tag("Guardian", Dynamic::map(m2));
        assert_eq!(name_float, "Guardian");
    }

    #[test]
    fn get_on_non_map_returns_null_defaulting_false() {
        let fix = EntityElderGuardianSplitFix::new(Schema, true);
        let tag = Dynamic {
            value: DynamicValue::String("not-a-map".to_string()),
        };
        let (name, out_tag) = fix.get_new_name_and_tag("Guardian", tag.clone());
        assert_eq!(name, "Guardian");
        assert_eq!(out_tag, tag);
        assert!(!Dynamic::null().as_boolean(false));
        assert!(Dynamic::null().as_boolean(true));
    }
}
