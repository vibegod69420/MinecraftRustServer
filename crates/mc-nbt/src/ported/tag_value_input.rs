use crate::NbtTag;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Problem {
    DecodeFromFieldFailed {
        name: String,
        tag: String,
        error: String,
    },
    DecodeFromListFailed {
        name: String,
        index: usize,
        tag: String,
        error: String,
    },
    DecodeFromMapFailed {
        error: String,
    },
    UnexpectedListElementType {
        name: String,
        index: usize,
        expected: String,
        actual: String,
    },
    UnexpectedNonNumber {
        name: String,
        actual: String,
    },
    UnexpectedType {
        name: String,
        expected: String,
        actual: String,
    },
}

impl Problem {
    // JAVA: net.minecraft.world.level.storage.TagValueInput$DecodeFromFieldFailedProblem#description
    // JAVA: net.minecraft.world.level.storage.TagValueInput$DecodeFromListFailedProblem#description
    // JAVA: net.minecraft.world.level.storage.TagValueInput$DecodeFromMapFailedProblem#description
    // JAVA: net.minecraft.world.level.storage.TagValueInput$UnexpectedListElementTypeProblem#description
    // JAVA: net.minecraft.world.level.storage.TagValueInput$UnexpectedNonNumberProblem#description
    // JAVA: net.minecraft.world.level.storage.TagValueInput$UnexpectedTypeProblem#description
    pub fn description(&self) -> String {
        match self {
            Problem::DecodeFromFieldFailed { name, tag, error } => {
                format!("Failed to decode value '{}' from field '{}': {}", tag, name, error)
            }
            Problem::DecodeFromListFailed {
                name,
                index,
                tag,
                error,
            } => format!(
                "Failed to decode value '{}' from field '{}' at index {}: {}",
                tag, name, index, error
            ),
            Problem::DecodeFromMapFailed { error } => format!("Failed to decode from map: {}", error),
            Problem::UnexpectedListElementType {
                name,
                index,
                expected,
                actual,
            } => format!(
                "Expected list '{}' to contain at index {} value of type {}, but got {}",
                name, index, expected, actual
            ),
            Problem::UnexpectedNonNumber { name, actual } => {
                format!("Expected field '{}' to contain number, but got {}", name, actual)
            }
            Problem::UnexpectedType {
                name,
                expected,
                actual,
            } => format!(
                "Expected field '{}' to contain value of type {}, but got {}",
                name, expected, actual
            ),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.description())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProblemReporter {
    problems: Rc<RefCell<Vec<Problem>>>,
}

impl ProblemReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report(&self, problem: Problem) {
        self.problems.borrow_mut().push(problem);
    }

    pub fn for_child_field(&self, _name: &str) -> Self {
        Self {
            problems: self.problems.clone(),
        }
    }

    pub fn for_child_index(&self, _index: usize) -> Self {
        Self {
            problems: self.problems.clone(),
        }
    }

    pub fn for_child_indexed_field(&self, _name: &str, _index: usize) -> Self {
        Self {
            problems: self.problems.clone(),
        }
    }

    pub fn problems(&self) -> Vec<Problem> {
        self.problems.borrow().clone()
    }

    pub fn is_empty(&self) -> bool {
        self.problems.borrow().is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbtType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

impl NbtType {
    pub fn from_tag(tag: &NbtTag) -> Self {
        match tag {
            NbtTag::Byte(_) => Self::Byte,
            NbtTag::Short(_) => Self::Short,
            NbtTag::Int(_) => Self::Int,
            NbtTag::Long(_) => Self::Long,
            NbtTag::Float(_) => Self::Float,
            NbtTag::Double(_) => Self::Double,
            NbtTag::ByteArray(_) => Self::ByteArray,
            NbtTag::String(_) => Self::String,
            NbtTag::List { .. } => Self::List,
            NbtTag::Compound(_) => Self::Compound,
            NbtTag::IntArray(_) => Self::IntArray,
            NbtTag::LongArray(_) => Self::LongArray,
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            Self::End => "END",
            Self::Byte => "BYTE",
            Self::Short => "SHORT",
            Self::Int => "INT",
            Self::Long => "LONG",
            Self::Float => "FLOAT",
            Self::Double => "DOUBLE",
            Self::ByteArray => "BYTE[]",
            Self::String => "STRING",
            Self::List => "LIST",
            Self::Compound => "COMPOUND",
            Self::IntArray => "INT[]",
            Self::LongArray => "LONG[]",
        }
    }
}

fn is_numeric(tag: &NbtTag) -> bool {
    matches!(
        tag,
        NbtTag::Byte(_)
            | NbtTag::Short(_)
            | NbtTag::Int(_)
            | NbtTag::Long(_)
            | NbtTag::Float(_)
            | NbtTag::Double(_)
    )
}

fn numeric_as_i8(tag: &NbtTag) -> Option<i8> {
    Some(match tag {
        NbtTag::Byte(v) => *v,
        NbtTag::Short(v) => *v as i8,
        NbtTag::Int(v) => *v as i8,
        NbtTag::Long(v) => *v as i8,
        NbtTag::Float(v) => (v.floor() as i32) as i8,
        NbtTag::Double(v) => (v.floor() as i32) as i8,
        _ => return None,
    })
}

fn numeric_as_i16(tag: &NbtTag) -> Option<i16> {
    Some(match tag {
        NbtTag::Byte(v) => *v as i16,
        NbtTag::Short(v) => *v,
        NbtTag::Int(v) => *v as i16,
        NbtTag::Long(v) => *v as i16,
        NbtTag::Float(v) => (v.floor() as i32) as i16,
        NbtTag::Double(v) => (v.floor() as i32) as i16,
        _ => return None,
    })
}

fn numeric_as_i32(tag: &NbtTag) -> Option<i32> {
    Some(match tag {
        NbtTag::Byte(v) => *v as i32,
        NbtTag::Short(v) => *v as i32,
        NbtTag::Int(v) => *v,
        NbtTag::Long(v) => *v as i32,
        NbtTag::Float(v) => v.floor() as i32,
        NbtTag::Double(v) => v.floor() as i32,
        _ => return None,
    })
}

fn numeric_as_i64(tag: &NbtTag) -> Option<i64> {
    Some(match tag {
        NbtTag::Byte(v) => *v as i64,
        NbtTag::Short(v) => *v as i64,
        NbtTag::Int(v) => *v as i64,
        NbtTag::Long(v) => *v,
        NbtTag::Float(v) => *v as i64,
        NbtTag::Double(v) => v.floor() as i64,
        _ => return None,
    })
}

fn numeric_as_f32(tag: &NbtTag) -> Option<f32> {
    Some(match tag {
        NbtTag::Byte(v) => *v as f32,
        NbtTag::Short(v) => *v as f32,
        NbtTag::Int(v) => *v as f32,
        NbtTag::Long(v) => *v as f32,
        NbtTag::Float(v) => *v,
        NbtTag::Double(v) => *v as f32,
        _ => return None,
    })
}

fn numeric_as_f64(tag: &NbtTag) -> Option<f64> {
    Some(match tag {
        NbtTag::Byte(v) => *v as f64,
        NbtTag::Short(v) => *v as f64,
        NbtTag::Int(v) => *v as f64,
        NbtTag::Long(v) => *v as f64,
        NbtTag::Float(v) => *v as f64,
        NbtTag::Double(v) => *v,
        _ => return None,
    })
}

fn format_tag(tag: &NbtTag) -> String {
    match tag {
        NbtTag::Byte(v) => format!("{}b", v),
        NbtTag::Short(v) => format!("{}s", v),
        NbtTag::Int(v) => format!("{}", v),
        NbtTag::Long(v) => format!("{}L", v),
        NbtTag::Float(v) => format!("{}f", v),
        NbtTag::Double(v) => format!("{}d", v),
        NbtTag::String(s) => format!("\"{}\"", s),
        NbtTag::ByteArray(v) => format!("[B; {} bytes]", v.len()),
        NbtTag::IntArray(v) => format!("[I; {} ints]", v.len()),
        NbtTag::LongArray(v) => format!("[L; {} longs]", v.len()),
        NbtTag::List { items, .. } => format!("[{} entries]", items.len()),
        NbtTag::Compound(entries) => format!("{{{} entries}}", entries.len()),
    }
}

#[derive(Clone, Debug)]
pub enum DecodeResult<T> {
    Success(T),
    Error { message: String, partial: Option<T> },
}

pub trait DynamicOpsAny {
    fn get_map_entries<'a>(&self, tag: &'a NbtTag) -> Result<&'a [(String, NbtTag)], String>;
}

#[derive(Clone, Debug, Default)]
pub struct NbtOps;

impl DynamicOpsAny for NbtOps {
    fn get_map_entries<'a>(&self, tag: &'a NbtTag) -> Result<&'a [(String, NbtTag)], String> {
        match tag {
            NbtTag::Compound(entries) => Ok(entries.as_slice()),
            _ => Err(format!("Not a map: {}", NbtType::from_tag(tag).type_name())),
        }
    }
}

