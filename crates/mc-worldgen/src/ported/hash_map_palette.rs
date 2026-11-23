use std::fmt;

/// Minimal self-contained VarInt helper.
// JAVA: net.minecraft.network.VarInt#getByteSize
pub fn var_int_byte_size(value: i32) -> i32 {
    let mut i: i32 = 1;
    while i < 5 {
        let mask = (-1i32).wrapping_shl(((i * 7) as u32) & 31);
        if (value & mask) == 0 {
            return i;
        }
        i += 1;
    }
    5
}

/// Minimal self-contained byte buffer for palette serialization.
#[derive(Clone, Debug, Default)]
pub struct FriendlyByteBuf {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.pos];
        self.pos += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte() as u32;
            result |= (b & 0x7F).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return result as i32;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }
}

/// Minimal self-contained id map.
pub trait IdMap<T> {
    // JAVA: net.minecraft.core.IdMap#byIdOrThrow
    fn by_id_or_throw(&self, id: i32) -> T;
    // JAVA: net.minecraft.core.IdMap#getId
    fn get_id(&self, value: &T) -> i32;
}

/// Minimal self-contained palette resize callback.
pub trait PaletteResize<T> {
    // JAVA: net.minecraft.world.level.chunk.PaletteResize#onResize
    fn on_resize(&mut self, bits: i32, value: &T) -> i32;
}

#[derive(Debug, Clone)]
pub struct MissingPaletteEntryException {
    pub id: i32,
}

impl fmt::Display for MissingPaletteEntryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing Palette entry for id {}.", self.id)
    }
}

impl std::error::Error for MissingPaletteEntryException {}

/// Minimal self-contained identity-like incremental int map.
/// Java uses identity semantics; this Rust port uses PartialEq for safe generics.
// JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap
#[derive(Clone, Debug)]
pub struct CrudeIncrementalIntIdentityHashBiMap<T> {
    values: Vec<T>,
}

