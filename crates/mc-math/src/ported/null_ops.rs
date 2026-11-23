use std::collections::HashMap;
use std::fmt;

/// Minimal representation of `net.minecraft.util.Unit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Unit;

impl Unit {
    pub const INSTANCE: Self = Self;
}

/// Minimal DataResult equivalent for this self-contained port.
pub type DataResult<T> = Result<T, String>;

/// Minimal numeric representation for `DynamicOps#createNumeric` / `getNumberValue`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl From<i8> for NumberValue {
    fn from(value: i8) -> Self {
        Self::Byte(value)
    }
}

impl From<i16> for NumberValue {
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<i32> for NumberValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for NumberValue {
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f32> for NumberValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for NumberValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

/// Minimal `com.mojang.serialization.MapLike` equivalent.
pub trait MapLike<T: Clone> {
    fn get(&self, key: &T) -> Option<T>;
    fn get_str(&self, key: &str) -> Option<T>;
    fn entries(&self) -> Vec<(T, T)>;
}

/// Minimal `com.mojang.serialization.DynamicOps` surface needed by `NullOps`.
pub trait DynamicOps<T: Clone> {
    fn empty(&self) -> T;
}

#[derive(Debug, Clone, Copy, Default)]
struct EmptyMapLike;

impl MapLike<Unit> for EmptyMapLike {
    // JAVA: net.minecraft.util.NullOps$1#get
    fn get(&self, _key: &Unit) -> Option<Unit> {
        None
    }

    // JAVA: net.minecraft.util.NullOps$1#get
    fn get_str(&self, _key: &str) -> Option<Unit> {
        None
    }

    // JAVA: net.minecraft.util.NullOps$1#entries
    fn entries(&self) -> Vec<(Unit, Unit)> {
        Vec::new()
    }
}

/// Port of `net.minecraft.util.NullOps`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullOps;

impl NullOps {
    pub const INSTANCE: Self = Self;

    // JAVA: net.minecraft.util.NullOps#convertTo
    pub fn convert_to<U: Clone, O: DynamicOps<U>>(&self, ops: &O, _input: Unit) -> U {
        ops.empty()
    }

    // JAVA: net.minecraft.util.NullOps#empty
    pub fn empty(&self) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#emptyMap
    pub fn empty_map(&self) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#emptyList
    pub fn empty_list(&self) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createNumeric
    pub fn create_numeric(&self, _value: NumberValue) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createByte
    pub fn create_byte(&self, _value: i8) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createShort
    pub fn create_short(&self, _value: i16) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createInt
    pub fn create_int(&self, _value: i32) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createLong
    pub fn create_long(&self, _value: i64) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createFloat
    pub fn create_float(&self, _value: f32) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createDouble
    pub fn create_double(&self, _value: f64) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createBoolean
    pub fn create_boolean(&self, _value: bool) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createString
    pub fn create_string(&self, _value: &str) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#getNumberValue
    pub fn get_number_value(&self, _input: Unit) -> DataResult<NumberValue> {
        Ok(NumberValue::Int(0))
    }

    // JAVA: net.minecraft.util.NullOps#getBooleanValue
    pub fn get_boolean_value(&self, _input: Unit) -> DataResult<bool> {
        Ok(false)
    }

    // JAVA: net.minecraft.util.NullOps#getStringValue
    pub fn get_string_value(&self, _input: Unit) -> DataResult<String> {
        Ok(String::new())
    }

    // JAVA: net.minecraft.util.NullOps#mergeToList
    pub fn merge_to_list(&self, _list: Unit, _value: Unit) -> DataResult<Unit> {
        Ok(Unit::INSTANCE)
    }

    // JAVA: net.minecraft.util.NullOps#mergeToList
    pub fn merge_to_list_values(&self, _list: Unit, _values: Vec<Unit>) -> DataResult<Unit> {
        Ok(Unit::INSTANCE)
    }

    // JAVA: net.minecraft.util.NullOps#mergeToMap
    pub fn merge_to_map(&self, _map: Unit, _key: Unit, _value: Unit) -> DataResult<Unit> {
        Ok(Unit::INSTANCE)
    }

    // JAVA: net.minecraft.util.NullOps#mergeToMap
    pub fn merge_to_map_entries(
        &self,
        _map: Unit,
        _values: HashMap<Unit, Unit>,
    ) -> DataResult<Unit> {
        Ok(Unit::INSTANCE)
    }

    // JAVA: net.minecraft.util.NullOps#mergeToMap
    pub fn merge_to_map_like(
        &self,
        _map: Unit,
        _values: &dyn MapLike<Unit>,
    ) -> DataResult<Unit> {
        Ok(Unit::INSTANCE)
    }

    // JAVA: net.minecraft.util.NullOps#getMapValues
    pub fn get_map_values(&self, _input: Unit) -> DataResult<Vec<(Unit, Unit)>> {
        Ok(Vec::new())
    }

    // JAVA: net.minecraft.util.NullOps#getMapEntries
    pub fn get_map_entries(
        &self,
        _input: Unit,
    ) -> DataResult<Box<dyn FnOnce(&mut dyn FnMut(Unit, Unit))>> {
        Ok(Box::new(|_consumer: &mut dyn FnMut(Unit, Unit)| {}))
    }

    // JAVA: net.minecraft.util.NullOps#getMap
    pub fn get_map(&self, _input: Unit) -> DataResult<Box<dyn MapLike<Unit>>> {
        Ok(Box::new(EmptyMapLike))
    }

    // JAVA: net.minecraft.util.NullOps#getStream
    pub fn get_stream(&self, _input: Unit) -> DataResult<Vec<Unit>> {
        Ok(Vec::new())
    }

    // JAVA: net.minecraft.util.NullOps#getList
    pub fn get_list(
        &self,
        _input: Unit,
    ) -> DataResult<Box<dyn FnOnce(&mut dyn FnMut(Unit))>> {
        Ok(Box::new(|_consumer: &mut dyn FnMut(Unit)| {}))
    }

    // JAVA: net.minecraft.util.NullOps#getByteBuffer
    pub fn get_byte_buffer(&self, _input: Unit) -> DataResult<Vec<u8>> {
        Ok(Vec::new())
    }

    // JAVA: net.minecraft.util.NullOps#getIntStream
    pub fn get_int_stream(&self, _input: Unit) -> DataResult<Vec<i32>> {
        Ok(Vec::new())
    }

    // JAVA: net.minecraft.util.NullOps#getLongStream
    pub fn get_long_stream(&self, _input: Unit) -> DataResult<Vec<i64>> {
        Ok(Vec::new())
    }

    // JAVA: net.minecraft.util.NullOps#createMap
    pub fn create_map(&self, _entries: Vec<(Unit, Unit)>) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createMap
    pub fn create_map_from_map(&self, _map: HashMap<Unit, Unit>) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createList
    pub fn create_list(&self, _values: Vec<Unit>) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createByteList
    pub fn create_byte_list(&self, _buf: &[u8]) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createIntList
    pub fn create_int_list(&self, _values: &[i32]) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#createLongList
    pub fn create_long_list(&self, _values: &[i64]) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps#remove
    pub fn remove(&self, input: Unit, _key: &str) -> Unit {
        input
    }

    // JAVA: net.minecraft.util.NullOps#mapBuilder
    pub fn map_builder(&self) -> NullMapBuilder {
        NullMapBuilder::new(*self)
    }

    // JAVA: net.minecraft.util.NullOps#listBuilder
    pub fn list_builder(&self) -> NullListBuilder {
        NullListBuilder::new(*self)
    }
}

impl DynamicOps<Unit> for NullOps {
    fn empty(&self) -> Unit {
        NullOps::empty(self)
    }
}

impl fmt::Display for NullOps {
    // JAVA: net.minecraft.util.NullOps#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Null")
    }
}

/// Port of `net.minecraft.util.NullOps.NullListBuilder`.
#[derive(Debug, Clone, Copy)]
pub struct NullListBuilder {
    _ops: NullOps,
}

impl NullListBuilder {
    // JAVA: net.minecraft.util.NullOps$NullListBuilder#<init>
    pub fn new(ops: NullOps) -> Self {
        Self { _ops: ops }
    }

