use std::fmt::Write as FmtWrite;

/// Minimal self-contained component abstraction for the pretty-printer.
/// This replaces `net.minecraft.network.chat.Component` / `MutableComponent`
/// and `ChatFormatting` for the purpose of this 1:1 port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatFormatting {
    Gray,
    Aqua,
    Green,
    Gold,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentPart {
    pub text: String,
    pub style: Option<ChatFormatting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub parts: Vec<ComponentPart>,
}

impl Component {
    // JAVA: net.minecraft.network.chat.Component#empty (adapted)
    pub fn empty() -> MutableComponent {
        MutableComponent {
            parts: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.Component#literal (adapted)
    pub fn literal(text: impl Into<String>) -> MutableComponent {
        MutableComponent {
            parts: vec![ComponentPart {
                text: text.into(),
                style: None,
            }],
        }
    }

    pub fn as_string(&self) -> String {
        let mut out = String::new();
        for part in &self.parts {
            out.push_str(&part.text);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    pub parts: Vec<ComponentPart>,
}

impl MutableComponent {
    // JAVA: net.minecraft.network.chat.MutableComponent#withStyle (adapted)
    pub fn with_style(mut self, style: ChatFormatting) -> Self {
        if let Some(last) = self.parts.last_mut() {
            last.style = Some(style);
        }
        self
    }

    // JAVA: net.minecraft.network.chat.MutableComponent#append(String) (adapted)
    pub fn append_str(&mut self, text: &str) -> &mut Self {
        self.parts.push(ComponentPart {
            text: text.to_string(),
            style: None,
        });
        self
    }

    // JAVA: net.minecraft.network.chat.MutableComponent#append(Component/MutableComponent) (adapted)
    pub fn append_component(&mut self, component: ComponentLike) -> &mut Self {
        match component {
            ComponentLike::Component(c) => self.parts.extend(c.parts),
            ComponentLike::Mutable(c) => self.parts.extend(c.parts),
        }
        self
    }

    pub fn into_component(self) -> Component {
        Component { parts: self.parts }
    }

    pub fn as_string(&self) -> String {
        let mut out = String::new();
        for part in &self.parts {
            out.push_str(&part.text);
        }
        out
    }
}

pub enum ComponentLike {
    Component(Component),
    Mutable(MutableComponent),
}

impl From<Component> for ComponentLike {
    fn from(c: Component) -> Self {
        ComponentLike::Component(c)
    }
}

impl From<MutableComponent> for ComponentLike {
    fn from(c: MutableComponent) -> Self {
        ComponentLike::Mutable(c)
    }
}

/// Minimal self-contained tag model for the visitor.
/// These mirror the Java NBT tag types used by `TextComponentTagVisitor`.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    // JAVA: net.minecraft.nbt.Tag#accept (dispatch)
    pub fn accept(&self, visitor: &mut TextComponentTagVisitor) {
        match self {
            Tag::End => visitor.visit_end(),
            Tag::Byte(v) => visitor.visit_byte(*v),
            Tag::Short(v) => visitor.visit_short(*v),
            Tag::Int(v) => visitor.visit_int(*v),
            Tag::Long(v) => visitor.visit_long(*v),
            Tag::Float(v) => visitor.visit_float(*v),
            Tag::Double(v) => visitor.visit_double(*v),
            Tag::ByteArray(v) => visitor.visit_byte_array(v),
            Tag::String(v) => visitor.visit_string(v),
            Tag::List(v) => visitor.visit_list(v),
            Tag::Compound(v) => visitor.visit_compound(v),
            Tag::IntArray(v) => visitor.visit_int_array(v),
            Tag::LongArray(v) => visitor.visit_long_array(v),
        }
    }
}

/// Helper trait to identify numeric tags for `should_wrap_list_elements`.
fn is_numeric_tag(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Byte(_) | Tag::Short(_) | Tag::Int(_) | Tag::Long(_) | Tag::Float(_) | Tag::Double(_)
    )
}

// JAVA: net.minecraft.nbt.StringTag#quoteAndEscape
pub fn quote_and_escape(value: &str) -> String {
    // Java StringTag.quoteAndEscape produces a quoted and escaped NBT string literal.
    // It uses single quotes if the string contains double quotes but no single quotes,
    // otherwise double quotes. Escapes backslash and the chosen quote character.
    let mut has_single = false;
    let mut has_double = false;
    for ch in value.chars() {
        if ch == '\'' {
            has_single = true;
        } else if ch == '"' {
            has_double = true;
        }
    }

    let quote = if has_single && !has_double { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for ch in value.chars() {
        if ch == '\\' || ch == quote {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push(quote);
    out
}

fn is_simple_value(s: &str) -> bool {
    // JAVA: Pattern "[A-Za-z0-9._+-]+"
    if s.is_empty() {
        return false;
    }
    s.bytes().all(|b| {
        b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'+' || b == b'-'
    })
}

fn repeat_string(s: &str, count: usize) -> String {
    // JAVA: Strings.repeat
    let mut out = String::new();
    for _ in 0..count {
        out.push_str(s);
    }
    out
}

const INLINE_LIST_THRESHOLD: usize = 8;
const MAX_DEPTH: i32 = 64;
const MAX_LENGTH: usize = 128;

const LIST_OPEN: &str = "[";
const LIST_CLOSE: &str = "]";
const LIST_TYPE_SEPARATOR: &str = ";";
const ELEMENT_SPACING: &str = " ";
const STRUCT_OPEN: &str = "{";
const STRUCT_CLOSE: &str = "}";
const NEWLINE: &str = "\n";
const NAME_VALUE_SEPARATOR: &str = ": ";
const ELEMENT_SEPARATOR: &str = ",";
const WRAPPED_ELEMENT_SEPARATOR: &str = ",\n";
const SPACED_ELEMENT_SEPARATOR: &str = ", ";

fn folded_component() -> Component {
    Component::literal("<...>")
        .with_style(ChatFormatting::Gray)
        .into_component()
}

fn byte_type_component() -> Component {
    Component::literal("b")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn short_type_component() -> Component {
    Component::literal("s")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn int_type_component() -> Component {
    Component::literal("I")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn long_type_component() -> Component {
    Component::literal("L")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn float_type_component() -> Component {
    Component::literal("f")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn double_type_component() -> Component {
    Component::literal("d")
        .with_style(ChatFormatting::Red)
        .into_component()
}

fn byte_array_type_component() -> Component {
    Component::literal("B")
        .with_style(ChatFormatting::Red)
        .into_component()
}

pub struct TextComponentTagVisitor {
    indentation: String,
    indent_depth: i32,
    depth: i32,
    result: MutableComponent,
}

impl TextComponentTagVisitor {
    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#<init>
    pub fn new(indentation: String) -> Self {
        Self {
            indentation,
            indent_depth: 0,
            depth: 0,
            result: Component::empty(),
        }
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visit
    pub fn visit(&mut self, tag: &Tag) -> Component {
        tag.accept(self);
        std::mem::take(&mut self.result).into_component()
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitString
    pub fn visit_string(&mut self, value: &str) {
        let escaped = quote_and_escape(value);
        let quote = &escaped[0..1];
        let inner = &escaped[1..escaped.len() - 1];
        let inner_component = Component::literal(inner.to_string()).with_style(ChatFormatting::Green);
        self.result.append_str(quote);
        self.result.append_component(inner_component.into());
        self.result.append_str(quote);
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitByte
    pub fn visit_byte(&mut self, value: i8) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
        self.result.append_component(byte_type_component().into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitShort
    pub fn visit_short(&mut self, value: i16) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
        self.result.append_component(short_type_component().into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitInt
    pub fn visit_int(&mut self, value: i32) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitLong
    pub fn visit_long(&mut self, value: i64) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
        self.result.append_component(long_type_component().into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitFloat
    pub fn visit_float(&mut self, value: f32) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
        self.result.append_component(float_type_component().into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitDouble
    pub fn visit_double(&mut self, value: f64) {
        let num = Component::literal(value.to_string()).with_style(ChatFormatting::Gold);
        self.result.append_component(num.into());
        self.result.append_component(double_type_component().into());
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitByteArray
    pub fn visit_byte_array(&mut self, values: &[i8]) {
        self.result.append_str(LIST_OPEN);
        self.result.append_component(byte_array_type_component().into());
        self.result.append_str(LIST_TYPE_SEPARATOR);

        let len = values.len();
        let mut i: usize = 0;
        while i < len && i < MAX_LENGTH {
            let v = values[i];
            let num = Component::literal(v.to_string()).with_style(ChatFormatting::Gold);
            self.result.append_str(ELEMENT_SPACING);
            self.result.append_component(num.into());
            self.result.append_component(byte_type_component().into());
            if i != len.wrapping_sub(1) {
                self.result.append_str(ELEMENT_SEPARATOR);
            }
            i = i.wrapping_add(1);
        }

        if len > MAX_LENGTH {
            self.result.append_component(folded_component().into());
        }

        self.result.append_str(LIST_CLOSE);
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitIntArray
    pub fn visit_int_array(&mut self, values: &[i32]) {
        self.result.append_str(LIST_OPEN);
        self.result.append_component(int_type_component().into());
        self.result.append_str(LIST_TYPE_SEPARATOR);

        let len = values.len();
        let mut i: usize = 0;
        while i < len && i < MAX_LENGTH {
            let v = values[i];
            let num = Component::literal(v.to_string()).with_style(ChatFormatting::Gold);
            self.result.append_str(ELEMENT_SPACING);
            self.result.append_component(num.into());
            if i != len.wrapping_sub(1) {
                self.result.append_str(ELEMENT_SEPARATOR);
            }
            i = i.wrapping_add(1);
        }

        if len > MAX_LENGTH {
            self.result.append_component(folded_component().into());
        }

        self.result.append_str(LIST_CLOSE);
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitLongArray
    pub fn visit_long_array(&mut self, values: &[i64]) {
        self.result.append_str(LIST_OPEN);
        self.result.append_component(long_type_component().into());
        self.result.append_str(LIST_TYPE_SEPARATOR);

        let len = values.len();
        let mut i: usize = 0;
        while i < len && i < MAX_LENGTH {
            let v = values[i];
            let num = Component::literal(v.to_string()).with_style(ChatFormatting::Gold);
            self.result.append_str(ELEMENT_SPACING);
            self.result.append_component(num.into());
            self.result.append_component(long_type_component().into());
            if i != len.wrapping_sub(1) {
                self.result.append_str(ELEMENT_SEPARATOR);
            }
            i = i.wrapping_add(1);
        }

        if len > MAX_LENGTH {
            self.result.append_component(folded_component().into());
        }

        self.result.append_str(LIST_CLOSE);
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#shouldWrapListElements
    fn should_wrap_list_elements(list: &[Tag]) -> bool {
        if list.len() >= INLINE_LIST_THRESHOLD {
            return false;
        }

        for item in list {
            if !is_numeric_tag(item) {
                return true;
            }
        }

        false
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitList
    pub fn visit_list(&mut self, list: &[Tag]) {
        if list.is_empty() {
            self.result.append_str("[]");
        } else if self.depth >= MAX_DEPTH {
            self.result.append_str(LIST_OPEN);
            self.result.append_component(folded_component().into());
            self.result.append_str(LIST_CLOSE);
        } else if !Self::should_wrap_list_elements(list) {
            self.result.append_str(LIST_OPEN);

            let mut i: usize = 0;
            let len = list.len();
            while i < len {
                if i != 0 {
                    self.result.append_str(SPACED_ELEMENT_SEPARATOR);
                }

                self.append_sub_tag(&list[i], false);
                i = i.wrapping_add(1);
            }

            self.result.append_str(LIST_CLOSE);
        } else {
            self.result.append_str(LIST_OPEN);
            if !self.indentation.is_empty() {
                self.result.append_str(NEWLINE);
            }

            let prefix = repeat_string(&self.indentation, (self.indent_depth + 1) as usize);

            let len = list.len();
            let mut i: usize = 0;
            while i < len && i < MAX_LENGTH {
                self.result.append_str(&prefix);
                self.append_sub_tag(&list[i], true);
                if i != len.wrapping_sub(1) {
                    if self.indentation.is_empty() {
                        self.result.append_str(SPACED_ELEMENT_SEPARATOR);
                    } else {
                        self.result.append_str(WRAPPED_ELEMENT_SEPARATOR);
                    }
                }
                i = i.wrapping_add(1);
            }

            if len > MAX_LENGTH {
                self.result.append_str(&prefix);
                self.result.append_component(folded_component().into());
            }

            if !self.indentation.is_empty() {
                let suffix_indent = repeat_string(&self.indentation, self.indent_depth as usize);
                self.result.append_str(NEWLINE);
                self.result.append_str(&suffix_indent);
            }

            self.result.append_str(LIST_CLOSE);
        }
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitCompound
    pub fn visit_compound(&mut self, entries: &[(String, Tag)]) {
        if entries.is_empty() {
            self.result.append_str("{}");
        } else if self.depth >= MAX_DEPTH {
            self.result.append_str(STRUCT_OPEN);
            self.result.append_component(folded_component().into());
            self.result.append_str(STRUCT_CLOSE);
        } else {
            self.result.append_str(STRUCT_OPEN);

            // Java: Collection<String> $$1 = $$0.keySet();
            // if (LOGGER.isDebugEnabled()) { sort... }
            // We preserve insertion order (no debug sort in this faithful port context).
            let keys: Vec<&String> = entries.iter().map(|(k, _)| k).collect();

            if !self.indentation.is_empty() {
                self.result.append_str(NEWLINE);
            }

            let prefix = repeat_string(&self.indentation, (self.indent_depth + 1) as usize);
            let len = keys.len();
            let mut i: usize = 0;

            while i < len {
                let key = keys[i];
                let value = &entries[i].1;
                self.result.append_str(&prefix);
                self.result
                    .append_component(Self::handle_escape_pretty(key).into());
                self.result.append_str(NAME_VALUE_SEPARATOR);
                self.append_sub_tag(value, true);
                if i != len.wrapping_sub(1) {
                    if self.indentation.is_empty() {
                        self.result.append_str(SPACED_ELEMENT_SEPARATOR);
                    } else {
                        self.result.append_str(WRAPPED_ELEMENT_SEPARATOR);
                    }
                }
                i = i.wrapping_add(1);
            }

            if !self.indentation.is_empty() {
                let suffix_indent = repeat_string(&self.indentation, self.indent_depth as usize);
                self.result.append_str(NEWLINE);
                self.result.append_str(&suffix_indent);
            }

            self.result.append_str(STRUCT_CLOSE);
        }
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#appendSubTag
    fn append_sub_tag(&mut self, tag: &Tag, indent: bool) {
        if indent {
            self.indent_depth = self.indent_depth.wrapping_add(1);
        }

        self.depth = self.depth.wrapping_add(1);

        // Java try/finally equivalent
        tag.accept(self);

        if indent {
            self.indent_depth = self.indent_depth.wrapping_sub(1);
        }

        self.depth = self.depth.wrapping_sub(1);
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#handleEscapePretty
    pub fn handle_escape_pretty(name: &str) -> Component {
        if is_simple_value(name) {
            return Component::literal(name.to_string())
                .with_style(ChatFormatting::Aqua)
                .into_component();
        }

        let escaped = quote_and_escape(name);
        let quote = &escaped[0..1];
        let inner = &escaped[1..escaped.len() - 1];
        let inner_component = Component::literal(inner.to_string()).with_style(ChatFormatting::Aqua);
        let mut out = Component::literal(quote.to_string());
        out.append_component(inner_component.into());
        out.append_str(quote);
        out.into_component()
    }

    // JAVA: net.minecraft.nbt.TextComponentTagVisitor#visitEnd
    pub fn visit_end(&mut self) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(tag: &Tag, indent: &str) -> String {
        let mut visitor = TextComponentTagVisitor::new(indent.to_string());
        visitor.visit(tag).as_string()
    }

    #[test]
    fn test_primitives_and_string_escaping() {
        assert_eq!(render(&Tag::Byte(-5), ""), "-5b");
        assert_eq!(render(&Tag::Short(1234), ""), "1234s");
        assert_eq!(render(&Tag::Int(-42), ""), "-42");
        assert_eq!(render(&Tag::Long(9_000_000_000_i64), ""), "9000000000L");
        assert_eq!(render(&Tag::Float(1.5_f32), ""), "1.5f");
        assert_eq!(render(&Tag::Double(2.25_f64), ""), "2.25d");

        // Simple string and escaped string
        assert_eq!(render(&Tag::String("hello".to_string()), ""), "'hello'");
        assert_eq!(render(&Tag::String("a\"b".to_string()), ""), "'a\"b'");
        assert_eq!(render(&Tag::String("it's".to_string()), ""), "\"it's\"");
    }

    #[test]
    fn test_arrays_truncation_and_format() {
        let bytes: Vec<i8> = (0..130).map(|i| (i as i8).wrapping_mul(1)).collect();
        let out = render(&Tag::ByteArray(bytes), "");
        assert!(out.starts_with("[B; 0b, 1b,"));
        assert!(out.contains("<...>"));
        assert!(out.ends_with(']'));

        let ints: Vec<i32> = (0..3).collect();
        assert_eq!(render(&Tag::IntArray(ints), ""), "[I; 0, 1, 2]");

        let longs: Vec<i64> = vec![1, -2];
        assert_eq!(render(&Tag::LongArray(longs), ""), "[L; 1L, -2L]");
    }

    #[test]
    fn test_list_inline_vs_wrapped_and_depth_limit() {
        // Inline numeric list (< 8, all numeric)
        let inline = Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]);
        assert_eq!(render(&inline, ""), "[1, 2, 3]");

        // Wrapped because contains non-numeric
        let wrapped = Tag::List(vec![Tag::Int(1), Tag::String("x".to_string())]);
        assert_eq!(render(&wrapped, ""), "[1, 'x']");

        // Pretty wrapped with indentation
        let pretty = render(&wrapped, "  ");
        assert_eq!(pretty, "[\n  1,\n  'x'\n]");

        // Depth limit: depth >= 64 renders folded
        let mut deep = Tag::Int(1);
        for _ in 0..70 {
            deep = Tag::List(vec![deep]);
        }
        let out = render(&deep, "");
        assert!(out.contains("<...>"));
    }

    #[test]
    fn test_compound_pretty_and_key_escaping() {
        let tag = Tag::Compound(vec![
            ("simple".to_string(), Tag::Int(1)),
            ("has space".to_string(), Tag::String("v".to_string())),
        ]);
        let compact = render(&tag, "");
        assert_eq!(compact, "{simple: 1, 'has space': 'v'}");

        let pretty = render(&tag, "  ");
        assert_eq!(pretty, "{\n  simple: 1,\n  'has space': 'v'\n}");
    }

    #[test]
    fn test_negative_and_overflow_like_values() {
        assert_eq!(render(&Tag::Byte(i8::MIN), ""), "-128b");
        assert_eq!(render(&Tag::Int(i32::MIN), ""), "-2147483648");
        assert_eq!(render(&Tag::Long(i64::MIN), ""), "-9223372036854775808L");

        // Float/double formatting follows Rust Display like Java String.valueOf for typical values;
        // we verify exact expected strings for these specific values.
        assert_eq!(render(&Tag::Float(-0.0_f32), ""), "-0f");
        assert_eq!(render(&Tag::Double(f64::INFINITY), ""), "infd");
    }

    #[test]
    fn test_handle_escape_pretty_and_end() {
        let simple = TextComponentTagVisitor::handle_escape_pretty("abc_1.+-");
        assert_eq!(simple.as_string(), "abc_1.+-");

        let quoted = TextComponentTagVisitor::handle_escape_pretty("a b");
        assert_eq!(quoted.as_string(), "'a b'");

        let mut v = TextComponentTagVisitor::new("".to_string());
        v.visit_end();
        assert_eq!(v.visit(&Tag::End).as_string(), "");
    }
}
