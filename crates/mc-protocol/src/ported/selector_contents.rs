use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style;

pub trait ContentConsumer<T> {
    fn accept(&mut self, text: &str) -> Option<T>;
}

pub trait StyledContentConsumer<T> {
    fn accept(&mut self, style: &Style, text: &str) -> Option<T>;
}

impl<T, F: FnMut(&str) -> Option<T>> ContentConsumer<T> for F {
    fn accept(&mut self, text: &str) -> Option<T> {
        self(text)
    }
}

impl<T, F: FnMut(&Style, &str) -> Option<T>> StyledContentConsumer<T> for F {
    fn accept(&mut self, style: &Style, text: &str) -> Option<T> {
        self(style, text)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapCodecStub;

pub const MAP_CODEC: MapCodecStub = MapCodecStub;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    pub text: String,
}

impl Component {
    pub fn empty() -> MutableComponent {
        MutableComponent {
            text: String::new(),
        }
    }
}

impl MutableComponent {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Debug, Clone, Default)]
pub struct CommandSourceStack;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entity {
    pub display_name: String,
}

impl Entity {
    pub fn get_display_name(&self) -> Component {
        Component {
            text: self.display_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorPattern {
    pub pattern: String,
}

impl SelectorPattern {
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn resolved(&self) -> ResolvedSelector {
        ResolvedSelector
    }
}

impl fmt::Display for SelectorPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedSelector;

impl ResolvedSelector {
    pub fn find_entities(
        &self,
        _source: &CommandSourceStack,
    ) -> Result<Vec<Entity>, CommandSyntaxException> {
        Ok(Vec::new())
    }
}

pub struct ComponentUtils;

impl ComponentUtils {
    pub fn update_for_entity(
        _source: &CommandSourceStack,
        separator: Option<&Component>,
        _entity: Option<&Entity>,
        _depth: i32,
    ) -> Result<Option<Component>, CommandSyntaxException> {
        Ok(separator.cloned())
    }

    pub fn format_list(
        entities: Vec<Entity>,
        separator: Option<Component>,
        get_name: impl Fn(&Entity) -> Component,
    ) -> MutableComponent {
        if entities.is_empty() {
            return MutableComponent::empty();
        }

        let sep = separator
            .map(|c| c.text)
            .unwrap_or_else(|| ", ".to_string());
        let text = entities
            .iter()
            .map(|e| get_name(e).text)
            .collect::<Vec<_>>()
            .join(&sep);
        MutableComponent { text }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorContents {
    pub selector: SelectorPattern,
    pub separator: Option<Component>,
}

impl SelectorContents {
    // JAVA: net.minecraft.network.chat.contents.SelectorContents#selector
    pub fn selector(&self) -> &SelectorPattern {
        &self.selector
    }

    // JAVA: net.minecraft.network.chat.contents.SelectorContents#separator
    pub fn separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }

    // JAVA: net.minecraft.network.chat.contents.SelectorContents#codec
    pub fn codec(&self) -> MapCodecStub {
        MAP_CODEC
    }

    // JAVA: net.minecraft.network.chat.contents.SelectorContents#resolve
    pub fn resolve(
        &self,
        source: Option<&CommandSourceStack>,
        entity: Option<&Entity>,
        depth: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        let Some(source) = source else {
            return Ok(Component::empty());
        };

        let sep =
            ComponentUtils::update_for_entity(source, self.separator.as_ref(), entity, depth)?;
        Ok(ComponentUtils::format_list(
            self.selector.resolved().find_entities(source)?,
            sep,
            |e| e.get_display_name(),
        ))
    }

    // JAVA: net.minecraft.network.chat.contents.SelectorContents#visit
    pub fn visit_styled_content<T>(
        &self,
        consumer: &mut impl StyledContentConsumer<T>,
        style: &Style,
    ) -> Option<T> {
        consumer.accept(style, self.selector.pattern())
    }

    // JAVA: net.minecraft.network.chat.contents.SelectorContents#visit
    pub fn visit_content<T>(&self, consumer: &mut impl ContentConsumer<T>) -> Option<T> {
        consumer.accept(self.selector.pattern())
    }
}

// JAVA: net.minecraft.network.chat.contents.SelectorContents#toString
impl fmt::Display for SelectorContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pattern{{{}}}", self.selector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contents(pattern: &str, separator: Option<&str>) -> SelectorContents {
        SelectorContents {
            selector: SelectorPattern {
                pattern: pattern.to_string(),
            },
            separator: separator.map(|s| Component {
                text: s.to_string(),
            }),
        }
    }

    #[test]
    fn resolve_none_source_returns_empty() {
        let sc = sample_contents("@p", None);
        let result = sc.resolve(None, None, 0).unwrap();
        assert_eq!(result, MutableComponent::empty());
    }

    #[test]
    fn resolve_with_source_and_negative_depth() {
        let sc = sample_contents("@a", Some(", "));
        let source = CommandSourceStack;
        let result = sc.resolve(Some(&source), None, -1).unwrap();
        assert_eq!(result, MutableComponent::empty());
        let result2 = sc.resolve(Some(&source), Some(&Entity::default()), i32::MIN).unwrap();
        assert_eq!(result2, MutableComponent::empty());
    }

    #[test]
    fn visit_returns_selector_pattern() {
        let sc = sample_contents("@e[type=pig]", None);
        let mut content = |s: &str| -> Option<String> { Some(s.to_string()) };
        assert_eq!(sc.visit_content(&mut content), Some("@e[type=pig]".to_string()));

        let style = Style;
        let mut styled = |_st: &Style, s: &str| -> Option<String> { Some(s.to_string()) };
        assert_eq!(
            sc.visit_styled_content(&mut styled, &style),
            Some("@e[type=pig]".to_string())
        );
    }

    #[test]
    fn to_string_and_accessors() {
        let sc = sample_contents("@r", Some("|"));
        assert_eq!(sc.to_string(), "pattern{@r}");
        assert_eq!(sc.selector().pattern(), "@r");
        assert_eq!(sc.separator(), Some(&Component { text: "|".to_string() }));
        assert_eq!(sc.codec(), MAP_CODEC);
    }

    #[test]
    fn format_list_stub_behavior() {
        let empty = ComponentUtils::format_list(Vec::new(), None, |e| e.get_display_name());
        assert_eq!(empty, MutableComponent::empty());

        let entities = vec![
            Entity { display_name: "A".to_string() },
            Entity { display_name: "B".to_string() },
        ];
        let formatted = ComponentUtils::format_list(
            entities,
            Some(Component { text: " & ".to_string() }),
            |e| e.get_display_name(),
        );
        assert_eq!(formatted.text, "A & B");
    }
}
