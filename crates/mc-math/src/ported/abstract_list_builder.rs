use std::fmt;

// JAVA: com.mojang.serialization.Lifecycle
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
}

fn combine_lifecycle(a: Lifecycle, b: Lifecycle) -> Lifecycle {
    if a == Lifecycle::Experimental || b == Lifecycle::Experimental {
        Lifecycle::Experimental
    } else {
        Lifecycle::Stable
    }
}

fn merge_errors(a: Option<String>, b: Option<String>) -> Option<String> {
    match (a, b) {
        (Some(x), Some(y)) => Some(format!("{x}; {y}")),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

// JAVA: com.mojang.serialization.DataResult
#[derive(Clone)]
pub struct DataResult<T> {
    value: Option<T>,
    error: Option<String>,
    lifecycle: Lifecycle,
}

impl<T: fmt::Debug> fmt::Debug for DataResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataResult")
            .field("value", &self.value)
            .field("error", &self.error)
            .field("lifecycle", &self.lifecycle)
            .finish()
    }
}

impl<T> DataResult<T> {
    // JAVA: com.mojang.serialization.DataResult#success
    pub fn success(value: T, lifecycle: Lifecycle) -> Self {
        Self {
            value: Some(value),
            error: None,
            lifecycle,
        }
    }

    // JAVA: com.mojang.serialization.DataResult#error
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            value: None,
            error: Some(message.into()),
            lifecycle: Lifecycle::Stable,
        }
    }

    // JAVA: com.mojang.serialization.DataResult#map
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> DataResult<U> {
        match self.value {
            Some(v) => DataResult {
                value: Some(f(v)),
                error: self.error,
                lifecycle: self.lifecycle,
            },
            None => DataResult {
                value: None,
                error: self.error,
                lifecycle: self.lifecycle,
            },
        }
    }

    // JAVA: com.mojang.serialization.DataResult#flatMap
    pub fn flat_map<U, F: FnOnce(T) -> DataResult<U>>(self, f: F) -> DataResult<U> {
        match self.value {
            Some(v) => {
                let r = f(v);
                DataResult {
                    value: r.value,
                    error: merge_errors(self.error, r.error),
                    lifecycle: combine_lifecycle(self.lifecycle, r.lifecycle),
                }
            }
            None => DataResult {
                value: None,
                error: self.error,
                lifecycle: self.lifecycle,
            },
        }
    }

    // JAVA: com.mojang.serialization.DataResult#apply2stable
    pub fn apply2stable<U, R, F: FnOnce(T, U) -> R>(
        self,
        second: DataResult<U>,
        f: F,
    ) -> DataResult<R> {
        let lifecycle = Lifecycle::Stable;
        let err = merge_errors(self.error, second.error);
        let val = match (self.value, second.value) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        };
        DataResult {
            value: val,
            error: err,
            lifecycle,
        }
    }

    // JAVA: com.mojang.serialization.DataResult#mapError
    pub fn map_error<F: FnOnce(String) -> String>(mut self, f: F) -> Self {
        if let Some(e) = self.error.take() {
            self.error = Some(f(e));
        }
        self
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.value.is_some()
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn into_result(self) -> Result<T, String> {
        match (self.value, self.error) {
            (Some(v), None) => Ok(v),
            (_, Some(e)) => Err(e),
            (None, None) => Err("empty DataResult".to_string()),
        }
    }
}

// JAVA: com.mojang.serialization.DynamicOps
pub trait DynamicOps<T> {
    fn empty(&self) -> T;
}

// JAVA: com.mojang.serialization.ListBuilder
pub trait ListBuilder<T> {
    fn ops(&self) -> &dyn DynamicOps<T>;
    fn add(&mut self, value: T) -> &mut Self;
    fn add_result(&mut self, value: DataResult<T>) -> &mut Self;
    fn with_errors_from<U>(&mut self, other: DataResult<U>) -> &mut Self;
    fn map_error_boxed(&mut self, f: Box<dyn FnOnce(String) -> String>) -> &mut Self;
    fn build(&mut self, prefix: T) -> DataResult<T>;
}

// JAVA: net.minecraft.util.AbstractListBuilder
pub struct AbstractListBuilder<T, B> {
    ops: Box<dyn DynamicOps<T>>,
    builder: Option<DataResult<B>>,
    init: Box<dyn Fn() -> B>,
    append_fn: Box<dyn Fn(B, T) -> B>,
    build_fn: Box<dyn Fn(B, T) -> DataResult<T>>,
}

