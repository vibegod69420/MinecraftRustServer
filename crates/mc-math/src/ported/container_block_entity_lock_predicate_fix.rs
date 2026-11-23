use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Double(f64),
    Float(f32),
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Bool(bool),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#renameAndFixField
    pub fn rename_and_fix_field(
        self,
        old_name: &str,
        new_name: &str,
        fixer: impl FnOnce(Dynamic) -> Dynamic,
    ) -> Dynamic {
        match self {
            Dynamic::Map(mut map) => {
                if let Some(value) = map.remove(old_name) {
                    map.insert(new_name.to_string(), fixer(value));
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DslRemainderFinder;

#[derive(Clone, Copy, Debug, Default)]
pub struct Dsl;

// JAVA: com.mojang.datafixers.DSL#remainderFinder
pub fn dsl_remainder_finder() -> DslRemainderFinder {
    DslRemainderFinder
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update(self, _finder: DslRemainderFinder, f: impl FnOnce(Dynamic) -> Dynamic) -> Typed {
        Typed {
            value: f(self.value),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#findChoiceType
    pub fn find_choice_type(&self, _type_ref: &str) -> TypeReference {
        TypeReference
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TypeReference;

#[derive(Clone, Copy, Debug, Default)]
pub struct TypeRewriteRule;

pub struct References;

impl References {
    pub const BLOCK_ENTITY: &'static str = "block_entity";
}

pub struct DataFix {
    input_schema: Schema,
    changes_type: bool,
}

impl DataFix {
    // JAVA: com.mojang.datafixers.DataFix#<init>
    pub fn new(input_schema: Schema, changes_type: bool) -> Self {
        Self {
            input_schema,
            changes_type,
        }
    }

    // JAVA: com.mojang.datafixers.DataFix#getInputSchema
    pub fn get_input_schema(&self) -> &Schema {
        &self.input_schema
    }

    // JAVA: com.mojang.datafixers.DataFix#fixTypeEverywhereTyped
    pub fn fix_type_everywhere_typed(
        &self,
        _name: &str,
        _type_ref: TypeReference,
        _fixer: fn(Typed) -> Typed,
    ) -> TypeRewriteRule {
        TypeRewriteRule
    }
}

// JAVA: net.minecraft.util.datafix.fixes.LockComponentPredicateFix#fixLock
pub fn lock_component_predicate_fix_lock(dynamic: Dynamic) -> Dynamic {
    // External dependency stub: original behavior is delegated to LockComponentPredicateFix.
    dynamic
}

pub struct ContainerBlockEntityLockPredicateFix {
    data_fix: DataFix,
}

impl ContainerBlockEntityLockPredicateFix {
    // JAVA: net.minecraft.util.datafix.fixes.ContainerBlockEntityLockPredicateFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            data_fix: DataFix::new(schema, false),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ContainerBlockEntityLockPredicateFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let type_ref = self
            .data_fix
            .get_input_schema()
            .find_choice_type(References::BLOCK_ENTITY);
        self.data_fix.fix_type_everywhere_typed(
            "ContainerBlockEntityLockPredicateFix",
            type_ref,
            Self::fix_block_entity,
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.ContainerBlockEntityLockPredicateFix#fixBlockEntity
    fn fix_block_entity(typed: Typed) -> Typed {
        typed.update(dsl_remainder_finder(), |d| {
            d.rename_and_fix_field("Lock", "lock", lock_component_predicate_fix_lock)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn renames_lock_to_lock_lowercase() {
        let input = Typed {
            value: make_map(vec![("Lock", Dynamic::String("my_key".to_string()))]),
        };
        let out = ContainerBlockEntityLockPredicateFix::fix_block_entity(input);
        let expected = make_map(vec![("lock", Dynamic::String("my_key".to_string()))]);
        assert_eq!(out.value, expected);
    }

    #[test]
    fn missing_lock_field_is_unchanged() {
        let input = Typed {
            value: make_map(vec![("Items", Dynamic::List(vec![]))]),
        };
        let out = ContainerBlockEntityLockPredicateFix::fix_block_entity(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn preserves_other_fields_and_removes_old_key() {
        let input = Typed {
            value: make_map(vec![
                ("Lock", Dynamic::String("k".to_string())),
                ("x", Dynamic::Int(7)),
                ("y", Dynamic::Long(-1)),
            ]),
        };
        let out = ContainerBlockEntityLockPredicateFix::fix_block_entity(input);
        match out.value {
            Dynamic::Map(m) => {
                assert_eq!(m.get("lock"), Some(&Dynamic::String("k".to_string())));
                assert_eq!(m.get("x"), Some(&Dynamic::Int(7)));
                assert_eq!(m.get("y"), Some(&Dynamic::Long(-1)));
                assert!(!m.contains_key("Lock"));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn make_rule_does_not_panic() {
        let fix = ContainerBlockEntityLockPredicateFix::new(Schema);
        let _rule = fix.make_rule();
    }
}
