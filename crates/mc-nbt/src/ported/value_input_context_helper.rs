//! Faithful port of `net.minecraft.world.level.storage.ValueInputContextHelper`.

use std::iter;
use std::marker::PhantomData;

/// Minimal placeholder for `com.mojang.serialization.DynamicOps<Tag>`.
pub trait DynamicOps: Clone {}

/// Minimal placeholder for `net.minecraft.core.HolderLookup.Provider`.
pub trait HolderLookupProvider: Clone {
    type Ops: DynamicOps;

    /// Creates a serialization context for the given ops.
    fn create_serialization_context(&self, ops: Self::Ops) -> Self::Ops;
}

/// Minimal placeholder for `com.mojang.serialization.Codec<T>`.
#[derive(Debug)]
pub struct Codec<T>(PhantomData<fn() -> T>);

impl<T> Codec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Codec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Codec<T> {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec<T>`.
#[derive(Debug)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for MapCodec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MapCodec<T> {}

/// Empty `ValueInput.ValueInputList` sentinel.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyChildList;

impl EmptyChildList {
    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#isEmpty
    pub fn is_empty(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#stream
    pub fn stream<'a, P: HolderLookupProvider>(&self) -> iter::Empty<EmptyValueInput<'a, P>> {
        iter::empty()
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#iterator
    pub fn iterator<'a, P: HolderLookupProvider>(&self) -> iter::Empty<EmptyValueInput<'a, P>> {
        iter::empty()
    }
}

/// Empty `ValueInput.TypedInputList<T>` sentinel.
#[derive(Debug)]
pub struct EmptyTypedInputList<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> EmptyTypedInputList<T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#isEmpty
    pub fn is_empty(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#stream
    pub fn stream(&self) -> iter::Empty<T> {
        iter::empty()
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#iterator
    pub fn iterator(&self) -> iter::Empty<T> {
        iter::empty()
    }
}

impl<T> Clone for EmptyTypedInputList<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EmptyTypedInputList<T> {}

impl<T> Default for EmptyTypedInputList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Faithful port of `net.minecraft.world.level.storage.ValueInputContextHelper`.
pub struct ValueInputContextHelper<P: HolderLookupProvider> {
    lookup: P,
    ops: P::Ops,
    empty_child_list: EmptyChildList,
    empty_typed_list: EmptyTypedInputList<()>,
}

impl<P: HolderLookupProvider> ValueInputContextHelper<P> {
    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#<init>
    pub fn new(lookup: P, ops: P::Ops) -> Self {
        let ops = lookup.create_serialization_context(ops);
        Self {
            lookup,
            ops,
            empty_child_list: EmptyChildList,
            empty_typed_list: EmptyTypedInputList::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#ops
    pub fn ops(&self) -> &P::Ops {
        &self.ops
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#lookup
    pub fn lookup(&self) -> &P {
        &self.lookup
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#empty
    pub fn empty(&self) -> EmptyValueInput<'_, P> {
        EmptyValueInput { helper: self }
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#emptyList
    pub fn empty_list(&self) -> &EmptyChildList {
        &self.empty_child_list
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#emptyTypedList
    pub fn empty_typed_list<T>(&self) -> EmptyTypedInputList<T> {
        EmptyTypedInputList::new()
    }
}

/// Empty `ValueInput` sentinel returned by `ValueInputContextHelper::empty()`.
#[derive(Clone, Copy)]
pub struct EmptyValueInput<'a, P: HolderLookupProvider> {
    helper: &'a ValueInputContextHelper<P>,
}

impl<'a, P: HolderLookupProvider> EmptyValueInput<'a, P> {
    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#read
    pub fn read<T>(&self, _key: &str, _codec: &Codec<T>) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#read
    pub fn read_map<T>(&self, _codec: &MapCodec<T>) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#child
    pub fn child(&self, _key: &str) -> Option<EmptyValueInput<'a, P>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#childOrEmpty
    pub fn child_or_empty(&self, _key: &str) -> EmptyValueInput<'a, P> {
        EmptyValueInput {
            helper: self.helper,
        }
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#childrenList
    pub fn children_list(&self, _key: &str) -> Option<&'a EmptyChildList> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#childrenListOrEmpty
    pub fn children_list_or_empty(&self, _key: &str) -> &'a EmptyChildList {
        &self.helper.empty_child_list
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#list
    pub fn list<T>(&self, _key: &str, _codec: &Codec<T>) -> Option<EmptyTypedInputList<T>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#listOrEmpty
    pub fn list_or_empty<T>(&self, _key: &str, _codec: &Codec<T>) -> EmptyTypedInputList<T> {
        self.helper.empty_typed_list()
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getBooleanOr
    pub fn get_boolean_or(&self, _key: &str, default: bool) -> bool {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getByteOr
    pub fn get_byte_or(&self, _key: &str, default: i8) -> i8 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getShortOr
    pub fn get_short_or(&self, _key: &str, default: i16) -> i32 {
        default as i32
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getInt
    pub fn get_int(&self, _key: &str) -> Option<i32> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getIntOr
    pub fn get_int_or(&self, _key: &str, default: i32) -> i32 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getLongOr
    pub fn get_long_or(&self, _key: &str, default: i64) -> i64 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getLong
    pub fn get_long(&self, _key: &str) -> Option<i64> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getFloatOr
    pub fn get_float_or(&self, _key: &str, default: f32) -> f32 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getDoubleOr
    pub fn get_double_or(&self, _key: &str, default: f64) -> f64 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getString
    pub fn get_string(&self, _key: &str) -> Option<String> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getStringOr
    pub fn get_string_or(&self, _key: &str, default: &str) -> String {
        default.to_owned()
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#lookup
    pub fn lookup(&self) -> &P {
        &self.helper.lookup
    }

    // JAVA: net.minecraft.world.level.storage.ValueInputContextHelper#getIntArray
    pub fn get_int_array(&self, _key: &str) -> Option<Vec<i32>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockOps(u32);
    impl DynamicOps for MockOps {}

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockProvider(u32);
    impl HolderLookupProvider for MockProvider {
        type Ops = MockOps;

        fn create_serialization_context(&self, ops: Self::Ops) -> Self::Ops {
            MockOps(ops.0.wrapping_add(self.0))
        }
    }

    #[test]
    fn constructor_and_accessors() {
        let provider = MockProvider(7);
        let helper = ValueInputContextHelper::new(provider.clone(), MockOps(10));
        assert_eq!(helper.ops(), &MockOps(17));
        assert_eq!(helper.lookup(), &provider);
        assert!(helper.empty_list().is_empty());
        assert!(helper.empty_typed_list::<i32>().is_empty());
    }

    #[test]
    fn empty_scalar_defaults_edge_cases() {
        let helper = ValueInputContextHelper::new(MockProvider(0), MockOps(0));
        let empty = helper.empty();

        assert!(empty.get_boolean_or("k", true));
        assert!(!empty.get_boolean_or("k", false));
        assert_eq!(empty.get_byte_or("k", i8::MIN), i8::MIN);
        assert_eq!(empty.get_byte_or("k", -1i8), -1i8);
        assert_eq!(empty.get_short_or("k", i16::MIN), i16::MIN as i32);
        assert_eq!(empty.get_short_or("k", i16::MAX), i16::MAX as i32);
        assert_eq!(empty.get_int("k"), None);
        assert_eq!(empty.get_int_or("k", i32::MIN), i32::MIN);
        assert_eq!(empty.get_int_or("k", i32::MAX), i32::MAX);
        assert_eq!(empty.get_long("k"), None);
        assert_eq!(empty.get_long_or("k", i64::MIN), i64::MIN);
        assert_eq!(empty.get_long_or("k", i64::MAX), i64::MAX);
        assert_eq!(empty.get_string("k"), None);
        assert_eq!(empty.get_string_or("k", "default"), "default");
        assert_eq!(empty.get_int_array("k"), None);
    }

    #[test]
    fn empty_float_bits_preserved() {
        let helper = ValueInputContextHelper::new(MockProvider(0), MockOps(0));
        let empty = helper.empty();

        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let out_f32 = empty.get_float_or("k", nan_f32);
        assert_eq!(out_f32.to_bits(), nan_f32.to_bits());

        let neg_zero_f32 = -0.0f32;
        let out_neg_zero = empty.get_float_or("k", neg_zero_f32);
        assert_eq!(out_neg_zero.to_bits(), neg_zero_f32.to_bits());

        let neg_inf_f64 = f64::NEG_INFINITY;
        let out_f64 = empty.get_double_or("k", neg_inf_f64);
        assert_eq!(out_f64.to_bits(), neg_inf_f64.to_bits());

        let nan_f64 = f64::from_bits(0x7FF8_0000_0000_0001);
        let out_nan_f64 = empty.get_double_or("k", nan_f64);
        assert_eq!(out_nan_f64.to_bits(), nan_f64.to_bits());
    }

    #[test]
    fn empty_lists_and_navigation() {
        let helper = ValueInputContextHelper::new(MockProvider(0), MockOps(0));
        let empty = helper.empty();

        assert!(empty.child("missing").is_none());
        let child = empty.child_or_empty("missing");
        assert_eq!(child.get_int_or("x", 42), 42);

        assert!(empty.children_list("missing").is_none());
        let child_list = empty.children_list_or_empty("missing");
        assert!(child_list.is_empty());
        assert_eq!(child_list.stream::<MockProvider>().count(), 0);
        assert_eq!(child_list.iterator::<MockProvider>().count(), 0);

        let codec: Codec<i32> = Codec::new();
        assert!(empty.list("missing", &codec).is_none());
        let typed = empty.list_or_empty("missing", &codec);
        assert!(typed.is_empty());
        assert_eq!(typed.stream().count(), 0);
        assert_eq!(typed.iterator().count(), 0);
    }

    #[test]
    fn empty_read_and_lookup() {
        let helper = ValueInputContextHelper::new(MockProvider(3), MockOps(1));
        let empty = helper.empty();
        let codec: Codec<String> = Codec::default();
        let map_codec: MapCodec<i64> = MapCodec::default();

        assert_eq!(empty.read("k", &codec), None::<String>);
        assert_eq!(empty.read_map(&map_codec), None::<i64>);
        assert_eq!(empty.lookup(), &MockProvider(3));
        assert_eq!(helper.lookup(), &MockProvider(3));
    }
}
