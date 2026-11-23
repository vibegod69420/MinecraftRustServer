//! Faithful port of `net.minecraft.util.datafix.fixes.BannerEntityCustomNameToOverrideComponentFix`
//! from Minecraft Java Edition 1.21.11.

use std::collections::BTreeMap;
use std::marker::PhantomData;

/// Minimal dynamic value representation for datafix operations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn create_string(s: impl Into<String>) -> Self {
        Dynamic::String(s.into())
    }

    pub fn create_map(entries: Vec<(Dynamic, Dynamic)>) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            if let Dynamic::String(key) = k {
                map.insert(key, v);
            }
        }
        Dynamic::Map(map)
    }

    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.insert(key.to_string(), value);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Self {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.remove(key);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// Minimal pair type matching DFU's `com.mojang.datafixers.util.Pair`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn get_first(&self) -> &A {
        &self.0
    }

    pub fn get_second(&self) -> &B {
        &self.1
    }
}

#[derive(Clone, Debug)]
enum OpticFinderKind {
    Field(String),
    TextComponent,
}

/// Minimal optic finder for field/type access on typed values.
#[derive(Clone, Debug)]
pub struct OpticFinder<T = ()> {
    kind: OpticFinderKind,
    _marker: PhantomData<T>,
}

impl OpticFinder<()> {
    pub fn field(name: impl Into<String>) -> Self {
        Self {
            kind: OpticFinderKind::Field(name.into()),
            _marker: PhantomData,
        }
    }
}

impl OpticFinder<Pair<String, String>> {
    pub fn text_component() -> Self {
        Self {
            kind: OpticFinderKind::TextComponent,
            _marker: PhantomData,
        }
    }
}

/// Minimal typed wrapper carrying an optional choice type tag and a dynamic value.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub type_name: Option<String>,
    pub value: Dynamic,
}

impl Typed {
    pub fn new(type_name: Option<String>, value: Dynamic) -> Self {
        Self { type_name, value }
    }

    pub fn get_choice_type_name(&self) -> &str {
        self.type_name.as_deref().unwrap_or("")
    }

