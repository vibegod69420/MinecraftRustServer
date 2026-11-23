use std::marker::PhantomData;

/// Placeholder for unported `net.minecraft.server.level.ServerLevel`.
#[derive(Debug, Default, Clone)]
pub struct ServerLevel;

/// Placeholder for unported `net.minecraft.util.debug.DebugSubscription<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscription<T> {
    _marker: PhantomData<T>,
}

impl<T> DebugSubscription<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for DebugSubscription<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ValueGetter<T> {
    // JAVA: net.minecraft.util.debug.DebugValueSource$ValueGetter#get
    fn get(&self) -> Option<T>;
}

pub trait Registration {
    // JAVA: net.minecraft.util.debug.DebugValueSource$Registration#register
    fn register<T>(&mut self, subscription: DebugSubscription<T>, getter: &dyn ValueGetter<T>);
}

pub trait DebugValueSource {
    // JAVA: net.minecraft.util.debug.DebugValueSource#registerDebugValues
    fn register_debug_values<R: Registration>(&self, level: &ServerLevel, registration: &mut R);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstGetter<T: Clone>(Option<T>);

    impl<T: Clone> ValueGetter<T> for ConstGetter<T> {
        fn get(&self) -> Option<T> {
            self.0.clone()
        }
    }

    #[test]
    fn value_getter_negative_i32() {
        let g = ConstGetter(Some(-1_i32));
        assert_eq!(g.get(), Some(-1_i32));
    }

    #[test]
    fn value_getter_i64_overflow_edges() {
        let g_min = ConstGetter(Some(i64::MIN));
        let g_max = ConstGetter(Some(i64::MAX));
        assert_eq!(g_min.get(), Some(i64::MIN));
        assert_eq!(g_max.get(), Some(i64::MAX));
        let wrapped = i64::MAX.wrapping_add(1);
        let g_wrap = ConstGetter(Some(wrapped));
        assert_eq!(g_wrap.get(), Some(i64::MIN));
    }

    #[test]
    fn value_getter_float_bits_nan() {
        let bits = 0x7FC0_0001_u32;
        let v = f32::from_bits(bits);
        let g = ConstGetter(Some(v));
        assert_eq!(g.get().unwrap().to_bits(), bits);
    }

    #[test]
    fn value_getter_negative_zero_f64_bits() {
        let v = -0.0_f64;
        let g = ConstGetter(Some(v));
        let got = g.get().unwrap();
        assert_eq!(got.to_bits(), v.to_bits());
        assert_ne!(got.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn value_getter_none() {
        let g: ConstGetter<i32> = ConstGetter(None);
        assert_eq!(g.get(), None);
    }

    struct CountingRegistration {
        calls: i32,
        saw_none: bool,
    }

    impl Registration for CountingRegistration {
        fn register<T>(&mut self, _subscription: DebugSubscription<T>, getter: &dyn ValueGetter<T>) {
            if getter.get().is_none() {
                self.saw_none = true;
            }
            self.calls = self.calls.wrapping_add(1);
        }
    }

    struct TestSource;

    impl DebugValueSource for TestSource {
        fn register_debug_values<R: Registration>(&self, _level: &ServerLevel, registration: &mut R) {
            registration.register(DebugSubscription::<i32>::new(), &ConstGetter(Some(i32::MIN)));
            registration.register(
                DebugSubscription::<f64>::new(),
                &ConstGetter(Some(f64::from_bits(0xFFFF_FFFF_FFFF_FFFF_u64))),
            );
            registration.register(DebugSubscription::<i32>::new(), &ConstGetter::<i32>(None));
        }
    }

    #[test]
    fn debug_value_source_registers_values() {
        let src = TestSource;
        let level = ServerLevel;
        let mut reg = CountingRegistration {
            calls: 0,
            saw_none: false,
        };
        src.register_debug_values(&level, &mut reg);
        assert_eq!(reg.calls, 3);
        assert!(reg.saw_none);
    }
}
