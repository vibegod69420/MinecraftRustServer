//! Port of `net.minecraft.network.chat.ChatTypeDecoration` (Minecraft 1.21.11).

/// Minimal stand-in for `net.minecraft.ChatFormatting`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatFormatting {
    Gray,
}

/// Minimal stand-in for `net.minecraft.network.chat.Style`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub color: Option<ChatFormatting>,
    pub italic: Option<bool>,
}

impl Style {
    pub const EMPTY: Self = Self {
        color: None,
        italic: None,
    };

    pub fn with_color(mut self, color: ChatFormatting) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }
}

/// Minimal stand-in for `net.minecraft.network.chat.Component` content.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ComponentContent {
    #[default]
    Empty,
    Literal(String),
    Translatable {
        key: String,
        args: Vec<Component>,
    },
}

/// Minimal stand-in for `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Component {
    pub content: ComponentContent,
    pub style: Style,
}

impl Component {
    pub fn empty() -> Self {
        Self {
            content: ComponentContent::Empty,
            style: Style::EMPTY,
        }
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Literal(text.into()),
            style: Style::EMPTY,
        }
    }

    pub fn translatable(key: impl Into<String>, args: Vec<Component>) -> Self {
        Self {
            content: ComponentContent::Translatable {
                key: key.into(),
                args,
            },
            style: Style::EMPTY,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// Minimal stand-in for `net.minecraft.network.chat.ChatType.Bound`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ChatTypeBound {
    pub name: Component,
    pub target_name: Option<Component>,
}

/// Port of `net.minecraft.network.chat.ChatTypeDecoration.Parameter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Parameter {
    Sender,
    Target,
    Content,
}

impl Parameter {
    // JAVA: net.minecraft.network.chat.ChatTypeDecoration$Parameter#select
    pub fn select(&self, content: &Component, bound: &ChatTypeBound) -> Component {
        match self {
            Self::Sender => bound.name.clone(),
            Self::Target => bound.target_name.clone().unwrap_or_else(Component::empty),
            Self::Content => content.clone(),
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration$Parameter#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::Sender => "sender",
            Self::Target => "target",
            Self::Content => "content",
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            Self::Sender => 0,
            Self::Target => 1,
            Self::Content => 2,
        }
    }

    /// Mirrors `ByIdMap.continuous(..., OutOfBoundsStrategy.ZERO)`.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Target,
            2 => Self::Content,
            _ => Self::Sender,
        }
    }
}

/// Port of `net.minecraft.network.chat.ChatTypeDecoration`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatTypeDecoration {
    pub translation_key: String,
    pub parameters: Vec<Parameter>,
    pub style: Style,
}

