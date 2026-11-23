// === FILE: hash_ops.rs ===
use std::cmp::Ordering;
use std::fmt;
use std::sync::LazyLock;

const TAG_EMPTY: u8 = 1;
const TAG_MAP_START: u8 = 2;
const TAG_MAP_END: u8 = 3;
const TAG_LIST_START: u8 = 4;
const TAG_LIST_END: u8 = 5;
const TAG_BYTE: u8 = 6;
const TAG_SHORT: u8 = 7;
const TAG_INT: u8 = 8;
const TAG_LONG: u8 = 9;
const TAG_FLOAT: u8 = 10;
const TAG_DOUBLE: u8 = 11;
const TAG_STRING: u8 = 12;
const TAG_BOOLEAN: u8 = 13;
const TAG_BYTE_ARRAY_START: u8 = 14;
const TAG_BYTE_ARRAY_END: u8 = 15;
const TAG_INT_ARRAY_START: u8 = 16;
const TAG_INT_ARRAY_END: u8 = 17;
const TAG_LONG_ARRAY_START: u8 = 18;
const TAG_LONG_ARRAY_END: u8 = 19;

const EMPTY_PAYLOAD: &[u8] = &[TAG_EMPTY];
const FALSE_PAYLOAD: &[u8] = &[TAG_BOOLEAN, 0];
const TRUE_PAYLOAD: &[u8] = &[TAG_BOOLEAN, 1];
pub const EMPTY_MAP_PAYLOAD: &[u8] = &[TAG_MAP_START, TAG_MAP_END];
pub const EMPTY_LIST_PAYLOAD: &[u8] = &[TAG_LIST_START, TAG_LIST_END];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HashCode {
    value: u32,
}

impl HashCode {
    fn from_u32(value: u32) -> Self {
        Self { value }
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.value.to_le_bytes()
    }

    pub fn pad_to_long(&self) -> i64 {
        self.value as i64
    }
}

const fn make_crc32c_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x82F63B78;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

static CRC32C_TABLE: [u32; 256] = make_crc32c_table();

#[derive(Clone, Debug)]
struct Crc32cHasher {
    crc: u32,
}

impl Crc32cHasher {
    fn new() -> Self {
        Self { crc: 0xFFFF_FFFF }
    }

    fn put_u8(&mut self, b: u8) {
        let idx = ((self.crc ^ b as u32) & 0xFF) as usize;
        self.crc = (self.crc >> 8) ^ CRC32C_TABLE[idx];
    }

    fn put_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.put_u8(b);
        }
    }

    fn hash(&self) -> HashCode {
        HashCode::from_u32(self.crc ^ 0xFFFF_FFFF)
    }
}

#[derive(Clone, Debug)]
enum HasherKind {
    Crc32c(Crc32cHasher),
}

#[derive(Clone, Debug)]
pub struct Hasher {
    kind: HasherKind,
}

impl Hasher {
    pub fn put_u8(&mut self, b: u8) -> &mut Self {
        match &mut self.kind {
            HasherKind::Crc32c(h) => h.put_u8(b),
        }
        self
    }

    pub fn put_short(&mut self, value: i16) -> &mut Self {
        self.put_bytes(&value.to_le_bytes())
    }

    pub fn put_int(&mut self, value: i32) -> &mut Self {
        self.put_bytes(&value.to_le_bytes())
    }

    pub fn put_long(&mut self, value: i64) -> &mut Self {
        self.put_bytes(&value.to_le_bytes())
    }

    pub fn put_float(&mut self, value: f32) -> &mut Self {
        self.put_int(value.to_bits() as i32)
    }

    pub fn put_double(&mut self, value: f64) -> &mut Self {
        self.put_long(value.to_bits() as i64)
    }

    pub fn put_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        match &mut self.kind {
            HasherKind::Crc32c(h) => h.put_bytes(bytes),
        }
        self
    }

    pub fn put_unencoded_chars(&mut self, s: &str) -> &mut Self {
        for c in s.encode_utf16() {
            self.put_u8((c & 0xFF) as u8);
            self.put_u8(((c >> 8) & 0xFF) as u8);
        }
        self
    }

    pub fn hash(&self) -> HashCode {
        match &self.kind {
            HasherKind::Crc32c(h) => h.hash(),
        }
    }
}

