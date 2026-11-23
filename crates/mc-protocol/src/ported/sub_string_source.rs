use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Style {
    pub id: u32,
}

impl Style {
    pub const EMPTY: Style = Style { id: 0 };
}

#[derive(Clone, Debug)]
pub struct FormattedCharSequence {
    pub text: String,
    pub style: Style,
    pub backward: bool,
}

impl FormattedCharSequence {
    // JAVA: net.minecraft.util.FormattedCharSequence#forward
    pub fn forward(text: String, style: Style) -> Self {
        Self {
            text,
            style,
            backward: false,
        }
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#backward
    pub fn backward(text: String, style: Style, _reverse_char_modifier: &dyn Fn(i32) -> i32) -> Self {
        Self {
            text,
            style,
            backward: true,
        }
    }
}

pub trait FormattedText {
    fn visit(
        &self,
        acceptor: &mut dyn FnMut(Style, &str) -> Option<()>,
        style: Style,
    ) -> Option<()>;
}

pub struct StringDecomposer;

impl StringDecomposer {
    // JAVA: net.minecraft.util.StringDecomposer#iterateFormatted(String,Style,FormattedCharSink)
    pub fn iterate_formatted(
        text: &str,
        style: Style,
        sink: &mut dyn FnMut(i32, Style, i32) -> bool,
    ) -> bool {
        let mut index: i32 = 0;
        for ch in text.chars() {
            let cp = ch as i32;
            if !sink(index, style.clone(), cp) {
                return false;
            }
            let char_count = ch.len_utf16() as i32;
            index = index.wrapping_add(char_count);
        }
        true
    }
}

fn substring_utf16(s: &str, start: usize, end: usize) -> String {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    let s_start = start.min(utf16.len());
    let s_end = end.min(utf16.len());
    if s_start >= s_end {
        return String::new();
    }
    String::from_utf16_lossy(&utf16[s_start..s_end])
}

pub struct SubStringSource {
    plain_text: String,
    char_styles: Vec<Style>,
    reverse_char_modifier: Arc<dyn Fn(i32) -> i32 + Send + Sync>,
}

impl SubStringSource {
    // JAVA: net.minecraft.network.chat.SubStringSource#<init>
    fn new(
        plain_text: String,
        char_styles: Vec<Style>,
        reverse_char_modifier: Arc<dyn Fn(i32) -> i32 + Send + Sync>,
    ) -> Self {
        Self {
            plain_text,
            char_styles,
            reverse_char_modifier,
        }
    }

    // JAVA: net.minecraft.network.chat.SubStringSource#getPlainText
    pub fn get_plain_text(&self) -> &str {
        &self.plain_text
    }

    // JAVA: net.minecraft.network.chat.SubStringSource#substring
    pub fn substring(
        &self,
        start: usize,
        length: usize,
        reverse: bool,
    ) -> Vec<FormattedCharSequence> {
        if length == 0 {
            return Vec::new();
        }

        let mut result: Vec<FormattedCharSequence> = Vec::new();
        let mut current_style = self.char_styles[start].clone();
        let mut segment_start = start;

        let mut i: usize = 1;
        while i < length {
            let idx = start + i;
            let style = self.char_styles[idx].clone();
            if style != current_style {
                let text = substring_utf16(&self.plain_text, segment_start, idx);
                let seq = if reverse {
                    FormattedCharSequence::backward(
                        text,
                        current_style.clone(),
                        &*self.reverse_char_modifier,
                    )
                } else {
                    FormattedCharSequence::forward(text, current_style.clone())
                };
                result.push(seq);
                current_style = style;
                segment_start = idx;
            }
            i += 1;
        }

        let end = start + length;
        if segment_start < end {
            let text = substring_utf16(&self.plain_text, segment_start, end);
            let seq = if reverse {
                FormattedCharSequence::backward(
                    text,
                    current_style.clone(),
                    &*self.reverse_char_modifier,
                )
            } else {
                FormattedCharSequence::forward(text, current_style)
            };
            result.push(seq);
        }

        if reverse {
            result.reverse();
        }
        result
    }

