use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatableFormatException {
    pub message: String,
}

impl TranslatableFormatException {
    fn msg(key: &str, detail: &str) -> Self {
        Self {
            message: format!("Error parsing: {}: {}", key, detail),
        }
    }

    fn index(key: &str, index: i32) -> Self {
        Self {
            message: format!("Invalid index {} in: {}", index, key),
        }
    }
}

impl fmt::Display for TranslatableFormatException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TranslatableFormatException {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Style;

#[derive(Clone, Debug, Default)]
pub struct CommandSourceStack;

#[derive(Clone, Debug, Default)]
pub struct Entity;

#[derive(Debug, Clone)]
pub struct CommandSyntaxException(pub String);

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CommandSyntaxException {}

pub trait Language: Send + Sync + fmt::Debug {
    fn get_or_default(&self, key: &str) -> String;
    fn get_or_default_fallback(&self, key: &str, fallback: &str) -> String;
    fn identity(&self) -> usize;
}

#[derive(Debug)]
struct DefaultLanguage;

impl Language for DefaultLanguage {
    fn get_or_default(&self, key: &str) -> String {
        key.to_string()
    }

    fn get_or_default_fallback(&self, _key: &str, fallback: &str) -> String {
        fallback.to_string()
    }

    fn identity(&self) -> usize {
        1
    }
}

thread_local! {
    static CURRENT_LANGUAGE: RefCell<Arc<dyn Language>> = RefCell::new(Arc::new(DefaultLanguage));
}

pub fn get_language() -> Arc<dyn Language> {
    CURRENT_LANGUAGE.with(|c| c.borrow().clone())
}

pub fn set_language(lang: Arc<dyn Language>) {
    CURRENT_LANGUAGE.with(|c| *c.borrow_mut() = lang);
}

#[derive(Clone, Debug, PartialEq)]
pub enum Component {
    Literal(String),
    Translatable(Box<TranslatableContents>),
}

impl Component {
    pub fn literal(s: impl Into<String>) -> Self {
        Component::Literal(s.into())
    }

