// === FILE: options_force_vbo_fix.rs ===
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Minimal self-contained DFU stubs
// ---------------------------------------------------------------------------

/// Minimal schema stub.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

impl Schema {
    pub fn get_type(&self, _reference: TypeRef) -> TypeRef {
        TypeRef
    }
}

/// Minimal type reference stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeRef;

/// Minimal optic/finder stub for remainder updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpticFinder;

/// DSL stub.
pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> OpticFinder {
        OpticFinder
    }
}

/// References stub.
pub struct References;

impl References {
    pub const OPTIONS: TypeRef = TypeRef;
}

/// Minimal dynamic value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl Dynamic {
    pub fn create_string(value: impl Into<String>) -> Self {
        Dynamic::String(value.into())
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                map.insert(key.to_string(), value);
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Minimal typed wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn update<F>(self, _finder: OpticFinder, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed {
            value: f(self.value),
        }
    }
}

/// Minimal type rewrite rule.
pub struct TypeRewriteRule {
    name: &'static str,
    apply_fn: Box<dyn Fn(Typed) -> Typed>,
}

impl TypeRewriteRule {
    pub fn apply(&self, typed: Typed) -> Typed {
        (self.apply_fn)(typed)
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

/// Minimal DataFix base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFix {
    output_schema: Schema,
    changes_type: bool,
}

impl DataFix {
    pub fn new(output_schema: Schema, changes_type: bool) -> Self {
        Self {
            output_schema,
            changes_type,
        }
    }

    pub fn get_input_schema(&self) -> &Schema {
        &self.output_schema
    }

    pub fn fix_type_everywhere_typed<F>(
        &self,
        name: &'static str,
        _type_ref: TypeRef,
        f: F,
    ) -> TypeRewriteRule
    where
        F: Fn(Typed) -> Typed + 'static,
    {
        TypeRewriteRule {
            name,
            apply_fn: Box::new(f),
        }
    }
}

// ---------------------------------------------------------------------------
// Ported fix
// ---------------------------------------------------------------------------

/// Port of `net.minecraft.util.datafix.fixes.OptionsForceVBOFix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsForceVBOFix {
    data_fix: DataFix,
}

impl OptionsForceVBOFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsForceVBOFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            data_fix: DataFix::new(schema, changes_type),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsForceVBOFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        self.data_fix.fix_type_everywhere_typed(
            "OptionsForceVBOFix",
            self.data_fix
                .get_input_schema()
                .get_type(References::OPTIONS),
            |typed: Typed| {
                typed.update(Dsl::remainder_finder(), |dynamic: Dynamic| {
                    dynamic.set("useVbo", Dynamic::create_string("true"))
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn options_map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(k.to_string(), v);
        }
        Dynamic::Map(map)
    }

    #[test]
    fn sets_use_vbo_to_true_when_missing() {
        let fix = OptionsForceVBOFix::new(Schema, false);
        let rule = fix.make_rule();
        let input = Typed {
            value: options_map(vec![]),
        };
        let output = rule.apply(input);
        assert_eq!(
            output.value.get("useVbo").and_then(Dynamic::as_string),
            Some("true")
        );
    }

    #[test]
    fn overwrites_existing_use_vbo_false() {
        let fix = OptionsForceVBOFix::new(Schema, false);
        let rule = fix.make_rule();
        let input = Typed {
            value: options_map(vec![("useVbo", Dynamic::create_string("false"))]),
        };
        let output = rule.apply(input);
        assert_eq!(
            output.value.get("useVbo").and_then(Dynamic::as_string),
            Some("true")
        );
    }

    #[test]
    fn preserves_other_options_with_numeric_edge_cases() {
        let fix = OptionsForceVBOFix::new(Schema, true);
        let rule = fix.make_rule();
        let nan_bits: u32 = 0xFFC0_0001;
        let neg_inf_bits: u64 = f64::NEG_INFINITY.to_bits();
        let input = Typed {
            value: options_map(vec![
                ("renderDistance", Dynamic::Int(-1)),
                ("maxFps", Dynamic::Int(i32::MAX)),
                ("gamma", Dynamic::Float(f32::from_bits(nan_bits))),
                ("fov", Dynamic::Double(f64::from_bits(neg_inf_bits))),
                ("seedOffset", Dynamic::Long(i64::MIN)),
                ("useVbo", Dynamic::create_string("false")),
            ]),
        };
        let output = rule.apply(input);

        assert_eq!(
            output.value.get("useVbo").and_then(Dynamic::as_string),
            Some("true")
        );
        assert_eq!(output.value.get("renderDistance"), Some(&Dynamic::Int(-1)));
        assert_eq!(output.value.get("maxFps"), Some(&Dynamic::Int(i32::MAX)));
        assert_eq!(
            output.value.get("seedOffset"),
            Some(&Dynamic::Long(i64::MIN))
        );
        match output.value.get("gamma") {
            Some(Dynamic::Float(f)) => assert_eq!(f.to_bits(), nan_bits),
            other => panic!("expected float gamma, got {other:?}"),
        }
        match output.value.get("fov") {
            Some(Dynamic::Double(d)) => assert_eq!(d.to_bits(), neg_inf_bits),
            other => panic!("expected double fov, got {other:?}"),
        }
    }

    #[test]
    fn overwrites_non_string_use_vbo() {
        let fix = OptionsForceVBOFix::new(Schema, false);
        let rule = fix.make_rule();
        let input = Typed {
            value: options_map(vec![("useVbo", Dynamic::Bool(false))]),
        };
        let output = rule.apply(input);
        assert_eq!(
            output.value.get("useVbo").and_then(Dynamic::as_string),
            Some("true")
        );
    }

    #[test]
    fn rule_has_expected_name_and_applies() {
        let fix = OptionsForceVBOFix::new(Schema, false);
        let rule = fix.make_rule();
        assert_eq!(rule.name(), "OptionsForceVBOFix");

        let output = rule.apply(Typed {
            value: Dynamic::Map(BTreeMap::new()),
        });
        assert_eq!(
            output.value.get("useVbo"),
            Some(&Dynamic::String("true".to_string()))
        );
    }
}
