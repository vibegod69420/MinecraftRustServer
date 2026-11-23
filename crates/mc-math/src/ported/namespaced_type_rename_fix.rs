use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    pub tag: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeReference {
    pub type_name: String,
}

impl TypeReference {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

#[derive(Clone, Debug)]
pub struct Schema {
    version_key: String,
    expected_type_ref: Option<String>,
}

impl Schema {
    pub fn new(version_key: impl Into<String>) -> Self {
        Self {
            version_key: version_key.into(),
            expected_type_ref: None,
        }
    }

    pub fn new_matching(version_key: impl Into<String>, type_ref_name: impl Into<String>) -> Self {
        Self {
            version_key: version_key.into(),
            expected_type_ref: Some(type_ref_name.into()),
        }
    }

    pub fn get_type(&self, type_reference: &TypeReference) -> Type {
        if let Some(expected) = &self.expected_type_ref {
            if expected == &type_reference.type_name {
                return dsl_named(&type_reference.type_name, namespaced_string());
            }
        }
        Type {
            tag: format!("actual:{}", type_reference.type_name),
        }
    }

    pub fn get_version_key(&self) -> &str {
        &self.version_key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRewriteRule;

fn dsl_named(type_name: &str, inner: Type) -> Type {
    Type {
        tag: format!("named:{}:{}", type_name, inner.tag),
    }
}

fn namespaced_string() -> Type {
    Type {
        tag: "namespaced_string".to_string(),
    }
}

pub struct NamespacedTypeRenameFix {
    name: String,
    type_reference: TypeReference,
    renamer: Box<dyn Fn(String) -> String>,
    schema: Schema,
}

impl NamespacedTypeRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.NamespacedTypeRenameFix#<init>
    pub fn new(
        schema: Schema,
        name: String,
        type_reference: TypeReference,
        renamer: Box<dyn Fn(String) -> String>,
    ) -> Self {
        Self {
            name,
            type_reference,
            renamer,
            schema,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.NamespacedTypeRenameFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let expected = dsl_named(self.type_reference.type_name(), namespaced_string());
        let actual = self.schema.get_type(&self.type_reference);
        if expected != actual {
            panic!(
                "\"{}\" is not what was expected.",
                self.type_reference.type_name()
            );
        }
        TypeRewriteRule
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_reference(&self) -> &TypeReference {
        &self.type_reference
    }

    pub fn apply_renamer(&self, value: String) -> String {
        (self.renamer)(value)
    }
}

impl fmt::Debug for NamespacedTypeRenameFix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamespacedTypeRenameFix")
            .field("name", &self.name)
            .field("type_reference", &self.type_reference)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_rule_succeeds_when_type_matches() {
        let schema = Schema::new_matching("v1", "minecraft:block_name");
        let tr = TypeReference::new("minecraft:block_name");
        let fix = NamespacedTypeRenameFix::new(
            schema,
            "rename_blocks".to_string(),
            tr,
            Box::new(|s| s),
        );
        let _rule = fix.make_rule();
        assert_eq!(fix.name(), "rename_blocks");
    }

    #[test]
    #[should_panic(expected = "\"minecraft:item_name\" is not what was expected.")]
    fn make_rule_panics_on_type_mismatch() {
        let schema = Schema::new("v1");
        let tr = TypeReference::new("minecraft:item_name");
        let fix = NamespacedTypeRenameFix::new(
            schema,
            "rename_items".to_string(),
            tr,
            Box::new(|s| s),
        );
        let _ = fix.make_rule();
    }

    #[test]
    fn renamer_applies_namespace_and_preserves_existing() {
        let schema = Schema::new_matching("v1", "t");
        let tr = TypeReference::new("t");
        let fix = NamespacedTypeRenameFix::new(
            schema,
            "rename".to_string(),
            tr,
            Box::new(|s| {
                if s.contains(':') {
                    s
                } else {
                    format!("minecraft:{}", s)
                }
            }),
        );
        assert_eq!(fix.apply_renamer("stone".to_string()), "minecraft:stone");
        assert_eq!(
            fix.apply_renamer("custom:foo".to_string()),
            "custom:foo"
        );
    }

    #[test]
    fn renamer_handles_empty_and_negative_like_strings() {
        let schema = Schema::new_matching("v2", "t2");
        let tr = TypeReference::new("t2");
        let fix = NamespacedTypeRenameFix::new(
            schema,
            "edge".to_string(),
            tr,
            Box::new(|s| {
                if s.is_empty() {
                    "minecraft:air".to_string()
                } else if s == "-1" {
                    "minecraft:minus_one".to_string()
                } else {
                    s
                }
            }),
        );
        assert_eq!(fix.apply_renamer(String::new()), "minecraft:air");
        assert_eq!(fix.apply_renamer("-1".to_string()), "minecraft:minus_one");
        assert_eq!(fix.type_reference().type_name(), "t2");
    }

    #[test]
    fn type_equality_is_exact_for_named_namespaced_string() {
        let tr = TypeReference::new("my:type");
        let expected = dsl_named(tr.type_name(), namespaced_string());
        let schema = Schema::new_matching("v3", "my:type");
        let actual = schema.get_type(&tr);
        assert_eq!(expected, actual);
        let mismatch_schema = Schema::new("v3");
        let mismatch = mismatch_schema.get_type(&tr);
        assert_ne!(expected, mismatch);
    }
}
