use std::fmt;
use std::marker::PhantomData;

pub trait IdMap<T> {
    // JAVA: net.minecraft.core.IdMap#getId
    fn get_id(&self, value: &T) -> i32;
    // JAVA: net.minecraft.core.IdMap#byId
    fn by_id(&self, id: i32) -> Option<T>;
    // JAVA: net.minecraft.core.IdMap#size
    fn size(&self) -> i32;
}

pub trait PaletteResize<T> {
    // JAVA: net.minecraft.world.level.chunk.PaletteResize#onResize
    fn on_resize(&mut self, new_bits: i32, value: &T) -> i32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingPaletteEntryException {
    pub index: i32,
}

impl MissingPaletteEntryException {
    // JAVA: net.minecraft.world.level.chunk.MissingPaletteEntryException#<init>
    pub fn new(index: i32) -> Self {
        Self { index }
    }
}

impl fmt::Display for MissingPaletteEntryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing palette entry for index {}", self.index)
    }
}

impl std::error::Error for MissingPaletteEntryException {}

pub struct FriendlyByteBuf;

pub trait Palette<T> {
    // JAVA: net.minecraft.world.level.chunk.Palette#idFor
    fn id_for(&mut self, value: &T, resize: &mut dyn PaletteResize<T>) -> i32;
    // JAVA: net.minecraft.world.level.chunk.Palette#maybeHas
    fn maybe_has(&self, predicate: &dyn Fn(&T) -> bool) -> bool;
    // JAVA: net.minecraft.world.level.chunk.Palette#valueFor
    fn value_for(&self, id: i32) -> Result<T, MissingPaletteEntryException>;
    // JAVA: net.minecraft.world.level.chunk.Palette#read
    fn read(&mut self, buf: &mut FriendlyByteBuf, registry: &dyn IdMap<T>);
    // JAVA: net.minecraft.world.level.chunk.Palette#write
    fn write(&self, buf: &mut FriendlyByteBuf, registry: &dyn IdMap<T>);
    // JAVA: net.minecraft.world.level.chunk.Palette#getSerializedSize
    fn get_serialized_size(&self, registry: &dyn IdMap<T>) -> i32;
    // JAVA: net.minecraft.world.level.chunk.Palette#getSize
    fn get_size(&self) -> i32;
    // JAVA: net.minecraft.world.level.chunk.Palette#copy
    fn copy_palette(&self) -> Box<dyn Palette<T>>;
}

pub struct GlobalPalette<T, R: IdMap<T> + Clone + 'static> {
    registry: R,
    _marker: PhantomData<T>,
}

impl<T: 'static, R: IdMap<T> + Clone + 'static> GlobalPalette<T, R> {
    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#<init>
    pub fn new(registry: R) -> Self {
        Self {
            registry,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#idFor
    pub fn id_for_value(&self, value: &T) -> i32 {
        let id = self.registry.get_id(value);
        if id == -1 { 0 } else { id }
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#valueFor
    pub fn value_for_id(&self, id: i32) -> Result<T, MissingPaletteEntryException> {
        match self.registry.by_id(id) {
            Some(value) => Ok(value),
            None => Err(MissingPaletteEntryException::new(id)),
        }
    }
}

impl<T: 'static, R: IdMap<T> + Clone + 'static> Palette<T> for GlobalPalette<T, R> {
    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#idFor
    fn id_for(&mut self, value: &T, _resize: &mut dyn PaletteResize<T>) -> i32 {
        self.id_for_value(value)
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#maybeHas
    fn maybe_has(&self, _predicate: &dyn Fn(&T) -> bool) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#valueFor
    fn value_for(&self, id: i32) -> Result<T, MissingPaletteEntryException> {
        self.value_for_id(id)
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#read
    fn read(&mut self, _buf: &mut FriendlyByteBuf, _registry: &dyn IdMap<T>) {}

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#write
    fn write(&self, _buf: &mut FriendlyByteBuf, _registry: &dyn IdMap<T>) {}

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#getSerializedSize
    fn get_serialized_size(&self, _registry: &dyn IdMap<T>) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#getSize
    fn get_size(&self) -> i32 {
        self.registry.size()
    }

    // JAVA: net.minecraft.world.level.chunk.GlobalPalette#copy
    fn copy_palette(&self) -> Box<dyn Palette<T>> {
        Box::new(GlobalPalette {
            registry: self.registry.clone(),
            _marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestIdMap {
        values: Vec<i32>,
    }

    impl IdMap<i32> for TestIdMap {
        fn get_id(&self, value: &i32) -> i32 {
            self.values
                .iter()
                .position(|v| v == value)
                .map(|i| i as i32)
                .unwrap_or(-1)
        }

        fn by_id(&self, id: i32) -> Option<i32> {
            if id < 0 {
                return None;
            }
            self.values.get(id as usize).copied()
        }

        fn size(&self) -> i32 {
            self.values.len() as i32
        }
    }

    struct NoopResize;
    impl PaletteResize<i32> for NoopResize {
        fn on_resize(&mut self, _new_bits: i32, _value: &i32) -> i32 {
            -999
        }
    }

    #[test]
    fn id_for_returns_zero_for_missing() {
        let mut palette = GlobalPalette::new(TestIdMap { values: vec![10, 20, 30] });
        let mut resize = NoopResize;
        assert_eq!(palette.id_for(&20, &mut resize), 1);
        assert_eq!(palette.id_for(&999, &mut resize), 0);
        assert_eq!(palette.id_for(&-1, &mut resize), 0);
    }

    #[test]
    fn value_for_returns_value_or_error() {
        let palette = GlobalPalette::new(TestIdMap { values: vec![5, 6, 7] });
        assert_eq!(palette.value_for(0).unwrap(), 5);
        assert_eq!(palette.value_for(2).unwrap(), 7);
        let err = palette.value_for(3).unwrap_err();
        assert_eq!(err, MissingPaletteEntryException::new(3));
        let err_neg = palette.value_for(-1).unwrap_err();
        assert_eq!(err_neg.index, -1);
    }

    #[test]
    fn maybe_has_is_always_true() {
        let palette = GlobalPalette::new(TestIdMap { values: vec![1] });
        assert!(palette.maybe_has(&|_| false));
        assert!(palette.maybe_has(&|v: &i32| *v == 42));
    }

    #[test]
    fn serialized_size_and_size_and_io_noop() {
        let mut palette = GlobalPalette::new(TestIdMap { values: vec![1, 2, 3, 4] });
        let registry = TestIdMap { values: vec![1, 2, 3, 4] };
        assert_eq!(palette.get_size(), 4);
        assert_eq!(palette.get_serialized_size(&registry), 0);
        let mut buf = FriendlyByteBuf;
        palette.read(&mut buf, &registry);
        palette.write(&mut buf, &registry);
        assert_eq!(palette.get_size(), 4);
    }

    #[test]
    fn copy_returns_functional_palette() {
        let palette = GlobalPalette::new(TestIdMap { values: vec![100, 200] });
        let copied = palette.copy_palette();
        assert_eq!(copied.get_size(), 2);
        assert_eq!(copied.value_for(1).unwrap(), 200);
        assert!(copied.maybe_has(&|_| false));
    }

    #[test]
    fn empty_registry_edge_cases() {
        let mut palette = GlobalPalette::new(TestIdMap { values: vec![] });
        let mut resize = NoopResize;
        assert_eq!(palette.get_size(), 0);
        assert_eq!(palette.id_for(&0, &mut resize), 0);
        assert!(palette.value_for(0).is_err());
        assert!(palette.value_for(i32::MIN).is_err());
    }
}
