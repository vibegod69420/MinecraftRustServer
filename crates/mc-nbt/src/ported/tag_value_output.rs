// === FILE: tag_value_output.rs ===
use crate::NbtTag;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Minimal self-contained support types (Java: ProblemReporter / DFU codecs)
// ---------------------------------------------------------------------------

/// Minimal problem descriptor (Java: `ProblemReporter.Problem`).
pub trait Problem {
    fn description(&self) -> String;
}

/// Path element for child reporters (Java: `ProblemReporter.FieldPathElement`).
#[derive(Debug, Clone)]
pub struct FieldPathElement(pub String);

/// Indexed path element for child reporters (Java: `ProblemReporter.IndexedFieldPathElement`).
#[derive(Debug, Clone)]
pub struct IndexedFieldPathElement {
    pub name: String,
    pub index: usize,
}

/// Path element union used by `ProblemReporter::for_child`.
#[derive(Debug, Clone)]
pub enum PathElement {
    Field(FieldPathElement),
    Indexed(IndexedFieldPathElement),
}

impl From<FieldPathElement> for PathElement {
    fn from(value: FieldPathElement) -> Self {
        PathElement::Field(value)
    }
}

impl From<IndexedFieldPathElement> for PathElement {
    fn from(value: IndexedFieldPathElement) -> Self {
        PathElement::Indexed(value)
    }
}

/// Minimal problem reporter (Java: `ProblemReporter`).
#[derive(Clone, Default, Debug)]
pub struct ProblemReporter {
    sink: Rc<RefCell<Vec<String>>>,
}

impl ProblemReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report(&self, problem: impl Problem) {
        self.sink.borrow_mut().push(problem.description());
    }

    pub fn for_child(&self, _element: PathElement) -> Self {
        Self {
            sink: Rc::clone(&self.sink),
        }
    }

    pub fn problems(&self) -> Vec<String> {
        self.sink.borrow().clone()
    }
}

/// Minimal DFU-like encode result (Java: `DataResult`).
#[derive(Debug)]
pub enum DataResult<T> {
    Success(T),
    Error {
        message: String,
        partial: Option<T>,
    },
}

/// Minimal dynamic ops marker (Java: `DynamicOps<Tag>`).
pub trait DynamicOps {}

/// Minimal NBT ops (Java: `NbtOps.INSTANCE`).
#[derive(Debug, Clone, Copy, Default)]
pub struct NbtOps;

impl DynamicOps for NbtOps {}

/// Minimal encoder (Java: DFU `Encoder<T>`).
pub trait Encoder<T: ?Sized> {
    fn encode_start(&self, ops: &dyn DynamicOps, input: &T) -> DataResult<NbtTag>;
}

/// Minimal codec (Java: DFU `Codec<T>`).
pub trait Codec<T: ?Sized>: Encoder<T> {}

/// Minimal map codec (Java: DFU `MapCodec<T>`).
/// `encode_start` represents `encoder().encodeStart(...)`.
pub trait MapCodec<T: ?Sized> {
    fn encode_start(&self, ops: &dyn DynamicOps, input: &T) -> DataResult<NbtTag>;
}

/// Minimal holder lookup provider (Java: `HolderLookup.Provider`).
pub trait HolderLookupProvider {
    type Ops: DynamicOps;
    fn create_serialization_context(&self, base: NbtOps) -> Self::Ops;
}

// ---------------------------------------------------------------------------
// Shared mutable NBT node wrappers (faithful Java reference semantics)
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct CompoundTag {
    inner: Rc<RefCell<CompoundInner>>,
}

#[derive(Default, Debug)]
struct CompoundInner {
    entries: Vec<(String, TagNode)>,
}

#[derive(Clone, Default)]
pub struct ListTag {
    inner: Rc<RefCell<ListInner>>,
}

#[derive(Default, Debug)]
struct ListInner {
    items: Vec<TagNode>,
}

