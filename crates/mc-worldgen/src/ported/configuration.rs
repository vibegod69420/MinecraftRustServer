//! Port of `net.minecraft.world.level.chunk.Configuration`.

/// Minimal palette abstraction used by [`Configuration`].
pub trait Palette<T> {}

/// Strategy abstraction used by [`Configuration::create_palette`].
pub trait Strategy<T> {
    /// Returns the global palette for this strategy.
    fn global_palette(&self) -> Box<dyn Palette<T>>;
}

/// Palette factory abstraction used by [`Configuration::Simple`].
pub trait PaletteFactory {
    /// Creates a palette for the given bit width and initial values.
    fn create<T: 'static>(&self, bits: i32, values: &[T]) -> Box<dyn Palette<T>>;
}

/// Port of `net.minecraft.world.level.chunk.Configuration`.
#[derive(Debug)]
pub enum Configuration<F> {
    /// Port of `net.minecraft.world.level.chunk.Configuration$Global`.
    Global {
        bits_in_memory: i32,
        bits_in_storage: i32,
    },
    /// Port of `net.minecraft.world.level.chunk.Configuration$Simple`.
    Simple { factory: F, bits: i32 },
}

impl<F> Configuration<F> {
    // JAVA: net.minecraft.world.level.chunk.Configuration#alwaysRepack
    pub fn always_repack(&self) -> bool {
        match self {
            Self::Global { .. } => true,
            Self::Simple { .. } => false,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.Configuration#bitsInMemory
    pub fn bits_in_memory(&self) -> i32 {
        match self {
            Self::Global { bits_in_memory, .. } => *bits_in_memory,
            Self::Simple { bits, .. } => *bits,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.Configuration#bitsInStorage
    pub fn bits_in_storage(&self) -> i32 {
        match self {
            Self::Global { bits_in_storage, .. } => *bits_in_storage,
            Self::Simple { bits, .. } => *bits,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.Configuration#createPalette
    pub fn create_palette<T: 'static>(
        &self,
        strategy: &dyn Strategy<T>,
        values: &[T],
    ) -> Box<dyn Palette<T>>
    where
        F: PaletteFactory,
    {
        match self {
            Self::Global { .. } => strategy.global_palette(),
            Self::Simple { factory, bits } => factory.create(*bits, values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct EmptyPalette;
    impl<T> Palette<T> for EmptyPalette {}

    struct NoopFactory;
    impl PaletteFactory for NoopFactory {
        fn create<T: 'static>(&self, _bits: i32, _values: &[T]) -> Box<dyn Palette<T>> {
            panic!("factory should not be called");
        }
    }

    struct PanicStrategy;
    impl<T: 'static> Strategy<T> for PanicStrategy {
        fn global_palette(&self) -> Box<dyn Palette<T>> {
            panic!("strategy should not be called");
        }
    }

    struct TrackingStrategy {
        called: Cell<bool>,
    }

    impl TrackingStrategy {
        fn new() -> Self {
            Self {
                called: Cell::new(false),
            }
        }
    }

    impl<T: 'static> Strategy<T> for TrackingStrategy {
        fn global_palette(&self) -> Box<dyn Palette<T>> {
            self.called.set(true);
            Box::new(EmptyPalette)
        }
    }

    struct TrackingFactory {
        bits: Cell<i32>,
        values_len: Cell<usize>,
    }

    impl TrackingFactory {
        fn new() -> Self {
            Self {
                bits: Cell::new(0),
                values_len: Cell::new(0),
            }
        }
    }

    impl PaletteFactory for TrackingFactory {
        fn create<T: 'static>(&self, bits: i32, values: &[T]) -> Box<dyn Palette<T>> {
            self.bits.set(bits);
            self.values_len.set(values.len());
            Box::new(EmptyPalette)
        }
    }

    #[test]
    fn global_always_repacks_and_exposes_bits() {
        let cfg = Configuration::<NoopFactory>::Global {
            bits_in_memory: 8,
            bits_in_storage: 4,
        };
        assert!(cfg.always_repack());
        assert_eq!(cfg.bits_in_memory(), 8);
        assert_eq!(cfg.bits_in_storage(), 4);
    }

    #[test]
    fn simple_never_repacks_and_uses_same_bits() {
        let cfg = Configuration::Simple {
            factory: NoopFactory,
            bits: 5,
        };
        assert!(!cfg.always_repack());
        assert_eq!(cfg.bits_in_memory(), 5);
        assert_eq!(cfg.bits_in_storage(), 5);
    }

    #[test]
    fn global_create_palette_delegates_to_strategy() {
        let strategy = TrackingStrategy::new();
        let cfg = Configuration::<NoopFactory>::Global {
            bits_in_memory: 1,
            bits_in_storage: 2,
        };
        let _palette = cfg.create_palette(&strategy, &[10i32, 20, 30]);
        assert!(strategy.called.get());
    }

    #[test]
    fn simple_create_palette_delegates_to_factory() {
        let cfg = Configuration::Simple {
            factory: TrackingFactory::new(),
            bits: 6,
        };
        let values = [1i32, 2, 3, 4];
        let _palette = cfg.create_palette(&PanicStrategy, &values);
        match &cfg {
            Configuration::Simple { factory, .. } => {
                assert_eq!(factory.bits.get(), 6);
                assert_eq!(factory.values_len.get(), 4);
            }
            _ => panic!("expected Simple"),
        }
    }

    #[test]
    fn negative_and_extreme_bit_values_preserved() {
        let global = Configuration::<NoopFactory>::Global {
            bits_in_memory: i32::MIN,
            bits_in_storage: -1,
        };
        assert_eq!(global.bits_in_memory(), i32::MIN);
        assert_eq!(global.bits_in_storage(), -1);
        assert!(global.always_repack());

        let simple = Configuration::Simple {
            factory: TrackingFactory::new(),
            bits: i32::MAX,
        };
        assert_eq!(simple.bits_in_memory(), i32::MAX);
        assert_eq!(simple.bits_in_storage(), i32::MAX);
        assert!(!simple.always_repack());

        let _palette = simple.create_palette(&PanicStrategy, &[0i32]);
        match &simple {
            Configuration::Simple { factory, .. } => {
                assert_eq!(factory.bits.get(), i32::MAX);
                assert_eq!(factory.values_len.get(), 1);
            }
            _ => panic!("expected Simple"),
        }
    }
}