    pub fn try_collapse_to_string(&self) -> Option<String> {
        match self {
            Component::Literal(s) => Some(s.clone()),
            Component::Translatable(t) => {
                if t.args.is_empty() && t.fallback.is_none() {
                    Some(t.key.clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn visit_styled<T>(
        &self,
        consumer: &mut dyn FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        match self {
            Component::Literal(s) => consumer(style, s),
            Component::Translatable(t) => t.visit_styled(consumer, style),
        }
    }

    pub fn visit_content<T>(&self, consumer: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            Component::Literal(s) => consumer(s),
            Component::Translatable(t) => t.visit_content(consumer),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FormattedText {
    Literal(String),
    Component(Component),
}

impl FormattedText {
    pub fn of(s: impl Into<String>) -> Self {
        FormattedText::Literal(s.into())
    }

    pub fn visit_styled<T>(
        &self,
        consumer: &mut dyn FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        match self {
            FormattedText::Literal(s) => consumer(style, s),
            FormattedText::Component(c) => c.visit_styled(consumer, style),
        }
    }

    pub fn visit_content<T>(&self, consumer: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            FormattedText::Literal(s) => consumer(s),
            FormattedText::Component(c) => c.visit_content(consumer),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Component(Component),
}

impl Arg {
    pub fn to_java_string(&self) -> String {
        match self {
            Arg::Bool(b) => b.to_string(),
            Arg::Int(n) => n.to_string(),
            Arg::Long(n) => n.to_string(),
            Arg::Float(f) => java_float_to_string(*f),
            Arg::Double(d) => java_double_to_string(*d),
            Arg::String(s) => s.clone(),
            Arg::Component(c) => match c {
                Component::Literal(s) => format!("literal{{{}}}", s),
                Component::Translatable(t) => t.to_string_java(),
            },
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_java_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MutableComponent {
    pub contents: TranslatableContents,
}

impl MutableComponent {
    pub fn create(contents: TranslatableContents) -> Self {
        Self { contents }
    }
}

#[derive(Clone, Debug, Default)]
struct DecomposeCache {
    decomposed_with: Option<usize>,
    decomposed_parts: Vec<FormattedText>,
}

#[derive(Clone, Debug)]
pub struct TranslatableContents {
    key: String,
    fallback: Option<String>,
    args: Vec<Arg>,
    cache: RefCell<DecomposeCache>,
}

impl PartialEq for TranslatableContents {
    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#equals
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.fallback == other.fallback && self.args == other.args
    }
}

impl fmt::Display for TranslatableContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_java())
    }
}

impl TranslatableContents {
    pub const NO_ARGS: &'static [Arg] = &[];

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#<init>
    pub fn new(key: String, fallback: Option<String>, args: Vec<Arg>) -> Self {
        Self {
            key,
            fallback,
            args,
            cache: RefCell::new(DecomposeCache::default()),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#filterAllowedArguments
    pub fn filter_allowed_arguments(arg: &Arg) -> Result<(), String> {
        if Self::is_allowed_primitive_argument(arg) {
            Ok(())
        } else {
            Err("This value needs to be parsed as component".to_string())
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#isAllowedPrimitiveArgument
    pub fn is_allowed_primitive_argument(arg: &Arg) -> bool {
        !matches!(arg, Arg::Component(_))
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#adjustArgs
    pub fn adjust_args_to_optional(args: &[Arg]) -> Option<Vec<Arg>> {
        if args.is_empty() {
            None
        } else {
            Some(args.to_vec())
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#adjustArgs
    pub fn adjust_args_from_optional(args: Option<Vec<Arg>>) -> Vec<Arg> {
        match args {
            Some(v) if !v.is_empty() => v,
            _ => Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#create
    pub fn create(key: String, fallback: Option<String>, args: Option<Vec<Arg>>) -> Self {
        Self::new(key, fallback, Self::adjust_args_from_optional(args))
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#decompose
    fn decompose(&self) {
        let lang = get_language();
        let lang_id = lang.identity();
        {
            let cache = self.cache.borrow();
            if cache.decomposed_with == Some(lang_id) {
                return;
            }
        }
        let template = match &self.fallback {
            Some(fb) => lang.get_or_default_fallback(&self.key, fb),
            None => lang.get_or_default(&self.key),
        };
        {
            let mut cache = self.cache.borrow_mut();
            cache.decomposed_with = Some(lang_id);
        }
        let mut parts = Vec::new();
        let result = self.decompose_template(&template, &mut parts);
        let mut cache = self.cache.borrow_mut();
        match result {
            Ok(()) => cache.decomposed_parts = parts,
            Err(_) => cache.decomposed_parts = vec![FormattedText::Literal(template)],
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#decomposeTemplate
    fn decompose_template(
        &self,
        template: &str,
        out: &mut Vec<FormattedText>,
    ) -> Result<(), TranslatableFormatException> {
        let len = template.len();
        let mut cursor: usize = 0;
        let mut implicit_index: i32 = 0;
        let mut pos: usize = 0;

        while pos < len {
            let Some((match_start, match_end, explicit_idx_str, fmt_ch)) =
                find_format_match(template, pos)
            else {
                break;
            };

            if match_start > cursor {
                let segment = &template[cursor..match_start];
                if segment.contains('%') {
                    return Err(TranslatableFormatException::msg(&self.key, "Illegal argument"));
                }
                out.push(FormattedText::Literal(segment.to_string()));
            }

            let matched_str = &template[match_start..match_end];
            if fmt_ch == '%' && matched_str == "%%" {
                out.push(FormattedText::Literal("%".to_string()));
            } else if fmt_ch == 's' {
                let arg_idx = if let Some(ei_str) = explicit_idx_str {
                    let ei: i32 = ei_str
                        .parse()
                        .map_err(|_| TranslatableFormatException::msg(&self.key, "Illegal argument"))?;
                    ei.wrapping_sub(1)
                } else {
                    let idx = implicit_index;
                    implicit_index = implicit_index.wrapping_add(1);
                    idx
                };
                let arg = self.get_argument(arg_idx)?;
                out.push(arg);
            } else {
                return Err(TranslatableFormatException::msg(
                    &self.key,
                    &format!("Unsupported format: '{}'", matched_str),
                ));
            }

            cursor = match_end;
            pos = match_end;
        }

        if cursor < len {
            let segment = &template[cursor..];
            if segment.contains('%') {
                return Err(TranslatableFormatException::msg(&self.key, "Illegal argument"));
            }
            out.push(FormattedText::Literal(segment.to_string()));
        }

        Ok(())
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#getArgument
    fn get_argument(&self, index: i32) -> Result<FormattedText, TranslatableFormatException> {
        if index >= 0 && (index as usize) < self.args.len() {
            let arg = &self.args[index as usize];
            match arg {
                Arg::Component(c) => Ok(FormattedText::Component(c.clone())),
                _ => Ok(FormattedText::Literal(arg.to_java_string())),
            }
        } else {
            Err(TranslatableFormatException::index(&self.key, index))
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#visit
    pub fn visit_styled<T>(
        &self,
        consumer: &mut dyn FnMut(&Style, &str) -> Option<T>,
        style: &Style,
    ) -> Option<T> {
        self.decompose();
        let parts = self.cache.borrow().decomposed_parts.clone();
        for part in &parts {
            if let Some(r) = part.visit_styled(consumer, style) {
                return Some(r);
            }
        }
        None
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#visit
    pub fn visit_content<T>(&self, consumer: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        self.decompose();
        let parts = self.cache.borrow().decomposed_parts.clone();
        for part in &parts {
            if let Some(r) = part.visit_content(consumer) {
                return Some(r);
            }
        }
        None
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#resolve
    pub fn resolve(
        &self,
        source: Option<&CommandSourceStack>,
        entity: Option<&Entity>,
        depth: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        let mut resolved_args: Vec<Arg> = Vec::with_capacity(self.args.len());
        for arg in &self.args {
            match arg {
                Arg::Component(c) => {
                    let resolved = update_component_for_entity(source, c, entity, depth)?;
                    resolved_args.push(Arg::Component(resolved));
                }
                other => resolved_args.push(other.clone()),
            }
        }
        Ok(MutableComponent::create(TranslatableContents::new(
            self.key.clone(),
            self.fallback.clone(),
            resolved_args,
        )))
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut h: i32 = java_string_hash(&self.key);
        h = h.wrapping_mul(31).wrapping_add(match &self.fallback {
            Some(s) => java_string_hash(s),
            None => 0,
        });
        h = h
            .wrapping_mul(31)
            .wrapping_add(java_args_hash(&self.args));
        h
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#toString
    pub fn to_string_java(&self) -> String {
        let mut s = format!("translation{{key='{}'", self.key);
        if let Some(fb) = &self.fallback {
            s.push_str(&format!(", fallback='{}'", fb));
        }
        s.push_str(", args=");
        s.push_str(&java_args_to_string(&self.args));
        s.push('}');
        s
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#getKey
    pub fn get_key(&self) -> &str {
        &self.key
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#getFallback
    pub fn get_fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableContents#getArgs
    pub fn get_args(&self) -> &[Arg] {
        &self.args
    }
}

fn update_component_for_entity(
    _source: Option<&CommandSourceStack>,
    component: &Component,
    _entity: Option<&Entity>,
    _depth: i32,
) -> Result<Component, CommandSyntaxException> {
    Ok(component.clone())
}

fn find_format_match(template: &str, from: usize) -> Option<(usize, usize, Option<&str>, char)> {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = from;
    while i < len {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + 1;

        let digit_start = j;
        while j < len && bytes[j].is_ascii_digit() {
            j += 1;
        }
        let explicit_idx_str = if j > digit_start && j < len && bytes[j] == b'$' {
            let s = &template[digit_start..j];
            j += 1;
            Some(s)
        } else {
            j = i + 1;
            None
        };

        if j < len {
            let b = bytes[j];
            if b.is_ascii_alphabetic() || b == b'%' {
                return Some((start, j + 1, explicit_idx_str, b as char));
            }
            i += 1;
            continue;
        } else {
            return Some((start, j, explicit_idx_str, '\0'));
        }
    }
    None
}

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for cu in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(cu as i32);
    }
    h
}

fn java_float_to_int_bits(f: f32) -> i32 {
    if f.is_nan() {
        0x7fc0_0000u32 as i32
    } else {
        f.to_bits() as i32
    }
}

fn java_double_to_long_bits(f: f64) -> i64 {
    if f.is_nan() {
        0x7ff8_0000_0000_0000u64 as i64
    } else {
        f.to_bits() as i64
    }
}

fn java_float_to_string(v: f32) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    let s = format!("{}", v);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn java_double_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    let s = format!("{}", v);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn java_arg_hash(arg: &Arg) -> i32 {
    match arg {
        Arg::Bool(b) => {
            if *b {
                1231
            } else {
                1237
            }
        }
        Arg::Int(n) => *n,
        Arg::Long(n) => ((*n as u64 ^ ((*n as u64) >> 32)) as i32),
        Arg::Float(f) => java_float_to_int_bits(*f),
        Arg::Double(d) => {
            let bits = java_double_to_long_bits(*d);
            ((bits as u64 ^ ((bits as u64) >> 32)) as i32)
        }
        Arg::String(s) => java_string_hash(s),
        Arg::Component(c) => match c {
            Component::Literal(s) => java_string_hash(s),
            Component::Translatable(t) => t.hash_code(),
        },
    }
}

fn java_args_hash(args: &[Arg]) -> i32 {
    let mut result: i32 = 1;
    for arg in args {
        result = result.wrapping_mul(31).wrapping_add(java_arg_hash(arg));
    }
    result
}

fn java_args_to_string(args: &[Arg]) -> String {
    let mut s = String::from("[");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&arg.to_java_string());
    }
    s.push(']');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct MapLanguage {
        map: HashMap<String, String>,
        id: usize,
    }

    impl Language for MapLanguage {
        fn get_or_default(&self, key: &str) -> String {
            self.map
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.to_string())
        }

        fn get_or_default_fallback(&self, key: &str, fallback: &str) -> String {
            self.map
                .get(key)
                .cloned()
                .unwrap_or_else(|| fallback.to_string())
        }

        fn identity(&self) -> usize {
            self.id
        }
    }

    #[test]
    fn decompose_basic_and_percent() {
        let mut map = HashMap::new();
        map.insert("greet".to_string(), "Hello %s! 100%%".to_string());
        set_language(Arc::new(MapLanguage { map, id: 10 }));
        let c = TranslatableContents::new(
            "greet".to_string(),
            None,
            vec![Arg::String("world".to_string())],
        );
        let mut out = Vec::new();
        c.visit_content(&mut |s: &str| {
            out.push(s.to_string());
            None::<()>
        });
        assert_eq!(out.join(""), "Hello world! 100%");
    }

    #[test]
    fn explicit_index_and_out_of_range() {
        let mut map = HashMap::new();
        map.insert("swap".to_string(), "%2$s-%1$s".to_string());
        set_language(Arc::new(MapLanguage { map, id: 20 }));
        let c = TranslatableContents::new(
            "swap".to_string(),
            None,
            vec![Arg::String("A".to_string()), Arg::String("B".to_string())],
        );
        let mut out = Vec::new();
        c.visit_content(&mut |s: &str| {
            out.push(s.to_string());
            None::<()>
        });
        assert_eq!(out.join(""), "B-A");

        let mut map2 = HashMap::new();
        map2.insert("bad".to_string(), "%5$s".to_string());
        set_language(Arc::new(MapLanguage { map: map2, id: 21 }));
        let c2 = TranslatableContents::new("bad".to_string(), None, vec![]);
        let mut out2 = Vec::new();
        c2.visit_content(&mut |s: &str| {
            out2.push(s.to_string());
            None::<()>
        });
        assert_eq!(out2.join(""), "%5$s");
    }

    #[test]
    fn negative_and_overflow_index() {
        let mut map = HashMap::new();
        map.insert("neg".to_string(), "%0$s".to_string());
        map.insert("ovf".to_string(), "%2147483648$s".to_string());
        set_language(Arc::new(MapLanguage { map, id: 22 }));

        let c1 = TranslatableContents::new("neg".to_string(), None, vec![Arg::String("x".to_string())]);
        let mut out1 = Vec::new();
        c1.visit_content(&mut |s: &str| {
            out1.push(s.to_string());
            None::<()>
        });
        assert_eq!(out1.join(""), "%0$s");

        let c2 = TranslatableContents::new("ovf".to_string(), None, vec![Arg::String("x".to_string())]);
        let mut out2 = Vec::new();
        c2.visit_content(&mut |s: &str| {
            out2.push(s.to_string());
            None::<()>
        });
        assert_eq!(out2.join(""), "%2147483648$s");
    }

    #[test]
    fn float_bits_nan_and_hash() {
        let nan1 = f32::NAN;
        let nan2 = f32::from_bits(0x7fc0_0001);
        assert_eq!(java_float_to_int_bits(nan1), 0x7fc0_0000u32 as i32);
        assert_eq!(java_float_to_int_bits(nan2), 0x7fc0_0000u32 as i32);
        assert_eq!(
            java_double_to_long_bits(f64::NAN),
            0x7ff8_0000_0000_0000u64 as i64
        );
        assert_ne!(
            java_float_to_int_bits(-0.0f32),
            java_float_to_int_bits(0.0f32)
        );

        let c = TranslatableContents::new("key".to_string(), None, vec![Arg::Float(f32::NAN)]);
        let mut expected = java_string_hash("key");
        expected = expected.wrapping_mul(31).wrapping_add(0);
        expected = expected
            .wrapping_mul(31)
            .wrapping_add(java_args_hash(&[Arg::Float(f32::NAN)]));
        assert_eq!(c.hash_code(), expected);
    }

    #[test]
    fn equals_hash_to_string_and_resolve() {
        let a = TranslatableContents::new(
            "k".to_string(),
            Some("fb".to_string()),
            vec![Arg::Int(1), Arg::String("x".to_string())],
        );
        let b = TranslatableContents::new(
            "k".to_string(),
            Some("fb".to_string()),
            vec![Arg::Int(1), Arg::String("x".to_string())],
        );
        let c = TranslatableContents::new("k".to_string(), None, vec![Arg::Int(1)]);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.hash_code(), b.hash_code());
        let s = a.to_string_java();
        assert!(s.contains("translation{key='k'"));
        assert!(s.contains("fallback='fb'"));
        assert!(s.contains("args=[1, x]"));

        let resolved = a.resolve(None, None, 0).unwrap();
        assert_eq!(resolved.contents.get_key(), "k");
        assert_eq!(resolved.contents.get_fallback(), Some("fb"));
    }

    #[test]
    fn primitive_check_and_adjust_args() {
        assert!(TranslatableContents::is_allowed_primitive_argument(&Arg::Int(1)));
        assert!(TranslatableContents::is_allowed_primitive_argument(&Arg::Bool(true)));
        assert!(TranslatableContents::is_allowed_primitive_argument(&Arg::String(
            "s".to_string()
        )));
        assert!(!TranslatableContents::is_allowed_primitive_argument(
            &Arg::Component(Component::literal("x"))
        ));
        assert!(TranslatableContents::filter_allowed_arguments(&Arg::Int(1)).is_ok());
        assert!(TranslatableContents::filter_allowed_arguments(&Arg::Component(
            Component::literal("x")
        ))
        .is_err());

        assert_eq!(TranslatableContents::adjust_args_to_optional(&[]), None);
        let args = vec![Arg::Int(1)];
        assert_eq!(
            TranslatableContents::adjust_args_to_optional(&args),
            Some(args.clone())
        );
        assert_eq!(
            TranslatableContents::adjust_args_from_optional(None),
            Vec::<Arg>::new()
        );
        assert_eq!(
            TranslatableContents::adjust_args_from_optional(Some(vec![])),
            Vec::<Arg>::new()
        );
    }
}