#[derive(Clone, Debug)]
enum TagNode {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(ListTag),
    Compound(CompoundTag),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl TagNode {
    fn from_nbt_tag(tag: &NbtTag) -> Self {
        match tag {
            NbtTag::Byte(v) => TagNode::Byte(*v),
            NbtTag::Short(v) => TagNode::Short(*v),
            NbtTag::Int(v) => TagNode::Int(*v),
            NbtTag::Long(v) => TagNode::Long(*v),
            NbtTag::Float(v) => TagNode::Float(*v),
            NbtTag::Double(v) => TagNode::Double(*v),
            NbtTag::ByteArray(v) => TagNode::ByteArray(v.clone()),
            NbtTag::String(v) => TagNode::String(v.clone()),
            NbtTag::List { items, .. } => {
                let list = ListTag::new();
                {
                    let mut inner = list.inner.borrow_mut();
                    inner.items = items.iter().map(TagNode::from_nbt_tag).collect();
                }
                TagNode::List(list)
            }
            NbtTag::Compound(entries) => {
                let compound = CompoundTag::from_entries(entries);
                TagNode::Compound(compound)
            }
            NbtTag::IntArray(v) => TagNode::IntArray(v.clone()),
            NbtTag::LongArray(v) => TagNode::LongArray(v.clone()),
        }
    }

    fn to_nbt_tag(&self) -> NbtTag {
        match self {
            TagNode::Byte(v) => NbtTag::Byte(*v),
            TagNode::Short(v) => NbtTag::Short(*v),
            TagNode::Int(v) => NbtTag::Int(*v),
            TagNode::Long(v) => NbtTag::Long(*v),
            TagNode::Float(v) => NbtTag::Float(*v),
            TagNode::Double(v) => NbtTag::Double(*v),
            TagNode::ByteArray(v) => NbtTag::ByteArray(v.clone()),
            TagNode::String(v) => NbtTag::String(v.clone()),
            TagNode::List(v) => v.to_nbt_tag(),
            TagNode::Compound(v) => v.to_nbt_tag(),
            TagNode::IntArray(v) => NbtTag::IntArray(v.clone()),
            TagNode::LongArray(v) => NbtTag::LongArray(v.clone()),
        }
    }

    fn deep_copy(&self) -> Self {
        match self {
            TagNode::Byte(v) => TagNode::Byte(*v),
            TagNode::Short(v) => TagNode::Short(*v),
            TagNode::Int(v) => TagNode::Int(*v),
            TagNode::Long(v) => TagNode::Long(*v),
            TagNode::Float(v) => TagNode::Float(*v),
            TagNode::Double(v) => TagNode::Double(*v),
            TagNode::ByteArray(v) => TagNode::ByteArray(v.clone()),
            TagNode::String(v) => TagNode::String(v.clone()),
            TagNode::List(v) => TagNode::List(v.deep_copy()),
            TagNode::Compound(v) => TagNode::Compound(v.deep_copy()),
            TagNode::IntArray(v) => TagNode::IntArray(v.clone()),
            TagNode::LongArray(v) => TagNode::LongArray(v.clone()),
        }
    }
}

impl fmt::Debug for CompoundTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompoundTag")
            .field("entries", &self.inner.borrow().entries)
            .finish()
    }
}

