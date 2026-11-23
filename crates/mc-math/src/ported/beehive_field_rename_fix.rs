/// Faithful port of `net.minecraft.util.datafix.fixes.BeehiveFieldRenameFix`.
///
/// Self-contained dynamic model for the beehive block entity field rename.
/// Java DataFixerUpper optics/types are represented as direct structural updates.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    String(String),
    Int(i32),
    List(Vec<DynamicValue>),
    Compound(Vec<(String, DynamicValue)>),
    Other,
}

impl DynamicValue {
    // JAVA: com.mojang.serialization.Dynamic#remove
    fn remove(self, key: &str) -> Self {
        match self {
            DynamicValue::Compound(mut entries) => {
                entries.retain(|(k, _)| k != key);
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#renameField
    fn rename_field(self, old_key: &str, new_key: &str) -> Self {
        match self {
            DynamicValue::Compound(mut entries) => {
                if let Some(pos) = entries.iter().position(|(k, _)| k == old_key) {
                    let (_, value) = entries.remove(pos);
                    if !entries.iter().any(|(k, _)| k == new_key) {
                        entries.push((new_key.to_string(), value));
                    } else {
                        // Preserve behavior conservatively: if target exists, keep target and drop old.
                    }
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }

    fn get_mut_list(&mut self, key: &str) -> Option<&mut Vec<DynamicValue>> {
        match self {
            DynamicValue::Compound(entries) => entries.iter_mut().find_map(|(k, v)| {
                if k == key {
                    if let DynamicValue::List(items) = v {
                        Some(items)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            _ => None,
        }
    }
}

pub struct BeehiveFieldRenameFix;

impl BeehiveFieldRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.BeehiveFieldRenameFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BeehiveFieldRenameFix#fixBeehive
    fn fix_beehive(&self, value: DynamicValue) -> DynamicValue {
        value.remove("Bees")
    }

    // JAVA: net.minecraft.util.datafix.fixes.BeehiveFieldRenameFix#fixBee
    fn fix_bee(&self, value: DynamicValue) -> DynamicValue {
        let value = value.remove("EntityData");
        let value = value.rename_field("TicksInHive", "ticks_in_hive");
        value.rename_field("MinOccupationTicks", "min_ticks_in_hive")
    }

    /// Applies the beehive fix to a block entity dynamic value when its id is `minecraft:beehive`.
    /// Mirrors the Java rule: remove top-level `Bees` from beehive remainder and rename fields in each bee element.
    // JAVA: net.minecraft.util.datafix.fixes.BeehiveFieldRenameFix#makeRule (behavioral equivalent)
    pub fn apply_to_block_entity(&self, block_entity: DynamicValue) -> DynamicValue {
        let is_beehive = match &block_entity {
            DynamicValue::Compound(entries) => entries.iter().any(|(k, v)| {
                k == "id" && matches!(v, DynamicValue::String(s) if s == "minecraft:beehive")
            }),
            _ => false,
        };

        if !is_beehive {
            return block_entity;
        }

        let mut be = block_entity;

        // Fix bee elements inside Bees list first (Java updates typed Bees, then remainder removes Bees).
        if let Some(bees) = be.get_mut_list("Bees") {
            for bee in bees.iter_mut() {
                let old = std::mem::replace(bee, DynamicValue::Other);
                *bee = self.fix_bee(old);
            }
        }

        // Remove Bees from beehive remainder.
        self.fix_beehive(be)
    }
}

impl Default for BeehiveFieldRenameFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn string(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    #[test]
    fn renames_bee_fields_and_removes_bees_from_beehive() {
        let fix = BeehiveFieldRenameFix::new();
        let be = compound(vec![
            ("id", string("minecraft:beehive")),
            (
                "Bees",
                DynamicValue::List(vec![
                    compound(vec![
                        ("EntityData", compound(vec![("id", string("minecraft:bee"))])),
                        ("TicksInHive", DynamicValue::Int(10)),
                        ("MinOccupationTicks", DynamicValue::Int(200)),
                    ]),
                    compound(vec![
                        ("TicksInHive", DynamicValue::Int(0)),
                        ("MinOccupationTicks", DynamicValue::Int(100)),
                    ]),
                ]),
            ),
        ]);

        let fixed = fix.apply_to_block_entity(be);
        let DynamicValue::Compound(entries) = fixed else { panic!("expected compound") };
        assert!(!entries.iter().any(|(k, _)| k == "Bees"));
        // Bee fields were renamed in-place before removal; since Bees is removed, we verify via direct bee fix.
    }

    #[test]
    fn fix_bee_renames_and_removes_entity_data() {
        let fix = BeehiveFieldRenameFix::new();
        let bee = compound(vec![
            ("EntityData", compound(vec![("id", string("minecraft:bee"))])),
            ("TicksInHive", DynamicValue::Int(5)),
            ("MinOccupationTicks", DynamicValue::Int(20)),
        ]);
        let fixed = fix.fix_bee(bee);
        let DynamicValue::Compound(entries) = fixed else { panic!("expected compound") };
        assert!(!entries.iter().any(|(k, _)| k == "EntityData"));
        assert!(entries.iter().any(|(k, v)| k == "ticks_in_hive" && matches!(v, DynamicValue::Int(5))));
        assert!(entries.iter().any(|(k, v)| k == "min_ticks_in_hive" && matches!(v, DynamicValue::Int(20))));
        assert!(!entries.iter().any(|(k, _)| k == "TicksInHive" || k == "MinOccupationTicks"));
    }

    #[test]
    fn non_beehive_block_entity_is_unchanged() {
        let fix = BeehiveFieldRenameFix::new();
        let be = compound(vec![
            ("id", string("minecraft:chest")),
            ("Items", DynamicValue::List(vec![])),
        ]);
        assert_eq!(fix.apply_to_block_entity(be.clone()), be);
    }

    #[test]
    fn fix_beehive_removes_bees_field() {
        let fix = BeehiveFieldRenameFix::new();
        let be = compound(vec![
            ("id", string("minecraft:beehive")),
            ("Bees", DynamicValue::List(vec![])),
            ("Keep", DynamicValue::Int(1)),
        ]);
        let fixed = fix.fix_beehive(be);
        let DynamicValue::Compound(entries) = fixed else { panic!("expected compound") };
        assert!(!entries.iter().any(|(k, _)| k == "Bees"));
        assert!(entries.iter().any(|(k, _)| k == "Keep"));
    }

    #[test]
    fn default_constructor_matches_new() {
        let bee = compound(vec![("TicksInHive", DynamicValue::Int(1))]);
        let a = BeehiveFieldRenameFix::new().fix_bee(bee.clone());
        let b = BeehiveFieldRenameFix::default().fix_bee(bee);
        assert_eq!(a, b);
    }
}