impl ChatTypeDecoration {
    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#withSender
    pub fn with_sender(translation_key: impl Into<String>) -> Self {
        Self {
            translation_key: translation_key.into(),
            parameters: vec![Parameter::Sender, Parameter::Content],
            style: Style::EMPTY,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#incomingDirectMessage
    pub fn incoming_direct_message(translation_key: impl Into<String>) -> Self {
        let style = Style::EMPTY
            .with_color(ChatFormatting::Gray)
            .with_italic(true);
        Self {
            translation_key: translation_key.into(),
            parameters: vec![Parameter::Sender, Parameter::Content],
            style,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#outgoingDirectMessage
    pub fn outgoing_direct_message(translation_key: impl Into<String>) -> Self {
        let style = Style::EMPTY
            .with_color(ChatFormatting::Gray)
            .with_italic(true);
        Self {
            translation_key: translation_key.into(),
            parameters: vec![Parameter::Target, Parameter::Content],
            style,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#teamMessage
    pub fn team_message(translation_key: impl Into<String>) -> Self {
        Self {
            translation_key: translation_key.into(),
            parameters: vec![Parameter::Target, Parameter::Sender, Parameter::Content],
            style: Style::EMPTY,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#decorate
    pub fn decorate(&self, content: &Component, bound: &ChatTypeBound) -> Component {
        let args = self.resolve_parameters(content, bound);
        Component::translatable(self.translation_key.clone(), args).with_style(self.style)
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#resolveParameters
    fn resolve_parameters(&self, content: &Component, bound: &ChatTypeBound) -> Vec<Component> {
        let mut result: Vec<Component> = Vec::with_capacity(self.parameters.len());
        let mut i: usize = 0;
        while i < self.parameters.len() {
            let param = self.parameters[i];
            result.push(param.select(content, bound));
            i = i.wrapping_add(1);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bound() -> ChatTypeBound {
        ChatTypeBound {
            name: Component::literal("Alice"),
            target_name: Some(Component::literal("Bob")),
        }
    }

    #[test]
    fn with_sender_decoration_structure() {
        let dec = ChatTypeDecoration::with_sender("chat.type.text");
        assert_eq!(dec.translation_key, "chat.type.text");
        assert_eq!(
            dec.parameters,
            vec![Parameter::Sender, Parameter::Content]
        );
        assert_eq!(dec.style, Style::EMPTY);
    }

    #[test]
    fn decorate_resolves_parameters_and_applies_style() {
        let dec =
            ChatTypeDecoration::incoming_direct_message("commands.message.display.incoming");
        let content = Component::literal("hello");
        let bound = sample_bound();
        let result = dec.decorate(&content, &bound);

        let expected = Component::translatable(
            "commands.message.display.incoming",
            vec![Component::literal("Alice"), Component::literal("hello")],
        )
        .with_style(
            Style::EMPTY
                .with_color(ChatFormatting::Gray)
                .with_italic(true),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn target_parameter_falls_back_to_empty_when_missing() {
        let dec =
            ChatTypeDecoration::outgoing_direct_message("commands.message.display.outgoing");
        let content = Component::literal("secret");
        let bound = ChatTypeBound {
            name: Component::literal("Alice"),
            target_name: None,
        };
        let result = dec.decorate(&content, &bound);

        let args = match &result.content {
            ComponentContent::Translatable { args, .. } => args,
            _ => panic!("expected translatable component"),
        };
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Component::empty());
        assert_eq!(args[1], Component::literal("secret"));
    }

    #[test]
    fn team_message_parameter_order_and_resolution() {
        let dec = ChatTypeDecoration::team_message("chat.type.team.text");
        assert_eq!(
            dec.parameters,
            vec![Parameter::Target, Parameter::Sender, Parameter::Content]
        );
        assert_eq!(dec.style, Style::EMPTY);

        let result = dec.decorate(&Component::literal("go"), &sample_bound());
        let args = match result.content {
            ComponentContent::Translatable { args, .. } => args,
            _ => panic!("expected translatable component"),
        };
        assert_eq!(args[0], Component::literal("Bob"));
        assert_eq!(args[1], Component::literal("Alice"));
        assert_eq!(args[2], Component::literal("go"));
    }

    #[test]
    fn parameter_metadata_and_negative_id_edge_cases() {
        assert_eq!(Parameter::Sender.id(), 0);
        assert_eq!(Parameter::Target.id(), 1);
        assert_eq!(Parameter::Content.id(), 2);
        assert_eq!(Parameter::Sender.get_serialized_name(), "sender");
        assert_eq!(Parameter::Target.get_serialized_name(), "target");
        assert_eq!(Parameter::Content.get_serialized_name(), "content");

        // OutOfBoundsStrategy.ZERO: negative and out-of-range ids map to Sender.
        assert_eq!(Parameter::from_id(-1), Parameter::Sender);
        assert_eq!(Parameter::from_id(i32::MIN), Parameter::Sender);
        assert_eq!(Parameter::from_id(3), Parameter::Sender);
        assert_eq!(Parameter::from_id(i32::MAX), Parameter::Sender);
        assert_eq!(Parameter::from_id(1), Parameter::Target);
        assert_eq!(Parameter::from_id(2), Parameter::Content);
    }

    #[test]
    fn decorate_with_empty_parameters_produces_empty_args() {
        let dec = ChatTypeDecoration {
            translation_key: "empty.key".to_string(),
            parameters: vec![],
            style: Style::EMPTY,
        };
        let result = dec.decorate(&Component::literal("ignored"), &sample_bound());
        assert_eq!(result, Component::translatable("empty.key", vec![]));
    }
}