impl fmt::Debug for ListTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListTag")
            .field("items", &self.inner.borrow().items)
            .finish()
    }
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_nbt_tag(tag: &NbtTag) -> Self {
        match tag {
            NbtTag::Compound(entries) => Self::from_entries(entries),
            _ => Self::new(),
        }
    }

    fn from_entries(entries: &[(String, NbtTag)]) -> Self {
        let compound = Self::new();
        {
            let mut inner = compound.inner.borrow_mut();
            inner.entries = entries
                .iter()
                .map(|(k, v)| (k.clone(), TagNode::from_nbt_tag(v)))
                .collect();
        }
        compound
    }

    pub fn deep_copy(&self) -> Self {
        let inner = self.inner.borrow();
        let entries = inner
            .entries
            .iter()
            .map(|(k, v)| (k.clone(), v.deep_copy()))
            .collect();
        Self {
            inner: Rc::new(RefCell::new(CompoundInner { entries })),
        }
    }

    fn put_node(&self, key: String, value: TagNode) {
        let mut inner = self.inner.borrow_mut();
        if let Some(pos) = inner.entries.iter().rposition(|(k, _)| k == &key) {
            inner.entries[pos].1 = value;
        } else {
            inner.entries.push((key, value));
        }
    }

    pub fn put_nbt(&self, key: String, value: NbtTag) {
        self.put_node(key, TagNode::from_nbt_tag(&value));
    }

    pub fn merge(&self, other: &CompoundTag) {
        let other_entries: Vec<(String, TagNode)> = {
            let inner = other.inner.borrow();
            inner
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.deep_copy()))
                .collect()
        };
        for (k, v) in other_entries {
            self.merge_node(k, v);
        }
    }

    fn merge_node(&self, key: String, value: TagNode) {
        let mut inner = self.inner.borrow_mut();
        if let Some(pos) = inner.entries.iter().rposition(|(k, _)| k == &key) {
            if let (TagNode::Compound(existing), TagNode::Compound(incoming)) =
                (&inner.entries[pos].1, &value)
            {
                existing.merge(incoming);
                return;
            }
            inner.entries[pos].1 = value;
        } else {
            inner.entries.push((key, value));
        }
    }

    pub fn put_boolean(&self, key: &str, value: bool) {
        self.put_node(
            key.to_string(),
            TagNode::Byte(if value { 1 } else { 0 }),
        );
    }

    pub fn put_byte(&self, key: &str, value: i8) {
        self.put_node(key.to_string(), TagNode::Byte(value));
    }

    pub fn put_short(&self, key: &str, value: i16) {
        self.put_node(key.to_string(), TagNode::Short(value));
    }

    pub fn put_int(&self, key: &str, value: i32) {
        self.put_node(key.to_string(), TagNode::Int(value));
    }

    pub fn put_long(&self, key: &str, value: i64) {
        self.put_node(key.to_string(), TagNode::Long(value));
    }

    pub fn put_float(&self, key: &str, value: f32) {
        self.put_node(key.to_string(), TagNode::Float(value));
    }

    pub fn put_double(&self, key: &str, value: f64) {
        self.put_node(key.to_string(), TagNode::Double(value));
    }

    pub fn put_string(&self, key: &str, value: &str) {
        self.put_node(key.to_string(), TagNode::String(value.to_string()));
    }

    pub fn put_int_array(&self, key: &str, value: Vec<i32>) {
        self.put_node(key.to_string(), TagNode::IntArray(value));
    }

    pub fn remove(&self, key: &str) {
        let mut inner = self.inner.borrow_mut();
        if let Some(pos) = inner.entries.iter().rposition(|(k, _)| k == key) {
            inner.entries.remove(pos);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().entries.is_empty()
    }

    pub fn to_nbt_tag(&self) -> NbtTag {
        let inner = self.inner.borrow();
        NbtTag::Compound(
            inner
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.to_nbt_tag()))
                .collect(),
        )
    }
}

impl ListTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn deep_copy(&self) -> Self {
        let inner = self.inner.borrow();
        let items = inner.items.iter().map(|v| v.deep_copy()).collect();
        Self {
            inner: Rc::new(RefCell::new(ListInner { items })),
        }
    }

    pub fn add(&self, value: TagNode) {
        self.inner.borrow_mut().items.push(value);
    }

    pub fn add_nbt(&self, value: NbtTag) {
        self.add(TagNode::from_nbt_tag(&value));
    }

    pub fn remove_last(&self) {
        self.inner.borrow_mut().items.pop();
    }

    pub fn size(&self) -> usize {
        self.inner.borrow().items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().items.is_empty()
    }

    pub fn to_nbt_tag(&self) -> NbtTag {
        let inner = self.inner.borrow();
        let items: Vec<NbtTag> = inner.items.iter().map(|v| v.to_nbt_tag()).collect();
        let elem_type = items.first().map(|t| t.id()).unwrap_or(0);
        NbtTag::List { elem_type, items }
    }
}

// ---------------------------------------------------------------------------
// Problem records (Java: TagValueOutput.EncodeTo*Problem)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EncodeToFieldFailedProblem {
    pub name: String,
    pub value: String,
    pub error: String,
}

impl Problem for EncodeToFieldFailedProblem {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.EncodeToFieldFailedProblem#description
    fn description(&self) -> String {
        format!(
            "Failed to encode value '{}' to field '{}': {}",
            self.value, self.name, self.error
        )
    }
}

#[derive(Debug, Clone)]
pub struct EncodeToListFailedProblem {
    pub name: String,
    pub value: String,
    pub error: String,
}

