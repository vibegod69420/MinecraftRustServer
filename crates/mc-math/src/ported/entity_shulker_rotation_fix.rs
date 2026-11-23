use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Double(f64),
    Float(f32),
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Bool(bool),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get_field(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asDouble
    pub fn as_double(&self, default: f64) -> f64 {
        match self {
            Dynamic::Double(v) => *v,
            Dynamic::Float(v) => f64::from(*v),
            Dynamic::Long(v) => *v as f64,
            Dynamic::Int(v) => f64::from(*v),
            Dynamic::Short(v) => f64::from(*v),
            Dynamic::Byte(v) => f64::from(*v),
            _ => default,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asList
    pub fn as_list_as_double(&self, default: f64) -> Vec<f64> {
        match self {
            Dynamic::List(list) => list.iter().map(|d| d.as_double(default)).collect(),
            _ => Vec::new(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    pub fn create_list(values: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(values)
    }

    // JAVA: com.mojang.serialization.Dynamic#createDouble
    pub fn create_double(v: f64) -> Dynamic {
        Dynamic::Double(v)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DslRemainderFinder;

// JAVA: com.mojang.datafixers.DSL#remainderFinder
pub fn dsl_remainder_finder() -> DslRemainderFinder {
    DslRemainderFinder
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update(self, _finder: DslRemainderFinder, f: impl FnOnce(Dynamic) -> Dynamic) -> Typed {
        Typed {
            value: f(self.value),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct References;

impl References {
    pub const ENTITY: &'static str = "entity";
}

pub struct EntityShulkerRotationFix {
    _schema: Schema,
}

impl EntityShulkerRotationFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerRotationFix#<init>
    pub fn new(schema: Schema) -> Self {
        // super(schema, false, "EntityShulkerRotationFix", References.ENTITY, "minecraft:shulker")
        let _ = References::ENTITY;
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerRotationFix#fixTag
    pub fn fix_tag(&self, d: Dynamic) -> Dynamic {
        let rotation: Vec<f64> = match d.get_field("Rotation") {
            Some(r) => r.as_list_as_double(180.0),
            None => Vec::new(),
        };
        if !rotation.is_empty() {
            let mut rotation = rotation;
            rotation[0] = rotation[0] - 180.0;
            let list = Dynamic::create_list(
                rotation.into_iter().map(Dynamic::create_double).collect(),
            );
            d.set("Rotation", list)
        } else {
            d
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerRotationFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(dsl_remainder_finder(), |d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn subtracts_180_from_first_rotation() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let d = make_map(vec![(
            "Rotation",
            Dynamic::List(vec![Dynamic::Double(180.0), Dynamic::Double(0.0)]),
        )]);
        let out = fix.fix_tag(d);
        match out {
            Dynamic::Map(m) => match m.get("Rotation") {
                Some(Dynamic::List(v)) => {
                    assert_eq!(v[0], Dynamic::Double(0.0));
                    assert_eq!(v[1], Dynamic::Double(0.0));
                }
                other => panic!("expected rotation list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn empty_rotation_is_unchanged() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let d = make_map(vec![("Rotation", Dynamic::List(vec![]))]);
        let out = fix.fix_tag(d.clone());
        assert_eq!(out, d);
    }

    #[test]
    fn missing_rotation_is_unchanged() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let d = make_map(vec![("id", Dynamic::String("minecraft:shulker".to_string()))]);
        let out = fix.fix_tag(d.clone());
        assert_eq!(out, d);
    }

    #[test]
    fn non_numeric_rotation_uses_default_180() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let d = make_map(vec![(
            "Rotation",
            Dynamic::List(vec![Dynamic::String("bad".to_string()), Dynamic::Double(45.0)]),
        )]);
        let out = fix.fix_tag(d);
        match out {
            Dynamic::Map(m) => match m.get("Rotation") {
                Some(Dynamic::List(v)) => {
                    // default 180.0 - 180.0 = 0.0
                    assert_eq!(v[0], Dynamic::Double(0.0));
                    assert_eq!(v[1], Dynamic::Double(45.0));
                }
                other => panic!("expected rotation list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn negative_rotation_and_float_bits_preserved() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let input = -90.0f64;
        let d = make_map(vec![("Rotation", Dynamic::List(vec![Dynamic::Double(input)]))]);
        let out = fix.fix_tag(d);
        match out {
            Dynamic::Map(m) => match m.get("Rotation") {
                Some(Dynamic::List(v)) => {
                    let expected = input - 180.0;
                    match &v[0] {
                        Dynamic::Double(got) => assert_eq!(got.to_bits(), expected.to_bits()),
                        other => panic!("expected double, got {:?}", other),
                    }
                }
                other => panic!("expected rotation list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn fix_updates_typed_value() {
        let fix = EntityShulkerRotationFix::new(Schema);
        let typed = Typed {
            value: make_map(vec![(
                "Rotation",
                Dynamic::List(vec![Dynamic::Double(0.0), Dynamic::Double(0.0)]),
            )]),
        };
        let out = fix.fix(typed);
        match out.value {
            Dynamic::Map(m) => match m.get("Rotation") {
                Some(Dynamic::List(v)) => assert_eq!(v[0], Dynamic::Double(-180.0)),
                other => panic!("expected rotation list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }
}
