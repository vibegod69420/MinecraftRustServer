use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(PhantomData<T>);

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

#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(PhantomData<T>);

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

#[derive(Debug, Clone, Copy, Default)]
pub struct HolderLookupProvider;

// JAVA: net.minecraft.world.level.storage.ValueInput$TypedInputList
pub trait TypedInputList<T: Clone> {
    // JAVA: net.minecraft.world.level.storage.ValueInput$TypedInputList#isEmpty
    fn is_empty(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.ValueInput$TypedInputList#stream
    fn stream(&self) -> Box<dyn Iterator<Item = T> + '_>;
}

// JAVA: net.minecraft.world.level.storage.ValueInput$ValueInputList
pub trait ValueInputList {
    // JAVA: net.minecraft.world.level.storage.ValueInput$ValueInputList#isEmpty
    fn is_empty(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.ValueInput$ValueInputList#stream
    fn stream<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn ValueInput> + 'a>;
}

// JAVA: net.minecraft.world.level.storage.ValueInput
pub trait ValueInput {
    // JAVA: net.minecraft.world.level.storage.ValueInput#child
    fn child(&self, name: &str) -> Option<Box<dyn ValueInput + '_>>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#childOrEmpty
    fn child_or_empty(&self, name: &str) -> Box<dyn ValueInput + '_>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#childrenList
    fn children_list(&self, name: &str) -> Option<Box<dyn ValueInputList + '_>>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#childrenListOrEmpty
    fn children_list_or_empty(&self, name: &str) -> Box<dyn ValueInputList + '_>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getBooleanOr
    fn get_boolean_or(&self, name: &str, default: bool) -> bool;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getByteOr
    fn get_byte_or(&self, name: &str, default: i8) -> i8;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getShortOr
    fn get_short_or(&self, name: &str, default: i16) -> i32;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getInt
    fn get_int(&self, name: &str) -> Option<i32>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getIntOr
    fn get_int_or(&self, name: &str, default: i32) -> i32;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getLongOr
    fn get_long_or(&self, name: &str, default: i64) -> i64;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getLong
    fn get_long(&self, name: &str) -> Option<i64>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getFloatOr
    fn get_float_or(&self, name: &str, default: f32) -> f32;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getDoubleOr
    fn get_double_or(&self, name: &str, default: f64) -> f64;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getString
    fn get_string(&self, name: &str) -> Option<String>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getStringOr
    fn get_string_or(&self, name: &str, default: &str) -> String;

    // JAVA: net.minecraft.world.level.storage.ValueInput#getIntArray
    fn get_int_array(&self, name: &str) -> Option<Vec<i32>>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#lookup
    fn lookup(&self) -> HolderLookupProvider;
}

// JAVA: net.minecraft.world.level.storage.ValueInput
pub trait ValueInputCodec {
    // JAVA: net.minecraft.world.level.storage.ValueInput#read
    fn read<T>(&self, name: &str, codec: &Codec<T>) -> Option<T>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#read
    fn read_map<T>(&self, codec: &MapCodec<T>) -> Option<T>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#list
    fn list<T: Clone + 'static>(
        &self,
        name: &str,
        codec: &Codec<T>,
    ) -> Option<Box<dyn TypedInputList<T> + '_>>;

    // JAVA: net.minecraft.world.level.storage.ValueInput#listOrEmpty
    fn list_or_empty<T: Clone + 'static>(
        &self,
        name: &str,
        codec: &Codec<T>,
    ) -> Box<dyn TypedInputList<T> + '_>;
}

pub struct EmptyValueInput;

impl EmptyValueInput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmptyValueInput {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EmptyValueInputList;

impl ValueInputList for EmptyValueInputList {
    // JAVA: net.minecraft.world.level.storage.ValueInput$ValueInputList#isEmpty
    fn is_empty(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput$ValueInputList#stream
    fn stream<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn ValueInput> + 'a> {
        Box::new(std::iter::empty())
    }
}

pub struct EmptyTypedInputList<T>(PhantomData<T>);

impl<T> EmptyTypedInputList<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for EmptyTypedInputList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + 'static> TypedInputList<T> for EmptyTypedInputList<T> {
    // JAVA: net.minecraft.world.level.storage.ValueInput$TypedInputList#isEmpty
    fn is_empty(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput$TypedInputList#stream
    fn stream(&self) -> Box<dyn Iterator<Item = T> + '_> {
        Box::new(std::iter::empty())
    }
}

