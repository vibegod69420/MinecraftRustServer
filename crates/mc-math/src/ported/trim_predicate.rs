//! Port of `net.minecraft.core.component.predicates.TrimPredicate`.

use std::marker::PhantomData;

/// Minimal stand-in for `net.minecraft.world.item.equipment.trim.TrimMaterial`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimMaterial(pub String);

/// Minimal stand-in for `net.minecraft.world.item.equipment.trim.TrimPattern`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimPattern(pub String);

/// Minimal stand-in for `net.minecraft.world.item.equipment.trim.ArmorTrim`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmorTrim {
    pub material: TrimMaterial,
    pub pattern: TrimPattern,
}

impl ArmorTrim {
    // JAVA: net.minecraft.world.item.equipment.trim.ArmorTrim#material
    pub fn material(&self) -> &TrimMaterial {
        &self.material
    }

    // JAVA: net.minecraft.world.item.equipment.trim.ArmorTrim#pattern
    pub fn pattern(&self) -> &TrimPattern {
        &self.pattern
    }
}

/// Minimal stand-in for `net.minecraft.core.HolderSet`.
pub trait HolderSet<T> {
    fn contains(&self, value: &T) -> bool;
}

/// Minimal stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataComponentType<T>(PhantomData<T>);

/// Minimal stand-in for `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;
impl DataComponents {
    pub const TRIM: DataComponentType<ArmorTrim> = DataComponentType(PhantomData);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimPredicate<MS, PS> {
    pub material: Option<MS>,
    pub pattern: Option<PS>,
}

impl<MS, PS> TrimPredicate<MS, PS> {
    // JAVA: net.minecraft.core.component.predicates.TrimPredicate#material
    pub fn material(&self) -> Option<&MS> {
        self.material.as_ref()
    }

    // JAVA: net.minecraft.core.component.predicates.TrimPredicate#pattern
    pub fn pattern(&self) -> Option<&PS> {
        self.pattern.as_ref()
    }

    // JAVA: net.minecraft.core.component.predicates.TrimPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<ArmorTrim> {
        DataComponents::TRIM
    }
}

impl<MS, PS> TrimPredicate<MS, PS>
where
    MS: HolderSet<TrimMaterial>,
    PS: HolderSet<TrimPattern>,
{
    // JAVA: net.minecraft.core.component.predicates.TrimPredicate#matches
    pub fn matches(&self, trim: &ArmorTrim) -> bool {
        if let Some(ref material) = self.material {
            if !material.contains(trim.material()) {
                return false;
            }
        }
        match &self.pattern {
            Some(pattern) => pattern.contains(trim.pattern()),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MaterialSet(Vec<String>);
    impl HolderSet<TrimMaterial> for MaterialSet {
        fn contains(&self, value: &TrimMaterial) -> bool {
            self.0.iter().any(|s| s == &value.0)
        }
    }

    struct PatternSet(Vec<String>);
    impl HolderSet<TrimPattern> for PatternSet {
        fn contains(&self, value: &TrimPattern) -> bool {
            self.0.iter().any(|s| s == &value.0)
        }
    }

    fn trim(material: &str, pattern: &str) -> ArmorTrim {
        ArmorTrim {
            material: TrimMaterial(material.to_string()),
            pattern: TrimPattern(pattern.to_string()),
        }
    }

    #[test]
    fn both_absent_matches_any_trim() {
        let pred = TrimPredicate::<MaterialSet, PatternSet> {
            material: None,
            pattern: None,
        };
        assert!(pred.matches(&trim("gold", "swirl")));
        assert!(pred.matches(&trim("", "")));
    }

    #[test]
    fn material_present_and_mismatch_returns_false() {
        let pred = TrimPredicate {
            material: Some(MaterialSet(vec!["iron".into()])),
            pattern: None::<PatternSet>,
        };
        assert!(!pred.matches(&trim("gold", "swirl")));
    }

    #[test]
    fn material_matches_and_pattern_absent_returns_true() {
        let pred = TrimPredicate {
            material: Some(MaterialSet(vec!["gold".into()])),
            pattern: None::<PatternSet>,
        };
        assert!(pred.matches(&trim("gold", "swirl")));
    }

    #[test]
    fn pattern_present_and_mismatch_returns_false() {
        let pred = TrimPredicate {
            material: None::<MaterialSet>,
            pattern: Some(PatternSet(vec!["eye".into()])),
        };
        assert!(!pred.matches(&trim("gold", "swirl")));
    }

    #[test]
    fn both_present_require_both_to_match() {
        let pred = TrimPredicate {
            material: Some(MaterialSet(vec!["gold".into(), "iron".into()])),
            pattern: Some(PatternSet(vec!["swirl".into()])),
        };
        assert!(pred.matches(&trim("gold", "swirl")));
        assert!(pred.matches(&trim("iron", "swirl")));
        assert!(!pred.matches(&trim("diamond", "swirl")));
        assert!(!pred.matches(&trim("gold", "eye")));
    }

    #[test]
    fn component_type_is_trim() {
        let pred = TrimPredicate::<MaterialSet, PatternSet> {
            material: None,
            pattern: None,
        };
        assert_eq!(pred.component_type(), DataComponents::TRIM);
    }
}
