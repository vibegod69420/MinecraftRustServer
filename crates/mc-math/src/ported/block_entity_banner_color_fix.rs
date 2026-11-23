use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

impl Dynamic {
    pub fn create_int(&self, value: i32) -> Dynamic {
        Dynamic::Int(value)
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            Dynamic::Float(v) => *v as i32,
            Dynamic::Double(v) => *v as i32,
            _ => default,
        }
    }

    pub fn update<F>(&self, key: &str, fixer: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(map) => {
                let mut new_map = map.clone();
                let current = new_map.get(key).cloned().unwrap_or(Dynamic::Null);
                new_map.insert(key.to_string(), fixer(current));
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }

    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    pub fn create_list(&self, items: impl Iterator<Item = Dynamic>) -> Dynamic {
        Dynamic::List(items.collect())
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        match self {
            Dynamic::Map(map) => map.get(key).and_then(|v| match v {
                Dynamic::Int(i) => Some(*i),
                _ => None,
            }),
            _ => None,
        }
    }

    pub fn get_list(&self, key: &str) -> Option<&Vec<Dynamic>> {
        match self {
            Dynamic::Map(map) => map.get(key).and_then(|v| match v {
                Dynamic::List(l) => Some(l),
                _ => None,
            }),
            _ => None,
        }
    }
}

/// Minimal DFU stubs for self-contained compilation.
#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, Default)]
pub struct RemainderFinder;

pub mod dsl {
    use super::RemainderFinder;
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn update<F>(&self, _finder: RemainderFinder, fixer: F) -> Typed
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed::new(fixer(self.remainder.clone()))
    }

    pub fn remainder(&self) -> &Dynamic {
        &self.remainder
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum References {
    #[default]
    BlockEntity,
}

/// Minimal base stub mirroring `NamedEntityFix`.
pub struct NamedEntityFix {
    _schema: Schema,
    _changes_type: bool,
    _name: String,
    _reference: References,
    _entity_name: String,
}

impl NamedEntityFix {
    // JAVA: net.minecraft.util.datafix.fixes.NamedEntityFix#<init>
    pub fn new(
        schema: Schema,
        changes_type: bool,
        name: String,
        reference: References,
        entity_name: String,
    ) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
            _name: name,
            _reference: reference,
            _entity_name: entity_name,
        }
    }
}

/// Port of `net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix`.
pub struct BlockEntityBannerColorFix {
    _base: NamedEntityFix,
}

impl BlockEntityBannerColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _base: NamedEntityFix::new(
                schema,
                changes_type,
                "BlockEntityBannerColorFix".to_string(),
                References::BlockEntity,
                "minecraft:banner".to_string(),
            ),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        let tag = tag.update("Base", |d| d.create_int(15 - d.as_int(0)));
        tag.update("Patterns", |patterns| {
            let fixed = patterns
                .as_stream_opt()
                .map(|stream| {
                    let mapped = stream.into_iter().map(|p| {
                        p.update("Color", |c| c.create_int(15 - c.as_int(0)))
                    });
                    patterns.create_list(mapped)
                });
            fixed.unwrap_or(patterns)
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(dsl::remainder_finder(), |d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banner_tag(base: i32, pattern_colors: &[i32]) -> Dynamic {
        let mut map = BTreeMap::new();
        map.insert("Base".to_string(), Dynamic::Int(base));
        let patterns: Vec<Dynamic> = pattern_colors
            .iter()
            .map(|c| {
                let mut pm = BTreeMap::new();
                pm.insert("Color".to_string(), Dynamic::Int(*c));
                pm.insert("Pattern".to_string(), Dynamic::String("stripe".to_string()));
                Dynamic::Map(pm)
            })
            .collect();
        map.insert("Patterns".to_string(), Dynamic::List(patterns));
        Dynamic::Map(map)
    }

    #[test]
    fn fix_tag_inverts_base_color() {
        let fix = BlockEntityBannerColorFix::new(Schema, true);
        let tag = banner_tag(0, &[]);
        let fixed = fix.fix_tag(tag);
        assert_eq!(fixed.get_int("Base"), Some(15));
    }

    #[test]
    fn fix_tag_inverts_pattern_colors() {
        let fix = BlockEntityBannerColorFix::new(Schema, true);
        let tag = banner_tag(15, &[0, 1, 15]);
        let fixed = fix.fix_tag(tag);
        let patterns = fixed.get_list("Patterns").unwrap();
        assert_eq!(patterns.len(), 3);
        let colors: Vec<i32> = patterns
            .iter()
            .map(|p| p.get_int("Color").unwrap())
            .collect();
        assert_eq!(colors, vec![15, 14, 0]);
    }

    #[test]
    fn fix_tag_missing_base_defaults_to_15() {
        let fix = BlockEntityBannerColorFix::new(Schema, true);
        let mut map = BTreeMap::new();
        map.insert("Patterns".to_string(), Dynamic::List(vec![]));
        let fixed = fix.fix_tag(Dynamic::Map(map));
        // as_int(0) on Null gives 0, so 15 - 0 = 15
        assert_eq!(fixed.get_int("Base"), Some(15));
    }

    #[test]
    fn fix_tag_non_list_patterns_preserved() {
        let fix = BlockEntityBannerColorFix::new(Schema, true);
        let mut map = BTreeMap::new();
        map.insert("Base".to_string(), Dynamic::Int(1));
        map.insert("Patterns".to_string(), Dynamic::String("bad".to_string()));
        let fixed = fix.fix_tag(Dynamic::Map(map));
        assert_eq!(fixed.get_int("Base"), Some(14));
        match &fixed {
            Dynamic::Map(m) => assert!(matches!(m.get("Patterns"), Some(Dynamic::String(s)) if s == "bad")),
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn fix_updates_typed_remainder() {
        let fix = BlockEntityBannerColorFix::new(Schema, true);
        let typed = Typed::new(banner_tag(2, &[3]));
        let fixed = fix.fix(typed);
        assert_eq!(fixed.remainder().get_int("Base"), Some(13));
        let patterns = fixed.remainder().get_list("Patterns").unwrap();
        assert_eq!(patterns[0].get_int("Color"), Some(12));
    }
}
