use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style;

pub trait FormattedCharSink {
    fn accept(&mut self, index: i32, style: &Style, codepoint: i32) -> bool;
}

impl<F> FormattedCharSink for F
where
    F: FnMut(i32, &Style, i32) -> bool,
{
    fn accept(&mut self, index: i32, style: &Style, codepoint: i32) -> bool {
        self(index, style, codepoint)
    }
}

pub struct DecoratedSink<'a, F> {
    inner: &'a mut dyn FormattedCharSink,
    map: F,
}

impl<'a, F: Fn(i32) -> i32> FormattedCharSink for DecoratedSink<'a, F> {
    fn accept(&mut self, index: i32, style: &Style, codepoint: i32) -> bool {
        self.inner.accept(index, style, (self.map)(codepoint))
    }
}

#[derive(Clone)]
pub struct FormattedCharSequence {
    inner: Arc<dyn Fn(&mut dyn FormattedCharSink) -> bool>,
}

fn iterate_forward(text: &str, style: &Style, sink: &mut dyn FormattedCharSink) -> bool {
    let mut index: i32 = 0;
    for ch in text.chars() {
        let cp = ch as i32;
        if !sink.accept(index, style, cp) {
            return false;
        }
        index = index.wrapping_add(ch.len_utf16() as i32);
    }
    true
}

fn iterate_backward(text: &str, style: &Style, sink: &mut dyn FormattedCharSink) -> bool {
    let mut entries: Vec<(i32, i32)> = Vec::with_capacity(text.chars().count());
    let mut index: i32 = 0;
    for ch in text.chars() {
        entries.push((index, ch as i32));
        index = index.wrapping_add(ch.len_utf16() as i32);
    }
    for (i, cp) in entries.into_iter().rev() {
        if !sink.accept(i, style, cp) {
            return false;
        }
    }
    true
}

