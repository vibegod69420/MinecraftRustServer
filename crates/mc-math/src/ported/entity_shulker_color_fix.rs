use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number(pub f64);

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
}

impl Dynamic {
    pub fn compound(entries: Vec<(String, Dynamic)>) -> Self {
        Dynamic::Compound(entries.into_iter().collect())
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Compound(map) => OptionalDynamic {
                inner: map.get(key).cloned(),
            },
            _ => OptionalDynamic { inner: None },
        }
    }

    pub fn get_field(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Compound(map) => map.get(key),
            _ => None,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Compound(map) => {
                let mut new_map = map.clone();
                new_map.insert(key.to_string(), value);
                Dynamic::Compound(new_map)
            }
            _ => self.clone(),
        }
    }

    pub fn create_byte(&self, v: i8) -> Dynamic {
        Dynamic::Byte(v)
    }

    pub fn as_number(d: &Dynamic) -> Option<Number> {
        match d {
            Dynamic::Byte(v) => Some(Number(f64::from(*v))),
            Dynamic::Short(v) => Some(Number(f64::from(*v))),
            Dynamic::Int(v) => Some(Number(f64::from(*v))),
            Dynamic::Long(v) => Some(Number(*v as f64)),
            Dynamic::Float(v) => Some(Number(f64::from(*v))),
            Dynamic::Double(v) => Some(Number(*v)),
            _ => None,
        }
    }
}

pub struct OptionalDynamic {
    inner: Option<Dynamic>,
}

impl OptionalDynamic {
    pub fn map<T, F>(self, f: F) -> DataResult<T>
    where
        F: FnOnce(&Dynamic) -> Option<T>,
    {
        DataResult {
            inner: self.inner.as_ref().and_then(f),
        }
    }

    pub fn result(self) -> Option<Dynamic> {
        self.inner
    }
}

pub struct DataResult<T> {
    inner: Option<T>,
}

impl<T> DataResult<T> {
    pub fn result(self) -> Option<T> {
        self.inner
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typed {
    pub data: Dynamic,
}

impl Typed {
    pub fn update<F>(&self, _finder: (), f: F) -> Typed
    where
        F: FnOnce(&Dynamic) -> Dynamic,
    {
        Typed {
            data: f(&self.data),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityShulkerColorFix {
    pub name: &'static str,
    pub schema: Schema,
    pub changes_type: bool,
    pub entity_name: &'static str,
}

impl EntityShulkerColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerColorFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            name: "EntityShulkerColorFix",
            schema,
            changes_type,
            entity_name: "minecraft:shulker",
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerColorFix#fixTag
    pub fn fix_tag(&self, dynamic: &Dynamic) -> Dynamic {
        if dynamic
            .get("Color")
            .map(Dynamic::as_number)
            .result()
            .is_none()
        {
            dynamic.set("Color", dynamic.create_byte(10))
        } else {
            dynamic.clone()
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityShulkerColorFix#fix
    pub fn fix(&self, typed: &Typed) -> Typed {
        typed.update((), |d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_tag_adds_default_color_when_missing() {
        let fix = EntityShulkerColorFix::new(Schema, false);
        let input = Dynamic::compound(vec![("id".to_string(), Dynamic::String("minecraft:shulker".to_string()))]);
        let out = fix.fix_tag(&input);
        assert_eq!(out.get_field("Color"), Some(&Dynamic::Byte(10)));
        assert_eq!(
            out.get_field("id"),
            Some(&Dynamic::String("minecraft:shulker".to_string()))
        );
    }

    #[test]
    fn fix_tag_preserves_existing_numeric_color() {
        let fix = EntityShulkerColorFix::new(Schema, false);
        let input = Dynamic::compound(vec![("Color".to_string(), Dynamic::Byte(5))]);
        let out = fix.fix_tag(&input);
        assert_eq!(out, input);

        let input_int = Dynamic::compound(vec![("Color".to_string(), Dynamic::Int(3))]);
        let out_int = fix.fix_tag(&input_int);
        assert_eq!(out_int.get_field("Color"), Some(&Dynamic::Int(3)));
    }

    #[test]
    fn fix_tag_replaces_non_numeric_color() {
        let fix = EntityShulkerColorFix::new(Schema, false);
        let input = Dynamic::compound(vec![("Color".to_string(), Dynamic::String("red".to_string()))]);
        let out = fix.fix_tag(&input);
        assert_eq!(out.get_field("Color"), Some(&Dynamic::Byte(10)));
    }

    #[test]
    fn fix_updates_typed_data() {
        let fix = EntityShulkerColorFix::new(Schema, false);
        let typed = Typed {
            data: Dynamic::compound(vec![]),
        };
        let out = fix.fix(&typed);
        assert_eq!(out.data.get_field("Color"), Some(&Dynamic::Byte(10)));
    }

    #[test]
    fn constructor_sets_metadata() {
        let fix = EntityShulkerColorFix::new(Schema, true);
        assert_eq!(fix.name, "EntityShulkerColorFix");
        assert_eq!(fix.entity_name, "minecraft:shulker");
        assert!(fix.changes_type);
    }
}
