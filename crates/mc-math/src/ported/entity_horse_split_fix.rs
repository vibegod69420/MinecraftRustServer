use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Int(i32),
    Long(i64),
    Short(i16),
    Byte(i8),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    Map(BTreeMap<String, DynValue>),
    List(Vec<DynValue>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#new
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => Dynamic::new(m.get(key).cloned().unwrap_or(DynValue::Null)),
            _ => Dynamic::new(DynValue::Null),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asInt
    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            DynValue::Int(v) => *v,
            DynValue::Long(v) => *v as i32,
            DynValue::Short(v) => i32::from(*v),
            DynValue::Byte(v) => i32::from(*v),
            DynValue::Float(v) => *v as i32,
            DynValue::Double(v) => *v as i32,
            _ => default,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#remove
    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut new_map = m.clone();
                new_map.remove(key);
                Dynamic::new(DynValue::Map(new_map))
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#get (remainder)
    pub fn get_remainder(&self) -> Dynamic {
        self.remainder.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    // JAVA: com.mojang.datafixers.util.Pair#of
    pub fn of(first: A, second: B) -> Self {
        Pair(first, second)
    }
}

// JAVA: net.minecraft.util.Util#writeAndReadTypedOrThrow
fn write_and_read_typed_or_throw<F: FnOnce(Dynamic) -> Dynamic>(
    typed: Typed,
    _output_type: (),
    f: F,
) -> Typed {
    Typed {
        remainder: f(typed.remainder),
    }
}

pub struct EntityHorseSplitFix;

impl EntityHorseSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityHorseSplitFix#<init>
    pub fn new(_schema: (), _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityHorseSplitFix#fix
    pub fn fix(&self, name: &str, typed: Typed) -> Pair<String, Typed> {
        if name == "EntityHorse" {
            let dynamic = typed.get_remainder();
            let type_id = dynamic.get("Type").as_int(0);

            let new_name = match type_id {
                1 => "Donkey",
                2 => "Mule",
                3 => "ZombieHorse",
                4 => "SkeletonHorse",
                _ => "Horse",
            };

            // this.getOutputSchema().findChoiceType(References.ENTITY).types().get(new_name)
            let output_type = ();
            let new_typed =
                write_and_read_typed_or_throw(typed, output_type, |d| d.remove("Type"));
            Pair::of(new_name.to_string(), new_typed)
        } else {
            Pair::of(name.to_string(), typed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn horse_dynamic(type_id: i32, extra: &[(&str, DynValue)]) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("Type".to_string(), DynValue::Int(type_id));
        for (k, v) in extra {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic::new(DynValue::Map(m))
    }

    #[test]
    fn splits_horse_types() {
        let fix = EntityHorseSplitFix::new((), true);
        let cases = [
            (0, "Horse"),
            (1, "Donkey"),
            (2, "Mule"),
            (3, "ZombieHorse"),
            (4, "SkeletonHorse"),
            (99, "Horse"),
        ];
        for (type_id, expected) in cases {
            let typed = Typed {
                remainder: horse_dynamic(type_id, &[]),
            };
            let Pair(name, out) = fix.fix("EntityHorse", typed);
            assert_eq!(name, expected);
            assert_eq!(out.remainder.get("Type").value, DynValue::Null);
        }
    }

    #[test]
    fn non_horse_passthrough() {
        let fix = EntityHorseSplitFix::new((), false);
        let mut m = BTreeMap::new();
        m.insert("Type".to_string(), DynValue::Int(1));
        let original = DynValue::Map(m);
        let typed = Typed {
            remainder: Dynamic::new(original.clone()),
        };
        let Pair(name, out) = fix.fix("Zombie", typed);
        assert_eq!(name, "Zombie");
        assert_eq!(out.remainder.value, original);
    }

    #[test]
    fn missing_type_defaults_to_horse_and_preserves_fields() {
        let fix = EntityHorseSplitFix::new((), true);
        let mut m = BTreeMap::new();
        m.insert("CustomName".to_string(), DynValue::String("Bob".to_string()));
        let typed = Typed {
            remainder: Dynamic::new(DynValue::Map(m)),
        };
        let Pair(name, out) = fix.fix("EntityHorse", typed);
        assert_eq!(name, "Horse");
        assert_eq!(
            out.remainder.get("CustomName").value,
            DynValue::String("Bob".to_string())
        );
    }

    #[test]
    fn preserves_other_fields_and_removes_type() {
        let fix = EntityHorseSplitFix::new((), true);
        let typed = Typed {
            remainder: horse_dynamic(
                2,
                &[
                    ("Health", DynValue::Float(10.0)),
                    ("Tame", DynValue::Bool(true)),
                ],
            ),
        };
        let Pair(name, out) = fix.fix("EntityHorse", typed);
        assert_eq!(name, "Mule");
        assert_eq!(out.remainder.get("Health").value, DynValue::Float(10.0));
        assert_eq!(out.remainder.get("Tame").value, DynValue::Bool(true));
        assert_eq!(out.remainder.get("Type").value, DynValue::Null);
    }

    #[test]
    fn negative_and_extreme_type_values_default_correctly() {
        let fix = EntityHorseSplitFix::new((), true);
        for type_id in [-1, i32::MIN, i32::MAX] {
            let typed = Typed {
                remainder: horse_dynamic(type_id, &[]),
            };
            let Pair(name, _) = fix.fix("EntityHorse", typed);
            assert_eq!(name, "Horse", "type_id {type_id} should default to Horse");
        }
    }
}
