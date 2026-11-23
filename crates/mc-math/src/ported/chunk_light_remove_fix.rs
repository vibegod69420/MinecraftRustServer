/// Minimal self-contained dynamic value for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(Vec<(String, Dynamic)>),
    Other,
}

impl Dynamic {
    fn remove(self, key: &str) -> Self {
        match self {
            Self::Map(mut entries) => {
                entries.retain(|(k, _)| k != key);
                Self::Map(entries)
            }
            other => other,
        }
    }

    fn update<F: FnOnce(Self) -> Self>(self, key: &str, f: F) -> Self {
        match self {
            Self::Map(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    let old = std::mem::replace(v, Self::Other);
                    *v = f(old);
                }
                Self::Map(entries)
            }
            other => other,
        }
    }
}

/// Minimal stub for DFU TypeRewriteRule.
pub struct TypeRewriteRule {
    apply_fn: Box<dyn Fn(Dynamic) -> Dynamic>,
}

impl TypeRewriteRule {
    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.apply_fn)(input)
    }
}

/// Minimal stub for DFU Schema.
pub struct Schema;

pub struct ChunkLightRemoveFix;

impl ChunkLightRemoveFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkLightRemoveFix#<init>
    pub fn new(_schema: Schema, _version_key: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkLightRemoveFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            apply_fn: Box::new(|chunk| chunk.update("Level", |level| level.remove("isLightOn"))),
        }
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
    fn removes_is_light_on_from_level() {
        let fix = ChunkLightRemoveFix::new(Schema, true);
        let input = map(vec![(
            "Level",
            map(vec![("isLightOn", Dynamic::Other), ("xPos", Dynamic::Other)]),
        )]);
        let out = fix.make_rule().apply(input);
        assert_eq!(
            out,
            map(vec![("Level", map(vec![("xPos", Dynamic::Other)]))])
        );
    }

    #[test]
    fn preserves_level_when_is_light_on_missing() {
        let fix = ChunkLightRemoveFix::new(Schema, false);
        let input = map(vec![("Level", map(vec![("xPos", Dynamic::Other)]))]);
        let out = fix.make_rule().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_level_is_noop() {
        let fix = ChunkLightRemoveFix::new(Schema, true);
        let input = map(vec![("DataVersion", Dynamic::Other)]);
        let out = fix.make_rule().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_map_level_is_noop() {
        let fix = ChunkLightRemoveFix::new(Schema, true);
        let input = map(vec![("Level", Dynamic::Other)]);
        let out = fix.make_rule().apply(input.clone());
        assert_eq!(out, input);
    }
}
