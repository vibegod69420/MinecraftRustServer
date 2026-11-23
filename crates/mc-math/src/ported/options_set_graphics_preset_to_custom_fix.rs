use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

static DYNAMIC_NULL: Dynamic = Dynamic::Null;

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn create_string(&self, s: &str) -> Self {
        Dynamic::String(s.to_string())
    }

    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).unwrap_or(&DYNAMIC_NULL),
            _ => &DYNAMIC_NULL,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut next = m.clone();
                next.insert(key.to_string(), value);
                Dynamic::Map(next)
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct TypeRewriteRule {
    name: String,
    apply_fn: Box<dyn Fn(Dynamic) -> Dynamic>,
}

impl TypeRewriteRule {
    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.apply_fn)(input)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct OptionsSetGraphicsPresetToCustomFix {
    #[allow(dead_code)]
    schema: Schema,
    #[allow(dead_code)]
    changes_type: bool,
}

impl OptionsSetGraphicsPresetToCustomFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsSetGraphicsPresetToCustomFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            changes_type: true,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsSetGraphicsPresetToCustomFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "graphicsPreset set to \"custom\"".to_string(),
            apply_fn: Box::new(|remainder: Dynamic| {
                remainder.set("graphicsPreset", remainder.create_string("custom"))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn sets_graphics_preset_to_custom() {
        let fix = OptionsSetGraphicsPresetToCustomFix::new(Schema);
        let out = fix.make_rule().apply(Dynamic::empty_map());
        assert_eq!(
            out.get("graphicsPreset"),
            &Dynamic::String("custom".to_string())
        );
    }

    #[test]
    fn overwrites_existing_preset() {
        let fix = OptionsSetGraphicsPresetToCustomFix::new(Schema);
        let input = map(vec![(
            "graphicsPreset",
            Dynamic::String("fancy".to_string()),
        )]);
        let out = fix.make_rule().apply(input);
        assert_eq!(
            out.get("graphicsPreset"),
            &Dynamic::String("custom".to_string())
        );
    }

    #[test]
    fn preserves_other_fields() {
        let fix = OptionsSetGraphicsPresetToCustomFix::new(Schema);
        let input = map(vec![
            ("renderDistance", Dynamic::String("12".to_string())),
            ("graphicsPreset", Dynamic::String("fast".to_string())),
        ]);
        let out = fix.make_rule().apply(input);
        assert_eq!(
            out.get("renderDistance"),
            &Dynamic::String("12".to_string())
        );
        assert_eq!(
            out.get("graphicsPreset"),
            &Dynamic::String("custom".to_string())
        );
    }

    #[test]
    fn rule_name_matches_java() {
        let fix = OptionsSetGraphicsPresetToCustomFix::new(Schema);
        assert_eq!(fix.make_rule().name(), "graphicsPreset set to \"custom\"");
    }
}
