use std::fmt;

pub struct FriendlyByteBuf;

pub trait Strategy<T> {
    // JAVA: net.minecraft.world.level.chunk.Strategy#<placeholder>
    fn strategy_id(&self) -> i32 {
        0
    }
}

pub trait CountConsumer<T> {
    // JAVA: net.minecraft.world.level.chunk.PalettedContainer$CountConsumer#accept
    fn accept(&mut self, value: &T, count: i32);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedData<T> {
    pub palette_entries: Vec<T>,
    pub storage: Option<Vec<i64>>,
    pub bits_per_entry: i32,
}

impl<T> PackedData<T> {
    pub const UNKNOWN_BITS_PER_ENTRY: i32 = -1;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO$PackedData#<init>
    pub fn new(palette_entries: Vec<T>, storage: Option<Vec<i64>>) -> Self {
        Self {
            palette_entries,
            storage,
            bits_per_entry: Self::UNKNOWN_BITS_PER_ENTRY,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO$PackedData#<init>
    pub fn with_bits(palette_entries: Vec<T>, storage: Option<Vec<i64>>, bits_per_entry: i32) -> Self {
        Self {
            palette_entries,
            storage,
            bits_per_entry,
        }
    }
}

pub trait Unpacker<T, C> {
    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO$Unpacker#read
    fn read(&self, strategy: &dyn Strategy<T>, data: PackedData<T>) -> Result<C, String>;
}

pub trait PalettedContainerRO<T> {
    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#get
    fn get(&self, x: i32, y: i32, z: i32) -> &T;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#getAll
    fn get_all(&self, consumer: &mut dyn FnMut(&T));

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#write
    fn write(&self, buf: &mut FriendlyByteBuf);

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#getSerializedSize
    fn get_serialized_size(&self) -> i32;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#bitsPerEntry
    fn bits_per_entry(&self) -> i32;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#maybeHas
    fn maybe_has(&self, predicate: &dyn Fn(&T) -> bool) -> bool;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#count
    fn count(&self, consumer: &mut dyn CountConsumer<T>);

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#copy
    fn copy_container(&self) -> Box<dyn PalettedContainerRO<T>>;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#recreate
    fn recreate(&self) -> Box<dyn PalettedContainerRO<T>>;

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerRO#pack
    fn pack(&self, strategy: &dyn Strategy<T>) -> PackedData<T>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleDataResultError(pub String);

impl fmt::Display for SimpleDataResultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleDataResultError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct DummyStrategy;
    impl Strategy<i32> for DummyStrategy {}

    struct MockContainer {
        data: Vec<i32>,
    }

    impl PalettedContainerRO<i32> for MockContainer {
        fn get(&self, x: i32, y: i32, z: i32) -> &i32 {
            let idx = ((y * 4 + z) * 4 + x) as usize;
            &self.data[idx]
        }
        fn get_all(&self, consumer: &mut dyn FnMut(&i32)) {
            for v in &self.data {
                consumer(v);
            }
        }
        fn write(&self, _buf: &mut FriendlyByteBuf) {}
        fn get_serialized_size(&self) -> i32 {
            self.data.len() as i32
        }
        fn bits_per_entry(&self) -> i32 {
            4
        }
        fn maybe_has(&self, predicate: &dyn Fn(&i32) -> bool) -> bool {
            self.data.iter().any(predicate)
        }
        fn count(&self, consumer: &mut dyn CountConsumer<i32>) {
            if let Some(v) = self.data.first() {
                consumer.accept(v, self.data.len() as i32);
            }
        }
        fn copy_container(&self) -> Box<dyn PalettedContainerRO<i32>> {
            Box::new(MockContainer {
                data: self.data.clone(),
            })
        }
        fn recreate(&self) -> Box<dyn PalettedContainerRO<i32>> {
            Box::new(MockContainer {
                data: vec![0; self.data.len()],
            })
        }
        fn pack(&self, _strategy: &dyn Strategy<i32>) -> PackedData<i32> {
            PackedData::with_bits(vec![0, 1], Some(vec![0i64; 2]), 4)
        }
    }

    struct CollectConsumer(RefCell<Vec<(i32, i32)>>);
    impl CountConsumer<i32> for CollectConsumer {
        fn accept(&mut self, value: &i32, count: i32) {
            self.0.borrow_mut().push((*value, count));
        }
    }

    struct MockUnpacker;
    impl Unpacker<i32, Vec<i32>> for MockUnpacker {
        fn read(&self, _strategy: &dyn Strategy<i32>, data: PackedData<i32>) -> Result<Vec<i32>, String> {
            if data.bits_per_entry < 0 && data.storage.is_none() {
                return Err("missing storage".to_string());
            }
            Ok(data.palette_entries)
        }
    }

    #[test]
    fn packed_data_constructors_and_constant() {
        assert_eq!(PackedData::<i32>::UNKNOWN_BITS_PER_ENTRY, -1);
        let a = PackedData::new(vec![1, 2], None);
        assert_eq!(a.palette_entries, vec![1, 2]);
        assert_eq!(a.storage, None);
        assert_eq!(a.bits_per_entry, -1);

        let b = PackedData::with_bits(vec![3], Some(vec![7i64, -1i64]), 5);
        assert_eq!(b.bits_per_entry, 5);
        assert_eq!(b.storage.as_ref().unwrap()[1], -1i64);
    }

    #[test]
    fn packed_data_negative_and_extreme_bits() {
        let p = PackedData::with_bits(vec![0u8], Some(vec![i64::MIN, i64::MAX]), i32::MIN);
        assert_eq!(p.bits_per_entry, i32::MIN);
        assert_eq!(p.storage.as_ref().unwrap()[0], i64::MIN);
        assert_eq!(p.storage.as_ref().unwrap()[1], i64::MAX);
    }

    #[test]
    fn paletted_container_ro_trait_mock() {
        let data: Vec<i32> = (0..64).collect();
        let c = MockContainer { data: data.clone() };
        assert_eq!(*c.get(1, 0, 0), 1);
        assert_eq!(*c.get(3, 3, 3), 63);
        assert_eq!(c.get_serialized_size(), 64);
        assert_eq!(c.bits_per_entry(), 4);
        assert!(c.maybe_has(&|v: &i32| *v == 63));
        assert!(!c.maybe_has(&|v: &i32| *v < 0));
    }

    #[test]
    fn get_all_count_copy_recreate_pack() {
        let c = MockContainer { data: vec![7; 64] };
        let mut all = Vec::new();
        c.get_all(&mut |v| all.push(*v));
        assert_eq!(all.len(), 64);
        assert!(all.iter().all(|v| *v == 7));

        let mut consumer = CollectConsumer(RefCell::new(Vec::new()));
        c.count(&mut consumer);
        assert_eq!(consumer.0.borrow().as_slice(), &[(7, 64)]);

        let copy = c.copy_container();
        assert_eq!(*copy.get(0, 0, 0), 7);
        let recreated = c.recreate();
        assert_eq!(*recreated.get(0, 0, 0), 0);

        let packed = c.pack(&DummyStrategy);
        assert_eq!(packed.bits_per_entry, 4);
        assert_eq!(packed.palette_entries, vec![0, 1]);
    }

    #[test]
    fn unpacker_reads_or_errors() {
        let unpacker = MockUnpacker;
        let ok = unpacker
            .read(&DummyStrategy, PackedData::with_bits(vec![1, 2], None, 1))
            .unwrap();
        assert_eq!(ok, vec![1, 2]);
        let err = unpacker
            .read(&DummyStrategy, PackedData::new(vec![9], None))
            .unwrap_err();
        assert_eq!(err, "missing storage");
    }
}
