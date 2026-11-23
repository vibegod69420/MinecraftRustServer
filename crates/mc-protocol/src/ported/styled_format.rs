//! Port of `net.minecraft.network.chat.numbers.StyledFormat`.

/// Minimal placeholder for `net.minecraft.ChatFormatting`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFormatting {
    Red,
    Yellow,
}

/// Minimal placeholder for `net.minecraft.network.chat.Style`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub color: Option<ChatFormatting>,
}

impl Style {
    pub const EMPTY: Self = Self { color: None };

    /// Mirrors `Style#withColor(ChatFormatting)`.
    pub const fn with_color(self, color: ChatFormatting) -> Self {
        Self { color: Some(color) }
    }
}

/// Minimal placeholder for `net.minecraft.network.chat.MutableComponent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableComponent {
    pub text: String,
    pub style: Style,
}

impl MutableComponent {
    /// Mirrors `MutableComponent#withStyle(Style)`.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// Minimal placeholder for `net.minecraft.network.chat.Component`.
pub struct Component;

impl Component {
    /// Mirrors `Component#literal(String)`.
    pub fn literal(text: impl Into<String>) -> MutableComponent {
        MutableComponent {
            text: text.into(),
            style: Style::EMPTY,
        }
    }
}

/// Minimal placeholder for `net.minecraft.network.chat.numbers.NumberFormatType<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormatType;

impl NumberFormatType {
    // JAVA: net.minecraft.network.chat.numbers.StyledFormat$1#mapCodec
    pub fn map_codec(&self) {
        // Codec infrastructure is not ported; structural placeholder for 1:1 correspondence.
    }

    // JAVA: net.minecraft.network.chat.numbers.StyledFormat$1#streamCodec
    pub fn stream_codec(&self) {
        // Codec infrastructure is not ported; structural placeholder for 1:1 correspondence.
    }
}

/// Minimal placeholder for `net.minecraft.network.chat.numbers.NumberFormat`.
pub trait NumberFormat {
    fn format(&self, value: i32) -> MutableComponent;
    fn number_format_type(&self) -> NumberFormatType;
}

/// Port of `net.minecraft.network.chat.numbers.StyledFormat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledFormat {
    pub style: Style,
}

impl StyledFormat {
    pub const TYPE: NumberFormatType = NumberFormatType;
    pub const NO_STYLE: Self = Self { style: Style::EMPTY };
    pub const SIDEBAR_DEFAULT: Self = Self {
        style: Style::EMPTY.with_color(ChatFormatting::Red),
    };
    pub const PLAYER_LIST_DEFAULT: Self = Self {
        style: Style::EMPTY.with_color(ChatFormatting::Yellow),
    };

    // JAVA: net.minecraft.network.chat.numbers.StyledFormat#<init>
    pub const fn new(style: Style) -> Self {
        Self { style }
    }

    // JAVA: net.minecraft.network.chat.numbers.StyledFormat#style
    pub fn style(&self) -> &Style {
        &self.style
    }
}

impl NumberFormat for StyledFormat {
    // JAVA: net.minecraft.network.chat.numbers.StyledFormat#format
    fn format(&self, value: i32) -> MutableComponent {
        Component::literal(value.to_string()).with_style(self.style.clone())
    }

    // JAVA: net.minecraft.network.chat.numbers.StyledFormat#type
    fn number_format_type(&self) -> NumberFormatType {
        Self::TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_positive_zero_and_negative() {
        let fmt = StyledFormat::NO_STYLE;
        assert_eq!(fmt.format(0).text, "0");
        assert_eq!(fmt.format(42).text, "42");
        assert_eq!(fmt.format(-1).text, "-1");
        assert_eq!(fmt.format(-12_345).text, "-12345");
    }

    #[test]
    fn format_integer_extremes() {
        let fmt = StyledFormat::NO_STYLE;
        assert_eq!(fmt.format(i32::MAX).text, "2147483647");
        assert_eq!(fmt.format(i32::MIN).text, "-2147483648");
    }

    #[test]
    fn constants_have_expected_styles() {
        assert_eq!(StyledFormat::NO_STYLE.style(), &Style::EMPTY);
        assert_eq!(
            StyledFormat::SIDEBAR_DEFAULT.style(),
            &Style::EMPTY.with_color(ChatFormatting::Red)
        );
        assert_eq!(
            StyledFormat::PLAYER_LIST_DEFAULT.style(),
            &Style::EMPTY.with_color(ChatFormatting::Yellow)
        );
    }

    #[test]
    fn format_applies_style() {
        let style = Style::EMPTY.with_color(ChatFormatting::Red);
        let fmt = StyledFormat::new(style.clone());
        let component = fmt.format(7);
        assert_eq!(component.text, "7");
        assert_eq!(component.style, style);
    }

    #[test]
    fn type_returns_styled_format_type_constant() {
        let fmt = StyledFormat::NO_STYLE;
        assert_eq!(fmt.number_format_type(), StyledFormat::TYPE);
        let _ = StyledFormat::TYPE.map_codec();
        let _ = StyledFormat::TYPE.stream_codec();
    }
}
