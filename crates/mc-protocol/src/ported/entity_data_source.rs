use std::fmt;
use std::hash::{Hash, Hasher};

// ---- Stubs for external Minecraft/Brigadier types (self-contained) ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTag {
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Debug, Clone)]
pub struct CommandSourceStack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub tag: CompoundTag,
}

#[derive(Debug, Clone)]
pub struct EntitySelector {
    entities: Vec<Entity>,
}

impl EntitySelector {
    pub fn with_entities(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    pub fn find_entities(
        &self,
        _source: &CommandSourceStack,
    ) -> Result<Vec<Entity>, CommandSyntaxException> {
        Ok(self.entities.clone())
    }
}

pub struct NbtPredicate;

impl NbtPredicate {
    pub fn get_entity_tag_to_compare(entity: &Entity) -> CompoundTag {
        entity.tag.clone()
    }
}

pub struct StringReader {
    _input: String,
}

impl StringReader {
    pub fn new(input: &str) -> Self {
        Self {
            _input: input.to_string(),
        }
    }
}

pub struct EntitySelectorParser {
    _reader: StringReader,
    _allow_selector: bool,
}

impl EntitySelectorParser {
    pub fn new(reader: StringReader, allow_selector: bool) -> Self {
        Self {
            _reader: reader,
            _allow_selector: allow_selector,
        }
    }

    pub fn parse(&mut self) -> Result<EntitySelector, CommandSyntaxException> {
        Err(CommandSyntaxException {
            message: "unimplemented selector parser".to_string(),
        })
    }
}

pub struct MapCodec<T>(std::marker::PhantomData<T>);

// ---- Ported class: net.minecraft.network.chat.contents.data.EntityDataSource ----

#[derive(Debug, Clone)]
pub struct EntityDataSource {
    selector_pattern: String,
    compiled_selector: Option<EntitySelector>,
}

impl EntityDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#<init>
    pub fn new(selector_pattern: String) -> Self {
        let compiled_selector = Self::compile_selector(&selector_pattern);
        Self {
            selector_pattern,
            compiled_selector,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#<init>
    pub fn with_compiled(
        selector_pattern: String,
        compiled_selector: Option<EntitySelector>,
    ) -> Self {
        Self {
            selector_pattern,
            compiled_selector,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#compileSelector
    fn compile_selector(pattern: &str) -> Option<EntitySelector> {
        let mut parser = EntitySelectorParser::new(StringReader::new(pattern), true);
        match parser.parse() {
            Ok(selector) => Some(selector),
            Err(_e) => None,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#getData
    pub fn get_data(
        &self,
        source: &CommandSourceStack,
    ) -> Result<Vec<CompoundTag>, CommandSyntaxException> {
        if let Some(ref selector) = self.compiled_selector {
            let entities = selector.find_entities(source)?;
            Ok(entities
                .iter()
                .map(NbtPredicate::get_entity_tag_to_compare)
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#codec
    pub fn codec(&self) -> MapCodec<EntityDataSource> {
        MapCodec(std::marker::PhantomData)
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#selectorPattern
    pub fn selector_pattern(&self) -> &str {
        &self.selector_pattern
    }

    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#compiledSelector
    pub fn compiled_selector(&self) -> Option<&EntitySelector> {
        self.compiled_selector.as_ref()
    }
}

impl fmt::Display for EntityDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "entity={}", self.selector_pattern)
    }
}

impl PartialEq for EntityDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#equals
    fn eq(&self, other: &Self) -> bool {
        self.selector_pattern == other.selector_pattern
    }
}

impl Eq for EntityDataSource {}

impl Hash for EntityDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.EntityDataSource#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.selector_pattern.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn new_stores_pattern_and_compile_failure_yields_none() {
        let ds = EntityDataSource::new("@e".to_string());
        assert_eq!(ds.selector_pattern(), "@e");
        assert!(ds.compiled_selector().is_none());
    }

    #[test]
    fn get_data_returns_empty_when_no_compiled_selector() {
        let ds = EntityDataSource::new("@p".to_string());
        let source = CommandSourceStack;
        let data = ds.get_data(&source).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn get_data_returns_entity_tags_when_selector_present() {
        let selector = EntitySelector::with_entities(vec![
            Entity {
                tag: CompoundTag {
                    data: "a".to_string(),
                },
            },
            Entity {
                tag: CompoundTag {
                    data: "b".to_string(),
                },
            },
        ]);
        let ds = EntityDataSource::with_compiled("@e".to_string(), Some(selector));
        let source = CommandSourceStack;
        let data = ds.get_data(&source).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].data, "a");
        assert_eq!(data[1].data, "b");
    }

    #[test]
    fn equals_and_hash_are_based_on_pattern_only() {
        let selector = EntitySelector::with_entities(vec![Entity {
            tag: CompoundTag {
                data: "x".to_string(),
            },
        }]);
        let a = EntityDataSource::with_compiled("@e[type=pig]".to_string(), Some(selector));
        let b = EntityDataSource::new("@e[type=pig]".to_string());
        let c = EntityDataSource::new("@a".to_string());

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn to_string_formats_entity_pattern() {
        let ds = EntityDataSource::new("@a[distance=..5]".to_string());
        assert_eq!(ds.to_string(), "entity=@a[distance=..5]");
    }

    #[test]
    fn empty_pattern_is_preserved() {
        let ds = EntityDataSource::new(String::new());
        assert_eq!(ds.selector_pattern(), "");
        assert_eq!(ds.to_string(), "entity=");
        assert!(ds.get_data(&CommandSourceStack).unwrap().is_empty());
    }
}