    // JAVA: net.minecraft.network.chat.SubStringSource#create(FormattedText)
    pub fn create(formatted_text: &dyn FormattedText) -> Self {
        Self::create_with_transforms(formatted_text, Arc::new(|x: i32| x), |s: String| s)
    }

    // JAVA: net.minecraft.network.chat.SubStringSource#create(FormattedText,Int2IntFunction,UnaryOperator)
    pub fn create_with_transforms(
        formatted_text: &dyn FormattedText,
        reverse_char_modifier: Arc<dyn Fn(i32) -> i32 + Send + Sync>,
        text_transform: impl Fn(String) -> String,
    ) -> Self {
        let mut plain_text = String::new();
        let mut char_styles: Vec<Style> = Vec::new();

        let mut acceptor = |style: Style, text: &str| -> Option<()> {
            StringDecomposer::iterate_formatted(
                text,
                style,
                &mut |_idx: i32, s: Style, cp: i32| {
                    if let Some(ch) = char::from_u32(cp as u32) {
                        plain_text.push(ch);
                        let count = ch.len_utf16();
                        let mut j: usize = 0;
                        while j < count {
                            char_styles.push(s.clone());
                            j += 1;
                        }
                    }
                    true
                },
            );
            None
        };

        formatted_text.visit(&mut acceptor, Style::EMPTY);

        Self::new(text_transform(plain_text), char_styles, reverse_char_modifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleFormattedText {
        parts: Vec<(Style, String)>,
    }

    impl FormattedText for SimpleFormattedText {
        fn visit(
            &self,
            acceptor: &mut dyn FnMut(Style, &str) -> Option<()>,
            _style: Style,
        ) -> Option<()> {
            for (style, text) in &self.parts {
                if acceptor(style.clone(), text).is_some() {
                    return Some(());
                }
            }
            None
        }
    }

    #[test]
    fn create_plain_text_and_substring() {
        let src = SubStringSource::create(&SimpleFormattedText {
            parts: vec![(Style { id: 1 }, "hello".to_string())],
        });
        assert_eq!(src.get_plain_text(), "hello");
        let seqs = src.substring(0, 5, false);
        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0].text, "hello");
        assert_eq!(seqs[0].style.id, 1);
        assert!(!seqs[0].backward);
    }

    #[test]
    fn substring_splits_on_style_change() {
        let src = SubStringSource::create(&SimpleFormattedText {
            parts: vec![
                (Style { id: 1 }, "ab".to_string()),
                (Style { id: 2 }, "cd".to_string()),
            ],
        });
        let seqs = src.substring(0, 4, false);
        assert_eq!(seqs.len(), 2);
        assert_eq!(seqs[0].text, "ab");
        assert_eq!(seqs[0].style.id, 1);
        assert_eq!(seqs[1].text, "cd");
        assert_eq!(seqs[1].style.id, 2);
    }

    #[test]
    fn substring_reverse_reverses_order() {
        let src = SubStringSource::create(&SimpleFormattedText {
            parts: vec![
                (Style { id: 1 }, "ab".to_string()),
                (Style { id: 2 }, "cd".to_string()),
            ],
        });
        let seqs = src.substring(0, 4, true);
        assert_eq!(seqs.len(), 2);
        assert_eq!(seqs[0].text, "cd");
        assert!(seqs[0].backward);
        assert_eq!(seqs[1].text, "ab");
        assert!(seqs[1].backward);
    }

    #[test]
    fn empty_length_returns_empty() {
        let src = SubStringSource::create(&SimpleFormattedText {
            parts: vec![(Style { id: 1 }, "abc".to_string())],
        });
        let seqs = src.substring(0, 0, false);
        assert!(seqs.is_empty());
    }

    #[test]
    fn create_applies_text_transform() {
        let src = SubStringSource::create_with_transforms(
            &SimpleFormattedText {
                parts: vec![(Style { id: 1 }, "abc".to_string())],
            },
            Arc::new(|x| x),
            |s| s.to_uppercase(),
        );
        assert_eq!(src.get_plain_text(), "ABC");
        let seqs = src.substring(0, 3, false);
        assert_eq!(seqs[0].text, "ABC");
    }
}