    pub fn get_optional_typed(&self, finder: &OpticFinder<()>) -> Option<Typed> {
        match &finder.kind {
            OpticFinderKind::Field(name) => match &self.value {
                Dynamic::Map(m) => m.get(name).cloned().map(|v| Typed::new(None, v)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn get_optional(
        &self,
        finder: &OpticFinder<Pair<String, String>>,
    ) -> Option<Pair<String, String>> {
        match &finder.kind {
            OpticFinderKind::TextComponent => match &self.value {
                Dynamic::String(s) => Some(Pair(s.clone(), s.clone())),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Minimal type rewrite rule representation.
pub struct TypeRewriteRule {
    pub name: String,
    rule: Box<dyn Fn(Typed) -> Typed>,
}

impl TypeRewriteRule {
    pub fn apply(&self, typed: Typed) -> Typed {
        (self.rule)(typed)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BannerEntityCustomNameToOverrideComponentFix;

impl BannerEntityCustomNameToOverrideComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.BannerEntityCustomNameToOverrideComponentFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BannerEntityCustomNameToOverrideComponentFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let this = *self;
        let text_component_finder = OpticFinder::<Pair<String, String>>::text_component();
        let custom_name_finder = OpticFinder::<()>::field("CustomName");
        TypeRewriteRule {
            name: "Banner entity custom_name to item_name component fix".to_string(),
            rule: Box::new(move |typed: Typed| {
                let type_name = typed.get_choice_type_name().to_string();
                if type_name == "minecraft:banner" {
                    this.fix(typed, &text_component_finder, &custom_name_finder)
                } else {
                    typed
                }
            }),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BannerEntityCustomNameToOverrideComponentFix#fix
    fn fix(
        &self,
        typed: Typed,
        text_component_finder: &OpticFinder<Pair<String, String>>,
        custom_name_finder: &OpticFinder<()>,
    ) -> Typed {
        let name: Option<String> = typed
            .get_optional_typed(custom_name_finder)
            .and_then(|t| {
                t.get_optional(text_component_finder)
                    .map(|p| p.get_second().clone())
            });

        let is_ominous: bool = name
            .as_ref()
            .and_then(|s| extract_translation_string(s))
            .filter(|s| s == "block.minecraft.ominous_banner")
            .is_some();

        if is_ominous {
            let name_string = name.unwrap();
            write_and_read_typed_or_throw(typed, |d| {
                let components = Dynamic::create_map(vec![
                    (
                        Dynamic::create_string("minecraft:item_name"),
                        Dynamic::create_string(name_string),
                    ),
                    (
                        Dynamic::create_string("minecraft:hide_additional_tooltip"),
                        Dynamic::empty_map(),
                    ),
                ]);
                d.set("components", components).remove("CustomName")
            })
        } else {
            typed
        }
    }
}

fn write_and_read_typed_or_throw(typed: Typed, f: impl FnOnce(Dynamic) -> Dynamic) -> Typed {
    let new_value = f(typed.value);
    Typed {
        type_name: typed.type_name,
        value: new_value,
    }
}

fn extract_translation_string(component: &str) -> Option<String> {
    let trimmed = component.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    extract_json_string_field(trimmed, "translate")
}

fn extract_json_string_field(json: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\"", field);
    let bytes = json.as_bytes();
    let mut i = 0usize;
    let mut depth: i32 = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b']' => {
                depth -= 1;
                i += 1;
            }
            b'"' => {
                if depth == 1 && json[i..].starts_with(&needle) {
                    let after_key = i + needle.len();
                    let rest = &json[after_key..];
                    let rest_trimmed = rest.trim_start();
                    if let Some(colon_rest) = rest_trimmed.strip_prefix(':') {
                        let value_trimmed = colon_rest.trim_start();
                        if let Some(stripped) = value_trimmed.strip_prefix('"') {
                            if let Some(end) = find_json_string_end(stripped) {
                                return Some(unescape_json_string(&stripped[..end]));
                            }
                        }
                    }
                }
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i = i.saturating_add(2);
                        continue;
                    }
                    if bytes[i] == b'"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

fn find_json_string_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = i.saturating_add(2);
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u16::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code as u32) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ominous_banner_custom_name_converted() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:banner".to_string()),
            Dynamic::Map(BTreeMap::from([
                (
                    "id".to_string(),
                    Dynamic::String("minecraft:banner".to_string()),
                ),
                (
                    "CustomName".to_string(),
                    Dynamic::String(
                        "{\"translate\":\"block.minecraft.ominous_banner\",\"color\":\"gold\"}"
                            .to_string(),
                    ),
                ),
            ])),
        );

        let output = rule.apply(input);
        assert_eq!(output.get_choice_type_name(), "minecraft:banner");
        assert!(output.value.get("CustomName").is_none());
        let components = output.value.get("components").unwrap();
        assert_eq!(
            components.get("minecraft:item_name"),
            Some(&Dynamic::String(
                "{\"translate\":\"block.minecraft.ominous_banner\",\"color\":\"gold\"}"
                    .to_string(),
            ))
        );
        assert_eq!(
            components.get("minecraft:hide_additional_tooltip"),
            Some(&Dynamic::empty_map())
        );
    }

    #[test]
    fn non_banner_unchanged() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:chest".to_string()),
            Dynamic::Map(BTreeMap::from([(
                "CustomName".to_string(),
                Dynamic::String("{\"translate\":\"block.minecraft.ominous_banner\"}".to_string()),
            )])),
        );

        let expected = input.clone();
        let output = rule.apply(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn banner_with_other_translation_unchanged() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:banner".to_string()),
            Dynamic::Map(BTreeMap::from([(
                "CustomName".to_string(),
                Dynamic::String("{\"translate\":\"block.minecraft.banner\"}".to_string()),
            )])),
        );

        let expected = input.clone();
        let output = rule.apply(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn banner_without_custom_name_unchanged() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:banner".to_string()),
            Dynamic::Map(BTreeMap::from([(
                "id".to_string(),
                Dynamic::String("minecraft:banner".to_string()),
            )])),
        );

        let expected = input.clone();
        let output = rule.apply(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn ominous_banner_preserves_other_fields() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:banner".to_string()),
            Dynamic::Map(BTreeMap::from([
                (
                    "id".to_string(),
                    Dynamic::String("minecraft:banner".to_string()),
                ),
                ("x".to_string(), Dynamic::String("10".to_string())),
                (
                    "CustomName".to_string(),
                    Dynamic::String("{\"translate\":\"block.minecraft.ominous_banner\"}".to_string()),
                ),
            ])),
        );

        let output = rule.apply(input);
        assert!(output.value.get("CustomName").is_none());
        assert_eq!(
            output.value.get("id"),
            Some(&Dynamic::String("minecraft:banner".to_string()))
        );
        assert_eq!(
            output.value.get("x"),
            Some(&Dynamic::String("10".to_string()))
        );
        assert!(output.value.get("components").is_some());
    }

    #[test]
    fn banner_with_plain_text_custom_name_unchanged() {
        let fix = BannerEntityCustomNameToOverrideComponentFix::new();
        let rule = fix.make_rule();

        let input = Typed::new(
            Some("minecraft:banner".to_string()),
            Dynamic::Map(BTreeMap::from([(
                "CustomName".to_string(),
                Dynamic::String("{\"text\":\"block.minecraft.ominous_banner\"}".to_string()),
            )])),
        );

        let expected = input.clone();
        let output = rule.apply(input);
        assert_eq!(output, expected);
    }
}
