use std::fmt;
use std::marker::PhantomData;

pub trait PaletteResize<T> {
    // JAVA: net.minecraft.world.level.chunk.PaletteResize#onResize
    fn on_resize(&mut self, var1: i32, var2: &T) -> i32;
}

#[derive(Debug)]
pub struct NoResizeExpected<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> NoResizeExpected<T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for NoResizeExpected<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> PaletteResize<T> for NoResizeExpected<T> {
    // JAVA: net.minecraft.world.level.chunk.PaletteResize#onResize
    fn on_resize(&mut self, var1: i32, var2: &T) -> i32 {
        panic!(
            "Unexpected palette resize, bits = {}, added value = {:?}",
            var1, var2
        );
    }
}

// JAVA: net.minecraft.world.level.chunk.PaletteResize#noResizeExpected
pub fn no_resize_expected<T: fmt::Debug>() -> NoResizeExpected<T> {
    NoResizeExpected::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddOneResize;

    impl PaletteResize<i32> for AddOneResize {
        fn on_resize(&mut self, var1: i32, _var2: &i32) -> i32 {
            var1.wrapping_add(1)
        }
    }

    #[test]
    fn custom_resize_returns_value() {
        let mut r = AddOneResize;
        assert_eq!(r.on_resize(4, &10), 5);
        assert_eq!(r.on_resize(-1, &0), 0);
    }

    #[test]
    fn custom_resize_wrapping() {
        let mut r = AddOneResize;
        assert_eq!(r.on_resize(i32::MAX, &0), i32::MIN);
    }

    #[test]
    #[should_panic(expected = "Unexpected palette resize, bits = 5, added value = 42")]
    fn no_resize_expected_panics() {
        let mut r = no_resize_expected::<i32>();
        r.on_resize(5, &42);
    }

    #[test]
    #[should_panic(expected = "Unexpected palette resize, bits = -1, added value = \"test\"")]
    fn no_resize_expected_panics_negative_bits() {
        let mut r = no_resize_expected::<&str>();
        r.on_resize(-1, &"test");
    }
}