    // JAVA: net.minecraft.util.NullOps$NullListBuilder#initBuilder
    pub fn init_builder(&self) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps$NullListBuilder#append
    pub fn append(&self, prefix: Unit, _value: Unit) -> Unit {
        prefix
    }

    // JAVA: net.minecraft.util.NullOps$NullListBuilder#build
    pub fn build(&self, builder: Unit, _prefix: Unit) -> DataResult<Unit> {
        Ok(builder)
    }
}

/// Port of `net.minecraft.util.NullOps.NullMapBuilder`.
#[derive(Debug, Clone, Copy)]
pub struct NullMapBuilder {
    _ops: NullOps,
}

impl NullMapBuilder {
    // JAVA: net.minecraft.util.NullOps$NullMapBuilder#<init>
    pub fn new(ops: NullOps) -> Self {
        Self { _ops: ops }
    }

    // JAVA: net.minecraft.util.NullOps$NullMapBuilder#initBuilder
    pub fn init_builder(&self) -> Unit {
        Unit::INSTANCE
    }

    // JAVA: net.minecraft.util.NullOps$NullMapBuilder#append
    pub fn append(&self, _key: Unit, _value: Unit, builder: Unit) -> Unit {
        builder
    }

    // JAVA: net.minecraft.util.NullOps$NullMapBuilder#build
    pub fn build(&self, _builder: Unit, prefix: Unit) -> DataResult<Unit> {
        Ok(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_methods_handle_negative_overflow_and_bounds() {
        let ops = NullOps::INSTANCE;
        assert_eq!(ops.create_byte(i8::MIN), Unit::INSTANCE);
        assert_eq!(ops.create_byte(-1), Unit::INSTANCE);
        assert_eq!(ops.create_short(i16::MIN), Unit::INSTANCE);
        assert_eq!(ops.create_int(i32::MIN), Unit::INSTANCE);
        assert_eq!(ops.create_int(i32::MAX), Unit::INSTANCE);
        assert_eq!(ops.create_long(i64::MIN), Unit::INSTANCE);
        assert_eq!(ops.create_long(i64::MAX), Unit::INSTANCE);
        assert_eq!(ops.create_boolean(true), Unit::INSTANCE);
        assert_eq!(ops.create_string(""), Unit::INSTANCE);
        assert_eq!(ops.create_numeric(NumberValue::Int(i32::MIN)), Unit::INSTANCE);
    }

    #[test]
    fn create_float_and_double_handle_special_float_bits() {
        let ops = NullOps::INSTANCE;
        let neg_zero_f32 = f32::from_bits(0x8000_0000);
        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let neg_inf_f64 = f64::NEG_INFINITY;
        let nan_f64 = f64::from_bits(0x7FF8_0000_0000_0001);
        let max_f64 = f64::from_bits(0x7FEF_FFFF_FFFF_FFFF);

        assert_eq!(ops.create_float(neg_zero_f32), Unit::INSTANCE);
        assert_eq!(ops.create_float(nan_f32), Unit::INSTANCE);
        assert_eq!(ops.create_float(f32::MIN), Unit::INSTANCE);
        assert_eq!(ops.create_double(neg_inf_f64), Unit::INSTANCE);
        assert_eq!(ops.create_double(nan_f64), Unit::INSTANCE);
        assert_eq!(ops.create_double(max_f64), Unit::INSTANCE);

        assert_eq!(neg_zero_f32.to_bits(), 0x8000_0000);
        assert!(nan_f32.is_nan());
        assert!(nan_f64.is_nan());
        assert_eq!(max_f64.to_bits(), 0x7FEF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn get_methods_return_java_defaults() {
        let ops = NullOps::INSTANCE;
        assert_eq!(
            ops.get_number_value(Unit::INSTANCE).unwrap(),
            NumberValue::Int(0)
        );
        assert_eq!(ops.get_boolean_value(Unit::INSTANCE).unwrap(), false);
        assert_eq!(ops.get_string_value(Unit::INSTANCE).unwrap(), "");
        assert!(ops.get_byte_buffer(Unit::INSTANCE).unwrap().is_empty());
        assert!(ops.get_int_stream(Unit::INSTANCE).unwrap().is_empty());
        assert!(ops.get_long_stream(Unit::INSTANCE).unwrap().is_empty());
        assert!(ops.get_stream(Unit::INSTANCE).unwrap().is_empty());
        assert!(ops.get_map_values(Unit::INSTANCE).unwrap().is_empty());
    }

    #[test]
    fn merge_remove_and_empty_map_behave_as_null() {
        let ops = NullOps::INSTANCE;
        assert_eq!(
            ops.merge_to_list(Unit::INSTANCE, Unit::INSTANCE).unwrap(),
            Unit::INSTANCE
        );
        assert_eq!(
            ops.merge_to_list_values(Unit::INSTANCE, vec![Unit::INSTANCE])
                .unwrap(),
            Unit::INSTANCE
        );
        assert_eq!(
            ops.merge_to_map(Unit::INSTANCE, Unit::INSTANCE, Unit::INSTANCE)
                .unwrap(),
            Unit::INSTANCE
        );
        let mut map_entries = HashMap::new();
        map_entries.insert(Unit::INSTANCE, Unit::INSTANCE);
        assert_eq!(
            ops.merge_to_map_entries(Unit::INSTANCE, map_entries)
                .unwrap(),
            Unit::INSTANCE
        );
        assert_eq!(ops.remove(Unit::INSTANCE, "missing"), Unit::INSTANCE);

        let map = ops.get_map(Unit::INSTANCE).unwrap();
        assert_eq!(map.get(&Unit::INSTANCE), None);
        assert_eq!(map.get_str("missing"), None);
        assert!(map.entries().is_empty());
    }

    #[test]
    fn builders_convert_to_and_consumers() {
        let ops = NullOps::INSTANCE;
        let list_builder = ops.list_builder();
        let list_init = list_builder.init_builder();
        let list_appended = list_builder.append(list_init, Unit::INSTANCE);
        assert_eq!(
            list_builder.build(list_appended, Unit::INSTANCE).unwrap(),
            Unit::INSTANCE
        );

        let map_builder = ops.map_builder();
        let map_init = map_builder.init_builder();
        let map_appended = map_builder.append(Unit::INSTANCE, Unit::INSTANCE, map_init);
        assert_eq!(
            map_builder.build(map_appended, Unit::INSTANCE).unwrap(),
            Unit::INSTANCE
        );

        struct OtherOps;
        impl DynamicOps<u32> for OtherOps {
            fn empty(&self) -> u32 {
                0xFFFF_FFFFu32
            }
        }
        assert_eq!(ops.convert_to(&OtherOps, Unit::INSTANCE), 0xFFFF_FFFFu32);
        assert_eq!(ops.to_string(), "Null");

        let mut count: i32 = 0;
        let map_entries = ops.get_map_entries(Unit::INSTANCE).unwrap();
        map_entries(&mut |_k, _v| {
            count = count.wrapping_add(1);
        });
        let list_consumer = ops.get_list(Unit::INSTANCE).unwrap();
        list_consumer(&mut |_v| {
            count = count.wrapping_add(1);
        });
        assert_eq!(count, 0);
    }

    #[test]
    fn create_collections_and_stream_like_inputs() {
        let ops = NullOps::INSTANCE;
        assert_eq!(ops.create_map(vec![(Unit::INSTANCE, Unit::INSTANCE)]), Unit::INSTANCE);
        let mut hm = HashMap::new();
        hm.insert(Unit::INSTANCE, Unit::INSTANCE);
        assert_eq!(ops.create_map_from_map(hm), Unit::INSTANCE);
        assert_eq!(ops.create_list(vec![Unit::INSTANCE]), Unit::INSTANCE);
        assert_eq!(ops.create_byte_list(&[u8::MIN, u8::MAX]), Unit::INSTANCE);
        assert_eq!(ops.create_int_list(&[i32::MIN, -1, 0, i32::MAX]), Unit::INSTANCE);
        assert_eq!(ops.create_long_list(&[i64::MIN, -1, 0, i64::MAX]), Unit::INSTANCE);
    }
}