impl FormattedCharSequence {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut dyn FormattedCharSink) -> bool + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#accept
    pub fn accept(&self, sink: &mut dyn FormattedCharSink) -> bool {
        (self.inner)(sink)
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#EMPTY
    pub fn empty() -> Self {
        Self::new(|_| true)
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#codepoint
    pub fn codepoint(codepoint: i32, style: Style) -> Self {
        Self::new(move |sink| sink.accept(0, &style, codepoint))
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#forward
    pub fn forward(text: String, style: Style) -> Self {
        if text.is_empty() {
            return Self::empty();
        }
        Self::new(move |sink| iterate_forward(&text, &style, sink))
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#forward
    pub fn forward_with_map<F>(text: String, style: Style, codepoint_map: F) -> Self
    where
        F: Fn(i32) -> i32 + 'static,
    {
        if text.is_empty() {
            return Self::empty();
        }
        Self::new(move |sink| {
            let mut decorated = DecoratedSink {
                inner: sink,
                map: &codepoint_map,
            };
            iterate_forward(&text, &style, &mut decorated)
        })
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#backward
    pub fn backward(text: String, style: Style) -> Self {
        if text.is_empty() {
            return Self::empty();
        }
        Self::new(move |sink| iterate_backward(&text, &style, sink))
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#backward
    pub fn backward_with_map<F>(text: String, style: Style, codepoint_map: F) -> Self
    where
        F: Fn(i32) -> i32 + 'static,
    {
        if text.is_empty() {
            return Self::empty();
        }
        Self::new(move |sink| {
            let mut decorated = DecoratedSink {
                inner: sink,
                map: &codepoint_map,
            };
            iterate_backward(&text, &style, &mut decorated)
        })
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#decorateOutput
    pub fn decorate_output<'a, F>(sink: &'a mut dyn FormattedCharSink, map: F) -> DecoratedSink<'a, F>
    where
        F: Fn(i32) -> i32,
    {
        DecoratedSink { inner: sink, map }
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#composite
    pub fn composite_empty() -> Self {
        Self::empty()
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#composite
    pub fn composite_one(seq: Self) -> Self {
        seq
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#composite
    pub fn composite_two(first: Self, second: Self) -> Self {
        Self::from_pair(first, second)
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#composite
    pub fn composite_many(sequences: Vec<Self>) -> Self {
        Self::composite_list(sequences)
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#composite
    pub fn composite_list(sequences: Vec<Self>) -> Self {
        match sequences.len() {
            0 => Self::empty(),
            1 => sequences.into_iter().next().expect("len checked"),
            2 => {
                let mut it = sequences.into_iter();
                let first = it.next().expect("len checked");
                let second = it.next().expect("len checked");
                Self::from_pair(first, second)
            }
            _ => Self::from_list(sequences),
        }
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#fromPair
    pub fn from_pair(first: Self, second: Self) -> Self {
        Self::new(move |sink| first.accept(sink) && second.accept(sink))
    }

    // JAVA: net.minecraft.util.FormattedCharSequence#fromList
    pub fn from_list(sequences: Vec<Self>) -> Self {
        Self::new(move |sink| {
            for seq in &sequences {
                if !seq.accept(sink) {
                    return false;
                }
            }
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FormattedCharSequence, FormattedCharSink, Style};

    fn collect(seq: &FormattedCharSequence) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        seq.accept(&mut |index: i32, _style: &Style, cp: i32| {
            out.push((index, cp));
            true
        });
        out
    }

    #[test]
    fn empty_and_codepoint() {
        let mut called = false;
        let ok = FormattedCharSequence::empty().accept(&mut |_i: i32, _s: &Style, _cp: i32| {
            called = true;
            true
        });
        assert!(ok);
        assert!(!called);
        assert_eq!(
            collect(&FormattedCharSequence::codepoint(65, Style)),
            vec![(0, 65)]
        );
    }

    #[test]
    fn forward_ascii_and_utf16_indices() {
        assert_eq!(
            collect(&FormattedCharSequence::forward("ab".to_string(), Style)),
            vec![(0, 97), (1, 98)]
        );
        assert_eq!(
            collect(&FormattedCharSequence::forward("A😀B".to_string(), Style)),
            vec![(0, 65), (1, 0x1F600), (3, 66)]
        );
    }

    #[test]
    fn backward_reverses_with_utf16_indices() {
        assert_eq!(
            collect(&FormattedCharSequence::backward("ab".to_string(), Style)),
            vec![(1, 98), (0, 97)]
        );
        assert_eq!(
            collect(&FormattedCharSequence::backward("A😀B".to_string(), Style)),
            vec![(3, 66), (1, 0x1F600), (0, 65)]
        );
    }

    #[test]
    fn forward_map_and_decorate_output() {
        let mapped = FormattedCharSequence::forward_with_map(
            "ab".to_string(),
            Style,
            |cp| cp.wrapping_add(1),
        );
        assert_eq!(collect(&mapped), vec![(0, 98), (1, 99)]);

        let base = FormattedCharSequence::forward("ab".to_string(), Style);
        let mut out = Vec::new();
        base.accept(&mut |index: i32, style: &Style, cp: i32| {
            let mut sink = |i: i32, _s: &Style, c: i32| {
                out.push((i, c));
                true
            };
            let mut decorated = FormattedCharSequence::decorate_output(&mut sink, |c| c + 2);
            decorated.accept(index, style, cp)
        });
        assert_eq!(out, vec![(0, 99), (1, 100)]);
    }

    #[test]
    fn composite_short_circuits() {
        let a = FormattedCharSequence::codepoint(65, Style);
        let b = FormattedCharSequence::codepoint(66, Style);
        let seq = FormattedCharSequence::from_pair(a, b);
        let mut out = Vec::new();
        let ok = seq.accept(&mut |_i: i32, _s: &Style, cp: i32| {
            out.push(cp);
            false
        });
        assert!(!ok);
        assert_eq!(out, vec![65]);
    }

    #[test]
    fn composite_list_sizes() {
        let empty = FormattedCharSequence::composite_list(vec![]);
        let one = FormattedCharSequence::composite_list(vec![FormattedCharSequence::codepoint(65, Style)]);
        let two = FormattedCharSequence::composite_list(vec![
            FormattedCharSequence::codepoint(65, Style),
            FormattedCharSequence::codepoint(66, Style),
        ]);
        let many = FormattedCharSequence::composite_many(vec![
            FormattedCharSequence::codepoint(65, Style),
            FormattedCharSequence::codepoint(66, Style),
            FormattedCharSequence::codepoint(67, Style),
        ]);
        assert!(collect(&empty).is_empty());
        assert_eq!(collect(&one), vec![(0, 65)]);
        assert_eq!(collect(&two), vec![(0, 65), (0, 66)]);
        assert_eq!(collect(&many), vec![(0, 65), (0, 66), (0, 67)]);
    }
}
