//! Port of `net.minecraft.network.chat.contents.KeybindContents`.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

/// Minimal stub for `net.minecraft.network.chat.Style`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Style;

/// Minimal stub for `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn get_string(&self) -> &str {
        &self.text
    }

    pub fn visit_content<T>(&self, consumer: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        consumer(&self.text)
    }

    pub fn visit_styled<T>(
        &self,
        consumer: &mut impl FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        consumer(style, &self.text)
    }
}

/// Minimal stub for Mojang serialization `MapCodec`.
pub struct MapCodec<T>(std::marker::PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T> Clone for MapCodec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MapCodec<T> {}

impl<T> fmt::Debug for MapCodec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec")
    }
}

impl<T> PartialEq for MapCodec<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for MapCodec<T> {}

/// Minimal stub for `net.minecraft.network.chat.ComponentContents`.
pub trait ComponentContents {
    fn visit_content<T>(&self, consumer: &mut impl FnMut(&str) -> Option<T>) -> Option<T>;
    fn visit_styled<T>(
        &self,
        consumer: &mut impl FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T>;
}

/// Minimal stub for `net.minecraft.network.chat.contents.KeybindResolver`.
pub mod keybind_resolver {
    use super::Component;
    use std::sync::{OnceLock, RwLock};

    pub type NameSupplier = Box<dyn Fn() -> Component + Send + Sync>;
    pub type KeyResolverFn = Box<dyn Fn(&str) -> NameSupplier + Send + Sync>;

    fn default_resolver(name: &str) -> NameSupplier {
        let owned = name.to_owned();
        Box::new(move || Component::literal(owned.clone()))
    }

    fn resolver_cell() -> &'static RwLock<KeyResolverFn> {
        static KEY_RESOLVER: OnceLock<RwLock<KeyResolverFn>> = OnceLock::new();
        KEY_RESOLVER.get_or_init(|| RwLock::new(Box::new(default_resolver)))
    }

    pub fn resolve(name: &str) -> NameSupplier {
        let guard = resolver_cell()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard(name)
    }

    pub fn set_key_resolver(resolver: KeyResolverFn) {
        let mut guard = resolver_cell()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = resolver;
    }

    pub fn reset_key_resolver() {
        set_key_resolver(Box::new(default_resolver));
    }
}

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for cu in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(cu as i32);
    }
    h
}

// JAVA: net.minecraft.network.chat.contents.KeybindContents#MAP_CODEC
pub static MAP_CODEC: MapCodec<KeybindContents> = MapCodec::new();

/// Port of `net.minecraft.network.chat.contents.KeybindContents`.
pub struct KeybindContents {
    name: String,
    name_resolver: OnceLock<keybind_resolver::NameSupplier>,
}

