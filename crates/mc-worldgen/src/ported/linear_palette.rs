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

#[derive(Clone, Debug)]
pub struct LinearPalette<T> {
    values: Vec<Option<T>>,
    bits: i32,
    size: i32,
}

impl<T: PartialEq> LinearPalette<T> {
    // JAVA: net.minecraft.world.level.chunk.LinearPalette#<init>(int,List)
    fn new_with_values(bits: i32, values: Vec<T>) -> Self {
        let capacity = 1usize.wrapping_shl((bits as u32) & 31);
        assert!(
            values.len() <= capacity,
            "Can't initialize LinearPalette of size {} with {} entries",
            capacity,
            values.len()
        );
        let mut arr: Vec<Option<T>> = Vec::with_capacity(capacity);
        let mut i = 0usize;
        while i < capacity {
            arr.push(None);
            i = i.wrapping_add(1);
        }
        let size = values.len() as i32;
        for (idx, v) in values.into_iter().enumerate() {
            arr[idx] = Some(v);
        }
        Self { values: arr, bits, size }
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#<init>(Object[],int,int)
    fn from_parts(values: Vec<Option<T>>, bits: i32, size: i32) -> Self {
        Self { values, bits, size }
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#create
    pub fn create(bits: i32, values: Vec<T>) -> Self {
        Self::new_with_values(bits, values)
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#idFor
    pub fn id_for<R: PaletteResize<T>>(&mut self, value: T, resize: &mut R) -> i32
    where
        T: Clone,
    {
        let mut i: i32 = 0;
        while i < self.size {
            let idx = i as usize;
            if let Some(ref v) = self.values[idx] {
                // Java uses reference equality (==); PartialEq is the closest safe Rust equivalent.
                if *v == value {
                    return i;
                }
            }
            i = i.wrapping_add(1);
        }

        let next = self.size;
        let capacity = self.values.len() as i32;
        if next < capacity {
            self.values[next as usize] = Some(value);
            self.size = self.size.wrapping_add(1);
            next
        } else {
            resize.on_resize(self.bits.wrapping_add(1), &value)
        }
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#maybeHas
    pub fn maybe_has<F: Fn(&T) -> bool>(&self, predicate: F) -> bool {
        let mut i: i32 = 0;
        while i < self.size {
            if let Some(ref v) = self.values[i as usize] {
                if predicate(v) {
                    return true;
                }
            }
            i = i.wrapping_add(1);
        }
        false
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#valueFor
    pub fn value_for(&self, id: i32) -> Result<&T, MissingPaletteEntryException> {
        if id >= 0 && id < self.size {
            if let Some(ref v) = self.values[id as usize] {
                return Ok(v);
            }
        }
        Err(MissingPaletteEntryException { id })
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#getSize
    pub fn get_size(&self) -> i32 {
        self.size
    }
}

impl<T: PartialEq + Clone> LinearPalette<T> {
    // JAVA: net.minecraft.world.level.chunk.LinearPalette#read
    pub fn read<M: IdMap<T>>(&mut self, buf: &mut FriendlyByteBuf, id_map: &M) {
        self.size = buf.read_var_int();
        let mut i: i32 = 0;
        while i < self.size {
            self.values[i as usize] = Some(id_map.by_id_or_throw(buf.read_var_int()));
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#write
    pub fn write<M: IdMap<T>>(&self, buf: &mut FriendlyByteBuf, id_map: &M) {
        buf.write_var_int(self.size);
        let mut i: i32 = 0;
        while i < self.size {
            let v = self.values[i as usize]
                .as_ref()
                .expect("palette entry missing during write");
            buf.write_var_int(id_map.get_id(v));
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#getSerializedSize
    pub fn get_serialized_size<M: IdMap<T>>(&self, id_map: &M) -> i32 {
        let mut size = var_int_byte_size(self.get_size());
        let mut i: i32 = 0;
        while i < self.size {
            let v = self.values[i as usize]
                .as_ref()
                .expect("palette entry missing during size calculation");
            size = size.wrapping_add(var_int_byte_size(id_map.get_id(v)));
            i = i.wrapping_add(1);
        }
        size
    }

    // JAVA: net.minecraft.world.level.chunk.LinearPalette#copy
    pub fn copy(&self) -> Self {
        Self::from_parts(self.values.clone(), self.bits, self.size)
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
            777
        }
    }

    #[test]
    fn create_and_id_for_reuse() {
        let mut p = LinearPalette::create(2, vec![Val(5), Val(6)]);
        let mut nr = NoResize;
        assert_eq!(p.get_size(), 2);
        assert_eq!(p.id_for(Val(5), &mut nr), 0);
        assert_eq!(p.id_for(Val(6), &mut nr), 1);
        assert_eq!(p.id_for(Val(7), &mut nr), 2);
        assert_eq!(p.get_size(), 3);
    }

    #[test]
    fn id_for_resizes_when_full() {
        let mut p = LinearPalette::create(1, vec![Val(1), Val(2)]); // capacity 2
        let mut r = ResizeTo(0);
        let id = p.id_for(Val(3), &mut r);
        assert_eq!(id, 777);
        assert_eq!(r.0, 2);
        assert_eq!(p.get_size(), 2);
    }

    #[test]
    fn value_for_out_of_range_and_negative() {
        let p = LinearPalette::create(2, vec![Val(10)]);
        assert_eq!(p.value_for(0).unwrap(), &Val(10));
        assert!(p.value_for(-1).is_err());
        assert!(p.value_for(1).is_err());
    }

    #[test]
    fn read_write_roundtrip() {
        let mut p = LinearPalette::create(4, vec![]);
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(3);
        buf.write_var_int(-1);
        buf.write_var_int(0);
        buf.write_var_int(2147483647);
        buf.pos = 0;
        p.read(&mut buf, &TestIdMap);
        assert_eq!(p.get_size(), 3);
        assert_eq!(p.value_for(0).unwrap(), &Val(-1));
        assert_eq!(p.value_for(2).unwrap(), &Val(i32::MAX));

        let mut out = FriendlyByteBuf::default();
        p.write(&mut out, &TestIdMap);
        assert_eq!(p.get_serialized_size(&TestIdMap) as usize, out.data.len());
    }

    #[test]
    fn maybe_has_and_copy() {
        let p = LinearPalette::create(2, vec![Val(1), Val(2)]);
        assert!(p.maybe_has(|v| v.0 == 2));
        assert!(!p.maybe_has(|v| v.0 == 3));
        let c = p.copy();
        assert_eq!(c.value_for(1).unwrap(), &Val(2));
    }
}