impl<T, B> AbstractListBuilder<T, B> {
    // JAVA: net.minecraft.util.AbstractListBuilder#<init>
    pub fn new(
        ops: Box<dyn DynamicOps<T>>,
        init: Box<dyn Fn() -> B>,
        append_fn: Box<dyn Fn(B, T) -> B>,
        build_fn: Box<dyn Fn(B, T) -> DataResult<T>>,
    ) -> Self {
        let builder = DataResult::success(init(), Lifecycle::Stable);
        Self {
            ops,
            builder: Some(builder),
            init,
            append_fn,
            build_fn,
        }
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#ops
    pub fn ops(&self) -> &dyn DynamicOps<T> {
        &*self.ops
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#initBuilder
    fn init_builder(&self) -> B {
        (self.init)()
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#append
    fn append(&self, builder: B, value: T) -> B {
        (self.append_fn)(builder, value)
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#build(B,T)
    fn build_value(&self, builder: B, prefix: T) -> DataResult<T> {
        (self.build_fn)(builder, prefix)
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#add(T)
    pub fn add(&mut self, value: T) -> &mut Self {
        let b = self.builder.take().expect("builder present");
        self.builder = Some(b.map(|builder| (self.append_fn)(builder, value)));
        self
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#add(DataResult)
    pub fn add_result(&mut self, value: DataResult<T>) -> &mut Self {
        let b = self.builder.take().expect("builder present");
        self.builder = Some(b.apply2stable(value, |builder, v| (self.append_fn)(builder, v)));
        self
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#withErrorsFrom
    pub fn with_errors_from<U>(&mut self, other: DataResult<U>) -> &mut Self {
        let b = self.builder.take().expect("builder present");
        self.builder = Some(b.flat_map(|builder| other.map(|_| builder)));
        self
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#mapError
    pub fn map_error<F: FnOnce(String) -> String + 'static>(&mut self, f: F) -> &mut Self {
        let b = self.builder.take().expect("builder present");
        self.builder = Some(b.map_error(f));
        self
    }

    // JAVA: net.minecraft.util.AbstractListBuilder#build(T)
    pub fn build(&mut self, prefix: T) -> DataResult<T> {
        let b = self.builder.take().expect("builder present");
        let result = b.flat_map(|builder| (self.build_fn)(builder, prefix));
        self.builder = Some(DataResult::success(self.init_builder(), Lifecycle::Stable));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestVal {
        List(Vec<i32>),
        Empty,
    }

    struct TestOps;

    impl DynamicOps<TestVal> for TestOps {
        fn empty(&self) -> TestVal {
            TestVal::Empty
        }
    }

    fn make_builder() -> AbstractListBuilder<TestVal, Vec<i32>> {
        AbstractListBuilder::new(
            Box::new(TestOps),
            Box::new(Vec::<i32>::new),
            Box::new(|mut b: Vec<i32>, v: TestVal| {
                if let TestVal::List(items) = v {
                    b.extend(items);
                } else if let TestVal::Empty = v {
                    // no-op
                }
                b
            }),
            Box::new(|b: Vec<i32>, _prefix: TestVal| {
                DataResult::success(TestVal::List(b), Lifecycle::Stable)
            }),
        )
    }

    fn make_int_builder() -> AbstractListBuilder<i32, Vec<i32>> {
        struct IntOps;
        impl DynamicOps<i32> for IntOps {
            fn empty(&self) -> i32 {
                0
            }
        }
        AbstractListBuilder::new(
            Box::new(IntOps),
            Box::new(Vec::<i32>::new),
            Box::new(|mut b: Vec<i32>, v: i32| {
                b.push(v);
                b
            }),
            Box::new(|b: Vec<i32>, _prefix: i32| {
                let sum: i32 = b.iter().copied().sum();
                DataResult::success(sum, Lifecycle::Stable)
            }),
        )
    }

    #[test]
    fn add_and_build_success() {
        let mut builder = make_int_builder();
        builder.add(1).add(2).add(3);
        let result = builder.build(0);
        assert!(result.is_success());
        assert_eq!(result.value(), Some(&6));
    }

    #[test]
    fn add_result_error_propagates() {
        let mut builder = make_int_builder();
        builder.add(1);
        builder.add_result(DataResult::error("bad element"));
        builder.add(2);
        let result = builder.build(0);
        assert!(result.is_error());
        assert_eq!(result.error_message(), Some("bad element"));
    }

    #[test]
    fn build_resets_builder() {
        let mut builder = make_int_builder();
        builder.add(10).add(20);
        let first = builder.build(0);
        assert_eq!(first.value(), Some(&30));

        builder.add(5);
        let second = builder.build(0);
        assert_eq!(second.value(), Some(&5));
    }

    #[test]
    fn with_errors_from_propagates_error() {
        let mut builder = make_int_builder();
        builder.add(1);
        builder.with_errors_from(DataResult::<()>::error("external error"));
        let result = builder.build(0);
        assert!(result.is_error());
        assert!(result.error_message().unwrap().contains("external error"));
    }

    #[test]
    fn map_error_transforms_message() {
        let mut builder = make_int_builder();
        builder.add_result(DataResult::error("original"));
        builder.map_error(|e| format!("wrapped: {e}"));
        let result = builder.build(0);
        assert!(result.is_error());
        assert_eq!(result.error_message(), Some("wrapped: original"));
    }

    #[test]
    fn ops_and_empty_builder() {
        let builder = make_builder();
        assert_eq!(builder.ops().empty(), TestVal::Empty);
        let mut builder = make_builder();
        let result = builder.build(TestVal::Empty);
        assert!(result.is_success());
        assert_eq!(result.value(), Some(&TestVal::List(vec![])));
    }
}
