use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

/// Optional dynamic wrapper mirroring DFU's `OptionalDynamic` behavior used here.
#[derive(Clone, Debug)]
pub struct OptionalDynamic {
    value: Option<Dynamic>,
}

impl OptionalDynamic {
    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            Some(Dynamic::Int(v)) => *v,
            Some(Dynamic::Long(v)) => *v as i32,
            _ => default,
        }
    }
}

impl Dynamic {
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(map) => OptionalDynamic {
                value: map.get(key).cloned(),
            },
            _ => OptionalDynamic { value: None },
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(map) => {
                let mut new_map = map.clone();
                new_map.remove(key);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }
}

/// Minimal DFU type/reference stubs for self-contained compilation.
#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Type;

#[derive(Clone, Debug, Default)]
pub struct TypeChoiceTypes;

impl TypeChoiceTypes {
    pub fn get(&self, _name: &str) -> Type {
        Type
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChoiceType;

impl ChoiceType {
    pub fn types(&self) -> TypeChoiceTypes {
        TypeChoiceTypes
    }
}

#[derive(Clone, Debug, Default)]
pub struct SchemaTypeFinder;

impl SchemaTypeFinder {
    pub fn find_choice_type(&self, _reference: References) -> ChoiceType {
        ChoiceType
    }
}

impl Schema {
    pub fn find_choice_type(&self, _reference: References) -> ChoiceType {
        ChoiceType
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum References {
    #[default]
    Entity,
}

#[derive(Clone, Debug, Default)]
pub struct RemainderFinder;

pub mod dsl {
    use super::RemainderFinder;
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

/// Minimal Typed stub carrying a dynamic remainder payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn get_or_create(&self, _finder: RemainderFinder) -> Dynamic {
        self.remainder.clone()
    }

    pub fn remainder(&self) -> &Dynamic {
        &self.remainder
    }
}

/// Minimal Pair stub mirroring `com.mojang.datafixers.util.Pair`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pair<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> Pair<A, B> {
    pub fn of(first: A, second: B) -> Self {
        Self { first, second }
    }
}

/// Minimal Util stub mirroring `net.minecraft.util.Util#writeAndReadTypedOrThrow`.
pub struct Util;

impl Util {
    pub fn write_and_read_typed_or_throw<F>(typed: Typed, _target_type: Type, fixer: F) -> Typed
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed::new(fixer(typed.remainder))
    }
}

/// Minimal base stub mirroring `EntityRenameFix`.
pub struct EntityRenameFix {
    _name: String,
    output_schema: Schema,
    _changes_type: bool,
}

impl EntityRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityRenameFix#<init>
    pub fn new(name: String, output_schema: Schema, changes_type: bool) -> Self {
        Self {
            _name: name,
            output_schema,
            _changes_type: changes_type,
        }
    }

    pub fn get_output_schema(&self) -> &Schema {
        &self.output_schema
    }
}

/// Port of `net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix`.
pub struct EntityMinecartIdentifiersFix {
    base: EntityRenameFix,
}

impl EntityMinecartIdentifiersFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            base: EntityRenameFix::new(
                "EntityMinecartIdentifiersFix".to_string(),
                schema,
                true,
            ),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix#fix
    pub fn fix(&self, name: String, typed: Typed) -> Pair<String, Typed> {
        if name != "Minecart" {
            return Pair::of(name, typed);
        }

        let type_id = typed.get_or_create(dsl::remainder_finder()).get("Type").as_int(0);

        let new_name = match type_id {
            1 => "MinecartChest",
            2 => "MinecartFurnace",
            _ => "MinecartRideable",
        };

        let target_type = self
            .base
            .get_output_schema()
            .find_choice_type(References::Entity)
            .types()
            .get(new_name);

        let fixed_typed = Util::write_and_read_typed_or_throw(typed, target_type, |d| d.remove("Type"));
        Pair::of(new_name.to_string(), fixed_typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minecart_typed(type_id: i32, extra: Option<(&str, Dynamic)>) -> Typed {
        let mut map = BTreeMap::new();
        map.insert("Type".to_string(), Dynamic::Int(type_id));
        if let Some((k, v)) = extra {
            map.insert(k.to_string(), v);
        }
        Typed::new(Dynamic::Map(map))
    }

    #[test]
    fn non_minecart_is_unchanged() {
        let fix = EntityMinecartIdentifiersFix::new(Schema);
        let typed = minecart_typed(1, None);
        let result = fix.fix("Zombie".to_string(), typed.clone());
        assert_eq!(result.first, "Zombie");
        assert_eq!(result.second, typed);
    }

    #[test]
    fn minecart_type_1_becomes_chest_and_removes_type() {
        let fix = EntityMinecartIdentifiersFix::new(Schema);
        let typed = minecart_typed(1, Some(("id", Dynamic::String("Minecart".to_string()))));
        let result = fix.fix("Minecart".to_string(), typed);
        assert_eq!(result.first, "MinecartChest");
        let rem = result.second.remainder();
        assert_eq!(rem.get("Type").as_int(-1), -1);
        assert_eq!(rem.get("id").as_int(-1), -1); // not int, default
        if let Dynamic::Map(m) = rem {
            assert!(matches!(m.get("id"), Some(Dynamic::String(s)) if s == "Minecart"));
            assert!(!m.contains_key("Type"));
        } else {
            panic!("expected map remainder");
        }
    }

    #[test]
    fn minecart_type_2_becomes_furnace() {
        let fix = EntityMinecartIdentifiersFix::new(Schema);
        let result = fix.fix("Minecart".to_string(), minecart_typed(2, None));
        assert_eq!(result.first, "MinecartFurnace");
    }

    #[test]
    fn minecart_default_becomes_rideable() {
        let fix = EntityMinecartIdentifiersFix::new(Schema);
        let result0 = fix.fix("Minecart".to_string(), minecart_typed(0, None));
        assert_eq!(result0.first, "MinecartRideable");
        let result99 = fix.fix("Minecart".to_string(), minecart_typed(99, None));
        assert_eq!(result99.first, "MinecartRideable");
    }

    #[test]
    fn minecart_missing_type_defaults_to_rideable() {
        let fix = EntityMinecartIdentifiersFix::new(Schema);
        let typed = Typed::new(Dynamic::Map(BTreeMap::new()));
        let result = fix.fix("Minecart".to_string(), typed);
        assert_eq!(result.first, "MinecartRideable");
    }
}
