use std::fmt;
use std::sync::LazyLock;

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

#[derive(Debug, Clone, Default)]
pub struct CommandSourceStack;

#[derive(Debug, Clone, Default)]
pub struct Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatFormatting {
    Gray,
    Green,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClickEvent {
    CopyToClipboard(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HoverEvent {
    ShowText(Box<Component>),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Style {
    pub color: Option<ChatFormatting>,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub insertion: Option<String>,
}

impl Style {
    pub fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.click_event.is_none()
            && self.hover_event.is_none()
            && self.insertion.is_none()
    }

    pub fn apply_to(&self, other: Style) -> Style {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self.clone();
        }
        Style {
            color: self.color.or(other.color),
            click_event: self.click_event.clone().or(other.click_event),
            hover_event: self.hover_event.clone().or(other.hover_event),
            insertion: self.insertion.clone().or(other.insertion),
        }
    }

    pub fn with_color(mut self, color: ChatFormatting) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_click_event(mut self, event: ClickEvent) -> Self {
        self.click_event = Some(event);
        self
    }

    pub fn with_hover_event(mut self, event: HoverEvent) -> Self {
        self.hover_event = Some(event);
        self
    }

    pub fn with_insertion(mut self, insertion: impl Into<String>) -> Self {
        self.insertion = Some(insertion.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentContents {
    Literal(String),
    Translatable {
        key: String,
        fallback: Option<String>,
        args: Vec<Component>,
    },
    Empty,
}

impl ComponentContents {
    pub fn resolve(
        &self,
        _source: Option<&CommandSourceStack>,
        _entity: Option<&Entity>,
        _depth: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        match self {
            ComponentContents::Literal(s) => Ok(Component::literal(s)),
            ComponentContents::Empty => Ok(Component::empty()),
            ComponentContents::Translatable { key, fallback, .. } => {
                let text = fallback.clone().unwrap_or_else(|| key.clone());
                Ok(Component::literal(text))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    contents: ComponentContents,
    siblings: Vec<Component>,
    style: Style,
}

pub type MutableComponent = Component;

impl Component {
    pub fn literal(text: impl Into<String>) -> MutableComponent {
        Component {
            contents: ComponentContents::Literal(text.into()),
            siblings: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn empty() -> MutableComponent {
        Component {
            contents: ComponentContents::Empty,
            siblings: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn translatable(key: impl Into<String>, args: Vec<Component>) -> MutableComponent {
        Component {
            contents: ComponentContents::Translatable {
                key: key.into(),
                fallback: None,
                args,
            },
            siblings: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn get_contents(&self) -> &ComponentContents {
        &self.contents
    }

    pub fn get_siblings(&self) -> &[Component] {
        &self.siblings
    }

    pub fn copy(&self) -> MutableComponent {
        self.clone()
    }

    pub fn get_string(&self) -> String {
        let mut out = match &self.contents {
            ComponentContents::Literal(s) => s.clone(),
            ComponentContents::Empty => String::new(),
            ComponentContents::Translatable { key, fallback, .. } => {
                fallback.clone().unwrap_or_else(|| key.clone())
            }
        };
        for sibling in &self.siblings {
            out.push_str(&sibling.get_string());
        }
        out
    }

    pub fn set_style(mut self, style: Style) -> MutableComponent {
        self.style = style;
        self
    }

    pub fn with_style(mut self, style: Style) -> MutableComponent {
        self.style = style.apply_to(self.style);
        self
    }

    pub fn with_style_fn(mut self, f: impl FnOnce(Style) -> Style) -> MutableComponent {
        self.style = f(self.style);
        self
    }

    pub fn with_chat_formatting(self, fmt: ChatFormatting) -> MutableComponent {
        let style = self.style.clone().with_color(fmt);
        self.with_style(style)
    }

    pub fn append(mut self, component: Component) -> MutableComponent {
        self.siblings.push(component);
        self
    }
}

pub trait BrigadierMessage {
    fn get_string(&self) -> String;
    fn as_component_clone(&self) -> Option<Component> {
        None
    }
}

impl BrigadierMessage for Component {
    fn get_string(&self) -> String {
        Component::get_string(self)
    }

    fn as_component_clone(&self) -> Option<Component> {
        Some(self.clone())
    }
}

pub struct LiteralMessage(pub String);

impl BrigadierMessage for LiteralMessage {
    fn get_string(&self) -> String {
        self.0.clone()
    }
}

pub struct Language;

impl Language {
    pub fn get_instance() -> Language {
        Language
    }

    pub fn has(&self, _key: &str) -> bool {
        false
    }
}

pub const DEFAULT_SEPARATOR_TEXT: &str = ", ";

pub static DEFAULT_SEPARATOR: LazyLock<Component> = LazyLock::new(|| {
    Component::literal(", ").with_chat_formatting(ChatFormatting::Gray)
});

pub static DEFAULT_NO_STYLE_SEPARATOR: LazyLock<Component> =
    LazyLock::new(|| Component::literal(", "));

// JAVA: net.minecraft.network.chat.ComponentUtils#mergeStyles
pub fn merge_styles_mutable(component: MutableComponent, style: Style) -> MutableComponent {
    if style.is_empty() {
        return component;
    }
    let current = component.get_style().clone();
    if current.is_empty() {
        return component.set_style(style);
    }
    if current == style {
        return component;
    }
    component.set_style(current.apply_to(style))
}

// JAVA: net.minecraft.network.chat.ComponentUtils#mergeStyles
pub fn merge_styles(component: Component, style: Style) -> Component {
    if style.is_empty() {
        return component;
    }
    let current = component.get_style().clone();
    if current.is_empty() {
        return component.copy().set_style(style);
    }
    if current == style {
        return component;
    }
    component.copy().set_style(current.apply_to(style))
}

// JAVA: net.minecraft.network.chat.ComponentUtils#updateForEntity
pub fn update_for_entity_optional(
    source: Option<&CommandSourceStack>,
    component: Option<Component>,
    entity: Option<&Entity>,
    depth: i32,
) -> Result<Option<MutableComponent>, CommandSyntaxException> {
    match component {
        Some(c) => Ok(Some(update_for_entity(source, c, entity, depth)?)),
        None => Ok(None),
    }
}

// JAVA: net.minecraft.network.chat.ComponentUtils#updateForEntity
pub fn update_for_entity(
    source: Option<&CommandSourceStack>,
    component: Component,
    entity: Option<&Entity>,
    depth: i32,
) -> Result<MutableComponent, CommandSyntaxException> {
    if depth > 100 {
        return Ok(component.copy());
    }

    let mut result = component
        .get_contents()
        .resolve(source, entity, depth.wrapping_add(1))?;

    for sibling in component.get_siblings() {
        let resolved_sibling =
            update_for_entity(source, sibling.clone(), entity, depth.wrapping_add(1))?;
        result = result.append(resolved_sibling);
    }

    let resolved_style = resolve_style(source, component.get_style().clone(), entity, depth)?;
    Ok(result.with_style(resolved_style))
}

// JAVA: net.minecraft.network.chat.ComponentUtils#resolveStyle
fn resolve_style(
    source: Option<&CommandSourceStack>,
    style: Style,
    entity: Option<&Entity>,
    depth: i32,
) -> Result<Style, CommandSyntaxException> {
    if let Some(HoverEvent::ShowText(text)) = style.hover_event.clone() {
        let resolved = update_for_entity(source, *text, entity, depth.wrapping_add(1))?;
        return Ok(style.with_hover_event(HoverEvent::ShowText(Box::new(resolved))));
    }
    Ok(style)
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatList
pub fn format_string_list(items: &[String]) -> Component {
    format_and_sort_list(items, |s: &String| {
        Component::literal(s.as_str()).with_chat_formatting(ChatFormatting::Green)
    })
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatAndSortList
pub fn format_and_sort_list<T: Ord + Clone, F>(items: &[T], to_component: F) -> Component
where
    F: Fn(&T) -> Component,
{
    if items.is_empty() {
        return Component::empty();
    }
    if items.len() == 1 {
        return to_component(&items[0]);
    }
    let mut sorted: Vec<T> = items.to_vec();
    sorted.sort();
    format_list(&sorted, &DEFAULT_SEPARATOR, to_component)
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatList
pub fn format_list_default<T, F>(items: &[T], to_component: F) -> Component
where
    F: Fn(&T) -> Component,
{
    format_list(items, &DEFAULT_SEPARATOR, to_component)
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatList
pub fn format_list_optional_separator<T, F>(
    items: &[T],
    separator: Option<&Component>,
    to_component: F,
) -> MutableComponent
where
    F: Fn(&T) -> Component,
{
    let sep = separator.unwrap_or(&DEFAULT_SEPARATOR);
    format_list(items, sep, to_component)
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatList
pub fn format_component_list(items: &[Component], separator: &Component) -> Component {
    format_list(items, separator, |c: &Component| c.clone())
}

// JAVA: net.minecraft.network.chat.ComponentUtils#formatList
pub fn format_list<T, F>(items: &[T], separator: &Component, to_component: F) -> MutableComponent
where
    F: Fn(&T) -> Component,
{
    if items.is_empty() {
        return Component::empty();
    }
    if items.len() == 1 {
        return to_component(&items[0]).copy();
    }
    let mut result = Component::empty();
    let mut first = true;
    for item in items {
        if !first {
            result = result.append(separator.clone());
        }
        result = result.append(to_component(item));
        first = false;
    }
    result
}

// JAVA: net.minecraft.network.chat.ComponentUtils#wrapInSquareBrackets
pub fn wrap_in_square_brackets(component: Component) -> MutableComponent {
    Component::translatable("chat.square_brackets", vec![component])
}

// JAVA: net.minecraft.network.chat.ComponentUtils#fromMessage
pub fn from_message(msg: &dyn BrigadierMessage) -> Component {
    if let Some(c) = msg.as_component_clone() {
        c
    } else {
        Component::literal(msg.get_string())
    }
}

// JAVA: net.minecraft.network.chat.ComponentUtils#isTranslationResolvable
pub fn is_translation_resolvable(component: Option<&Component>) -> bool {
    match component {
        Some(c) => match c.get_contents() {
            ComponentContents::Translatable { key, fallback, .. } => {
                fallback.is_some() || Language::get_instance().has(key)
            }
            _ => true,
        },
        None => true,
    }
}

// JAVA: net.minecraft.network.chat.ComponentUtils#copyOnClickText
pub fn copy_on_click_text(text: &str) -> MutableComponent {
    let text_owned = text.to_string();
    wrap_in_square_brackets(Component::literal(text).with_style_fn({
        let text_for_click = text_owned.clone();
        let text_for_insert = text_owned;
        move |style| {
            style
                .with_color(ChatFormatting::Green)
                .with_click_event(ClickEvent::CopyToClipboard(text_for_click))
                .with_hover_event(HoverEvent::ShowText(Box::new(Component::translatable(
                    "chat.copy.click",
                    vec![],
                ))))
                .with_insertion(text_for_insert)
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_styles_edge_cases() {
        let empty_style = Style::default();
        let green = Style::default().with_color(ChatFormatting::Green);
        let gray = Style::default().with_color(ChatFormatting::Gray);

        let base = Component::literal("a");
        let unchanged = merge_styles(base.clone(), empty_style.clone());
        assert_eq!(unchanged.get_style(), base.get_style());

        let green_comp = Component::literal("b").set_style(green.clone());
        let same = merge_styles(green_comp.clone(), green.clone());
        assert_eq!(same.get_style(), &green);

        let applied = merge_styles(Component::literal("c"), green.clone());
        assert_eq!(applied.get_style().color, Some(ChatFormatting::Green));

        let kept_existing = merge_styles_mutable(Component::literal("d").set_style(gray.clone()), green);
        assert_eq!(kept_existing.get_style().color, Some(ChatFormatting::Gray));
        assert!(empty_style.is_empty());
    }

    #[test]
    fn test_format_list_empty_single_multiple() {
        let empty: Vec<Component> = vec![];
        let r0 = format_component_list(&empty, &DEFAULT_NO_STYLE_SEPARATOR);
        assert_eq!(r0.get_string(), "");

        let single = vec![Component::literal("one")];
        let r1 = format_component_list(&single, &DEFAULT_NO_STYLE_SEPARATOR);
        assert_eq!(r1.get_string(), "one");

        let multi = vec![
            Component::literal("a"),
            Component::literal("b"),
            Component::literal("c"),
        ];
        let r2 = format_component_list(&multi, &DEFAULT_NO_STYLE_SEPARATOR);
        assert_eq!(r2.get_string(), "a, b, c");
    }

    #[test]
    fn test_format_and_sort_list_with_negative_and_extreme_integers() {
        let items: Vec<i32> = vec![1, i32::MIN, 0, i32::MAX, -1];
        let c = format_and_sort_list(&items, |n: &i32| Component::literal(n.to_string()));
        assert_eq!(
            c.get_string(),
            format!("{}, {}, {}, {}, {}", i32::MIN, -1, 0, 1, i32::MAX)
        );
    }

    #[test]
    fn test_update_for_entity_depth_limit_and_negative_depth() {
        let source = None;
        let entity = None;
        let c = Component::literal("root").append(Component::literal("child"));
        let resolved = update_for_entity(source, c.clone(), entity, 0).unwrap();
        assert_eq!(resolved.get_string(), "rootchild");

        let deep = update_for_entity(source, c.clone(), entity, 101).unwrap();
        assert_eq!(deep.get_string(), "rootchild");

        let at_max = update_for_entity(source, Component::literal("x"), entity, i32::MAX).unwrap();
        assert_eq!(at_max.get_string(), "x");

        let at_min = update_for_entity(source, Component::literal("y"), entity, i32::MIN).unwrap();
        assert_eq!(at_min.get_string(), "y");
    }

    #[test]
    fn test_is_translation_resolvable_and_copy_on_click() {
        assert!(is_translation_resolvable(None));
        assert!(is_translation_resolvable(Some(&Component::literal("hi"))));

        let with_fallback = Component {
            contents: ComponentContents::Translatable {
                key: "missing.key".to_string(),
                fallback: Some("fallback".to_string()),
                args: vec![],
            },
            siblings: vec![],
            style: Style::default(),
        };
        assert!(is_translation_resolvable(Some(&with_fallback)));

        let no_fallback = Component {
            contents: ComponentContents::Translatable {
                key: "missing.key".to_string(),
                fallback: None,
                args: vec![],
            },
            siblings: vec![],
            style: Style::default(),
        };
        assert!(!is_translation_resolvable(Some(&no_fallback)));

        let c = copy_on_click_text("abc");
        assert_eq!(c.get_string(), "chat.square_brackets");
        match c.get_contents() {
            ComponentContents::Translatable { key, args, .. } => {
                assert_eq!(key, "chat.square_brackets");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].get_string(), "abc");
                assert_eq!(args[0].get_style().color, Some(ChatFormatting::Green));
                assert!(matches!(
                    args[0].get_style().click_event,
                    Some(ClickEvent::CopyToClipboard(ref s)) if s == "abc"
                ));
                assert_eq!(args[0].get_style().insertion.as_deref(), Some("abc"));
            }
            _ => panic!("expected translatable"),
        }
    }

    #[test]
    fn test_from_message_and_float_bits_literal() {
        let bits: u64 = 0xBFF8_0000_0000_0001u64;
        let f = f64::from_bits(bits);
        let text = format!("{:016X}", f.to_bits());
        let c = Component::literal(&text);
        assert_eq!(c.get_string(), text);
        assert_eq!(f.to_bits(), bits);

        let msg = LiteralMessage("hello".to_string());
        let from = from_message(&msg);
        assert_eq!(from.get_string(), "hello");

        let comp = Component::literal("direct");
        let from_comp = from_message(&comp);
        assert_eq!(from_comp.get_string(), "direct");
    }
}