#[derive(Clone, Debug)]
enum HashFunctionKind {
    Crc32c,
}

#[derive(Clone, Debug)]
pub struct HashFunction {
    kind: HashFunctionKind,
}

impl HashFunction {
    pub fn crc32c() -> Self {
        Self {
            kind: HashFunctionKind::Crc32c,
        }
    }

    pub fn new_hasher(&self) -> Hasher {
        match self.kind {
            HashFunctionKind::Crc32c => Hasher {
                kind: HasherKind::Crc32c(Crc32cHasher::new()),
            },
        }
    }

    pub fn new_hasher_with_size(&self, _expected_size: usize) -> Hasher {
        self.new_hasher()
    }

    pub fn hash_bytes(&self, bytes: &[u8]) -> HashCode {
        let mut h = self.new_hasher();
        h.put_bytes(bytes);
        h.hash()
    }
}

impl fmt::Display for HashFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            HashFunctionKind::Crc32c => write!(f, "Hashing.crc32c()"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataResult<T> {
    Success(T),
    Error(String),
}

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn of(first: A, second: B) -> Self {
        Self(first, second)
    }

    pub fn get_first(&self) -> &A {
        &self.0
    }

    pub fn get_second(&self) -> &B {
        &self.1
    }
}

#[derive(Clone, Copy, Debug)]
pub enum JavaNumber {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

fn hash_comparator(a: &HashCode, b: &HashCode) -> Ordering {
    a.pad_to_long().cmp(&b.pad_to_long())
}

// JAVA: net.minecraft.util.HashOps#hashMap
fn hash_map(hasher: &mut Hasher, map: impl IntoIterator<Item = (HashCode, HashCode)>) {
    hasher.put_u8(TAG_MAP_START);
    let mut entries: Vec<(HashCode, HashCode)> = map.into_iter().collect();
    entries.sort_by(|a, b| hash_comparator(&a.0, &b.0).then_with(|| hash_comparator(&a.1, &b.1)));
    for (k, v) in &entries {
        hasher.put_bytes(&k.as_bytes());
        hasher.put_bytes(&v.as_bytes());
    }
    hasher.put_u8(TAG_MAP_END);
}

// JAVA: net.minecraft.util.HashOps#hashMap
fn hash_map_pairs(hasher: &mut Hasher, pairs: impl IntoIterator<Item = Pair<HashCode, HashCode>>) {
    hasher.put_u8(TAG_MAP_START);
    let mut entries: Vec<Pair<HashCode, HashCode>> = pairs.into_iter().collect();
    entries.sort_by(|a, b| {
        hash_comparator(a.get_first(), b.get_first())
            .then_with(|| hash_comparator(a.get_second(), b.get_second()))
    });
    for p in &entries {
        hasher.put_bytes(&p.get_first().as_bytes());
        hasher.put_bytes(&p.get_second().as_bytes());
    }
    hasher.put_u8(TAG_MAP_END);
}

// JAVA: net.minecraft.util.HashOps#unsupported
fn unsupported<T>() -> DataResult<T> {
    DataResult::Error("Unsupported operation".to_string())
}

pub struct HashOps {
    hash_function: HashFunction,
    empty: HashCode,
    empty_map: HashCode,
    empty_list: HashCode,
    true_hash: HashCode,
    false_hash: HashCode,
}

pub static CRC32C_INSTANCE: LazyLock<HashOps> =
    LazyLock::new(|| HashOps::new(HashFunction::crc32c()));

impl HashOps {
    // JAVA: net.minecraft.util.HashOps#<init>
    pub fn new(hash_function: HashFunction) -> Self {
        let empty = hash_function.hash_bytes(EMPTY_PAYLOAD);
        let empty_map = hash_function.hash_bytes(EMPTY_MAP_PAYLOAD);
        let empty_list = hash_function.hash_bytes(EMPTY_LIST_PAYLOAD);
        let false_hash = hash_function.hash_bytes(FALSE_PAYLOAD);
        let true_hash = hash_function.hash_bytes(TRUE_PAYLOAD);
        Self {
            hash_function,
            empty,
            empty_map,
            empty_list,
            true_hash,
            false_hash,
        }
    }

