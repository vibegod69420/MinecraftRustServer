use std::collections::BTreeMap;

/// Minimal self-contained dynamic/typed representation for datafix porting.
/// Mirrors the subset of DFU `Dynamic`/`Typed` operations used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Byte(i8),
    Short(i16),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s.as_str()),
            _ => None,
        }
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

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                let child = map.remove(key).unwrap_or(Dynamic::Null);
                let updated = f(child);
                map.insert(key.to_string(), updated);
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

/// Minimal typed wrapper around a `Dynamic` value, representing a DFU `Typed<?>`.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn new(value: Dynamic) -> Self {
        Self { value }
    }

    pub fn get_optional_string_field(&self, key: &str) -> Option<String> {
        self.value.get(key).and_then(|v| v.as_string()).map(|s| s.to_string())
    }

    /// Gets a nested field like the OpticFinder for "id" which in DFU resolves to
    /// a Pair<String, String> in the named item name type; here we simplify to reading
    /// the string id directly from the "id" field.
    pub fn get_optional_id(&self) -> Option<String> {
        self.get_optional_string_field("id")
    }

    /// Updates the "tag" field using the provided fixer.
    pub fn update_tag<F>(self, fixer: F) -> Self
    where
        F: FnOnce(Typed) -> Typed,
    {
        let tag = match &self.value {
            Dynamic::Map(map) => map.get("tag").cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        };
        let fixed_tag = fixer(Typed::new(tag));
        Typed::new(self.value.set("tag", fixed_tag.value))
    }
}

/// Faithful structural port of `net.minecraft.util.datafix.fixes.ItemStackTagFix`.
/// Abstract datafix that applies a transformation to the `tag` of matching item stacks.
pub struct ItemStackTagFix {
    pub name: String,
    pub id_filter: fn(&str) -> bool,
    pub fix_item_stack_tag: fn(Typed) -> Typed,
}

impl ItemStackTagFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#<init>
    pub fn new(
        name: String,
        id_filter: fn(&str) -> bool,
        fix_item_stack_tag: fn(Typed) -> Typed,
    ) -> Self {
        Self {
            name,
            id_filter,
            fix_item_stack_tag,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#createFixer
    pub fn create_fixer(
        id_filter: fn(&str) -> bool,
        fix_item_stack_tag: fn(Typed) -> Typed,
    ) -> impl Fn(Typed) -> Typed {
        move |typed: Typed| {
            let id_opt = typed.get_optional_id();
            match id_opt {
                Some(id) if id_filter(&id) => typed.update_tag(fix_item_stack_tag),
                _ => typed,
            }
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Typed) -> Typed {
        Self::create_fixer(self.id_filter, self.fix_item_stack_tag)
    }

    /// Applies this fix to a typed item stack.
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagFix#fixItemStackTag
    pub fn apply(&self, typed: Typed) -> Typed {
        let rule = self.make_rule();
        rule(typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_stack(id: &str, tag: Option<Dynamic>) -> Typed {
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), Dynamic::String(id.to_string()));
        if let Some(t) = tag {
            map.insert("tag".to_string(), t);
        }
        Typed::new(Dynamic::Map(map))
    }

    fn is_diamond(id: &str) -> bool {
        id == "minecraft:diamond_sword"
    }

    fn rename_damage_tag(typed: Typed) -> Typed {
        Typed::new(typed.value.update("Damage", |_old| Dynamic::Int(0)))
    }

    #[test]
    fn test_matching_id_applies_fix() {
        let fix = ItemStackTagFix::new("TestFix".to_string(), is_diamond, rename_damage_tag);
        let mut tag = BTreeMap::new();
        tag.insert("Damage".to_string(), Dynamic::Int(5));
        let input = item_stack("minecraft:diamond_sword", Some(Dynamic::Map(tag)));
        let output = fix.apply(input);
        let out_tag = output.value.get("tag").unwrap();
        assert_eq!(out_tag.get("Damage"), Some(&Dynamic::Int(0)));
    }

    #[test]
    fn test_non_matching_id_unchanged() {
        let fix = ItemStackTagFix::new("TestFix".to_string(), is_diamond, rename_damage_tag);
        let mut tag = BTreeMap::new();
        tag.insert("Damage".to_string(), Dynamic::Int(5));
        let input = item_stack("minecraft:apple", Some(Dynamic::Map(tag)));
        let output = fix.apply(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn test_missing_id_unchanged() {
        let fix = ItemStackTagFix::new("TestFix".to_string(), is_diamond, rename_damage_tag);
        let mut map = BTreeMap::new();
        map.insert("tag".to_string(), Dynamic::Null);
        let input = Typed::new(Dynamic::Map(map));
        let output = fix.apply(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn test_create_fixer_directly() {
        let fixer = ItemStackTagFix::create_fixer(is_diamond, rename_damage_tag);
        let mut tag = BTreeMap::new();
        tag.insert("Damage".to_string(), Dynamic::Int(10));
        let input = item_stack("minecraft:diamond_sword", Some(Dynamic::Map(tag)));
        let output = fixer(input);
        assert_eq!(output.value.get("tag").unwrap().get("Damage"), Some(&Dynamic::Int(0)));
    }

    #[test]
    fn test_missing_tag_creates_null_tag_updated() {
        let fix = ItemStackTagFix::new("TestFix".to_string(), is_diamond, |typed| {
            Typed::new(match typed.value {
                Dynamic::Null => {
                    let mut m = BTreeMap::new();
                    m.insert("fixed".to_string(), Dynamic::Bool(true));
                    Dynamic::Map(m)
                }
                other => other,
            })
        });
        let input = item_stack("minecraft:diamond_sword", None);
        let output = fix.apply(input);
        let tag = output.value.get("tag").unwrap();
        assert_eq!(tag.get("fixed"), Some(&Dynamic::Bool(true)));
    }
}
