use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    Bool(bool),
    Byte(i8),
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Compound(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_bool(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(b) => *b,
            Dynamic::Byte(b) => *b != 0,
            _ => default,
        }
    }

    pub fn as_byte(&self, default: i8) -> i8 {
        match self {
            Dynamic::Byte(b) => *b,
            Dynamic::Int(i) => *i as i8,
            Dynamic::Long(l) => *l as i8,
            _ => default,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_byte(&self, v: i8) -> Dynamic {
        Dynamic::Byte(v)
    }

    pub fn create_string(&self, v: &str) -> Dynamic {
        Dynamic::String(v.to_string())
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_string(), value);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }
}

pub struct JigsawPropertiesFix;

impl JigsawPropertiesFix {
    // JAVA: net.minecraft.util.datafix.fixes.JigsawPropertiesFix#fixTag
    pub fn fix_tag(d: &Dynamic) -> Dynamic {
        let attachement_type = d.get("attachement_type").as_string("minecraft:empty");
        let target_pool = d.get("target_pool").as_string("minecraft:empty");
        d.set("name", d.create_string(&attachement_type))
            .set("target", d.create_string(&attachement_type))
            .remove("attachement_type")
            .set("pool", d.create_string(&target_pool))
            .remove("target_pool")
    }

    // JAVA: net.minecraft.util.datafix.fixes.JigsawPropertiesFix#fix
    pub fn fix(d: &Dynamic) -> Dynamic {
        Self::fix_tag(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn compound(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Compound(m)
    }

    #[test]
    fn remaps_attachement_and_target_pool() {
        let d = compound(vec![
            ("attachement_type", Dynamic::String("minecraft:foo".to_string())),
            ("target_pool", Dynamic::String("minecraft:bar".to_string())),
        ]);
        let fixed = JigsawPropertiesFix::fix_tag(&d);
        assert_eq!(fixed.get("name"), Dynamic::String("minecraft:foo".to_string()));
        assert_eq!(fixed.get("target"), Dynamic::String("minecraft:foo".to_string()));
        assert_eq!(fixed.get("pool"), Dynamic::String("minecraft:bar".to_string()));
        assert_eq!(fixed.get("attachement_type"), Dynamic::Missing);
        assert_eq!(fixed.get("target_pool"), Dynamic::Missing);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let d = compound(vec![("joint", Dynamic::String("aligned".to_string()))]);
        let fixed = JigsawPropertiesFix::fix_tag(&d);
        assert_eq!(fixed.get("name"), Dynamic::String("minecraft:empty".to_string()));
        assert_eq!(fixed.get("target"), Dynamic::String("minecraft:empty".to_string()));
        assert_eq!(fixed.get("pool"), Dynamic::String("minecraft:empty".to_string()));
        assert_eq!(fixed.get("joint"), Dynamic::String("aligned".to_string()));
    }

    #[test]
    fn overwrites_existing_name_target_pool() {
        let d = compound(vec![
            ("attachement_type", Dynamic::String("minecraft:new".to_string())),
            ("target_pool", Dynamic::String("minecraft:pool".to_string())),
            ("name", Dynamic::String("old".to_string())),
            ("target", Dynamic::String("old".to_string())),
            ("pool", Dynamic::String("old".to_string())),
        ]);
        let fixed = JigsawPropertiesFix::fix_tag(&d);
        assert_eq!(fixed.get("name"), Dynamic::String("minecraft:new".to_string()));
        assert_eq!(fixed.get("target"), Dynamic::String("minecraft:new".to_string()));
        assert_eq!(fixed.get("pool"), Dynamic::String("minecraft:pool".to_string()));
    }

    #[test]
    fn preserves_negative_and_float_bit_fields() {
        let fbits = f64::to_bits(f64::NAN) as i64;
        let d = compound(vec![
            ("attachement_type", Dynamic::String("minecraft:a".to_string())),
            ("target_pool", Dynamic::String("minecraft:b".to_string())),
            ("x", Dynamic::Int(i32::MIN)),
            ("y", Dynamic::Int(-7)),
            ("floatBits", Dynamic::Long(fbits)),
        ]);
        let fixed = JigsawPropertiesFix::fix(&d);
        assert_eq!(fixed.get("x"), Dynamic::Int(i32::MIN));
        assert_eq!(fixed.get("y"), Dynamic::Int(-7));
        assert_eq!(fixed.get("floatBits"), Dynamic::Long(fbits));
    }
}
