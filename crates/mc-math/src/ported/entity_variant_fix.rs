use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
/// Represents only the node kinds needed by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Long(i64),
    Int(i32),
}

impl Dynamic {
    fn as_number_i32(&self) -> Option<i32> {
        match self {
            Dynamic::Int(v) => Some(*v),
            Dynamic::Long(v) => Some(*v as i32),
            _ => None,
        }
    }
}

/// Port of `net.minecraft.util.datafix.fixes.EntityVariantFix`.
///
/// DFU schema/Typed/DSL infrastructure is not ported; this module preserves
/// the faithful field-rename and variant-conversion logic.
pub struct EntityVariantFix {
    field_name: String,
    id_conversions: fn(i32) -> String,
}

impl EntityVariantFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityVariantFix#EntityVariantFix
    pub fn new(field_name: &str, id_conversions: fn(i32) -> String) -> Self {
        Self {
            field_name: field_name.to_string(),
            id_conversions,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityVariantFix#updateAndRename
    fn update_and_rename(
        value: Dynamic,
        old_key: &str,
        new_key: &str,
        f: impl Fn(Dynamic) -> Dynamic,
    ) -> Dynamic {
        match value {
            Dynamic::Map(mut map) => {
                if let Some(old_val) = map.get(old_key).cloned() {
                    let new_val = f(old_val);
                    map.insert(new_key.to_string(), new_val);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityVariantFix#fix
    pub fn fix(&self, value: Dynamic) -> SelfFixResult {
        SelfFixResult(Self::update_and_rename(
            value,
            &self.field_name,
            "variant",
            |dyn_val| {
                if let Some(n) = dyn_val.as_number_i32() {
                    Dynamic::String((self.id_conversions)(n))
                } else {
                    dyn_val
                }
            },
        ))
    }
}

/// Wrapper to keep the fix API explicit and testable in a self-contained module.
#[derive(Clone, Debug, PartialEq)]
pub struct SelfFixResult(pub Dynamic);

impl SelfFixResult {
    pub fn into_dynamic(self) -> Dynamic {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Dynamic, EntityVariantFix};
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    fn conv(id: i32) -> String {
        format!("minecraft:variant_{id}")
    }

    #[test]
    fn converts_numeric_field_to_variant_string() {
        let fix = EntityVariantFix::new("Type", conv);
        let input = map(vec![("Type", Dynamic::Int(2)), ("id", Dynamic::String("e".into()))]);
        let out = fix.fix(input).into_dynamic();
        let Dynamic::Map(m) = out else { panic!("expected map") };
        assert_eq!(m.get("variant"), Some(&Dynamic::String("minecraft:variant_2".into())));
        // Java code sets new key; old key remains present in the original map structure.
        assert_eq!(m.get("Type"), Some(&Dynamic::Int(2)));
    }

    #[test]
    fn preserves_non_numeric_field_value() {
        let fix = EntityVariantFix::new("Type", conv);
        let input = map(vec![("Type", Dynamic::String("already_string".into()))]);
        let out = fix.fix(input).into_dynamic();
        let Dynamic::Map(m) = out else { panic!("expected map") };
        assert_eq!(m.get("variant"), Some(&Dynamic::String("already_string".into())));
    }

    #[test]
    fn missing_field_leaves_value_unchanged() {
        let fix = EntityVariantFix::new("Type", conv);
        let input = map(vec![("id", Dynamic::String("entity".into()))]);
        let out = fix.fix(input.clone()).into_dynamic();
        assert_eq!(out, input);
    }

    #[test]
    fn handles_negative_and_long_truncation() {
        let fix = EntityVariantFix::new("Type", conv);
        let neg = map(vec![("Type", Dynamic::Int(-1))]);
        let out_neg = fix.fix(neg).into_dynamic();
        let Dynamic::Map(m1) = out_neg else { panic!("expected map") };
        assert_eq!(m1.get("variant"), Some(&Dynamic::String("minecraft:variant_-1".into())));

        // Java Number.intValue() truncates long to int.
        let long_val = (i32::MAX as i64) + 1;
        let long_input = map(vec![("Type", Dynamic::Long(long_val))]);
        let out_long = fix.fix(long_input).into_dynamic();
        let Dynamic::Map(m2) = out_long else { panic!("expected map") };
        assert_eq!(
            m2.get("variant"),
            Some(&Dynamic::String(format!("minecraft:variant_{}", long_val as i32)))
        );
    }

    #[test]
    fn non_map_input_is_returned_as_is() {
        let fix = EntityVariantFix::new("Type", conv);
        let input = Dynamic::Int(5);
        let out = fix.fix(input.clone()).into_dynamic();
        assert_eq!(out, input);
    }
}