impl<T: PartialEq> CrudeIncrementalIntIdentityHashBiMap<T> {
    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#create
    pub fn create(_initial_capacity: i32) -> Self {
        Self { values: Vec::new() }
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#getId
    pub fn get_id(&self, value: &T) -> i32 {
        for (i, v) in self.values.iter().enumerate() {
            if v == value {
                return i as i32;
            }
        }
        -1
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#add
    pub fn add(&mut self, value: T) -> i32 {
        let id = self.values.len() as i32;
        self.values.push(value);
        id
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#byId
    pub fn by_id(&self, id: i32) -> Option<&T> {
        if id < 0 {
            return None;
        }
        self.values.get(id as usize)
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#size
    pub fn size(&self) -> i32 {
        self.values.len() as i32
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#clear
    pub fn clear(&mut self) {
        self.values.clear();
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#iterator
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    // JAVA: net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap#copy
    pub fn copy_map(&self) -> Self
    where
        T: Clone,
    {
        self.clone()
    }
}

#[derive(Clone, Debug)]
pub struct HashMapPalette<T> {
    values: CrudeIncrementalIntIdentityHashBiMap<T>,
    bits: i32,
}

impl<T: PartialEq> HashMapPalette<T> {
    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#<init>(int,List)
    pub fn new(bits: i32, values: Vec<T>) -> Self {
        let mut map = CrudeIncrementalIntIdentityHashBiMap::create(1i32.wrapping_shl((bits as u32) & 31));
        for v in values {
            map.add(v);
        }
        Self { values: map, bits }
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#<init>(int)
    pub fn with_bits(bits: i32) -> Self {
        Self {
            bits,
            values: CrudeIncrementalIntIdentityHashBiMap::create(1i32.wrapping_shl((bits as u32) & 31)),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#create
    pub fn create(bits: i32, values: Vec<T>) -> Self {
        Self::new(bits, values)
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#idFor
    pub fn id_for<R: PaletteResize<T>>(&mut self, value: T, resize: &mut R) -> i32
    where
        T: Clone,
    {
        let mut id = self.values.get_id(&value);
        if id == -1 {
            id = self.values.add(value.clone());
            let capacity = 1i32.wrapping_shl((self.bits as u32) & 31);
            if id >= capacity {
                id = resize.on_resize(self.bits.wrapping_add(1), &value);
            }
        }
        id
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#maybeHas
    pub fn maybe_has<F: Fn(&T) -> bool>(&self, predicate: F) -> bool {
        let size = self.get_size();
        let mut i: i32 = 0;
        while i < size {
            if let Some(v) = self.values.by_id(i) {
                if predicate(v) {
                    return true;
                }
            }
            i = i.wrapping_add(1);
        }
        false
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#valueFor
    pub fn value_for(&self, id: i32) -> Result<&T, MissingPaletteEntryException> {
        match self.values.by_id(id) {
            Some(v) => Ok(v),
            None => Err(MissingPaletteEntryException { id }),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#getSize
    pub fn get_size(&self) -> i32 {
        self.values.size()
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#getEntries
    pub fn get_entries(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.values.iter().cloned().collect()
    }
}

impl<T: PartialEq + Clone> HashMapPalette<T> {
    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#read
    pub fn read<M: IdMap<T>>(&mut self, buf: &mut FriendlyByteBuf, id_map: &M) {
        self.values.clear();
        let count = buf.read_var_int();
        let mut i: i32 = 0;
        while i < count {
            self.values.add(id_map.by_id_or_throw(buf.read_var_int()));
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#write
    pub fn write<M: IdMap<T>>(&self, buf: &mut FriendlyByteBuf, id_map: &M) {
        let size = self.get_size();
        buf.write_var_int(size);
        let mut i: i32 = 0;
        while i < size {
            let v = self.values.by_id(i).expect("palette entry missing during write");
            buf.write_var_int(id_map.get_id(v));
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#getSerializedSize
    pub fn get_serialized_size<M: IdMap<T>>(&self, id_map: &M) -> i32 {
        let mut size = var_int_byte_size(self.get_size());
        let count = self.get_size();
        let mut i: i32 = 0;
        while i < count {
            let v = self.values.by_id(i).expect("palette entry missing during size calculation");
            size = size.wrapping_add(var_int_byte_size(id_map.get_id(v)));
            i = i.wrapping_add(1);
        }
        size
    }

    // JAVA: net.minecraft.world.level.chunk.HashMapPalette#copy
    pub fn copy(&self) -> Self {
        Self {
            bits: self.bits,
            values: self.values.copy_map(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Val(i32);

    struct TestIdMap;
    impl IdMap<Val> for TestIdMap {
        fn by_id_or_throw(&self, id: i32) -> Val {
            Val(id)
        }
        fn get_id(&self, value: &Val) -> i32 {
            value.0
        }
    }

    struct NoResize;
    impl PaletteResize<Val> for NoResize {
        fn on_resize(&mut self, _bits: i32, _value: &Val) -> i32 {
            panic!("unexpected resize")
        }
    }

    struct ResizeTo(i32);
    impl PaletteResize<Val> for ResizeTo {
        fn on_resize(&mut self, bits: i32, _value: &Val) -> i32 {
            self.0 = bits;
            999
        }
    }

    #[test]
    fn id_for_adds_and_reuses() {
        let mut p = HashMapPalette::with_bits(4);
        let mut nr = NoResize;
        assert_eq!(p.id_for(Val(10), &mut nr), 0);
        assert_eq!(p.id_for(Val(20), &mut nr), 1);
        assert_eq!(p.id_for(Val(10), &mut nr), 0);
        assert_eq!(p.get_size(), 2);
        assert_eq!(p.value_for(1).unwrap(), &Val(20));
    }

    #[test]
    fn id_for_resizes_when_full() {
        let mut p = HashMapPalette::with_bits(1); // capacity 2
        let mut nr = NoResize;
        assert_eq!(p.id_for(Val(1), &mut nr), 0);
        assert_eq!(p.id_for(Val(2), &mut nr), 1);
        let mut r = ResizeTo(0);
        let id = p.id_for(Val(3), &mut r);
        assert_eq!(id, 999);
        assert_eq!(r.0, 2); // bits + 1
        // Value was still added to this palette, matching Java behavior.
        assert_eq!(p.get_size(), 3);
    }

    #[test]
    fn missing_entry_returns_error() {
        let p: HashMapPalette<Val> = HashMapPalette::with_bits(2);
        let err = p.value_for(5).unwrap_err();
        assert_eq!(err.id, 5);
        assert_eq!(err.to_string(), "Missing Palette entry for id 5.");
    }

    #[test]
    fn read_write_roundtrip_with_negative_ids() {
        let mut p = HashMapPalette::with_bits(4);
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(2); // count
        buf.write_var_int(-1);
        buf.write_var_int(-123456);
        buf.pos = 0;
        p.read(&mut buf, &TestIdMap);
        assert_eq!(p.get_size(), 2);
        assert_eq!(p.value_for(0).unwrap(), &Val(-1));
        assert_eq!(p.value_for(1).unwrap(), &Val(-123456));

        let mut out = FriendlyByteBuf::default();
        p.write(&mut out, &TestIdMap);
        assert_eq!(p.get_serialized_size(&TestIdMap) as usize, out.data.len());
    }

    #[test]
    fn copy_and_entries_are_independent() {
        let mut p = HashMapPalette::with_bits(4);
        let mut nr = NoResize;
        p.id_for(Val(7), &mut nr);
        let mut c = p.copy();
        c.id_for(Val(8), &mut nr);
        assert_eq!(p.get_size(), 1);
        assert_eq!(c.get_size(), 2);
        assert_eq!(c.get_entries(), vec![Val(7), Val(8)]);
    }
}
