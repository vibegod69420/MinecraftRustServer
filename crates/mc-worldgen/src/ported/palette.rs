/// Minimal self-contained placeholder for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    pub data: Vec<u8>,
}

/// Minimal self-contained placeholder for `net.minecraft.core.IdMap`.
pub trait IdMap<T> {
    fn get_id(&self, value: &T) -> Option<i32>;
    fn by_id(&self, id: i32) -> Option<&T>;
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.chunk.PaletteResize`.
pub trait PaletteResize<T> {
    // JAVA: net.minecraft.world.level.chunk.PaletteResize#onResize
    fn on_resize(&mut self, bits: i32, value: &T) -> i32;
}

/// Port of `net.minecraft.world.level.chunk.Palette`.
pub trait Palette<T> {
    // JAVA: net.minecraft.world.level.chunk.Palette#idFor
    fn id_for(&mut self, value: &T, resize: &mut dyn PaletteResize<T>) -> i32;

    // JAVA: net.minecraft.world.level.chunk.Palette#maybeHas
    fn maybe_has(&self, predicate: &dyn Fn(&T) -> bool) -> bool;

    // JAVA: net.minecraft.world.level.chunk.Palette#valueFor
    fn value_for(&self, id: i32) -> Option<&T>;

    // JAVA: net.minecraft.world.level.chunk.Palette#read
    fn read(&mut self, buf: &mut FriendlyByteBuf, id_map: &dyn IdMap<T>);

    // JAVA: net.minecraft.world.level.chunk.Palette#write
    fn write(&self, buf: &mut FriendlyByteBuf, id_map: &dyn IdMap<T>);

    // JAVA: net.minecraft.world.level.chunk.Palette#getSerializedSize
    fn get_serialized_size(&self, id_map: &dyn IdMap<T>) -> i32;

    // JAVA: net.minecraft.world.level.chunk.Palette#getSize
    fn get_size(&self) -> i32;

    // JAVA: net.minecraft.world.level.chunk.Palette#copy
    fn copy_palette(&self) -> Box<dyn Palette<T>>;
}

/// Port of `net.minecraft.world.level.chunk.Palette.Factory`.
pub trait Factory {
    // JAVA: net.minecraft.world.level.chunk.Palette$Factory#create
    fn create<A: Clone + 'static>(&self, bits: i32, values: &[A]) -> Box<dyn Palette<A>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default)]
    struct TestIdMap {
        values: Vec<i32>,
    }

    impl IdMap<i32> for TestIdMap {
        fn get_id(&self, value: &i32) -> Option<i32> {
            self.values.iter().position(|v| v == value).map(|i| i as i32)
        }

        fn by_id(&self, id: i32) -> Option<&i32> {
            if id < 0 {
                return None;
            }
            self.values.get(id as usize)
        }
    }

    struct NoResize;
    impl PaletteResize<i32> for NoResize {
        fn on_resize(&mut self, bits: i32, _value: &i32) -> i32 {
            bits
        }
    }

    #[derive(Clone, Debug)]
    struct SimplePalette {
        entries: Vec<i32>,
    }

    impl Palette<i32> for SimplePalette {
        fn id_for(&mut self, value: &i32, _resize: &mut dyn PaletteResize<i32>) -> i32 {
            if let Some(idx) = self.entries.iter().position(|v| v == value) {
                return idx as i32;
            }
            self.entries.push(*value);
            (self.entries.len() as i32).wrapping_sub(1)
        }

        fn maybe_has(&self, predicate: &dyn Fn(&i32) -> bool) -> bool {
            self.entries.iter().any(predicate)
        }

        fn value_for(&self, id: i32) -> Option<&i32> {
            if id < 0 {
                return None;
            }
            self.entries.get(id as usize)
        }

        fn read(&mut self, buf: &mut FriendlyByteBuf, _id_map: &dyn IdMap<i32>) {
            self.entries.clear();
            for chunk in buf.data.chunks_exact(4) {
                let val = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                self.entries.push(val);
            }
        }

        fn write(&self, buf: &mut FriendlyByteBuf, _id_map: &dyn IdMap<i32>) {
            for v in &self.entries {
                buf.data.extend_from_slice(&v.to_le_bytes());
            }
        }

        fn get_serialized_size(&self, _id_map: &dyn IdMap<i32>) -> i32 {
            (self.entries.len() as i32).wrapping_mul(4)
        }

        fn get_size(&self) -> i32 {
            self.entries.len() as i32
        }

        fn copy_palette(&self) -> Box<dyn Palette<i32>> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn id_for_and_value_for_round_trip() {
        let mut p = SimplePalette {
            entries: Vec::new(),
        };
        let mut r = NoResize;
        let id0 = p.id_for(&10, &mut r);
        let id1 = p.id_for(&-5, &mut r);
        let id0b = p.id_for(&10, &mut r);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id0b, 0);
        assert_eq!(p.value_for(0), Some(&10));
        assert_eq!(p.value_for(1), Some(&-5));
        assert_eq!(p.value_for(-1), None);
        assert_eq!(p.value_for(99), None);
    }

    #[test]
    fn maybe_has_with_negative_and_extreme_values() {
        let mut p = SimplePalette {
            entries: Vec::new(),
        };
        let mut r = NoResize;
        p.id_for(&i32::MIN, &mut r);
        p.id_for(&i32::MAX, &mut r);
        assert!(p.maybe_has(&|v| *v == i32::MIN));
        assert!(p.maybe_has(&|v| *v == i32::MAX));
        assert!(!p.maybe_has(&|v| *v == 0));
    }

    #[test]
    fn write_read_round_trip_preserves_int_bits() {
        let mut p = SimplePalette {
            entries: vec![1, -2, i32::MAX, i32::MIN],
        };
        let id_map = TestIdMap::default();
        let mut buf = FriendlyByteBuf::default();
        p.write(&mut buf, &id_map);
        assert_eq!(p.get_serialized_size(&id_map), 16);

        let mut p2 = SimplePalette {
            entries: Vec::new(),
        };
        p2.read(&mut buf, &id_map);
        assert_eq!(p2.entries, vec![1, -2, i32::MAX, i32::MIN]);
    }

    #[test]
    fn copy_palette_is_independent() {
        let mut p = SimplePalette { entries: vec![7] };
        let mut copy = p.copy_palette();
        let mut r = NoResize;
        p.id_for(&8, &mut r);
        assert_eq!(copy.get_size(), 1);
        assert_eq!(p.get_size(), 2);
        assert_eq!(copy.value_for(0), Some(&7));
    }

    #[test]
    fn factory_trait_is_implementable() {
        struct SimpleFactory;
        impl Factory for SimpleFactory {
            fn create<A: Clone + 'static>(&self, _bits: i32, values: &[A]) -> Box<dyn Palette<A>> {
                let _ = values;
                // Shape-only placeholder implementation for this self-contained module.
                panic!("not used in this test")
            }
        }

        let _factory = SimpleFactory;
    }
}
