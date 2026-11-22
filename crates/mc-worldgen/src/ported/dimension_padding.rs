//! Port of `net.minecraft.world.level.levelgen.structure.pools.DimensionPadding`.

use std::marker::PhantomData;

/// Placeholder for Mojang `Codec<T>` (codec infrastructure not yet ported).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Codec<T> {
    pub const fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Port of `DimensionPadding(int bottom, int top)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DimensionPadding {
    pub bottom: i32,
    pub top: i32,
}

#[allow(dead_code)]
const RECORD_CODEC: Codec<DimensionPadding> = Codec::placeholder();
pub const CODEC: Codec<DimensionPadding> = Codec::placeholder();

pub const ZERO: DimensionPadding = DimensionPadding { bottom: 0, top: 0 };

impl DimensionPadding {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.DimensionPadding#<init>
    pub const fn new(padding: i32) -> Self {
        Self {
            bottom: padding,
            top: padding,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.DimensionPadding#hasEqualTopAndBottom
    pub const fn has_equal_top_and_bottom(&self) -> bool {
        self.top == self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_constant() {
        assert_eq!(ZERO, DimensionPadding { bottom: 0, top: 0 });
        assert!(ZERO.has_equal_top_and_bottom());
    }

    #[test]
    fn symmetric_constructor_sets_both() {
        let p = DimensionPadding::new(5);
        assert_eq!(p.bottom, 5);
        assert_eq!(p.top, 5);
        assert!(p.has_equal_top_and_bottom());
    }

    #[test]
    fn different_top_and_bottom() {
        let p = DimensionPadding { bottom: 1, top: 2 };
        assert!(!p.has_equal_top_and_bottom());
        assert_eq!(p.bottom, 1);
        assert_eq!(p.top, 2);
    }

    #[test]
    fn negative_values_preserved() {
        let p = DimensionPadding { bottom: -7, top: -3 };
        assert_eq!(p.bottom, -7);
        assert_eq!(p.top, -3);
        assert!(!p.has_equal_top_and_bottom());

        let sym = DimensionPadding::new(-4);
        assert_eq!(sym.bottom, -4);
        assert_eq!(sym.top, -4);
        assert!(sym.has_equal_top_and_bottom());
    }

    #[test]
    fn extreme_i32_values() {
        let p = DimensionPadding {
            bottom: i32::MIN,
            top: i32::MAX,
        };
        assert_eq!(p.bottom, i32::MIN);
        assert_eq!(p.top, i32::MAX);
        assert!(!p.has_equal_top_and_bottom());

        let q = DimensionPadding::new(i32::MIN);
        assert!(q.has_equal_top_and_bottom());
    }
}