    // JAVA: net.minecraft.util.HashOps#empty
    pub fn empty(&self) -> HashCode {
        self.empty
    }

    // JAVA: net.minecraft.util.HashOps#emptyMap
    pub fn empty_map(&self) -> HashCode {
        self.empty_map
    }

    // JAVA: net.minecraft.util.HashOps#emptyList
    pub fn empty_list(&self) -> HashCode {
        self.empty_list
    }

    // JAVA: net.minecraft.util.HashOps#createNumeric
    pub fn create_numeric(&self, value: JavaNumber) -> HashCode {
        match value {
            JavaNumber::Byte(v) => self.create_byte(v),
            JavaNumber::Short(v) => self.create_short(v),
            JavaNumber::Int(v) => self.create_int(v),
            JavaNumber::Long(v) => self.create_long(v),
            JavaNumber::Double(v) => self.create_double(v),
            JavaNumber::Float(v) => self.create_float(v),
        }
    }

    // JAVA: net.minecraft.util.HashOps#createByte
    pub fn create_byte(&self, value: i8) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(2);
        h.put_u8(TAG_BYTE);
        h.put_u8(value as u8);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createShort
    pub fn create_short(&self, value: i16) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(3);
        h.put_u8(TAG_SHORT);
        h.put_short(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createInt
    pub fn create_int(&self, value: i32) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(5);
        h.put_u8(TAG_INT);
        h.put_int(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createLong
    pub fn create_long(&self, value: i64) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(9);
        h.put_u8(TAG_LONG);
        h.put_long(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createFloat
    pub fn create_float(&self, value: f32) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(5);
        h.put_u8(TAG_FLOAT);
        h.put_float(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createDouble
    pub fn create_double(&self, value: f64) -> HashCode {
        let mut h = self.hash_function.new_hasher_with_size(9);
        h.put_u8(TAG_DOUBLE);
        h.put_double(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createString
    pub fn create_string(&self, value: &str) -> HashCode {
        let utf16_len = value.encode_utf16().count() as i32;
        let mut h = self.hash_function.new_hasher();
        h.put_u8(TAG_STRING);
        h.put_int(utf16_len);
        h.put_unencoded_chars(value);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createBoolean
    pub fn create_boolean(&self, value: bool) -> HashCode {
        if value {
            self.true_hash
        } else {
            self.false_hash
        }
    }

    // JAVA: net.minecraft.util.HashOps#createMap
    pub fn create_map(&self, entries: impl IntoIterator<Item = (HashCode, HashCode)>) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        hash_map(&mut h, entries);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createMap
    pub fn create_map_pairs(
        &self,
        entries: impl IntoIterator<Item = Pair<HashCode, HashCode>>,
    ) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        hash_map_pairs(&mut h, entries);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createList
    pub fn create_list(&self, values: impl IntoIterator<Item = HashCode>) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        h.put_u8(TAG_LIST_START);
        for v in values {
            h.put_bytes(&v.as_bytes());
        }
        h.put_u8(TAG_LIST_END);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createByteList
    pub fn create_byte_list(&self, data: &[u8]) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        h.put_u8(TAG_BYTE_ARRAY_START);
        h.put_bytes(data);
        h.put_u8(TAG_BYTE_ARRAY_END);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createIntList
    pub fn create_int_list(&self, values: impl IntoIterator<Item = i32>) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        h.put_u8(TAG_INT_ARRAY_START);
        for v in values {
            h.put_int(v);
        }
        h.put_u8(TAG_INT_ARRAY_END);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#createLongList
    pub fn create_long_list(&self, values: impl IntoIterator<Item = i64>) -> HashCode {
        let mut h = self.hash_function.new_hasher();
        h.put_u8(TAG_LONG_ARRAY_START);
        for v in values {
            h.put_long(v);
        }
        h.put_u8(TAG_LONG_ARRAY_END);
        h.hash()
    }

    // JAVA: net.minecraft.util.HashOps#remove
    pub fn remove(&self, input: HashCode, _key: &str) -> HashCode {
        input
    }

    // JAVA: net.minecraft.util.HashOps#mapBuilder
    pub fn map_builder(&self) -> MapHashBuilder<'_> {
        MapHashBuilder::new(self)
    }

    // JAVA: net.minecraft.util.HashOps#listBuilder
    pub fn list_builder(&self) -> ListHashBuilder<'_> {
        ListHashBuilder::new(self)
    }

    // JAVA: net.minecraft.util.HashOps#convertTo
    pub fn convert_to<U>(&self, _input: HashCode) -> U {
        panic!("Can't convert from this type")
    }

    // JAVA: net.minecraft.util.HashOps#getNumberValue
    pub fn get_number_value_or(&self, _input: HashCode, default: JavaNumber) -> JavaNumber {
        default
    }

    // JAVA: net.minecraft.util.HashOps#getNumberValue
    pub fn get_number_value(&self, _input: HashCode) -> DataResult<JavaNumber> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#set
    pub fn set(&self, input: HashCode, _key: &str, _value: HashCode) -> HashCode {
        input
    }

    // JAVA: net.minecraft.util.HashOps#update
    pub fn update<F: FnOnce(HashCode) -> HashCode>(
        &self,
        input: HashCode,
        _key: &str,
        _f: F,
    ) -> HashCode {
        input
    }

    // JAVA: net.minecraft.util.HashOps#updateGeneric
    pub fn update_generic<F: FnOnce(HashCode) -> HashCode>(
        &self,
        input: HashCode,
        _key: HashCode,
        _f: F,
    ) -> HashCode {
        input
    }

    // JAVA: net.minecraft.util.HashOps#get
    pub fn get(&self, _input: HashCode, _key: &str) -> DataResult<HashCode> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getGeneric
    pub fn get_generic(&self, _input: HashCode, _key: HashCode) -> DataResult<HashCode> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getBooleanValue
    pub fn get_boolean_value(&self, _input: HashCode) -> DataResult<bool> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getStringValue
    pub fn get_string_value(&self, _input: HashCode) -> DataResult<String> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#isEmpty
    fn is_empty(&self, value: HashCode) -> bool {
        value == self.empty
    }

    // JAVA: net.minecraft.util.HashOps#mergeToList
    pub fn merge_to_list(&self, list: HashCode, value: HashCode) -> DataResult<HashCode> {
        if self.is_empty(list) {
            DataResult::success(self.create_list([value]))
        } else {
            unsupported()
        }
    }

    // JAVA: net.minecraft.util.HashOps#mergeToList
    pub fn merge_to_list_values(
        &self,
        list: HashCode,
        values: impl IntoIterator<Item = HashCode>,
    ) -> DataResult<HashCode> {
        if self.is_empty(list) {
            DataResult::success(self.create_list(values))
        } else {
            unsupported()
        }
    }

    // JAVA: net.minecraft.util.HashOps#mergeToMap
    pub fn merge_to_map(
        &self,
        map: HashCode,
        key: HashCode,
        value: HashCode,
    ) -> DataResult<HashCode> {
        if self.is_empty(map) {
            DataResult::success(self.create_map([(key, value)]))
        } else {
            unsupported()
        }
    }

    // JAVA: net.minecraft.util.HashOps#mergeToMap
    pub fn merge_to_map_entries(
        &self,
        map: HashCode,
        entries: impl IntoIterator<Item = (HashCode, HashCode)>,
    ) -> DataResult<HashCode> {
        if self.is_empty(map) {
            DataResult::success(self.create_map(entries))
        } else {
            unsupported()
        }
    }

    // JAVA: net.minecraft.util.HashOps#mergeToMap
    pub fn merge_to_map_pairs(
        &self,
        map: HashCode,
        entries: impl IntoIterator<Item = Pair<HashCode, HashCode>>,
    ) -> DataResult<HashCode> {
        if self.is_empty(map) {
            DataResult::success(self.create_map_pairs(entries))
        } else {
            unsupported()
        }
    }

    // JAVA: net.minecraft.util.HashOps#getMapValues
    pub fn get_map_values(
        &self,
        _input: HashCode,
    ) -> DataResult<Vec<Pair<HashCode, HashCode>>> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getMapEntries
    pub fn get_map_entries(&self, _input: HashCode) -> DataResult<()> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getStream
    pub fn get_stream(&self, _input: HashCode) -> DataResult<Vec<HashCode>> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getList
    pub fn get_list(&self, _input: HashCode) -> DataResult<()> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getMap
    pub fn get_map(&self, _input: HashCode) -> DataResult<()> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getByteBuffer
    pub fn get_byte_buffer(&self, _input: HashCode) -> DataResult<Vec<u8>> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getIntStream
    pub fn get_int_stream(&self, _input: HashCode) -> DataResult<Vec<i32>> {
        unsupported()
    }

    // JAVA: net.minecraft.util.HashOps#getLongStream
    pub fn get_long_stream(&self, _input: HashCode) -> DataResult<Vec<i64>> {
        unsupported()
    }
}

impl fmt::Display for HashOps {
    // JAVA: net.minecraft.util.HashOps#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash {}", self.hash_function)
    }
}

pub struct ListHashBuilder<'a> {
    ops: &'a HashOps,
    builder: Hasher,
}

impl<'a> ListHashBuilder<'a> {
    // JAVA: net.minecraft.util.HashOps$ListHashBuilder#<init>
    fn new(ops: &'a HashOps) -> Self {
        Self {
            builder: Self::init_builder(ops),
            ops,
        }
    }

    // JAVA: net.minecraft.util.HashOps$ListHashBuilder#initBuilder
    fn init_builder(ops: &HashOps) -> Hasher {
        let mut h = ops.hash_function.new_hasher();
        h.put_u8(TAG_LIST_START);
        h
    }

    // JAVA: net.minecraft.util.HashOps$ListHashBuilder#append
    fn append(hasher: &mut Hasher, value: HashCode) {
        hasher.put_bytes(&value.as_bytes());
    }

    pub fn add(&mut self, value: HashCode) -> &mut Self {
        Self::append(&mut self.builder, value);
        self
    }

    // JAVA: net.minecraft.util.HashOps$ListHashBuilder#build
    pub fn build(mut self, prefix: HashCode) -> DataResult<HashCode> {
        debug_assert_eq!(prefix, self.ops.empty);
        self.builder.put_u8(TAG_LIST_END);
        DataResult::success(self.builder.hash())
    }
}

pub struct MapHashBuilder<'a> {
    ops: &'a HashOps,
    entries: Vec<(HashCode, HashCode)>,
}

impl<'a> MapHashBuilder<'a> {
    // JAVA: net.minecraft.util.HashOps$MapHashBuilder#<init>
    fn new(ops: &'a HashOps) -> Self {
        Self {
            ops,
            entries: Self::init_builder(),
        }
    }

    // JAVA: net.minecraft.util.HashOps$MapHashBuilder#initBuilder
    fn init_builder() -> Vec<(HashCode, HashCode)> {
        Vec::new()
    }

    // JAVA: net.minecraft.util.HashOps$MapHashBuilder#append
    fn append(entries: &mut Vec<(HashCode, HashCode)>, key: HashCode, value: HashCode) {
        entries.push((key, value));
    }

    pub fn add(&mut self, key: HashCode, value: HashCode) -> &mut Self {
        Self::append(&mut self.entries, key, value);
        self
    }

    // JAVA: net.minecraft.util.HashOps$MapHashBuilder#build
    pub fn build(mut self, prefix: HashCode) -> DataResult<HashCode> {
        debug_assert!(self.ops.is_empty(prefix));
        let mut h = self.ops.hash_function.new_hasher();
        hash_map(&mut h, self.entries.drain(..));
        DataResult::success(h.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32c_known_value_and_empty() {
        let h = HashFunction::crc32c().hash_bytes(b"123456789");
        assert_eq!(h.pad_to_long(), 0xE3069283i64);
        assert_eq!(h.as_bytes(), 0xE3069283u32.to_le_bytes());

        let ops = HashOps::new(HashFunction::crc32c());
        let mut expected = HashFunction::crc32c().new_hasher();
        expected.put_u8(TAG_EMPTY);
        assert_eq!(ops.empty(), expected.hash());
    }

    #[test]
    fn negative_and_extreme_integers() {
        let ops = HashOps::new(HashFunction::crc32c());

        let mut e = HashFunction::crc32c().new_hasher_with_size(2);
        e.put_u8(TAG_BYTE);
        e.put_u8(0x80);
        assert_eq!(ops.create_byte(i8::MIN), e.hash());

        let mut e2 = HashFunction::crc32c().new_hasher_with_size(5);
        e2.put_u8(TAG_INT);
        e2.put_int(i32::MIN);
        assert_eq!(ops.create_int(i32::MIN), e2.hash());

        let mut e3 = HashFunction::crc32c().new_hasher_with_size(9);
        e3.put_u8(TAG_LONG);
        e3.put_long(-1i64);
        assert_eq!(ops.create_long(-1i64), e3.hash());
    }

    #[test]
    fn float_and_double_raw_bits() {
        let ops = HashOps::new(HashFunction::crc32c());
        assert_ne!(ops.create_float(-0.0f32), ops.create_float(0.0f32));
        assert_ne!(ops.create_double(-0.0f64), ops.create_double(0.0f64));

        let mut e = HashFunction::crc32c().new_hasher_with_size(5);
        e.put_u8(TAG_FLOAT);
        e.put_int((-0.0f32).to_bits() as i32);
        assert_eq!(ops.create_float(-0.0f32), e.hash());

        let mut e2 = HashFunction::crc32c().new_hasher_with_size(9);
        e2.put_u8(TAG_DOUBLE);
        e2.put_long(f64::INFINITY.to_bits() as i64);
        assert_eq!(ops.create_double(f64::INFINITY), e2.hash());
    }

    #[test]
    fn map_order_is_deterministic() {
        let ops = HashOps::new(HashFunction::crc32c());
        let k1 = ops.create_int(1);
        let v1 = ops.create_string("a");
        let k2 = ops.create_int(2);
        let v2 = ops.create_string("b");
        let h1 = ops.create_map([(k1, v1), (k2, v2)]);
        let h2 = ops.create_map([(k2, v2), (k1, v1)]);
        assert_eq!(h1, h2);
        assert_ne!(h1, ops.empty_map());
    }

    #[test]
    fn string_utf16_and_list_encoding() {
        let ops = HashOps::new(HashFunction::crc32c());
        let s = "A😀B";
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut e = HashFunction::crc32c().new_hasher();
        e.put_u8(TAG_STRING);
        e.put_int(utf16.len() as i32);
        for c in &utf16 {
            e.put_u8((c & 0xFF) as u8);
            e.put_u8(((c >> 8) & 0xFF) as u8);
        }
        assert_eq!(ops.create_string(s), e.hash());

        let a = ops.create_byte(1);
        let b = ops.create_byte(2);
        let mut el = HashFunction::crc32c().new_hasher();
        el.put_u8(TAG_LIST_START);
        el.put_bytes(&a.as_bytes());
        el.put_bytes(&b.as_bytes());
        el.put_u8(TAG_LIST_END);
        assert_eq!(ops.create_list([a, b]), el.hash());
    }

    #[test]
    fn booleans_empty_and_merge() {
        let ops = HashOps::new(HashFunction::crc32c());
        assert_ne!(ops.create_boolean(true), ops.create_boolean(false));
        assert_eq!(ops.empty_map(), ops.create_map([]));
        assert_eq!(ops.empty_list(), ops.create_list([]));

        let v = ops.create_int(7);
        match ops.merge_to_list(ops.empty(), v) {
            DataResult::Success(h) => assert_eq!(h, ops.create_list([v])),
            DataResult::Error(_) => panic!("expected success"),
        }
        match ops.merge_to_list(v, v) {
            DataResult::Success(_) => panic!("expected error"),
            DataResult::Error(msg) => assert_eq!(msg, "Unsupported operation"),
        }
    }
}
