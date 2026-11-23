#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub fn empty() -> Self {
        Self {
            bold: false,
            italic: false,
        }
    }

    // JAVA: net.minecraft.network.chat.Style#applyTo
    pub fn apply_to(&self, other: &Style) -> Style {
        Style {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
        }
    }
}

pub const STOP_ITERATION: Option<()> = Some(());

// JAVA: net.minecraft.network.chat.FormattedText$ContentConsumer
pub trait ContentConsumer<T> {
    // JAVA: net.minecraft.network.chat.FormattedText$ContentConsumer#accept
    fn accept(&mut self, text: &str) -> Option<T>;
}

impl<T, F: FnMut(&str) -> Option<T>> ContentConsumer<T> for F {
    // JAVA: net.minecraft.network.chat.FormattedText$ContentConsumer#accept
    fn accept(&mut self, text: &str) -> Option<T> {
        self(text)
    }
}

// JAVA: net.minecraft.network.chat.FormattedText$StyledContentConsumer
pub trait StyledContentConsumer<T> {
    // JAVA: net.minecraft.network.chat.FormattedText$StyledContentConsumer#accept
    fn accept(&mut self, style: &Style, text: &str) -> Option<T>;
}

impl<T, F: FnMut(&Style, &str) -> Option<T>> StyledContentConsumer<T> for F {
    // JAVA: net.minecraft.network.chat.FormattedText$StyledContentConsumer#accept
    fn accept(&mut self, style: &Style, text: &str) -> Option<T> {
        self(style, text)
    }
}

// JAVA: net.minecraft.network.chat.FormattedText
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormattedText {
    Empty,
    Literal(String),
    StyledLiteral(String, Style),
    Composite(Vec<FormattedText>),
}

impl FormattedText {
    pub const EMPTY: FormattedText = FormattedText::Empty;

    // JAVA: net.minecraft.network.chat.FormattedText#of
    pub fn of(text: impl Into<String>) -> Self {
        FormattedText::Literal(text.into())
    }

    // JAVA: net.minecraft.network.chat.FormattedText#of
    pub fn of_styled(text: impl Into<String>, style: Style) -> Self {
        FormattedText::StyledLiteral(text.into(), style)
    }

    // JAVA: net.minecraft.network.chat.FormattedText#composite
    pub fn composite(parts: Vec<FormattedText>) -> Self {
        FormattedText::Composite(parts)
    }

    // JAVA: net.minecraft.network.chat.FormattedText#visit
    pub fn visit<T>(&self, acceptor: &mut impl ContentConsumer<T>) -> Option<T> {
        match self {
            FormattedText::Empty => None,
            FormattedText::Literal(s) => acceptor.accept(s),
            FormattedText::StyledLiteral(s, _) => acceptor.accept(s),
            FormattedText::Composite(parts) => {
                for part in parts {
                    let r = part.visit(acceptor);
                    if r.is_some() {
                        return r;
                    }
                }
                None
            }
        }
    }

    // JAVA: net.minecraft.network.chat.FormattedText#visit
    pub fn visit_styled<T>(
        &self,
        acceptor: &mut impl StyledContentConsumer<T>,
        style: &Style,
    ) -> Option<T> {
        match self {
            FormattedText::Empty => None,
            FormattedText::Literal(s) => acceptor.accept(style, s),
            FormattedText::StyledLiteral(s, s_style) => {
                let merged = s_style.apply_to(style);
                acceptor.accept(&merged, s)
            }
            FormattedText::Composite(parts) => {
                for part in parts {
                    let r = part.visit_styled(acceptor, style);
                    if r.is_some() {
                        return r;
                    }
                }
                None
            }
        }
    }

    // JAVA: net.minecraft.network.chat.FormattedText#getString
    pub fn get_string(&self) -> String {
        let mut out = String::new();
        self.visit(&mut |s: &str| {
            out.push_str(s);
            None::<()>
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none_and_empty_string() {
        let t = FormattedText::EMPTY;
        assert_eq!(t.get_string(), "");
        let r: Option<i32> = t.visit(&mut |_s: &str| Some(1));
        assert_eq!(r, None);
    }

    #[test]
    fn literal_visit_and_get_string() {
        let t = FormattedText::of("hello");
        assert_eq!(t.get_string(), "hello");
        let r = t.visit(&mut |s: &str| Some(s.to_string()));
        assert_eq!(r.as_deref(), Some("hello"));
    }

    #[test]
    fn styled_literal_applies_captured_style_to_argument() {
        let captured = Style {
            bold: true,
            italic: false,
        };
        let arg = Style {
            bold: false,
            italic: true,
        };
        let t = FormattedText::of_styled("x", captured);
        let mut seen: Option<(Style, String)> = None;
        t.visit_styled(
            &mut |st: &Style, s: &str| {
                seen = Some((st.clone(), s.to_string()));
                Some(())
            },
            &arg,
        );
        let (st, s) = seen.unwrap();
        assert_eq!(s, "x");
        assert!(st.bold);
        assert!(st.italic);
    }

    #[test]
    fn composite_short_circuits_and_concatenates() {
        let t = FormattedText::composite(vec![
            FormattedText::of("a"),
            FormattedText::of("b"),
            FormattedText::of("c"),
        ]);
        let mut visited = Vec::new();
        let r = t.visit(&mut |s: &str| {
            visited.push(s.to_string());
            if s == "b" {
                Some(s.to_string())
            } else {
                None
            }
        });
        assert_eq!(r.as_deref(), Some("b"));
        assert_eq!(visited, vec!["a", "b"]);
        assert_eq!(t.get_string(), "abc");
    }

    #[test]
    fn composite_styled_visit_propagates_style() {
        let t = FormattedText::composite(vec![
            FormattedText::of("plain"),
            FormattedText::of_styled("bold", Style { bold: true, italic: false }),
        ]);
        let mut styles = Vec::new();
        t.visit_styled(
            &mut |st: &Style, _s: &str| {
                styles.push(st.bold);
                None::<()>
            },
            &Style::empty(),
        );
        assert_eq!(styles, vec![false, true]);
    }

    #[test]
    fn stop_iteration_constant_is_some_unit() {
        assert_eq!(STOP_ITERATION, Some(()));
    }
}
