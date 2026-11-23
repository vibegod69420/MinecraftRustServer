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

#[derive(Clone, Debug)]
pub struct SingleValuePalette<T> {
    value: Option<T>,
}

impl<T> SingleValuePalette<T> {
    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#<init>
    pub fn new(values: Vec<T>) -> Self {
        if !values.is_empty() {
            assert!(
                values.len() <= 1,
                "Can't initialize SingleValuePalette with {} values.",
                values.len()
            );
            let mut it = values.into_iter();
            Self { value: it.next() }
        } else {
            Self { value: None }
        }
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#create
    pub fn create(_bits: i32, values: Vec<T>) -> Self {
        Self::new(values)
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#getSize
    pub fn get_size(&self) -> i32 {
        1
    }
}

impl<T: PartialEq> SingleValuePalette<T> {
    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#idFor
    pub fn id_for<R: PaletteResize<T>>(&mut self, value: T, resize: &mut R) -> i32 {
        if let Some(ref current) = self.value {
            // Java uses reference inequality (!=); PartialEq is the closest safe Rust equivalent.
            if *current != value {
                return resize.on_resize(1, &value);
            }
        }
        self.value = Some(value);
        0
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#maybeHas
    pub fn maybe_has<F: Fn(&T) -> bool>(&self, predicate: F) -> bool {
        match self.value.as_ref() {
            Some(v) => predicate(v),
            None => panic!("Use of an uninitialized palette"),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#valueFor
    pub fn value_for(&self, id: i32) -> &T {
        if let Some(ref v) = self.value {
            if id == 0 {
                return v;
            }
        }
        panic!("Missing Palette entry for id {}.", id);
    }
}

impl<T: Clone> SingleValuePalette<T> {
    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#read
    pub fn read<M: IdMap<T>>(&mut self, buf: &mut FriendlyByteBuf, id_map: &M) {
        self.value = Some(id_map.by_id_or_throw(buf.read_var_int()));
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#write
    pub fn write<M: IdMap<T>>(&self, buf: &mut FriendlyByteBuf, id_map: &M) {
        let value = self
            .value
            .as_ref()
            .expect("Use of an uninitialized palette");
        buf.write_var_int(id_map.get_id(value));
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#getSerializedSize
    pub fn get_serialized_size<M: IdMap<T>>(&self, id_map: &M) -> i32 {
        let value = self
            .value
            .as_ref()
            .expect("Use of an uninitialized palette");
        var_int_byte_size(id_map.get_id(value))
    }

    // JAVA: net.minecraft.world.level.chunk.SingleValuePalette#copy
    pub fn copy(&self) -> Self {
        if self.value.is_none() {
            panic!("Use of an uninitialized palette");
        }
        // Java returns `this`; cloning preserves observable behavior in safe Rust.
        self.clone()
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
            assert_eq!(bits, 1);
            self.0
        }
    }

    #[test]
    fn new_and_value_for() {
        let mut p = SingleValuePalette::new(vec![Val(7)]);
        assert_eq!(p.get_size(), 1);
        assert_eq!(p.value_for(0), &Val(7));
        assert_eq!(p.id_for(Val(7), &mut NoResize), 0);
    }

    #[test]
    fn id_for_triggers_resize_on_different_value() {
        let mut p = SingleValuePalette::new(vec![Val(1)]);
        let mut r = ResizeTo(42);
        assert_eq!(p.id_for(Val(2), &mut r), 42);
        // Value should remain unchanged after resize.
        assert_eq!(p.value_for(0), &Val(1));
    }

    #[test]
    #[should_panic(expected = "Use of an uninitialized palette")]
    fn maybe_has_panics_when_uninitialized() {
        let p: SingleValuePalette<Val> = SingleValuePalette::new(vec![]);
        let _ = p.maybe_has(|_| true);
    }

    #[test]
    #[should_panic(expected = "Missing Palette entry for id 1")]
    fn value_for_invalid_id_panics() {
        let p = SingleValuePalette::new(vec![Val(5)]);
        let _ = p.value_for(1);
    }

    #[test]
    fn read_write_and_serialized_size_negative_varint() {
        let mut p: SingleValuePalette<Val> = SingleValuePalette::new(vec![]);
        let mut buf = FriendlyByteBuf::new(vec![]);
        buf.write_var_int(-1);
        buf.pos = 0;
        p.read(&mut buf, &TestIdMap);
        assert_eq!(p.value_for(0), &Val(-1));

        let mut out = FriendlyByteBuf::default();
        p.write(&mut out, &TestIdMap);
        assert_eq!(p.get_serialized_size(&TestIdMap), 5);
        let mut read_back = FriendlyByteBuf::new(out.data);
        assert_eq!(read_back.read_var_int(), -1);
    }

    #[test]
    fn copy_returns_initialized_clone() {
        let p = SingleValuePalette::new(vec![Val(9)]);
        let c = p.copy();
        assert_eq!(c.value_for(0), &Val(9));
    }
}
