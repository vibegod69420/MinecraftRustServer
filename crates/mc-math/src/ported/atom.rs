//! Port of `net.minecraft.util.parsing.packrat.Atom` (Java 1.21.11).
//!
//! A named atom used by the packrat parser.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A named parser atom.
///
/// Java record: `Atom<T>(String name)`.
///
/// The type parameter `T` is a phantom type tag used for type-safe parser
/// composition, matching Java's generic `Atom<T>`.
pub struct Atom<T> {
    name: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Atom<T> {
    // JAVA: net.minecraft.util.parsing.packrat.Atom#of
    pub fn of(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.Atom#name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> fmt::Display for Atom<T> {
    // JAVA: net.minecraft.util.parsing.packrat.Atom#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Atom<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T> Eq for Atom<T> {}

impl<T> Hash for Atom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T> fmt::Debug for Atom<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Atom").field("name", &self.name).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn of_and_name() {
        let atom: Atom<String> = Atom::of("foo");
        assert_eq!(atom.name(), "foo");
    }

    #[test]
    fn display_format_matches_java_tostring() {
        let atom: Atom<()> = Atom::of("bar");
        assert_eq!(atom.to_string(), "<bar>");
    }

    #[test]
    fn empty_name() {
        let atom: Atom<i32> = Atom::of("");
        assert_eq!(atom.name(), "");
        assert_eq!(atom.to_string(), "<>");
    }

    #[test]
    fn equality_same_name() {
        let a: Atom<String> = Atom::of("test");
        let b: Atom<String> = Atom::of("test");
        assert_eq!(a, b);
    }

    #[test]
    fn inequality_different_name() {
        let a: Atom<String> = Atom::of("a");
        let b: Atom<String> = Atom::of("b");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_consistent_with_equality() {
        let a: Atom<u64> = Atom::of("same");
        let b: Atom<u64> = Atom::of("same");
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn hash_differs_for_different_names() {
        let a: Atom<u64> = Atom::of("alpha");
        let b: Atom<u64> = Atom::of("beta");
        // Not strictly guaranteed by hash contract, but extremely unlikely to collide.
        assert_ne!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn clone_equals_original() {
        let a: Atom<Vec<u8>> = Atom::of("clone_me");
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(a.name(), b.name());
    }

    #[test]
    fn unicode_name() {
        let atom: Atom<()> = Atom::of("café_こんにちは_🚀");
        assert_eq!(atom.name(), "café_こんにちは_🚀");
        assert_eq!(atom.to_string(), "<café_こんにちは_🚀>");
    }

    #[test]
    fn special_characters_and_whitespace() {
        let atom: Atom<()> = Atom::of("a<b>c & d\n\t\"quoted\"");
        assert_eq!(atom.to_string(), "<a<b>c & d\n\t\"quoted\">");
    }

    #[test]
    fn name_with_null_byte() {
        let atom: Atom<()> = Atom::of("a\0b");
        assert_eq!(atom.name(), "a\0b");
        assert_eq!(atom.to_string(), "<a\0b>");
    }

    #[test]
    fn long_name() {
        let name = "x".repeat(10_000);
        let atom: Atom<()> = Atom::of(name.clone());
        assert_eq!(atom.name(), name.as_str());
        assert_eq!(atom.to_string(), format!("<{}>", name));
    }

    #[test]
    fn different_type_params_same_name_properties() {
        let a: Atom<i32> = Atom::of("num");
        let b: Atom<String> = Atom::of("num");
        // Different Rust types cannot be compared with ==, but their observable
        // properties match the same Java record component.
        assert_eq!(a.name(), b.name());
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn debug_contains_struct_and_name() {
        let atom: Atom<()> = Atom::of("debug_test");
        let dbg = format!("{:?}", atom);
        assert!(dbg.contains("Atom"));
        assert!(dbg.contains("debug_test"));
    }

    #[test]
    fn of_accepts_string_and_str() {
        let from_str: Atom<()> = Atom::of("literal");
        let from_string: Atom<()> = Atom::of(String::from("literal"));
        assert_eq!(from_str, from_string);
        assert_eq!(from_str.name(), "literal");
        assert_eq!(from_string.name(), "literal");
    }

    #[test]
    fn clone_works_without_t_clone_bound() {
        struct NoClone;
        let a: Atom<NoClone> = Atom::of("x");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn send_sync_without_t_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}
        // Rc<()> is !Send + !Sync, but Atom<T> should still be Send + Sync
        // because T is only a phantom type parameter.
        assert_send_sync::<Atom<std::rc::Rc<()>>>();
    }

    #[test]
    fn negative_and_numeric_like_names() {
        // Edge-case string values that look like negative/overflowing numbers.
        let cases = [
            "-1",
            "-2147483648",
            "2147483647",
            "9223372036854775807",
            "-9223372036854775808",
            "NaN",
            "-0.0",
            "1e308",
        ];
        for case in cases {
            let atom: Atom<()> = Atom::of(case);
            assert_eq!(atom.name(), case);
            assert_eq!(atom.to_string(), format!("<{}>", case));
        }
    }
}
