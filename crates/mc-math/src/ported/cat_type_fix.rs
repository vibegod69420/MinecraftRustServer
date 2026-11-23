use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    FloatBits(u32),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_int(&self, i: i32) -> Dynamic {
        Dynamic::Int(i)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn as_int(&self, default: i32) -> i32 {
        match &self.0 {
            Some(Dynamic::Int(i)) => *i,
            _ => default,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn update_remainder<F: FnOnce(Dynamic) -> Dynamic>(self, f: F) -> Self {
        Typed {
            remainder: f(self.remainder),
        }
    }
}

pub struct CatTypeFix {
    _schema: Schema,
    _changes_type: bool,
}

impl CatTypeFix {
    // JAVA: net.minecraft.util.datafix.fixes.CatTypeFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.CatTypeFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        if tag.get("CatType").as_int(0) == 9 {
            let new_val = tag.create_int(10);
            tag.set("CatType", new_val)
        } else {
            tag
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.CatTypeFix#fix
    pub fn fix(&self, input: Typed) -> Typed {
        input.update_remainder(|d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat_tag(cat_type: i32) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("CatType".to_string(), Dynamic::Int(cat_type));
        Dynamic::Map(m)
    }

    #[test]
    fn cat_type_9_becomes_10() {
        let fix = CatTypeFix::new(Schema, false);
        let out = fix.fix_tag(cat_tag(9));
        assert_eq!(out.get("CatType").as_int(-1), 10);
    }

    #[test]
    fn cat_type_0_unchanged() {
        let fix = CatTypeFix::new(Schema, false);
        let input = cat_tag(0);
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_cat_type_defaults_to_0() {
        let fix = CatTypeFix::new(Schema, false);
        let input = Dynamic::empty_map();
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn negative_and_boundary_cat_type_unchanged() {
        let fix = CatTypeFix::new(Schema, false);
        for val in [i32::MIN, -1, i32::MAX] {
            let input = cat_tag(val);
            let out = fix.fix_tag(input.clone());
            assert_eq!(out.get("CatType").as_int(0), val);
        }
    }

    #[test]
    fn fix_wraps_fix_tag() {
        let fix = CatTypeFix::new(Schema, false);
        let input = Typed {
            remainder: cat_tag(9),
        };
        let out = fix.fix(input);
        assert_eq!(out.remainder.get("CatType").as_int(-1), 10);
    }

    #[test]
    fn float_bits_preserved_on_fix() {
        let fix = CatTypeFix::new(Schema, false);
        let mut m = BTreeMap::new();
        m.insert("CatType".to_string(), Dynamic::Int(9));
        m.insert("health".to_string(), Dynamic::FloatBits(f32::NAN.to_bits()));
        let out = fix.fix_tag(Dynamic::Map(m));
        assert_eq!(out.get("CatType").as_int(-1), 10);
        if let Dynamic::Map(ref map) = out {
            match map.get("health") {
                Some(Dynamic::FloatBits(b)) => assert!(f32::from_bits(*b).is_nan()),
                other => panic!("{:?}", other),
            }
        } else {
            panic!("expected map");
        }
    }
}