pub trait Codec<T> {
    fn parse(&self, ops: &dyn DynamicOpsAny, tag: &NbtTag) -> DecodeResult<T>;
}

pub trait MapCodec<T> {
    fn decode(&self, ops: &dyn DynamicOpsAny, entries: &[(String, NbtTag)]) -> DecodeResult<T>;
}

#[derive(Clone)]
pub struct ValueInputContextHelper<L: Clone> {
    lookup_provider: L,
    ops: NbtOps,
}

impl<L: Clone> ValueInputContextHelper<L> {
    pub fn new(lookup_provider: L, ops: NbtOps) -> Self {
        Self { lookup_provider, ops }
    }

    pub fn lookup(&self) -> &L {
        &self.lookup_provider
    }

    pub fn ops(&self) -> &NbtOps {
        &self.ops
    }

    pub fn empty(&self) -> TagValueInput<L> {
        TagValueInput::new(ProblemReporter::new(), self.clone(), Vec::new())
    }

    pub fn empty_list(&self) -> ListWrapper<L> {
        ListWrapper::empty(self.clone())
    }
}

pub struct TagValueInput<L: Clone> {
    problem_reporter: ProblemReporter,
    context: ValueInputContextHelper<L>,
    input: Vec<(String, NbtTag)>,
}

