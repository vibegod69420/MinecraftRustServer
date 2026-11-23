use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Long(i64),
    Int(i32),
}

/// Port of `net.minecraft.util.datafix.fixes.ProjectileStoredWeaponFix`.
///
/// DFU schema/type/typed optic infrastructure is not ported; this module
/// preserves the faithful choice selection and identity pass-through logic
/// for arrow projectile entities.
pub struct ProjectileStoredWeaponFix;

impl ProjectileStoredWeaponFix {
    // JAVA: net.minecraft.util.datafix.fixes.ProjectileStoredWeaponFix#ProjectileStoredWeaponFix
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ProjectileStoredWeaponFix#makeRule
    pub fn make_rule(&self, entity_id: &str, data: Dynamic) -> Dynamic {
        let data = self.fix_choice("minecraft:arrow", entity_id, data);
        self.fix_choice("minecraft:spectral_arrow", entity_id, data)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ProjectileStoredWeaponFix#fixChoice
    fn fix_choice(&self, choice_name: &str, entity_id: &str, data: Dynamic) -> Dynamic {
        Self::fix_choice_cap(choice_name, entity_id, data)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ProjectileStoredWeaponFix#fixChoiceCap
    fn fix_choice_cap(choice_name: &str, entity_id: &str, data: Dynamic) -> Dynamic {
        if entity_id == choice_name {
            // Mirrors Util.writeAndReadTypedOrThrow(..., UnaryOperator.identity())
            // as an identity transformation on the selected entity payload.
            data
        } else {
            data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dynamic, ProjectileStoredWeaponFix};
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn arrow_entity_passes_through_unchanged() {
        let fix = ProjectileStoredWeaponFix::new();
        let input = map(vec![
            ("id", Dynamic::String("minecraft:arrow".into())),
            ("damage", Dynamic::Int(2)),
        ]);
        let out = fix.make_rule("minecraft:arrow", input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn spectral_arrow_entity_passes_through_unchanged() {
        let fix = ProjectileStoredWeaponFix::new();
        let input = map(vec![
            ("id", Dynamic::String("minecraft:spectral_arrow".into())),
            ("pickup", Dynamic::Int(1)),
        ]);
        let out = fix.make_rule("minecraft:spectral_arrow", input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn unrelated_entity_is_unchanged() {
        let fix = ProjectileStoredWeaponFix::new();
        let input = map(vec![
            ("id", Dynamic::String("minecraft:trident".into())),
            ("damage", Dynamic::Int(8)),
        ]);
        let out = fix.make_rule("minecraft:trident", input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn identity_preserves_nested_structure() {
        let fix = ProjectileStoredWeaponFix::new();
        let input = map(vec![
            ("weapon", map(vec![("Count", Dynamic::Int(1))])),
            ("Owner", Dynamic::List(vec![Dynamic::Int(-1), Dynamic::Int(0)])),
        ]);
        let out = fix.make_rule("minecraft:arrow", input.clone());
        assert_eq!(out, input);
    }
}
