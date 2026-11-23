use std::fmt;
use std::marker::PhantomData;

/// Minimal stub for `com.mojang.brigadier.StringReader`.
#[derive(Debug, Clone)]
pub struct StringReader {
    input: String,
    cursor: usize,
}

impl StringReader {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub fn get_remaining(&self) -> &str {
        &self.input[self.cursor.min(self.input.len())..]
    }

    pub fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.cursor < bytes.len() {
            match bytes[self.cursor] {
                b' ' | b'\t' | b'\n' | b'\r' => self.cursor += 1,
                _ => break,
            }
        }
    }
}

/// Minimal stub for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal stub for `net.minecraft.util.parsing.packrat.ParseState`.
pub trait ParseState<I> {
    fn input(&mut self) -> &mut I;
    fn mark(&mut self) -> i32;
    fn restore(&mut self, cursor: i32);
}

/// Minimal stub for `net.minecraft.util.parsing.packrat.SuggestionSupplier`.
pub trait SuggestionSupplier<I> {
    fn possible_values<'a>(
        &'a self,
        state: &'a mut dyn ParseState<I>,
    ) -> Box<dyn Iterator<Item = String> + 'a>;
}

/// Port of `net.minecraft.util.parsing.packrat.commands.ResourceSuggestion`.
pub trait ResourceSuggestion: Send + Sync {
    // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceSuggestion#possibleResources
    fn possible_resources(&self) -> Box<dyn Iterator<Item = Identifier> + '_>;

    // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceSuggestion#possibleValues
    fn possible_values_default<'a>(
        &'a self,
        _state: &'a mut dyn ParseState<StringReader>,
    ) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(self.possible_resources().map(|id| id.to_string()))
    }
}

impl<T: ResourceSuggestion + ?Sized> SuggestionSupplier<StringReader> for T {
    fn possible_values<'a>(
        &'a self,
        state: &'a mut dyn ParseState<StringReader>,
    ) -> Box<dyn Iterator<Item = String> + 'a> {
        self.possible_values_default(state)
    }
}

/// Marker to keep generic parameter usage explicit in self-contained stubs.
#[allow(dead_code)]
fn _phantom_use<I>(_: PhantomData<I>) {}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockState {
        reader: StringReader,
    }

    impl MockState {
        fn new(s: &str) -> Self {
            Self {
                reader: StringReader::new(s),
            }
        }
    }

    impl ParseState<StringReader> for MockState {
        fn input(&mut self) -> &mut StringReader {
            &mut self.reader
        }
        fn mark(&mut self) -> i32 {
            self.reader.get_cursor() as i32
        }
        fn restore(&mut self, cursor: i32) {
            self.reader.set_cursor(cursor.max(0) as usize);
        }
    }

    struct FixedResources(Vec<Identifier>);

    impl ResourceSuggestion for FixedResources {
        // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceSuggestion#possibleResources
        fn possible_resources(&self) -> Box<dyn Iterator<Item = Identifier> + '_> {
            Box::new(self.0.iter().cloned())
        }
    }

    #[test]
    fn possible_values_maps_identifiers_to_strings() {
        let sugg = FixedResources(vec![
            Identifier::new("minecraft", "stone"),
            Identifier::new("minecraft", "dirt"),
        ]);
        let mut state = MockState::new("");
        let values: Vec<String> =
            SuggestionSupplier::<StringReader>::possible_values(&sugg, &mut state).collect();
        assert_eq!(values, vec!["minecraft:stone", "minecraft:dirt"]);
    }

    #[test]
    fn possible_values_empty_stream() {
        let sugg = FixedResources(vec![]);
        let mut state = MockState::new("abc");
        let values: Vec<String> =
            SuggestionSupplier::<StringReader>::possible_values(&sugg, &mut state).collect();
        assert!(values.is_empty());
    }

    #[test]
    fn possible_values_preserves_order_and_custom_namespace() {
        let sugg = FixedResources(vec![
            Identifier::new("custom", "zeta"),
            Identifier::new("minecraft", "alpha"),
            Identifier::new("other", "beta"),
        ]);
        let mut state = MockState::new("");
        let values: Vec<String> =
            SuggestionSupplier::<StringReader>::possible_values(&sugg, &mut state).collect();
        assert_eq!(
            values,
            vec!["custom:zeta", "minecraft:alpha", "other:beta"]
        );
    }

    #[test]
    fn possible_resources_returns_original_identifiers() {
        let sugg = FixedResources(vec![Identifier::new("ns", "path")]);
        let ids: Vec<Identifier> = sugg.possible_resources().collect();
        assert_eq!(ids, vec![Identifier::new("ns", "path")]);
    }
}
