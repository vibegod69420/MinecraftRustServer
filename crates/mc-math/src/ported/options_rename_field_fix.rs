/// Minimal self-contained dynamic value model for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i64),
    Bool(bool),
    Map(Vec<(String, Dynamic)>),
}

impl Dynamic {
    pub fn rename_field(&self, old_key: &str, new_key: &str) -> Self {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    if k == old_key {
                        new_entries.push((new_key.to_string(), v.clone()));
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                Dynamic::Map(new_entries)
            }
            _ => self.clone(),
        }
    }
}

/// Placeholder for DFU `Schema`.
#[derive(Clone, Debug, Default)]
pub struct Schema;

/// Placeholder for DFU `TypeRewriteRule`.
#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule {
    pub name: String,
}

pub struct OptionsRenameFieldFix {
    _schema: Schema,
    _changes_type: bool,
    fix_name: String,
    field_from: String,
    field_to: String,
}

impl OptionsRenameFieldFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsRenameFieldFix#<init>
    pub fn new(
        schema: Schema,
        changes_type: bool,
        fix_name: String,
        field_from: String,
        field_to: String,
    ) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
            fix_name,
            field_from,
            field_to,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsRenameFieldFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: self.fix_name.clone(),
        }
    }

    /// Applies the core rename operation modeled by this fix's rule.
    pub fn apply_to_options(&self, options: Dynamic) -> Dynamic {
        options.rename_field(&self.field_from, &self.field_to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn renames_existing_field() {
        let fix = OptionsRenameFieldFix::new(
            Schema,
            false,
            "RenameFoo".to_string(),
            "foo".to_string(),
            "bar".to_string(),
        );
        let input = map(vec![
            ("foo", Dynamic::Int(1)),
            ("keep", Dynamic::Bool(true)),
        ]);
        let output = fix.apply_to_options(input);
        let expected = map(vec![
            ("bar", Dynamic::Int(1)),
            ("keep", Dynamic::Bool(true)),
        ]);
        assert_eq!(output, expected);
        assert_eq!(fix.make_rule().name, "RenameFoo");
    }

    #[test]
    fn missing_field_leaves_dynamic_unchanged() {
        let fix = OptionsRenameFieldFix::new(
            Schema,
            false,
            "RenameMissing".to_string(),
            "old".to_string(),
            "new".to_string(),
        );
        let input = map(vec![("other", Dynamic::String("value".to_string()))]);
        let output = fix.apply_to_options(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn preserves_other_fields_and_order() {
        let fix = OptionsRenameFieldFix::new(
            Schema,
            true,
            "RenameOrdered".to_string(),
            "b".to_string(),
            "bee".to_string(),
        );
        let input = map(vec![
            ("a", Dynamic::Int(1)),
            ("b", Dynamic::Int(2)),
            ("c", Dynamic::Int(3)),
        ]);
        let output = fix.apply_to_options(input);
        let expected = map(vec![
            ("a", Dynamic::Int(1)),
            ("bee", Dynamic::Int(2)),
            ("c", Dynamic::Int(3)),
        ]);
        assert_eq!(output, expected);
    }

    #[test]
    fn non_map_dynamic_is_unchanged() {
        let fix = OptionsRenameFieldFix::new(
            Schema,
            false,
            "RenameNonMap".to_string(),
            "x".to_string(),
            "y".to_string(),
        );
        let input = Dynamic::String("not-a-map".to_string());
        assert_eq!(fix.apply_to_options(input.clone()), input);
    }
}
