use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl CommandSyntaxException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn empty() -> MutableComponent {
        MutableComponent {
            text: String::new(),
        }
    }

    pub fn literal(text: impl Into<String>) -> MutableComponent {
        MutableComponent { text: text.into() }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MutableComponent {
    pub text: String,
}

impl MutableComponent {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn append_component(mut self, other: &Component) -> Self {
        self.text.push_str(&other.text);
        self
    }

    pub fn append(mut self, other: MutableComponent) -> Self {
        self.text.push_str(&other.text);
        self
    }

    pub fn into_component(self) -> Component {
        Component { text: self.text }
    }
}

impl From<Component> for MutableComponent {
    fn from(value: Component) -> Self {
        Self { text: value.text }
    }
}

impl From<MutableComponent> for Component {
    fn from(value: MutableComponent) -> Self {
        value.into_component()
    }
}

impl fmt::Display for MutableComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

pub fn default_no_style_separator() -> Component {
    Component {
        text: ", ".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    String(String),
    Int(i32),
    Float(f32),
    Double(f64),
    Compound(BTreeMap<String, Tag>),
    Other(String),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::String(s) => write!(f, "\"{s}\""),
            Tag::Int(v) => write!(f, "{v}"),
            Tag::Float(v) => write!(f, "{v}f"),
            Tag::Double(v) => write!(f, "{v}d"),
            Tag::Compound(map) => {
                f.write_str("{")?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        f.write_str(",")?;
                    }
                    first = false;
                    write!(f, "{k}:{v}")?;
                }
                f.write_str("}")
            }
            Tag::Other(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtPath {
    pattern: String,
}

impl NbtPath {
    pub fn parse(pattern: &str) -> Result<Self, CommandSyntaxException> {
        if pattern.is_empty() {
            return Err(CommandSyntaxException::new("Empty NBT path"));
        }
        let mut depth_bracket: i32 = 0;
        let mut depth_brace: i32 = 0;
        let mut in_quote = false;
        let mut escape = false;
        for ch in pattern.chars() {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' if in_quote => escape = true,
                '"' => in_quote = !in_quote,
                '[' if !in_quote => depth_bracket = depth_bracket.wrapping_add(1),
                ']' if !in_quote => {
                    depth_bracket = depth_bracket.wrapping_sub(1);
                    if depth_bracket < 0 {
                        return Err(CommandSyntaxException::new(
                            "Unbalanced brackets in NBT path",
                        ));
                    }
                }
                '{' if !in_quote => depth_brace = depth_brace.wrapping_add(1),
                '}' if !in_quote => {
                    depth_brace = depth_brace.wrapping_sub(1);
                    if depth_brace < 0 {
                        return Err(CommandSyntaxException::new(
                            "Unbalanced braces in NBT path",
                        ));
                    }
                }
                _ => {}
            }
        }
        if in_quote || depth_bracket != 0 || depth_brace != 0 {
            return Err(CommandSyntaxException::new("Invalid NBT path"));
        }
        Ok(Self {
            pattern: pattern.to_string(),
        })
    }

    pub fn get(&self, tag: &Tag) -> Result<Vec<Tag>, CommandSyntaxException> {
        let segments: Vec<&str> = self
            .pattern
            .split('.')
            .filter(|s| !s.is_empty())
            .collect();
        let mut current = vec![tag.clone()];
        for seg in segments {
            let mut next = Vec::new();
            for t in &current {
                if let Tag::Compound(map) = t {
                    if let Some(v) = map.get(seg) {
                        next.push(v.clone());
                    }
                }
            }
            current = next;
        }
        Ok(current)
    }
}

pub trait EntityRef: fmt::Debug {}

pub trait CommandSourceStack {
    fn parse_component_from_nbt(&self, tag: &Tag) -> Result<Component, String>;
    fn update_component_for_entity(
        &self,
        component: Option<&Component>,
        entity: Option<&dyn EntityRef>,
        depth: i32,
    ) -> Option<Component>;
}

pub trait DataSource: fmt::Debug + fmt::Display + PartialEq {
    fn get_data(
        &self,
        source: &dyn CommandSourceStack,
    ) -> Result<Vec<Tag>, CommandSyntaxException>;
    fn java_hash_code(&self) -> i32;
}

#[derive(Debug, Clone)]
pub struct MapCodec<T> {
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct NbtContents<D: DataSource> {
    interpreting: bool,
    separator: Option<Component>,
    nbt_path_pattern: String,
    data_source: D,
    compiled_nbt_path: Option<NbtPath>,
}

impl<D: DataSource> NbtContents<D> {
    // JAVA: net.minecraft.network.chat.contents.NbtContents#<init>
    pub fn new(
        nbt_path_pattern: String,
        interpreting: bool,
        separator: Option<Component>,
        data_source: D,
    ) -> Self {
        let compiled_nbt_path = compile_nbt_path(&nbt_path_pattern);
        Self::new_with_compiled(
            nbt_path_pattern,
            compiled_nbt_path,
            interpreting,
            separator,
            data_source,
        )
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#<init>
    fn new_with_compiled(
        nbt_path_pattern: String,
        compiled_nbt_path: Option<NbtPath>,
        interpreting: bool,
        separator: Option<Component>,
        data_source: D,
    ) -> Self {
        Self {
            nbt_path_pattern,
            compiled_nbt_path,
            interpreting,
            separator,
            data_source,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#getNbtPath
    pub fn get_nbt_path(&self) -> &str {
        &self.nbt_path_pattern
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#isInterpreting
    pub fn is_interpreting(&self) -> bool {
        self.interpreting
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#getSeparator
    pub fn get_separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#getDataSource
    pub fn get_data_source(&self) -> &D {
        &self.data_source
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#equals
    pub fn java_equals(&self, other: &Self) -> bool {
        self.data_source == other.data_source
            && self.separator == other.separator
            && self.interpreting == other.interpreting
            && self.nbt_path_pattern == other.nbt_path_pattern
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#hashCode
    pub fn java_hash_code(&self) -> i32 {
        let mut h: i32 = if self.interpreting { 1 } else { 0 };
        h = h
            .wrapping_mul(31)
            .wrapping_add(java_hash_optional_component(&self.separator));
        h = h
            .wrapping_mul(31)
            .wrapping_add(java_hash_string(&self.nbt_path_pattern));
        h.wrapping_mul(31)
            .wrapping_add(self.data_source.java_hash_code())
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#toString
    pub fn java_to_string(&self) -> String {
        format!(
            "nbt{{{}, interpreting={}, separator={}}}",
            self.data_source,
            self.interpreting,
            java_optional_to_string(&self.separator)
        )
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#resolve
    pub fn resolve(
        &self,
        source: Option<&dyn CommandSourceStack>,
        entity: Option<&dyn EntityRef>,
        depth: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        let Some(source) = source else {
            return Ok(Component::empty());
        };
        let Some(compiled_nbt_path) = &self.compiled_nbt_path else {
            return Ok(Component::empty());
        };

        let data = self.data_source.get_data(source)?;
        let mut tags: Vec<Tag> = Vec::new();
        for t in data {
            match compiled_nbt_path.get(&t) {
                Ok(found) => tags.extend(found),
                Err(_) => {
                    // CommandSyntaxException from path lookup -> empty stream for this element
                }
            }
        }

        if self.interpreting {
            let separator = source
                .update_component_for_entity(self.separator.as_ref(), entity, depth)
                .unwrap_or_else(default_no_style_separator);
            let mut acc: Option<MutableComponent> = None;
            for tag in &tags {
                match source.parse_component_from_nbt(tag) {
                    Ok(parsed) => {
                        let updated = source
                            .update_component_for_entity(Some(&parsed), entity, depth)
                            .unwrap_or(parsed);
                        acc = Some(match acc {
                            Some(prev) => prev
                                .append_component(&separator)
                                .append(MutableComponent::from(updated)),
                            None => MutableComponent::from(updated),
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to parse component: {}: {}", tag, e);
                    }
                }
            }
            Ok(acc.unwrap_or_else(Component::empty))
        } else {
            let strings: Vec<String> = tags.iter().map(as_string).collect();
            let resolved_sep =
                source.update_component_for_entity(self.separator.as_ref(), entity, depth);
            match resolved_sep {
                Some(sep) => {
                    let mut acc: Option<MutableComponent> = None;
                    for s in strings {
                        let lit = Component::literal(s);
                        acc = Some(match acc {
                            Some(prev) => prev.append_component(&sep).append(lit),
                            None => lit,
                        });
                    }
                    Ok(acc.unwrap_or_else(Component::empty))
                }
                None => Ok(Component::literal(strings.join(", "))),
            }
        }
    }

    // JAVA: net.minecraft.network.chat.contents.NbtContents#codec
    pub fn codec(&self) -> MapCodec<NbtContents<D>> {
        map_codec()
    }
}

// JAVA: net.minecraft.network.chat.contents.NbtContents#MAP_CODEC
pub fn map_codec<D: DataSource>() -> MapCodec<NbtContents<D>> {
    MapCodec {
        _marker: std::marker::PhantomData,
    }
}

// JAVA: net.minecraft.network.chat.contents.NbtContents#compileNbtPath
fn compile_nbt_path(pattern: &str) -> Option<NbtPath> {
    NbtPath::parse(pattern).ok()
}

// JAVA: net.minecraft.network.chat.contents.NbtContents#asString
fn as_string(tag: &Tag) -> String {
    match tag {
        Tag::String(s) => s.clone(),
        _ => tag.to_string(),
    }
}

fn java_hash_string(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

fn java_hash_component(c: &Component) -> i32 {
    java_hash_string(&c.text)
}

fn java_hash_optional_component(c: &Option<Component>) -> i32 {
    match c {
        Some(v) => java_hash_component(v),
        None => 0,
    }
}

fn java_optional_to_string(c: &Option<Component>) -> String {
    match c {
        Some(v) => format!("Optional[{}]", v.text),
        None => "Optional.empty".to_string(),
    }
}

impl<D: DataSource> PartialEq for NbtContents<D> {
    fn eq(&self, other: &Self) -> bool {
        self.java_equals(other)
    }
}

impl<D: DataSource> Eq for NbtContents<D> {}

impl<D: DataSource> std::hash::Hash for NbtContents<D> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.java_hash_code().hash(state);
    }
}

impl<D: DataSource> fmt::Display for NbtContents<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.java_to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq)]
    struct TestDataSource {
        name: String,
        hash: i32,
        data: Vec<Tag>,
    }

    impl fmt::Display for TestDataSource {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.name)
        }
    }

    impl DataSource for TestDataSource {
        fn get_data(
            &self,
            _source: &dyn CommandSourceStack,
        ) -> Result<Vec<Tag>, CommandSyntaxException> {
            Ok(self.data.clone())
        }

        fn java_hash_code(&self) -> i32 {
            self.hash
        }
    }

    #[derive(Debug, Default)]
    struct TestCtx;

    impl CommandSourceStack for TestCtx {
        fn parse_component_from_nbt(&self, tag: &Tag) -> Result<Component, String> {
            match tag {
                Tag::String(s) => Ok(Component { text: s.clone() }),
                _ => Ok(Component {
                    text: tag.to_string(),
                }),
            }
        }

        fn update_component_for_entity(
            &self,
            component: Option<&Component>,
            _entity: Option<&dyn EntityRef>,
            _depth: i32,
        ) -> Option<Component> {
            component.cloned()
        }
    }

    #[test]
    fn hash_code_wrapping_and_negative_inputs() {
        let ds = TestDataSource {
            name: "ds".into(),
            hash: i32::MIN,
            data: vec![],
        };
        let c = NbtContents::new("path".into(), true, None, ds);
        let mut expected: i32 = 1;
        expected = expected.wrapping_mul(31).wrapping_add(0);
        expected = expected
            .wrapping_mul(31)
            .wrapping_add(java_hash_string("path"));
        expected = expected.wrapping_mul(31).wrapping_add(i32::MIN);
        assert_eq!(c.java_hash_code(), expected);

        let ds2 = TestDataSource {
            name: "ds2".into(),
            hash: -1,
            data: vec![],
        };
        let c2 = NbtContents::new(
            "p".into(),
            false,
            Some(Component { text: "sep".into() }),
            ds2,
        );
        let mut e2: i32 = 0;
        e2 = e2.wrapping_mul(31).wrapping_add(java_hash_string("sep"));
        e2 = e2.wrapping_mul(31).wrapping_add(java_hash_string("p"));
        e2 = e2.wrapping_mul(31).wrapping_add(-1);
        assert_eq!(c2.java_hash_code(), e2);
    }

    #[test]
    fn equals_ignores_compiled_nbt_path_and_checks_fields() {
        let ds = TestDataSource {
            name: "ds".into(),
            hash: 1,
            data: vec![],
        };
        let a = NbtContents::new_with_compiled(
            "x".into(),
            Some(NbtPath::parse("x").unwrap()),
            false,
            None,
            ds.clone(),
        );
        let b = NbtContents::new_with_compiled("x".into(), None, false, None, ds.clone());
        assert!(a.java_equals(&b));

        let c = NbtContents::new("x".into(), true, None, ds.clone());
        assert!(!a.java_equals(&c));
        let d = NbtContents::new(
            "x".into(),
            false,
            Some(Component { text: ",".into() }),
            ds,
        );
        assert!(!a.java_equals(&d));
    }

    #[test]
    fn resolve_empty_when_missing_and_as_string_negative() {
        let ds = TestDataSource {
            name: "ds".into(),
            hash: 0,
            data: vec![Tag::String("hi".into())],
        };
        let ctx = TestCtx::default();
        let c = NbtContents::new("name".into(), false, None, ds.clone());
        assert_eq!(c.resolve(None, None, 0).unwrap().text, "");

        let bad = NbtContents::new_with_compiled("[".into(), None, false, None, ds);
        assert_eq!(bad.resolve(Some(&ctx), None, -1).unwrap().text, "");

        assert_eq!(as_string(&Tag::String("raw".into())), "raw");
        assert_eq!(as_string(&Tag::Int(-42)), "-42");
    }

    #[test]
    fn resolve_non_interpreting_with_separator_and_join() {
        let mut m1 = BTreeMap::new();
        m1.insert("v".into(), Tag::Int(1));
        let mut m2 = BTreeMap::new();
        m2.insert("v".into(), Tag::Int(-2));
        let ds = TestDataSource {
            name: "ds".into(),
            hash: 0,
            data: vec![Tag::Compound(m1), Tag::Compound(m2)],
        };
        let ctx = TestCtx::default();
        let c = NbtContents::new(
            "v".into(),
            false,
            Some(Component { text: ";".into() }),
            ds.clone(),
        );
        assert_eq!(c.resolve(Some(&ctx), None, 0).unwrap().text, "1;-2");

        let c2 = NbtContents::new("v".into(), false, None, ds);
        assert_eq!(c2.resolve(Some(&ctx), None, 0).unwrap().text, "1, -2");
    }

    #[test]
    fn resolve_interpreting_uses_separator_and_depth() {
        let mut m1 = BTreeMap::new();
        m1.insert("k".into(), Tag::String("A".into()));
        let mut m2 = BTreeMap::new();
        m2.insert("k".into(), Tag::String("B".into()));
        let ds = TestDataSource {
            name: "ds".into(),
            hash: 7,
            data: vec![Tag::Compound(m1), Tag::Compound(m2)],
        };
        let ctx = TestCtx::default();
        let c = NbtContents::new(
            "k".into(),
            true,
            Some(Component { text: "|".into() }),
            ds,
        );
        let r = c.resolve(Some(&ctx), None, -5).unwrap();
        assert_eq!(r.text, "A|B");
    }

    #[test]
    fn float_bits_preserved_in_tag_display() {
        let f = f32::from_bits(0xFFC0_0001);
        let d = f64::from_bits(0x7FF8_0000_0000_0001);
        let tf = Tag::Float(f);
        let td = Tag::Double(d);
        match &tf {
            Tag::Float(v) => assert_eq!(v.to_bits(), 0xFFC0_0001),
            _ => panic!("expected float"),
        }
        match &td {
            Tag::Double(v) => assert_eq!(v.to_bits(), 0x7FF8_0000_0000_0001),
            _ => panic!("expected double"),
        }
        assert!(as_string(&tf).ends_with('f'));
        assert!(as_string(&td).ends_with('d'));
    }
}
