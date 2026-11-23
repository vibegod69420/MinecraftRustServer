//! Faithful port of `net.minecraft.util.datafix.fixes.OptionsFancyGraphicsToGraphicsModeFix`.

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(Vec<(String, Dynamic)>),
    Null,
}

impl Dynamic {
    fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_string())
    }

    fn rename_and_fix_field<F>(self, old_name: &str, new_name: &str, fixer: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                let mut found = None;
                for (k, v) in entries {
                    if k == old_name {
                        found = Some(v);
                    } else {
                        out.push((k, v));
                    }
                }
                if let Some(v) = found {
                    out.push((new_name.to_string(), fixer(v)));
                }
                Dynamic::Map(out)
            }
            other => other,
        }
    }
}

pub struct OptionsFancyGraphicsToGraphicsModeFix;

impl OptionsFancyGraphicsToGraphicsModeFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsFancyGraphicsToGraphicsModeFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsFancyGraphicsToGraphicsModeFix#fixGraphicsMode
    fn fix_graphics_mode(value: Dynamic) -> Dynamic {
        if value.as_string("true") == "true" {
            value.create_string("1")
        } else {
            value.create_string("0")
        }
    }

    /// Applies the core options transformation represented by the Java `makeRule`.
    pub fn apply(&self, options: Dynamic) -> Dynamic {
        options.rename_and_fix_field("fancyGraphics", "graphicsMode", Self::fix_graphics_mode)
    }
}

impl Default for OptionsFancyGraphicsToGraphicsModeFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_true_to_1() {
        let fix = OptionsFancyGraphicsToGraphicsModeFix::new();
        let input = Dynamic::Map(vec![(
            "fancyGraphics".into(),
            Dynamic::String("true".into()),
        )]);
        let out = fix.apply(input);
        assert_eq!(
            out,
            Dynamic::Map(vec![("graphicsMode".into(), Dynamic::String("1".into()))])
        );
    }

    #[test]
    fn fix_false_to_0() {
        let fix = OptionsFancyGraphicsToGraphicsModeFix::new();
        let input = Dynamic::Map(vec![(
            "fancyGraphics".into(),
            Dynamic::String("false".into()),
        )]);
        let out = fix.apply(input);
        assert_eq!(
            out,
            Dynamic::Map(vec![("graphicsMode".into(), Dynamic::String("0".into()))])
        );
    }

    #[test]
    fn fix_non_string_defaults_to_true_branch() {
        let out =
            OptionsFancyGraphicsToGraphicsModeFix::fix_graphics_mode(Dynamic::Int(0));
        assert_eq!(out, Dynamic::String("1".into()));
    }

    #[test]
    fn fix_arbitrary_string_to_0() {
        let out = OptionsFancyGraphicsToGraphicsModeFix::fix_graphics_mode(
            Dynamic::String("yes".into()),
        );
        assert_eq!(out, Dynamic::String("0".into()));
    }

    #[test]
    fn preserves_other_fields() {
        let fix = OptionsFancyGraphicsToGraphicsModeFix::new();
        let input = Dynamic::Map(vec![
            ("renderDistance".into(), Dynamic::Int(12)),
            ("fancyGraphics".into(), Dynamic::String("true".into())),
        ]);
        let out = fix.apply(input);
        assert_eq!(
            out,
            Dynamic::Map(vec![
                ("renderDistance".into(), Dynamic::Int(12)),
                ("graphicsMode".into(), Dynamic::String("1".into())),
            ])
        );
    }
}