impl ValueInput for EmptyValueInput {
    // JAVA: net.minecraft.world.level.storage.ValueInput#child
    fn child(&self, _name: &str) -> Option<Box<dyn ValueInput + '_>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#childOrEmpty
    fn child_or_empty(&self, _name: &str) -> Box<dyn ValueInput + '_> {
        Box::new(EmptyValueInput)
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#childrenList
    fn children_list(&self, _name: &str) -> Option<Box<dyn ValueInputList + '_>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#childrenListOrEmpty
    fn children_list_or_empty(&self, _name: &str) -> Box<dyn ValueInputList + '_> {
        Box::new(EmptyValueInputList)
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getBooleanOr
    fn get_boolean_or(&self, _name: &str, default: bool) -> bool {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getByteOr
    fn get_byte_or(&self, _name: &str, default: i8) -> i8 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getShortOr
    fn get_short_or(&self, _name: &str, default: i16) -> i32 {
        default as i32
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getInt
    fn get_int(&self, _name: &str) -> Option<i32> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getIntOr
    fn get_int_or(&self, _name: &str, default: i32) -> i32 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getLongOr
    fn get_long_or(&self, _name: &str, default: i64) -> i64 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getLong
    fn get_long(&self, _name: &str) -> Option<i64> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getFloatOr
    fn get_float_or(&self, _name: &str, default: f32) -> f32 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getDoubleOr
    fn get_double_or(&self, _name: &str, default: f64) -> f64 {
        default
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getString
    fn get_string(&self, _name: &str) -> Option<String> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getStringOr
    fn get_string_or(&self, _name: &str, default: &str) -> String {
        default.to_string()
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#getIntArray
    fn get_int_array(&self, _name: &str) -> Option<Vec<i32>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#lookup
    fn lookup(&self) -> HolderLookupProvider {
        HolderLookupProvider
    }
}

impl ValueInputCodec for EmptyValueInput {
    // JAVA: net.minecraft.world.level.storage.ValueInput#read
    fn read<T>(&self, _name: &str, _codec: &Codec<T>) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#read
    fn read_map<T>(&self, _codec: &MapCodec<T>) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#list
    fn list<T: Clone + 'static>(
        &self,
        _name: &str,
        _codec: &Codec<T>,
    ) -> Option<Box<dyn TypedInputList<T> + '_>> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.ValueInput#listOrEmpty
    fn list_or_empty<T: Clone + 'static>(
        &self,
        _name: &str,
        _codec: &Codec<T>,
    ) -> Box<dyn TypedInputList<T> + '_> {
        Box::new(EmptyTypedInputList::<T>::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_defaults_and_absent_optionals() {
        let vi = EmptyValueInput::new();
        assert_eq!(vi.get_boolean_or("b", true), true);
        assert_eq!(vi.get_byte_or("b", i8::MIN), i8::MIN);
        assert_eq!(vi.get_short_or("s", i16::MIN), i16::MIN as i32);
        assert_eq!(vi.get_int("i"), None);
        assert_eq!(vi.get_int_or("i", -1), -1);
        assert_eq!(vi.get_long("l"), None);
        assert_eq!(vi.get_long_or("l", i64::MIN), i64::MIN);
    }

    #[test]
    fn float_double_bits_and_string_defaults() {
        let vi = EmptyValueInput::new();
        let f = vi.get_float_or("f", -0.0f32);
        assert_eq!(f.to_bits(), (-0.0f32).to_bits());
        let nan = vi.get_double_or("d", f64::NAN);
        assert!(nan.is_nan());
        assert_eq!(
            vi.get_double_or("d2", f64::NEG_INFINITY),
            f64::NEG_INFINITY
        );
        assert_eq!(vi.get_string("s"), None);
        assert_eq!(vi.get_string_or("s", "default"), "default");
        assert_eq!(vi.get_int_array("a"), None);
    }

    #[test]
    fn child_and_lists_empty() {
        let vi = EmptyValueInput::new();
        assert!(vi.child("c").is_none());
        let child = vi.child_or_empty("c");
        assert_eq!(child.get_int_or("x", 7), 7);
        assert!(vi.children_list("l").is_none());
        let list = vi.children_list_or_empty("l");
        assert!(ValueInputList::is_empty(&*list));
        assert_eq!(list.stream().count(), 0);
    }

    #[test]
    fn typed_list_empty_and_read() {
        let vi = EmptyValueInput::new();
        let codec: Codec<i32> = Codec::new();
        let r: Option<i32> = vi.read("r", &codec);
        assert!(r.is_none());
        let rm: Option<i32> = vi.read_map(&MapCodec::<i32>::new());
        assert!(rm.is_none());
        let l: Option<Box<dyn TypedInputList<i32>>> = vi.list("l", &Codec::<i32>::new());
        assert!(l.is_none());
        let tl = vi.list_or_empty("l", &Codec::<i32>::new());
        assert!(TypedInputList::is_empty(&*tl));
        assert_eq!(tl.stream().count(), 0);
    }

    #[test]
    fn lookup_and_short_widening() {
        let vi = EmptyValueInput::new();
        let _p = vi.lookup();
        assert_eq!(vi.get_short_or("s", -1i16), -1i32);
        assert_eq!(vi.get_short_or("s", i16::MAX), 32767i32);
        assert_eq!(vi.get_int_array("missing"), None);
    }
}