impl<L: Clone> TagValueInput<L> {
    // JAVA: net.minecraft.world.level.storage.TagValueInput#<init>
    fn new(
        problem_reporter: ProblemReporter,
        context: ValueInputContextHelper<L>,
        input: Vec<(String, NbtTag)>,
    ) -> Self {
        Self {
            problem_reporter,
            context,
            input,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#create
    pub fn create(problem_reporter: ProblemReporter, lookup: L, input: Vec<(String, NbtTag)>) -> Self {
        Self::new(
            problem_reporter,
            ValueInputContextHelper::new(lookup, NbtOps),
            input,
        )
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#create
    pub fn create_list(
        problem_reporter: ProblemReporter,
        lookup: L,
        list: Vec<Vec<(String, NbtTag)>>,
    ) -> CompoundListWrapper<L> {
        CompoundListWrapper::new(
            problem_reporter,
            ValueInputContextHelper::new(lookup, NbtOps),
            list,
        )
    }

    fn compound_get(&self, name: &str) -> Option<&NbtTag> {
        self.input
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#read
    pub fn read<T>(&self, name: &str, codec: &dyn Codec<T>) -> Option<T> {
        let tag = self.compound_get(name)?;
        match codec.parse(self.context.ops(), tag) {
            DecodeResult::Success(v) => Some(v),
            DecodeResult::Error { message, partial } => {
                self.problem_reporter.report(Problem::DecodeFromFieldFailed {
                    name: name.to_string(),
                    tag: format_tag(tag),
                    error: message,
                });
                partial
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#read
    pub fn read_map<T>(&self, codec: &dyn MapCodec<T>) -> Option<T> {
        let ops = self.context.ops();
        let compound_tag = NbtTag::Compound(self.input.clone());
        let map_result = ops.get_map_entries(&compound_tag);
        match map_result {
            Ok(entries) => match codec.decode(ops, entries) {
                DecodeResult::Success(v) => Some(v),
                DecodeResult::Error { message, partial } => {
                    self.problem_reporter
                        .report(Problem::DecodeFromMapFailed { error: message });
                    partial
                }
            },
            Err(msg) => {
                self.problem_reporter
                    .report(Problem::DecodeFromMapFailed { error: msg });
                None
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getOptionalTypedTag
    fn get_optional_typed_tag(&self, name: &str, expected: NbtType) -> Option<&NbtTag> {
        let tag = self.compound_get(name)?;
        let actual = NbtType::from_tag(tag);
        if actual != expected {
            self.problem_reporter.report(Problem::UnexpectedType {
                name: name.to_string(),
                expected: expected.type_name().to_string(),
                actual: actual.type_name().to_string(),
            });
            return None;
        }
        Some(tag)
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getNumericTag
    fn get_numeric_tag(&self, name: &str) -> Option<&NbtTag> {
        let tag = self.compound_get(name)?;
        if is_numeric(tag) {
            Some(tag)
        } else {
            self.problem_reporter.report(Problem::UnexpectedNonNumber {
                name: name.to_string(),
                actual: NbtType::from_tag(tag).type_name().to_string(),
            });
            None
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#child
    pub fn child(&self, name: &str) -> Option<TagValueInput<L>> {
        let tag = self.get_optional_typed_tag(name, NbtType::Compound)?;
        let entries = match tag {
            NbtTag::Compound(e) => e.clone(),
            _ => return None,
        };
        Some(self.wrap_child(name, entries))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#childOrEmpty
    pub fn child_or_empty(&self, name: &str) -> TagValueInput<L> {
        let tag = self.get_optional_typed_tag(name, NbtType::Compound);
        match tag {
            Some(NbtTag::Compound(e)) => self.wrap_child(name, e.clone()),
            _ => self.context.empty(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#childrenList
    pub fn children_list(&self, name: &str) -> Option<ListWrapper<L>> {
        let tag = self.get_optional_typed_tag(name, NbtType::List)?;
        let list = tag.clone();
        Some(self.wrap_list(name, &self.context.clone(), list))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#childrenListOrEmpty
    pub fn children_list_or_empty(&self, name: &str) -> ListWrapper<L> {
        let tag = self.get_optional_typed_tag(name, NbtType::List);
        match tag {
            Some(t @ NbtTag::List { .. }) => self.wrap_list(name, &self.context.clone(), t.clone()),
            _ => self.context.empty_list(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#list
    pub fn list<T, C: Codec<T>>(&self, name: &str, codec: C) -> Option<TypedListWrapper<T, L, C>> {
        let tag = self.get_optional_typed_tag(name, NbtType::List)?;
        let list = tag.clone();
        Some(self.wrap_typed_list(name, list, codec))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#listOrEmpty
    pub fn list_or_empty<T, C: Codec<T>>(&self, name: &str, codec: C) -> TypedListWrapper<T, L, C> {
        let tag = self.get_optional_typed_tag(name, NbtType::List);
        match tag {
            Some(t @ NbtTag::List { .. }) => self.wrap_typed_list(name, t.clone(), codec),
            _ => TypedListWrapper::empty(self.context.clone(), codec),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getBooleanOr
    pub fn get_boolean_or(&self, name: &str, default: bool) -> bool {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_i8(t).unwrap_or(0) != 0,
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getByteOr
    pub fn get_byte_or(&self, name: &str, default: i8) -> i8 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_i8(t).unwrap_or(default),
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getShortOr
    pub fn get_short_or(&self, name: &str, default: i16) -> i32 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_i16(t).unwrap_or(default) as i32,
            None => default as i32,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getInt
    pub fn get_int(&self, name: &str) -> Option<i32> {
        let tag = self.get_numeric_tag(name)?;
        Some(numeric_as_i32(tag).unwrap_or(0))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getIntOr
    pub fn get_int_or(&self, name: &str, default: i32) -> i32 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_i32(t).unwrap_or(default),
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getLongOr
    pub fn get_long_or(&self, name: &str, default: i64) -> i64 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_i64(t).unwrap_or(default),
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getLong
    pub fn get_long(&self, name: &str) -> Option<i64> {
        let tag = self.get_numeric_tag(name)?;
        Some(numeric_as_i64(tag).unwrap_or(0))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getFloatOr
    pub fn get_float_or(&self, name: &str, default: f32) -> f32 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_f32(t).unwrap_or(default),
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getDoubleOr
    pub fn get_double_or(&self, name: &str, default: f64) -> f64 {
        match self.get_numeric_tag(name) {
            Some(t) => numeric_as_f64(t).unwrap_or(default),
            None => default,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getString
    pub fn get_string(&self, name: &str) -> Option<String> {
        let tag = self.get_optional_typed_tag(name, NbtType::String)?;
        match tag {
            NbtTag::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getStringOr
    pub fn get_string_or(&self, name: &str, default: &str) -> String {
        let tag = self.get_optional_typed_tag(name, NbtType::String);
        match tag {
            Some(NbtTag::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#getIntArray
    pub fn get_int_array(&self, name: &str) -> Option<Vec<i32>> {
        let tag = self.get_optional_typed_tag(name, NbtType::IntArray)?;
        match tag {
            NbtTag::IntArray(v) => Some(v.clone()),
            _ => None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#lookup
    pub fn lookup(&self) -> &L {
        self.context.lookup()
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#wrapChild
    fn wrap_child(&self, name: &str, compound: Vec<(String, NbtTag)>) -> TagValueInput<L> {
        if compound.is_empty() {
            self.context.empty()
        } else {
            TagValueInput::new(
                self.problem_reporter.for_child_field(name),
                self.context.clone(),
                compound,
            )
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#wrapChild
    fn wrap_child_static(
        problem_reporter: ProblemReporter,
        context: ValueInputContextHelper<L>,
        compound: Vec<(String, NbtTag)>,
    ) -> TagValueInput<L> {
        if compound.is_empty() {
            context.empty()
        } else {
            TagValueInput::new(problem_reporter, context, compound)
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#wrapList
    fn wrap_list(
        &self,
        name: &str,
        context: &ValueInputContextHelper<L>,
        list: NbtTag,
    ) -> ListWrapper<L> {
        match list {
            NbtTag::List { items, .. } if items.is_empty() => ListWrapper::empty(context.clone()),
            NbtTag::List { items, .. } => ListWrapper::new(
                self.problem_reporter.clone(),
                name.to_string(),
                context.clone(),
                items,
            ),
            _ => ListWrapper::empty(context.clone()),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput#wrapTypedList
    fn wrap_typed_list<T, C: Codec<T>>(
        &self,
        name: &str,
        list: NbtTag,
        codec: C,
    ) -> TypedListWrapper<T, L, C> {
        match list {
            NbtTag::List { items, .. } if items.is_empty() => {
                TypedListWrapper::empty(self.context.clone(), codec)
            }
            NbtTag::List { items, .. } => TypedListWrapper::new(
                self.problem_reporter.clone(),
                name.to_string(),
                self.context.clone(),
                codec,
                items,
            ),
            _ => TypedListWrapper::empty(self.context.clone(), codec),
        }
    }
}

pub struct CompoundListWrapper<L: Clone> {
    problem_reporter: ProblemReporter,
    context: ValueInputContextHelper<L>,
    list: Vec<Vec<(String, NbtTag)>>,
}

impl<L: Clone> CompoundListWrapper<L> {
    // JAVA: net.minecraft.world.level.storage.TagValueInput$CompoundListWrapper#<init>
    pub fn new(
        problem_reporter: ProblemReporter,
        context: ValueInputContextHelper<L>,
        list: Vec<Vec<(String, NbtTag)>>,
    ) -> Self {
        Self {
            problem_reporter,
            context,
            list,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$CompoundListWrapper#wrapChild
    fn wrap_child(&self, index: usize, compound: Vec<(String, NbtTag)>) -> TagValueInput<L> {
        TagValueInput::wrap_child_static(
            self.problem_reporter.for_child_index(index),
            self.context.clone(),
            compound,
        )
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$CompoundListWrapper#isEmpty
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$CompoundListWrapper#stream
    pub fn stream(&self) -> impl Iterator<Item = TagValueInput<L>> + '_ {
        self.list
            .iter()
            .enumerate()
            .map(|(i, c)| self.wrap_child(i, c.clone()))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$CompoundListWrapper#iterator
    pub fn iter(&self) -> impl Iterator<Item = TagValueInput<L>> + '_ {
        self.stream()
    }
}

pub struct ListWrapper<L: Clone> {
    problem_reporter: ProblemReporter,
    name: String,
    context: ValueInputContextHelper<L>,
    list: Vec<NbtTag>,
}

impl<L: Clone> ListWrapper<L> {
    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#<init>
    fn new(
        problem_reporter: ProblemReporter,
        name: String,
        context: ValueInputContextHelper<L>,
        list: Vec<NbtTag>,
    ) -> Self {
        Self {
            problem_reporter,
            name,
            context,
            list,
        }
    }

    fn empty(context: ValueInputContextHelper<L>) -> Self {
        Self {
            problem_reporter: ProblemReporter::new(),
            name: String::new(),
            context,
            list: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#isEmpty
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#reporterForChild
    fn reporter_for_child(&self, index: usize) -> ProblemReporter {
        self.problem_reporter
            .for_child_indexed_field(&self.name, index)
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#reportIndexUnwrapProblem
    fn report_index_unwrap_problem(&self, index: usize, tag: &NbtTag) {
        self.problem_reporter
            .report(Problem::UnexpectedListElementType {
                name: self.name.clone(),
                index,
                expected: NbtType::Compound.type_name().to_string(),
                actual: NbtType::from_tag(tag).type_name().to_string(),
            });
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#stream
    pub fn stream(&self) -> impl Iterator<Item = TagValueInput<L>> + '_ {
        self.list.iter().enumerate().filter_map(|(i, tag)| {
            if let NbtTag::Compound(entries) = tag {
                Some(TagValueInput::wrap_child_static(
                    self.reporter_for_child(i),
                    self.context.clone(),
                    entries.clone(),
                ))
            } else {
                self.report_index_unwrap_problem(i, tag);
                None
            }
        })
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$ListWrapper#iterator
    pub fn iter(&self) -> impl Iterator<Item = TagValueInput<L>> + '_ {
        self.stream()
    }
}

pub struct TypedListWrapper<T, L: Clone, C: Codec<T>> {
    problem_reporter: ProblemReporter,
    name: String,
    context: ValueInputContextHelper<L>,
    codec: C,
    list: Vec<NbtTag>,
    _marker: PhantomData<T>,
}

impl<T, L: Clone, C: Codec<T>> TypedListWrapper<T, L, C> {
    // JAVA: net.minecraft.world.level.storage.TagValueInput$TypedListWrapper#<init>
    fn new(
        problem_reporter: ProblemReporter,
        name: String,
        context: ValueInputContextHelper<L>,
        codec: C,
        list: Vec<NbtTag>,
    ) -> Self {
        Self {
            problem_reporter,
            name,
            context,
            codec,
            list,
            _marker: PhantomData,
        }
    }

    fn empty(context: ValueInputContextHelper<L>, codec: C) -> Self {
        Self {
            problem_reporter: ProblemReporter::new(),
            name: String::new(),
            context,
            codec,
            list: Vec::new(),
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$TypedListWrapper#isEmpty
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$TypedListWrapper#reportIndexUnwrapProblem
    fn report_index_unwrap_problem(&self, index: usize, tag: &NbtTag, error: &str) {
        self.problem_reporter
            .report(Problem::DecodeFromListFailed {
                name: self.name.clone(),
                index,
                tag: format_tag(tag),
                error: error.to_string(),
            });
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$TypedListWrapper#stream
    pub fn stream(&self) -> impl Iterator<Item = T> + '_ {
        self.list.iter().enumerate().filter_map(|(i, tag)| {
            match self.codec.parse(self.context.ops(), tag) {
                DecodeResult::Success(v) => Some(v),
                DecodeResult::Error { message, partial } => {
                    self.report_index_unwrap_problem(i, tag, &message);
                    partial
                }
            }
        })
    }

    // JAVA: net.minecraft.world.level.storage.TagValueInput$TypedListWrapper#iterator
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.stream()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct DummyLookup;

    #[test]
    fn scalar_reads_defaults_and_type_errors() {
        let reporter = ProblemReporter::new();
        let input = vec![
            ("b".to_string(), NbtTag::Byte(1)),
            ("i".to_string(), NbtTag::Int(-42)),
            ("s".to_string(), NbtTag::String("hello".to_string())),
        ];
        let vi = TagValueInput::create(reporter.clone(), DummyLookup, input);

        assert!(vi.get_boolean_or("b", false));
        assert_eq!(vi.get_byte_or("b", 0), 1);
        assert_eq!(vi.get_int_or("i", 0), -42);
        assert_eq!(vi.get_int_or("missing", 99), 99);
        assert_eq!(vi.get_string_or("s", "def"), "hello");
        assert_eq!(vi.get_string_or("i", "def"), "def");
        assert!(!reporter.is_empty());
        assert!(reporter
            .problems()
            .iter()
            .any(|p| matches!(p, Problem::UnexpectedType { .. })));
    }

    #[test]
    fn numeric_narrowing_negative_and_overflow() {
        let reporter = ProblemReporter::new();
        let input = vec![
            ("big".to_string(), NbtTag::Int(0x1FF)),
            ("neg".to_string(), NbtTag::Byte(-1)),
            ("longv".to_string(), NbtTag::Long(i64::MIN)),
            ("shortv".to_string(), NbtTag::Short(-32000)),
        ];
        let vi = TagValueInput::create(reporter, DummyLookup, input);

        assert_eq!(vi.get_byte_or("big", 0), -1i8); // 0x1FF truncates to i8
        assert_eq!(vi.get_int_or("neg", 0), -1);
        assert_eq!(vi.get_int_or("longv", 123), 0i32); // i64::MIN as i32
        assert_eq!(vi.get_long_or("longv", 0), i64::MIN);
        assert_eq!(vi.get_short_or("shortv", 0), -32000i32);
    }

    #[test]
    fn float_and_double_bits() {
        let reporter = ProblemReporter::new();
        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let neg_zero = -0.0f64;
        let input = vec![
            ("nan".to_string(), NbtTag::Float(nan_f32)),
            ("nz".to_string(), NbtTag::Double(neg_zero)),
            ("f".to_string(), NbtTag::Float(1.5f32)),
        ];
        let vi = TagValueInput::create(reporter, DummyLookup, input);

        let got_nan = vi.get_float_or("nan", 0.0);
        assert!(got_nan.is_nan());
        assert_eq!(got_nan.to_bits(), nan_f32.to_bits());

        let got_nz = vi.get_double_or("nz", 1.0);
        assert_eq!(got_nz.to_bits(), neg_zero.to_bits());
        assert!(got_nz.is_sign_negative());

        assert_eq!(vi.get_int_or("f", 0), 1); // floor(1.5) == 1
    }

    #[test]
    fn child_and_list_wrappers() {
        let reporter = ProblemReporter::new();
        let list = NbtTag::List {
            elem_type: 10,
            items: vec![
                NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(1))]),
                NbtTag::Int(999),
                NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(2))]),
            ],
        };
        let input = vec![
            ("child".to_string(), NbtTag::Compound(vec![("x".to_string(), NbtTag::Int(7))])),
            ("list".to_string(), list),
        ];
        let vi = TagValueInput::create(reporter.clone(), DummyLookup, input);

        let child = vi.child("child").expect("child present");
        assert_eq!(child.get_int_or("x", 0), 7);
        assert!(vi.child("missing").is_none());
        assert_eq!(vi.child_or_empty("missing").get_int_or("x", 5), 5);

        let lw = vi.children_list("list").expect("list present");
        let items: Vec<_> = lw.stream().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get_int_or("v", 0), 1);
        assert_eq!(items[1].get_int_or("v", 0), 2);
        assert!(reporter
            .problems()
            .iter()
            .any(|p| matches!(p, Problem::UnexpectedListElementType { .. })));
    }

    #[test]
    fn typed_list_decode_and_partial() {
        struct MaybeIntCodec;
        impl Codec<i32> for MaybeIntCodec {
            fn parse(&self, _ops: &dyn DynamicOpsAny, tag: &NbtTag) -> DecodeResult<i32> {
                match tag {
                    NbtTag::Int(v) => DecodeResult::Success(*v),
                    NbtTag::String(_) => DecodeResult::Error {
                        message: "bad".to_string(),
                        partial: Some(-1),
                    },
                    _ => DecodeResult::Error {
                        message: "bad".to_string(),
                        partial: None,
                    },
                }
            }
        }

        let reporter = ProblemReporter::new();
        let list = NbtTag::List {
            elem_type: 3,
            items: vec![
                NbtTag::Int(10),
                NbtTag::String("oops".to_string()),
                NbtTag::Byte(0),
                NbtTag::Int(20),
            ],
        };
        let input = vec![("vals".to_string(), list)];
        let vi = TagValueInput::create(reporter.clone(), DummyLookup, input);

        let tl = vi.list("vals", MaybeIntCodec).expect("typed list");
        let vals: Vec<i32> = tl.stream().collect();
        assert_eq!(vals, vec![10, -1, 20]);
        assert_eq!(
            reporter
                .problems()
                .iter()
                .filter(|p| matches!(p, Problem::DecodeFromListFailed { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn int_array_boolean_and_read_codec() {
        struct IntCodec;
        impl Codec<i32> for IntCodec {
            fn parse(&self, _ops: &dyn DynamicOpsAny, tag: &NbtTag) -> DecodeResult<i32> {
                match tag {
                    NbtTag::Int(v) => DecodeResult::Success(*v),
                    _ => DecodeResult::Error {
                        message: "not int".to_string(),
                        partial: None,
                    },
                }
            }
        }

        let reporter = ProblemReporter::new();
        let input = vec![
            ("arr".to_string(), NbtTag::IntArray(vec![1, -2, 3])),
            ("flag".to_string(), NbtTag::Byte(0)),
            ("flag2".to_string(), NbtTag::Int(2)),
            ("num".to_string(), NbtTag::Int(77)),
        ];
        let vi = TagValueInput::create(reporter.clone(), DummyLookup, input);

        assert_eq!(vi.get_int_array("arr"), Some(vec![1, -2, 3]));
        assert_eq!(vi.get_int_array("missing"), None);
        assert!(!vi.get_boolean_or("flag", true));
        assert!(vi.get_boolean_or("flag2", false));
        assert_eq!(vi.read("num", &IntCodec), Some(77));
        assert_eq!(vi.read("missing", &IntCodec), None);
    }
}
