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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralContents {
    pub text: String,
}

impl LiteralContents {
    // JAVA: net.minecraft.network.chat.contents.PlainTextContents$LiteralContents#text
    pub fn text(&self) -> &str {
        &self.text
    }

    // JAVA: net.minecraft.network.chat.contents.PlainTextContents$LiteralContents#visit
    pub fn visit_content<T>(&self, consumer: &mut impl ContentConsumer<T>) -> Option<T> {
        consumer.accept(&self.text)
    }

    // JAVA: net.minecraft.network.chat.contents.PlainTextContents$LiteralContents#visit
    pub fn visit_styled_content<T>(
        &self,
        consumer: &mut impl StyledContentConsumer<T>,
        style: &Style,
    ) -> Option<T> {
        consumer.accept(style, &self.text)
    }
}

// JAVA: net.minecraft.network.chat.contents.PlainTextContents$LiteralContents#toString
impl fmt::Display for LiteralContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "literal{{{}}}", self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlainTextContents {
    Empty,
    Literal(LiteralContents),
}

impl PlainTextContents {
    pub const EMPTY: Self = Self::Empty;

    // JAVA: net.minecraft.network.chat.contents.PlainTextContents#create
    pub fn create(text: String) -> Self {
        if text.is_empty() {
            Self::Empty
        } else {
            Self::Literal(LiteralContents { text })
        }
    }

    // JAVA: net.minecraft.network.chat.contents.PlainTextContents#text
    pub fn text(&self) -> &str {
        match self {
            Self::Empty => "",
            Self::Literal(lit) => lit.text(),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.PlainTextContents#codec
    pub fn codec(&self) -> MapCodecStub {
        MAP_CODEC
    }
}

// JAVA: net.minecraft.network.chat.contents.PlainTextContents$1#toString
impl fmt::Display for PlainTextContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty"),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_returns_empty() {
        let pt = PlainTextContents::create(String::new());
        assert_eq!(pt, PlainTextContents::EMPTY);
        assert_eq!(pt.text(), "");
        assert_eq!(pt.to_string(), "empty");
    }

    #[test]
    fn create_non_empty_returns_literal() {
        let pt = PlainTextContents::create("hello".to_string());
        assert_eq!(pt.text(), "hello");
        match &pt {
            PlainTextContents::Literal(lit) => assert_eq!(lit.text(), "hello"),
            PlainTextContents::Empty => panic!("expected literal"),
        }
        assert_eq!(pt.to_string(), "literal{hello}");
    }

    #[test]
    fn literal_visit_content() {
        let lit = LiteralContents {
            text: "test".to_string(),
        };
        let mut consumer = |s: &str| -> Option<usize> { Some(s.len()) };
        assert_eq!(lit.visit_content(&mut consumer), Some(4));

        let mut none_consumer = |_s: &str| -> Option<usize> { None };
        assert_eq!(lit.visit_content(&mut none_consumer), None);
    }

    #[test]
    fn literal_visit_styled_content() {
        let lit = LiteralContents {
            text: "ok".to_string(),
        };
        let style = Style;
        let mut consumer = |st: &Style, s: &str| -> Option<(Style, String)> {
            Some((st.clone(), s.to_uppercase()))
        };
        assert_eq!(
            lit.visit_styled_content(&mut consumer, &style),
            Some((Style, "OK".to_string()))
        );
    }

    #[test]
    fn codec_returns_map_codec_stub() {
        let pt = PlainTextContents::create("x".to_string());
        assert_eq!(pt.codec(), MAP_CODEC);
        assert_eq!(PlainTextContents::EMPTY.codec(), MAP_CODEC);
    }

    #[test]
    fn empty_and_unicode_edge_cases() {
        let lit = LiteralContents {
            text: String::new(),
        };
        assert_eq!(lit.to_string(), "literal{}");
        assert_eq!(lit.text(), "");

        let unicode = PlainTextContents::create("§c❤\n\t".to_string());
        assert_eq!(unicode.text(), "§c❤\n\t");
        assert_eq!(unicode.to_string(), "literal{§c❤\n\t}");
    }
}
