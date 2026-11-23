use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema;

pub struct References;

impl References {
    pub const BLOCK_ENTITY: &'static str = "block_entity";
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    Missing,
}

impl Dynamic {
    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(map) => map.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(map) => {
                let mut next = map.clone();
                next.insert(key.to_string(), value);
                Dynamic::Map(next)
            }
            _ => {
                let mut next = BTreeMap::new();
                next.insert(key.to_string(), value);
                Dynamic::Map(next)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typed {
    remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, _finder: RemainderFinder, f: F) -> Self {
        Self {
            remainder: f(self.remainder),
        }
    }

    pub fn remainder(&self) -> &Dynamic {
        &self.remainder
    }
}

pub struct WeaponSmithChestLootTableFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub reference: String,
    pub entity_name: String,
}

impl WeaponSmithChestLootTableFix {
    // JAVA: net.minecraft.util.datafix.fixes.WeaponSmithChestLootTableFix#WeaponSmithChestLootTableFix
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            schema,
            changes_type,
            name: "WeaponSmithChestLootTableFix".to_string(),
            reference: References::BLOCK_ENTITY.to_string(),
            entity_name: "minecraft:chest".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WeaponSmithChestLootTableFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |d| {
            let loot_table = d.get("LootTable").as_string("");
            if loot_table == "minecraft:chests/village_blacksmith" {
                let new_value = d.create_string("minecraft:chests/village/village_weaponsmith");
                d.set("LootTable", new_value)
            } else {
                d
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chest_dynamic(loot_table: &str) -> Dynamic {
        let mut map = BTreeMap::new();
        map.insert(
            "LootTable".to_string(),
            Dynamic::String(loot_table.to_string()),
        );
        Dynamic::Map(map)
    }

    #[test]
    fn replaces_blacksmith_loot_table() {
        let fix = WeaponSmithChestLootTableFix::new(Schema, true);
        let typed = Typed::new(chest_dynamic("minecraft:chests/village_blacksmith"));
        let out = fix.fix(typed);
        assert_eq!(
            out.remainder().get("LootTable").as_string(""),
            "minecraft:chests/village/village_weaponsmith"
        );
    }

    #[test]
    fn leaves_other_loot_table_unchanged() {
        let fix = WeaponSmithChestLootTableFix::new(Schema, false);
        let typed = Typed::new(chest_dynamic("minecraft:chests/simple_dungeon"));
        let out = fix.fix(typed);
        assert_eq!(
            out.remainder().get("LootTable").as_string(""),
            "minecraft:chests/simple_dungeon"
        );
    }

    #[test]
    fn missing_loot_table_uses_default_and_stays_unchanged() {
        let fix = WeaponSmithChestLootTableFix::new(Schema, true);
        let typed = Typed::new(Dynamic::Map(BTreeMap::new()));
        let out = fix.fix(typed);
        assert_eq!(out.remainder().get("LootTable").as_string(""), "");
        assert_eq!(out.remainder(), &Dynamic::Map(BTreeMap::new()));
    }

    #[test]
    fn non_string_loot_table_is_not_replaced() {
        let fix = WeaponSmithChestLootTableFix::new(Schema, true);
        let mut map = BTreeMap::new();
        map.insert("LootTable".to_string(), Dynamic::Map(BTreeMap::new()));
        let typed = Typed::new(Dynamic::Map(map.clone()));
        let out = fix.fix(typed);
        assert_eq!(out.remainder(), &Dynamic::Map(map));
    }

    #[test]
    fn numeric_like_loot_table_edge_case_unchanged() {
        let fix = WeaponSmithChestLootTableFix::new(Schema, true);
        let typed = Typed::new(chest_dynamic("-9223372036854775808"));
        let out = fix.fix(typed);
        assert_eq!(
            out.remainder().get("LootTable").as_string(""),
            "-9223372036854775808"
        );
    }
}
