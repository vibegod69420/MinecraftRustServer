#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuation {
    Continue,
    Abort,
}

impl Continuation {
    // JAVA: net.minecraft.util.AbortableIterationConsumer.Continuation#shouldAbort
    pub fn should_abort(self) -> bool {
        self == Continuation::Abort
    }
}

pub trait AbortableIterationConsumer<T> {
    // JAVA: net.minecraft.util.AbortableIterationConsumer#accept
    fn accept(&mut self, value: T) -> Continuation;
}

impl<T, F> AbortableIterationConsumer<T> for F
where
    F: FnMut(T) -> Continuation,
{
    // JAVA: net.minecraft.util.AbortableIterationConsumer#accept
    fn accept(&mut self, value: T) -> Continuation {
        self(value)
    }
}

// JAVA: net.minecraft.util.AbortableIterationConsumer#forConsumer
pub fn for_consumer<T, F>(mut consumer: F) -> impl AbortableIterationConsumer<T>
where
    F: FnMut(T),
{
    move |value: T| {
        consumer(value);
        Continuation::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn continuation_should_abort() {
        assert!(!Continuation::Continue.should_abort());
        assert!(Continuation::Abort.should_abort());
    }

    #[test]
    fn for_consumer_always_continues() {
        let seen = Cell::new(0i32);
        let mut c = for_consumer(|v: i32| {
            seen.set(v);
        });
        assert_eq!(seen.get(), 0);
        assert_eq!(c.accept(-1), Continuation::Continue);
        assert_eq!(seen.get(), -1);
        assert_eq!(c.accept(i32::MIN), Continuation::Continue);
        assert_eq!(seen.get(), i32::MIN);
        assert_eq!(c.accept(i32::MAX), Continuation::Continue);
        assert_eq!(seen.get(), i32::MAX);
    }

    #[test]
    fn accept_can_abort_iteration() {
        let mut c = |v: i32| {
            if v < 0 {
                Continuation::Abort
            } else {
                Continuation::Continue
            }
        };
        assert_eq!(c.accept(0), Continuation::Continue);
        assert_eq!(c.accept(i32::MAX), Continuation::Continue);
        assert_eq!(c.accept(-1), Continuation::Abort);
        assert_eq!(c.accept(i32::MIN), Continuation::Abort);
    }

    #[test]
    fn for_consumer_preserves_float_bits() {
        let bits = Cell::new(0u32);
        let mut c = for_consumer(|v: f32| {
            bits.set(v.to_bits());
        });
        let values = [f32::NAN, f32::NEG_INFINITY, -0.0f32, f32::MIN_POSITIVE, -1.5f32];
        for &v in &values {
            assert_eq!(c.accept(v), Continuation::Continue);
            assert_eq!(bits.get(), v.to_bits());
        }
    }

    #[test]
    fn for_consumer_preserves_u64_overflow_values() {
        let seen = Cell::new(0u64);
        let mut c = for_consumer(|v: u64| {
            seen.set(v);
        });
        let values = [0u64, 1u64, u64::MAX, u64::MAX.wrapping_add(1), u64::MAX.wrapping_mul(2)];
        for &v in &values {
            assert_eq!(c.accept(v), Continuation::Continue);
            assert_eq!(seen.get(), v);
        }
    }
}
