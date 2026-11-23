use std::any::Any;
use std::marker::PhantomData;

/// Minimal stand-in for `net.minecraft.util.context.ContextKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey {
    name: String,
}

impl ContextKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Minimal stand-in for `net.minecraft.util.StringRepresentable`.
pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &str;
}

/// Minimal stand-in for a target enum value that exposes a context parameter.
pub trait ContextParamHolder {
    fn context_param(&self) -> &ContextKey;
}

/// Minimal stand-in for `net.minecraft.world.level.storage.loot.LootContext`.
pub trait LootContextAccess {
    fn get_optional_parameter(&self, key: &ContextKey) -> Option<&dyn Any>;
}

/// Port of `net.minecraft.world.level.storage.loot.LootContextArg<R>`.
pub trait LootContextArg<R> {
    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#get
    fn get(&self, context: &dyn LootContextAccess) -> Option<R>;

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#contextParam
    fn context_param(&self) -> &ContextKey;
}

struct CastArg<R, U> {
    inner: Box<dyn LootContextArg<R>>,
    _marker: PhantomData<fn(R) -> U>,
}

impl<R: 'static, U: 'static> LootContextArg<U> for CastArg<R, U>
where
    R: Into<U>,
{
    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#get
    fn get(&self, context: &dyn LootContextAccess) -> Option<U> {
        self.inner.get(context).map(Into::into)
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#contextParam
    fn context_param(&self) -> &ContextKey {
        self.inner.context_param()
    }
}

// JAVA: net.minecraft.world.level.storage.loot.LootContextArg#cast
pub fn cast<R: 'static, U: 'static>(arg: Box<dyn LootContextArg<R>>) -> Box<dyn LootContextArg<U>>
where
    R: Into<U>,
{
    Box::new(CastArg {
        inner: arg,
        _marker: PhantomData,
    })
}

/// Minimal stand-in for the codec produced by `ArgCodecBuilder.build()`.
pub struct ArgCodec<R> {
    sources: Vec<(String, Box<dyn LootContextArg<R>>)>,
}

impl<R> ArgCodec<R> {
    pub fn get_by_name(&self, name: &str) -> Option<&dyn LootContextArg<R>> {
        self.sources
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, arg)| arg.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.sources.iter().map(|(n, _)| n.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder<R>`.
pub struct ArgCodecBuilder<R> {
    sources: Vec<(String, Box<dyn LootContextArg<R>>)>,
}

impl<R: 'static> ArgCodecBuilder<R> {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyOf
    pub fn any_of<T, FN, FA>(&mut self, values: &[T], name_fn: FN, arg_fn: FA) -> &mut Self
    where
        FN: Fn(&T) -> String,
        FA: Fn(&T) -> Box<dyn LootContextArg<R>>,
    {
        for v in values {
            self.sources.push((name_fn(v), arg_fn(v)));
        }
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyOf
    pub fn any_of_representable<T: StringRepresentable, FA>(
        &mut self,
        values: &[T],
        arg_fn: FA,
    ) -> &mut Self
    where
        FA: Fn(&T) -> Box<dyn LootContextArg<R>>,
    {
        for v in values {
            self.sources
                .push((v.get_serialized_name().to_string(), arg_fn(v)));
        }
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyOf
    pub fn any_of_self<T, S>(&mut self, values: &[T]) -> &mut Self
    where
        T: StringRepresentable + LootContextArg<S> + Clone + 'static,
        S: Into<R> + 'static,
    {
        for v in values {
            self.sources.push((
                v.get_serialized_name().to_string(),
                cast::<S, R>(Box::new(v.clone())),
            ));
        }
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyEntity
    pub fn any_entity<T, FA>(&mut self, values: &[T], arg_fn: FA) -> &mut Self
    where
        T: StringRepresentable + ContextParamHolder,
        FA: Fn(&ContextKey) -> Box<dyn LootContextArg<R>>,
    {
        self.any_of_representable(values, |v| arg_fn(v.context_param()))
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyBlockEntity
    pub fn any_block_entity<T, FA>(&mut self, values: &[T], arg_fn: FA) -> &mut Self
    where
        T: StringRepresentable + ContextParamHolder,
        FA: Fn(&ContextKey) -> Box<dyn LootContextArg<R>>,
    {
        self.any_of_representable(values, |v| arg_fn(v.context_param()))
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#anyItemStack
    pub fn any_item_stack<T, FA>(&mut self, values: &[T], arg_fn: FA) -> &mut Self
    where
        T: StringRepresentable + ContextParamHolder,
        FA: Fn(&ContextKey) -> Box<dyn LootContextArg<R>>,
    {
        self.any_of_representable(values, |v| arg_fn(v.context_param()))
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$ArgCodecBuilder#build
    pub fn build(self) -> ArgCodec<R> {
        ArgCodec {
            sources: self.sources,
        }
    }
}

impl<R: 'static> Default for ArgCodecBuilder<R> {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.world.level.storage.loot.LootContextArg#createArgCodec
pub fn create_arg_codec<R: 'static, F>(f: F) -> ArgCodec<R>
where
    F: FnOnce(&mut ArgCodecBuilder<R>) -> &mut ArgCodecBuilder<R>,
{
    let mut builder = ArgCodecBuilder::new();
    f(&mut builder);
    builder.build()
}

/// Port of `net.minecraft.world.level.storage.loot.LootContextArg$Getter<T, R>`.
pub trait Getter<T: 'static, R>: LootContextArg<R> {
    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg$Getter#get
    fn get_from(&self, value: &T) -> Option<R>;
}

// JAVA: net.minecraft.world.level.storage.loot.LootContextArg$Getter#get
pub fn getter_get_from_context<T: 'static, R, G: Getter<T, R> + ?Sized>(
    getter: &G,
    context: &dyn LootContextAccess,
) -> Option<R> {
    let value = context.get_optional_parameter(getter.context_param())?;
    let typed = value.downcast_ref::<T>()?;
    getter.get_from(typed)
}

/// Port of `net.minecraft.world.level.storage.loot.LootContextArg$SimpleGetter<T>`.
pub trait SimpleGetter<T: Clone + 'static>: LootContextArg<T> {}

// JAVA: net.minecraft.world.level.storage.loot.LootContextArg$SimpleGetter#get
pub fn simple_getter_get_from_context<T: Clone + 'static, G: SimpleGetter<T> + ?Sized>(
    getter: &G,
    context: &dyn LootContextAccess,
) -> Option<T> {
    context
        .get_optional_parameter(getter.context_param())?
        .downcast_ref::<T>()
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestContext {
        params: HashMap<String, Box<dyn Any>>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                params: HashMap::new(),
            }
        }

        fn with_param<T: 'static>(mut self, key: &ContextKey, value: T) -> Self {
            self.params.insert(key.name().to_string(), Box::new(value));
            self
        }
    }

    impl LootContextAccess for TestContext {
        fn get_optional_parameter(&self, key: &ContextKey) -> Option<&dyn Any> {
            self.params.get(key.name()).map(|v| v.as_ref())
        }
    }

    #[test]
    fn builder_any_of_and_lookup() {
        #[derive(Clone)]
        struct ConstArg {
            key: ContextKey,
            val: i32,
        }
        impl LootContextArg<i32> for ConstArg {
            fn get(&self, _context: &dyn LootContextAccess) -> Option<i32> {
                Some(self.val)
            }
            fn context_param(&self) -> &ContextKey {
                &self.key
            }
        }

        let key = ContextKey::new("test");
        let values = [1i32, 2, 3];
        let mut builder = ArgCodecBuilder::<i32>::new();
        builder.any_of(
            &values,
            |v| format!("n{}", v),
            |v| {
                Box::new(ConstArg {
                    key: key.clone(),
                    val: *v,
                })
            },
        );
        let codec = builder.build();
        assert_eq!(codec.len(), 3);
        assert!(!codec.is_empty());
        assert_eq!(codec.names(), vec!["n1", "n2", "n3"]);
        let ctx = TestContext::new();
        assert_eq!(codec.get_by_name("n2").unwrap().get(&ctx), Some(2));
        assert!(codec.get_by_name("missing").is_none());
    }

    #[test]
    fn create_arg_codec_builds() {
        #[derive(Clone)]
        struct DummyArg(ContextKey);
        impl LootContextArg<i32> for DummyArg {
            fn get(&self, _: &dyn LootContextAccess) -> Option<i32> {
                Some(42)
            }
            fn context_param(&self) -> &ContextKey {
                &self.0
            }
        }

        let codec = create_arg_codec::<i32, _>(|b| {
            b.any_of(
                &["a"],
                |s| s.to_string(),
                |_| Box::new(DummyArg(ContextKey::new("k"))),
            )
        });
        assert_eq!(codec.len(), 1);
        assert_eq!(
            codec.get_by_name("a").unwrap().get(&TestContext::new()),
            Some(42)
        );
    }

    #[test]
    fn getter_default_extracts_from_context() {
        struct MyGetter {
            key: ContextKey,
        }
        impl LootContextArg<i64> for MyGetter {
            fn get(&self, context: &dyn LootContextAccess) -> Option<i64> {
                getter_get_from_context(self, context)
            }
            fn context_param(&self) -> &ContextKey {
                &self.key
            }
        }
        impl Getter<String, i64> for MyGetter {
            fn get_from(&self, value: &String) -> Option<i64> {
                value.parse().ok()
            }
        }

        let key = ContextKey::new("s");
        let g = MyGetter { key: key.clone() };
        let ctx = TestContext::new().with_param(&key, String::from("123"));
        assert_eq!(g.get(&ctx), Some(123i64));

        let ctx_bad = TestContext::new().with_param(&key, String::from("nope"));
        assert_eq!(g.get(&ctx_bad), None);

        let ctx_missing = TestContext::new();
        assert_eq!(g.get(&ctx_missing), None);

        let ctx_wrong_type = TestContext::new().with_param(&key, 999i32);
        assert_eq!(g.get(&ctx_wrong_type), None);
    }

    #[test]
    fn simple_getter_returns_cloned_value() {
        struct MySimple {
            key: ContextKey,
        }
        impl LootContextArg<String> for MySimple {
            fn get(&self, context: &dyn LootContextAccess) -> Option<String> {
                simple_getter_get_from_context(self, context)
            }
            fn context_param(&self) -> &ContextKey {
                &self.key
            }
        }
        impl SimpleGetter<String> for MySimple {}

        let key = ContextKey::new("name");
        let g = MySimple { key: key.clone() };
        let ctx = TestContext::new().with_param(&key, String::from("hello"));
        assert_eq!(g.get(&ctx), Some(String::from("hello")));
        assert_eq!(g.get(&TestContext::new()), None);
    }

    #[test]
    fn cast_converts_arg_output() {
        #[derive(Clone)]
        struct I32Arg(ContextKey);
        impl LootContextArg<i32> for I32Arg {
            fn get(&self, _: &dyn LootContextAccess) -> Option<i32> {
                Some(7)
            }
            fn context_param(&self) -> &ContextKey {
                &self.0
            }
        }

        let key = ContextKey::new("k");
        let arg: Box<dyn LootContextArg<i32>> = Box::new(I32Arg(key.clone()));
        let casted: Box<dyn LootContextArg<i64>> = cast(arg);
        assert_eq!(casted.get(&TestContext::new()), Some(7i64));
        assert_eq!(casted.context_param(), &key);
    }

    #[test]
    fn any_of_representable_entity_block_item_and_self() {
        #[derive(Clone)]
        struct Target {
            name: &'static str,
            key: ContextKey,
        }
        impl StringRepresentable for Target {
            fn get_serialized_name(&self) -> &str {
                self.name
            }
        }
        impl ContextParamHolder for Target {
            fn context_param(&self) -> &ContextKey {
                &self.key
            }
        }

        #[derive(Clone)]
        struct KeyEchoArg(ContextKey);
        impl LootContextArg<String> for KeyEchoArg {
            fn get(&self, _: &dyn LootContextAccess) -> Option<String> {
                Some(self.0.name().to_string())
            }
            fn context_param(&self) -> &ContextKey {
                &self.0
            }
        }

        let targets = [
            Target {
                name: "this",
                key: ContextKey::new("this"),
            },
            Target {
                name: "killer",
                key: ContextKey::new("killer"),
            },
        ];

        let mut builder = ArgCodecBuilder::<String>::new();
        builder.any_entity(&targets, |k| Box::new(KeyEchoArg(k.clone())));
        let codec = builder.build();
        assert_eq!(codec.names(), vec!["this", "killer"]);
        let ctx = TestContext::new();
        assert_eq!(
            codec.get_by_name("killer").unwrap().get(&ctx),
            Some("killer".to_string())
        );

        let mut b2 = ArgCodecBuilder::<String>::new();
        b2.any_block_entity(&targets, |k| Box::new(KeyEchoArg(k.clone())));
        assert_eq!(b2.build().len(), 2);

        let mut b3 = ArgCodecBuilder::<String>::new();
        b3.any_item_stack(&targets, |k| Box::new(KeyEchoArg(k.clone())));
        assert_eq!(b3.build().len(), 2);

        #[derive(Clone)]
        struct SelfArg {
            name: &'static str,
            key: ContextKey,
        }
        impl StringRepresentable for SelfArg {
            fn get_serialized_name(&self) -> &str {
                self.name
            }
        }
        impl LootContextArg<i32> for SelfArg {
            fn get(&self, _: &dyn LootContextAccess) -> Option<i32> {
                Some(1)
            }
            fn context_param(&self) -> &ContextKey {
                &self.key
            }
        }

        let vals = [
            SelfArg {
                name: "a",
                key: ContextKey::new("ka"),
            },
            SelfArg {
                name: "b",
                key: ContextKey::new("kb"),
            },
        ];
        let mut b4 = ArgCodecBuilder::<i32>::new();
        b4.any_of_self(&vals);
        let codec4 = b4.build();
        assert_eq!(codec4.names(), vec!["a", "b"]);
        assert_eq!(codec4.get_by_name("a").unwrap().context_param().name(), "ka");
    }
}