impl Problem for EncodeToListFailedProblem {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.EncodeToListFailedProblem#description
    fn description(&self) -> String {
        format!(
            "Failed to append value '{}' to list '{}': {}",
            self.value, self.name, self.error
        )
    }
}

#[derive(Debug, Clone)]
pub struct EncodeToMapFailedProblem {
    pub value: String,
    pub error: String,
}

impl Problem for EncodeToMapFailedProblem {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.EncodeToMapFailedProblem#description
    fn description(&self) -> String {
        format!(
            "Failed to merge value '{}' to an object: {}",
            self.value, self.error
        )
    }
}

// ---------------------------------------------------------------------------
// TagValueOutput
// ---------------------------------------------------------------------------

pub struct TagValueOutput<O: DynamicOps = NbtOps> {
    problem_reporter: ProblemReporter,
    ops: O,
    output: CompoundTag,
}

impl TagValueOutput<NbtOps> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput#createWithoutContext
    pub fn create_without_context(problem_reporter: ProblemReporter) -> Self {
        Self::new(problem_reporter, NbtOps, CompoundTag::new())
    }
}

impl<O: DynamicOps> TagValueOutput<O> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput#<init>
    pub fn new(problem_reporter: ProblemReporter, ops: O, output: CompoundTag) -> Self {
        Self {
            problem_reporter,
            ops,
            output,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#createWithContext
    pub fn create_with_context<P: HolderLookupProvider<Ops = O>>(
        problem_reporter: ProblemReporter,
        provider: &P,
    ) -> Self {
        Self::new(
            problem_reporter,
            provider.create_serialization_context(NbtOps),
            CompoundTag::new(),
        )
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#store
    pub fn store<T: fmt::Debug + ?Sized, C: Codec<T>>(
        &mut self,
        key: &str,
        codec: &C,
        value: &T,
    ) {
        match codec.encode_start(&self.ops, value) {
            DataResult::Success(tag) => {
                self.output.put_nbt(key.to_string(), tag);
            }
            DataResult::Error { message, partial } => {
                self.problem_reporter.report(EncodeToFieldFailedProblem {
                    name: key.to_string(),
                    value: format!("{:?}", value),
                    error: message,
                });
                if let Some(tag) = partial {
                    self.output.put_nbt(key.to_string(), tag);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#storeNullable
    pub fn store_nullable<T: fmt::Debug, C: Codec<T>>(
        &mut self,
        key: &str,
        codec: &C,
        value: Option<&T>,
    ) {
        if let Some(v) = value {
            self.store(key, codec, v);
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#store
    pub fn store_map<T: fmt::Debug + ?Sized, M: MapCodec<T>>(&mut self, codec: &M, value: &T) {
        match codec.encode_start(&self.ops, value) {
            DataResult::Success(tag) => {
                if let NbtTag::Compound(_) = &tag {
                    let other = CompoundTag::from_nbt_tag(&tag);
                    self.output.merge(&other);
                }
            }
            DataResult::Error { message, partial } => {
                self.problem_reporter.report(EncodeToMapFailedProblem {
                    value: format!("{:?}", value),
                    error: message,
                });
                if let Some(tag) = partial {
                    if let NbtTag::Compound(_) = &tag {
                        let other = CompoundTag::from_nbt_tag(&tag);
                        self.output.merge(&other);
                    }
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putBoolean
    pub fn put_boolean(&mut self, key: &str, value: bool) {
        self.output.put_boolean(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putByte
    pub fn put_byte(&mut self, key: &str, value: i8) {
        self.output.put_byte(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putShort
    pub fn put_short(&mut self, key: &str, value: i16) {
        self.output.put_short(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putInt
    pub fn put_int(&mut self, key: &str, value: i32) {
        self.output.put_int(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putLong
    pub fn put_long(&mut self, key: &str, value: i64) {
        self.output.put_long(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putFloat
    pub fn put_float(&mut self, key: &str, value: f32) {
        self.output.put_float(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putDouble
    pub fn put_double(&mut self, key: &str, value: f64) {
        self.output.put_double(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putString
    pub fn put_string(&mut self, key: &str, value: &str) {
        self.output.put_string(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#putIntArray
    pub fn put_int_array(&mut self, key: &str, value: Vec<i32>) {
        self.output.put_int_array(key, value);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#reporterForChild
    fn reporter_for_child(&self, name: &str) -> ProblemReporter {
        self.problem_reporter
            .for_child(PathElement::Field(FieldPathElement(name.to_string())))
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#child
    pub fn child(&mut self, key: &str) -> TagValueOutput<O>
    where
        O: Clone,
    {
        let compound = CompoundTag::new();
        self.output
            .put_node(key.to_string(), TagNode::Compound(compound.clone()));
        TagValueOutput::new(self.reporter_for_child(key), self.ops.clone(), compound)
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#childrenList
    pub fn children_list(&mut self, key: &str) -> ListWrapper<O>
    where
        O: Clone,
    {
        let list = ListTag::new();
        self.output
            .put_node(key.to_string(), TagNode::List(list.clone()));
        ListWrapper::new(
            key.to_string(),
            self.problem_reporter.clone(),
            self.ops.clone(),
            list,
        )
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#list
    pub fn list<T: fmt::Debug + ?Sized, C: Codec<T>>(
        &mut self,
        key: &str,
        codec: C,
    ) -> TypedListWrapper<O, T, C>
    where
        O: Clone,
    {
        let list_tag = ListTag::new();
        self.output
            .put_node(key.to_string(), TagNode::List(list_tag.clone()));
        TypedListWrapper::new(
            self.problem_reporter.clone(),
            key.to_string(),
            self.ops.clone(),
            codec,
            list_tag,
        )
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#discard
    pub fn discard(&mut self, key: &str) {
        self.output.remove(key);
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#isEmpty
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#buildResult
    pub fn build_result(&self) -> CompoundTag {
        self.output.clone()
    }

    /// Convenience: convert the built result to an owned `NbtTag`.
    pub fn build_result_nbt(&self) -> NbtTag {
        self.output.to_nbt_tag()
    }
}

// ---------------------------------------------------------------------------
// ListWrapper (Java: TagValueOutput.ListWrapper)
// ---------------------------------------------------------------------------

pub struct ListWrapper<O: DynamicOps = NbtOps> {
    field_name: String,
    problem_reporter: ProblemReporter,
    ops: O,
    output: ListTag,
}

impl<O: DynamicOps> ListWrapper<O> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.ListWrapper#<init>
    fn new(field_name: String, problem_reporter: ProblemReporter, ops: O, output: ListTag) -> Self {
        Self {
            field_name,
            problem_reporter,
            ops,
            output,
        }
    }
}

impl<O: DynamicOps + Clone> ListWrapper<O> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.ListWrapper#addChild
    pub fn add_child(&mut self) -> TagValueOutput<O> {
        let idx = self.output.size();
        let compound = CompoundTag::new();
        self.output.add(TagNode::Compound(compound.clone()));
        let child_reporter = self.problem_reporter.for_child(PathElement::Indexed(
            IndexedFieldPathElement {
                name: self.field_name.clone(),
                index: idx,
            },
        ));
        TagValueOutput::new(child_reporter, self.ops.clone(), compound)
    }
}

impl<O: DynamicOps> ListWrapper<O> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.ListWrapper#discardLast
    pub fn discard_last(&mut self) {
        self.output.remove_last();
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput.ListWrapper#isEmpty
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

// ---------------------------------------------------------------------------
// TypedListWrapper (Java: TagValueOutput.TypedListWrapper)
// ---------------------------------------------------------------------------

pub struct TypedListWrapper<O: DynamicOps, T: fmt::Debug + ?Sized, C: Codec<T>> {
    problem_reporter: ProblemReporter,
    name: String,
    ops: O,
    codec: C,
    output: ListTag,
    _marker: PhantomData<fn(&T)>,
}

impl<O: DynamicOps, T: fmt::Debug + ?Sized, C: Codec<T>> TypedListWrapper<O, T, C> {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput.TypedListWrapper#<init>
    fn new(
        problem_reporter: ProblemReporter,
        name: String,
        ops: O,
        codec: C,
        output: ListTag,
    ) -> Self {
        Self {
            problem_reporter,
            name,
            ops,
            codec,
            output,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput.TypedListWrapper#add
    pub fn add(&mut self, value: &T) {
        match self.codec.encode_start(&self.ops, value) {
            DataResult::Success(tag) => {
                self.output.add_nbt(tag);
            }
            DataResult::Error { message, partial } => {
                self.problem_reporter
                    .report(EncodeToListFailedProblem {
                        name: self.name.clone(),
                        value: format!("{:?}", value),
                        error: message,
                    });
                if let Some(tag) = partial {
                    self.output.add_nbt(tag);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput.TypedListWrapper#isEmpty
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    struct IntCodec;
    impl Encoder<i32> for IntCodec {
        fn encode_start(&self, _ops: &dyn DynamicOps, input: &i32) -> DataResult<NbtTag> {
            DataResult::Success(NbtTag::Int(*input))
        }
    }
    impl Codec<i32> for IntCodec {}

    struct FailingIntCodec;
    impl Encoder<i32> for FailingIntCodec {
        fn encode_start(&self, _ops: &dyn DynamicOps, input: &i32) -> DataResult<NbtTag> {
            DataResult::Error {
                message: "bad int".to_string(),
                partial: Some(NbtTag::Int(input.wrapping_add(1))),
            }
        }
    }
    impl Codec<i32> for FailingIntCodec {}

    struct MapMergeCodec;
    impl MapCodec<i32> for MapMergeCodec {
        fn encode_start(&self, _ops: &dyn DynamicOps, input: &i32) -> DataResult<NbtTag> {
            DataResult::Success(NbtTag::Compound(vec![
                ("a".to_string(), NbtTag::Int(*input)),
                (
                    "nested".to_string(),
                    NbtTag::Compound(vec![("x".to_string(), NbtTag::Int(1))]),
                ),
            ]))
        }
    }

    struct MapMergeCodec2;
    impl MapCodec<i32> for MapMergeCodec2 {
        fn encode_start(&self, _ops: &dyn DynamicOps, input: &i32) -> DataResult<NbtTag> {
            DataResult::Success(NbtTag::Compound(vec![
                ("b".to_string(), NbtTag::Int(*input)),
                (
                    "nested".to_string(),
                    NbtTag::Compound(vec![("y".to_string(), NbtTag::Int(2))]),
                ),
            ]))
        }
    }

    #[test]
    fn primitives_negative_and_float_bits() {
        let mut out = TagValueOutput::create_without_context(ProblemReporter::new());
        out.put_boolean("bool_t", true);
        out.put_boolean("bool_f", false);
        out.put_byte("byte_min", i8::MIN);
        out.put_short("short_min", i16::MIN);
        out.put_int("int_min", i32::MIN);
        out.put_long("long_min", i64::MIN);
        out.put_float("f_neg_zero", -0.0_f32);
        out.put_float("f_nan", f32::NAN);
        out.put_double("d_neg_zero", -0.0_f64);
        out.put_double("d_nan", f64::NAN);
        out.put_string("s", "hello");
        out.put_int_array("arr", vec![-1, 0, i32::MAX]);

        let tag = out.build_result_nbt();
        assert_eq!(tag.compound_get("bool_t"), Some(&NbtTag::Byte(1)));
        assert_eq!(tag.compound_get("bool_f"), Some(&NbtTag::Byte(0)));
        assert_eq!(tag.compound_get("byte_min"), Some(&NbtTag::Byte(i8::MIN)));
        assert_eq!(tag.compound_get("short_min"), Some(&NbtTag::Short(i16::MIN)));
        assert_eq!(tag.compound_get("int_min"), Some(&NbtTag::Int(i32::MIN)));
        assert_eq!(tag.compound_get("long_min"), Some(&NbtTag::Long(i64::MIN)));
        assert_eq!(tag.compound_get("s"), Some(&NbtTag::String("hello".to_string())));
        assert_eq!(
            tag.compound_get("arr"),
            Some(&NbtTag::IntArray(vec![-1, 0, i32::MAX]))
        );

        match tag.compound_get("f_neg_zero") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), (-0.0_f32).to_bits()),
            other => panic!("unexpected: {:?}", other),
        }
        match tag.compound_get("f_nan") {
            Some(NbtTag::Float(v)) => assert!(v.is_nan()),
            other => panic!("unexpected: {:?}", other),
        }
        match tag.compound_get("d_neg_zero") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), (-0.0_f64).to_bits()),
            other => panic!("unexpected: {:?}", other),
        }
        match tag.compound_get("d_nan") {
            Some(NbtTag::Double(v)) => assert!(v.is_nan()),
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn child_children_list_and_discard() {
        let mut out = TagValueOutput::create_without_context(ProblemReporter::new());
        assert!(out.is_empty());

        {
            let mut child = out.child("child");
            child.put_int("x", 42);
            assert!(!child.is_empty());
        }

        {
            let mut list = out.children_list("items");
            assert!(list.is_empty());
            let mut c1 = list.add_child();
            c1.put_byte("v", -7);
            let mut c2 = list.add_child();
            c2.put_string("name", "b");
            assert!(!list.is_empty());
            list.discard_last();
            assert!(!list.is_empty());
        }

        out.put_int("tmp", 1);
        out.discard("tmp");
        let tag = out.build_result_nbt();
        assert!(tag.compound_get("tmp").is_none());

        match tag.compound_get("child") {
            Some(NbtTag::Compound(entries)) => {
                assert!(entries.iter().any(|(k, v)| k == "x" && v == &NbtTag::Int(42)));
            }
            other => panic!("unexpected: {:?}", other),
        }

        match tag.compound_get("items") {
            Some(NbtTag::List { items, .. }) => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    NbtTag::Compound(entries) => {
                        assert!(entries.iter().any(|(k, v)| k == "v" && v == &NbtTag::Byte(-7)));
                    }
                    other => panic!("unexpected: {:?}", other),
                }
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn typed_list_error_reports_and_partial() {
        let reporter = ProblemReporter::new();
        let mut out = TagValueOutput::create_without_context(reporter.clone());
        {
            let mut list = out.list("vals", FailingIntCodec);
            list.add(&10);
            assert!(!list.is_empty());
        }
        let tag = out.build_result_nbt();
        match tag.compound_get("vals") {
            Some(NbtTag::List { items, elem_type }) => {
                assert_eq!(*elem_type, 3);
                assert_eq!(items, &vec![NbtTag::Int(11)]);
            }
            other => panic!("unexpected: {:?}", other),
        }
        let problems = reporter.problems();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("Failed to append value"));
        assert!(problems[0].contains("bad int"));
    }

    #[test]
    fn store_store_nullable_and_store_map_merge() {
        let reporter = ProblemReporter::new();
        let mut out = TagValueOutput::create_without_context(reporter.clone());
        out.store("n", &IntCodec, &-5_i32);
        out.store_nullable("none", &IntCodec, None::<&i32>);
        out.store_nullable("some", &IntCodec, Some(&7_i32));

        out.store_map(&MapMergeCodec, &100_i32);
        out.store_map(&MapMergeCodec2, &200_i32);

        let tag = out.build_result_nbt();
        assert_eq!(tag.compound_get("n"), Some(&NbtTag::Int(-5)));
        assert!(tag.compound_get("none").is_none());
        assert_eq!(tag.compound_get("some"), Some(&NbtTag::Int(7)));
        assert_eq!(tag.compound_get("a"), Some(&NbtTag::Int(100)));
        assert_eq!(tag.compound_get("b"), Some(&NbtTag::Int(200)));

        match tag.compound_get("nested") {
            Some(NbtTag::Compound(entries)) => {
                assert!(entries.iter().any(|(k, v)| k == "x" && v == &NbtTag::Int(1)));
                assert!(entries.iter().any(|(k, v)| k == "y" && v == &NbtTag::Int(2)));
            }
            other => panic!("unexpected: {:?}", other),
        }
        assert!(reporter.problems().is_empty());
    }

    #[test]
    fn store_error_reports_and_keeps_partial() {
        let reporter = ProblemReporter::new();
        let mut out = TagValueOutput::create_without_context(reporter.clone());
        out.store("bad", &FailingIntCodec, &5_i32);
        let tag = out.build_result_nbt();
        assert_eq!(tag.compound_get("bad"), Some(&NbtTag::Int(6)));
        let problems = reporter.problems();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("Failed to encode value"));
        assert!(problems[0].contains("'bad'"));
    }
}
