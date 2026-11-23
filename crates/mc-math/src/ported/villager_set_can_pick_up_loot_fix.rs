use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_boolean(&self, value: bool) -> Dynamic {
        Dynamic::Bool(value)
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn map(entries: Vec<(String, Dynamic)>) -> Self {
        Dynamic::Map(entries.into_iter().collect())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum References {
    Entity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedEntityFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub reference: References,
    pub entity_name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn update<F>(self, _finder: RemainderFinder, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed {
            remainder: f(self.remainder),
        }
    }
}

pub const CAN_PICK_UP_LOOT: &str = "CanPickUpLoot";

#[derive(Clone, Debug, PartialEq)]
pub struct VillagerSetCanPickUpLootFix {
    pub base: NamedEntityFix,
}

impl VillagerSetCanPickUpLootFix {
    // JAVA: net.minecraft.util.datafix.fixes.VillagerSetCanPickUpLootFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            base: NamedEntityFix {
                schema,
                changes_type: true,
                name: "Villager CanPickUpLoot default value".to_string(),
                reference: References::Entity,
                entity_name: "Villager".to_string(),
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerSetCanPickUpLootFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), Self::fix_value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerSetCanPickUpLootFix#fixValue
    fn fix_value(d: Dynamic) -> Dynamic {
        let value = d.create_boolean(true);
        d.set(CAN_PICK_UP_LOOT, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_can_pick_up_loot_true() {
        let fix = VillagerSetCanPickUpLootFix::new(Schema);
        let input = Typed::new(Dynamic::map(vec![]));
        let out = fix.fix(input);
        assert_eq!(out.remainder.get(CAN_PICK_UP_LOOT), Dynamic::Bool(true));
    }

    #[test]
    fn overwrites_existing_false() {
        let fix = VillagerSetCanPickUpLootFix::new(Schema);
        let input = Typed::new(Dynamic::map(vec![(
            CAN_PICK_UP_LOOT.to_string(),
            Dynamic::Bool(false),
        )]));
        let out = fix.fix(input);
        assert_eq!(out.remainder.get(CAN_PICK_UP_LOOT), Dynamic::Bool(true));
    }

    #[test]
    fn preserves_other_fields() {
        let fix = VillagerSetCanPickUpLootFix::new(Schema);
        let input = Typed::new(Dynamic::map(vec![(
            "CustomName".to_string(),
            Dynamic::String("Bob".to_string()),
        )]));
        let out = fix.fix(input);
        assert_eq!(out.remainder.get(CAN_PICK_UP_LOOT), Dynamic::Bool(true));
        assert_eq!(out.remainder.get("CustomName").as_string(""), "Bob");
    }

    #[test]
    fn constructor_sets_base_fields() {
        let fix = VillagerSetCanPickUpLootFix::new(Schema);
        assert!(fix.base.changes_type);
        assert_eq!(fix.base.name, "Villager CanPickUpLoot default value");
        assert_eq!(fix.base.entity_name, "Villager");
        assert_eq!(fix.base.reference, References::Entity);
    }
}