impl KeybindContents {
    // JAVA: net.minecraft.network.chat.contents.KeybindContents#<init>
    pub fn new(name: String) -> Self {
        Self {
            name,
            name_resolver: OnceLock::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#getNestedComponent
    fn get_nested_component(&self) -> Component {
        let supplier = self
            .name_resolver
            .get_or_init(|| keybind_resolver::resolve(&self.name));
        supplier()
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#visit(FormattedText$ContentConsumer)
    pub fn visit_content<T>(&self, consumer: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        self.get_nested_component().visit_content(consumer)
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#visit(FormattedText$StyledContentConsumer, Style)
    pub fn visit_styled<T>(
        &self,
        consumer: &mut impl FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        self.get_nested_component().visit_styled(consumer, style)
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#getName
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#codec
    pub fn codec(&self) -> MapCodec<KeybindContents> {
        MAP_CODEC
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#hashCode
    pub fn hash_code(&self) -> i32 {
        java_string_hash(&self.name)
    }
}

impl ComponentContents for KeybindContents {
    // JAVA: net.minecraft.network.chat.contents.KeybindContents#visit(FormattedText$ContentConsumer)
    fn visit_content<T>(&self, consumer: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        KeybindContents::visit_content(self, consumer)
    }

    // JAVA: net.minecraft.network.chat.contents.KeybindContents#visit(FormattedText$StyledContentConsumer, Style)
    fn visit_styled<T>(
        &self,
        consumer: &mut impl FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        KeybindContents::visit_styled(self, consumer, style)
    }
}

impl PartialEq for KeybindContents {
    // JAVA: net.minecraft.network.chat.contents.KeybindContents#equals
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for KeybindContents {}

impl Hash for KeybindContents {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl fmt::Display for KeybindContents {
    // JAVA: net.minecraft.network.chat.contents.KeybindContents#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "keybind{{{}}}", self.name)
    }
}

impl fmt::Debug for KeybindContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeybindContents")
            .field("name", &self.name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_get_name() {
        let contents = KeybindContents::new("key.jump".to_string());
        assert_eq!(contents.get_name(), "key.jump");

        let empty = KeybindContents::new(String::new());
        assert_eq!(empty.get_name(), "");
    }

    #[test]
    fn to_string_format() {
        let contents = KeybindContents::new("key.inventory".to_string());
        assert_eq!(contents.to_string(), "keybind{key.inventory}");
        assert_eq!(
            KeybindContents::new(String::new()).to_string(),
            "keybind{}"
        );
    }

    #[test]
    fn equals_and_hash_code() {
        let a = KeybindContents::new("key.forward".to_string());
        let b = KeybindContents::new("key.forward".to_string());
        let c = KeybindContents::new("key.back".to_string());

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.hash_code(), b.hash_code());

        // Java String.hashCode() semantics (wrapping i32)
        assert_eq!(KeybindContents::new(String::new()).hash_code(), 0);
        assert_eq!(KeybindContents::new("a".to_string()).hash_code(), 97);
        assert_eq!(KeybindContents::new("abc".to_string()).hash_code(), 96354);

        // Non-BMP: hashed as UTF-16 code units
        let emoji = KeybindContents::new("😀".to_string());
        let mut expected: i32 = 0;
        for cu in "😀".encode_utf16() {
            expected = expected.wrapping_mul(31).wrapping_add(cu as i32);
        }
        assert_eq!(emoji.hash_code(), expected);

        // Overflow/wrapping case
        let long_name = "a".repeat(100);
        let mut long_expected: i32 = 0;
        for cu in long_name.encode_utf16() {
            long_expected = long_expected.wrapping_mul(31).wrapping_add(cu as i32);
        }
        assert_eq!(
            KeybindContents::new(long_name).hash_code(),
            long_expected
        );
    }

    #[test]
    fn visit_content_delegates_to_resolved_component() {
        let contents = KeybindContents::new("key.jump".to_string());
        let mut seen = String::new();
        let result: Option<()> = contents.visit_content(&mut |text| {
            seen.push_str(text);
            None
        });
        assert_eq!(result, None);
        assert_eq!(seen, "key.jump");

        let found: Option<String> =
            ComponentContents::visit_content(&contents, &mut |text| Some(text.to_string()));
        assert_eq!(found.as_deref(), Some("key.jump"));

        let again: Option<String> = contents.visit_content(&mut |text| Some(text.to_string()));
        assert_eq!(again, found);
    }

    #[test]
    fn visit_styled_delegates_to_resolved_component() {
        let contents = KeybindContents::new("key.sneak".to_string());
        let style = Style;
        let mut seen = String::new();
        let mut seen_style = None;
        let result: Option<i32> = contents.visit_styled(
            &mut |s, text| {
                seen_style = Some(s.clone());
                seen.push_str(text);
                Some(42)
            },
            &style,
        );
        assert_eq!(result, Some(42));
        assert_eq!(seen, "key.sneak");
        assert_eq!(seen_style, Some(Style));
    }

    #[test]
    fn codec_returns_static_map_codec() {
        let contents = KeybindContents::new("key.test".to_string());
        let codec = contents.codec();
        assert_eq!(codec, MAP_CODEC);
    }
}
