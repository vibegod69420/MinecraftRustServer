use std::collections::HashMap;

pub const PRE_BUILT_MAP_THRESHOLD: usize = 16;

pub trait StringRepresentable: Clone {
    // JAVA: net.minecraft.util.StringRepresentable#getSerializedName
    fn get_serialized_name(&self) -> &str;
}

// JAVA: net.minecraft.util.StringRepresentable#createNameLookup
pub fn create_name_lookup<T: StringRepresentable + Send + Sync + 'static>(
    values: &[T],
) -> Box<dyn Fn(&str) -> Option<T> + Send + Sync> {
    create_name_lookup_with(values, |v| v.get_serialized_name().to_owned())
}

// JAVA: net.minecraft.util.StringRepresentable#createNameLookup
pub fn create_name_lookup_with<T, F>(
    values: &[T],
    name_fn: F,
) -> Box<dyn Fn(&str) -> Option<T> + Send + Sync>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T) -> String + Send + Sync + 'static,
{
    if values.len() > PRE_BUILT_MAP_THRESHOLD {
        let map: HashMap<String, T> = values.iter().map(|v| (name_fn(v), v.clone())).collect();
        Box::new(move |name: &str| map.get(name).cloned())
    } else {
        let entries: Vec<(String, T)> = values.iter().map(|v| (name_fn(v), v.clone())).collect();
        Box::new(move |name: &str| {
            for (n, v) in &entries {
                if n == name {
                    return Some(v.clone());
                }
            }
            None
        })
    }
}

// JAVA: net.minecraft.util.StringRepresentable#fromEnum
pub fn from_enum<E>(
    supplier: impl Fn() -> Vec<E>,
) -> EnumCodec<E>
where
    E: StringRepresentable + Send + Sync + 'static,
{
    from_enum_with_mapping(supplier, |s: &str| s.to_owned())
}

// JAVA: net.minecraft.util.StringRepresentable#fromEnumWithMapping
pub fn from_enum_with_mapping<E, M>(
    supplier: impl Fn() -> Vec<E>,
    mapping: M,
) -> EnumCodec<E>
where
    E: StringRepresentable + Send + Sync + 'static,
    M: Fn(&str) -> String + Send + Sync + 'static,
{
    let values = supplier();
    let resolver = create_name_lookup_with(&values, move |v: &E| mapping(v.get_serialized_name()));
    EnumCodec::new(values, resolver)
}

// JAVA: net.minecraft.util.StringRepresentable#fromValues
pub fn from_values<T>(
    supplier: impl Fn() -> Vec<T>,
) -> StringRepresentableCodec<T>
where
    T: StringRepresentable + Send + Sync + 'static,
{
    let values = supplier();
    let resolver = create_name_lookup(&values);
    let values_for_id = values.clone();
    let to_id = Box::new(move |v: &T| -> i32 {
        let name = v.get_serialized_name();
        for (i, item) in values_for_id.iter().enumerate() {
            if item.get_serialized_name() == name {
                return i as i32;
            }
        }
        -1_i32
    });
    StringRepresentableCodec::new(values, resolver, to_id)
}

// JAVA: net.minecraft.util.StringRepresentable#keys
pub fn keys<S: StringRepresentable>(values: &[S]) -> Vec<String> {
    values
        .iter()
        .map(|v| v.get_serialized_name().to_owned())
        .collect()
}

pub struct StringRepresentableCodec<S> {
    values: Vec<S>,
    resolver: Box<dyn Fn(&str) -> Option<S> + Send + Sync>,
    to_id: Box<dyn Fn(&S) -> i32 + Send + Sync>,
}

impl<S: Clone + Send + Sync + 'static> StringRepresentableCodec<S> {
    // JAVA: net.minecraft.util.StringRepresentable$StringRepresentableCodec#<init>
    pub fn new(
        values: Vec<S>,
        resolver: Box<dyn Fn(&str) -> Option<S> + Send + Sync>,
        to_id: Box<dyn Fn(&S) -> i32 + Send + Sync>,
    ) -> Self {
        Self {
            values,
            resolver,
            to_id,
        }
    }

    pub fn by_name(&self, name: &str) -> Option<S> {
        (self.resolver)(name)
    }

    pub fn to_id(&self, value: &S) -> i32 {
        (self.to_id)(value)
    }

    pub fn by_id(&self, id: i32) -> Option<S> {
        if id >= 0 {
            let idx = id as usize;
            if idx < self.values.len() {
                return Some(self.values[idx].clone());
            }
        }
        None
    }

    pub fn values(&self) -> &[S] {
        &self.values
    }
}

pub struct EnumCodec<E> {
    base: StringRepresentableCodec<E>,
    resolver: Box<dyn Fn(&str) -> Option<E> + Send + Sync>,
}

impl<E: StringRepresentable + Send + Sync + 'static> EnumCodec<E> {
    // JAVA: net.minecraft.util.StringRepresentable$EnumCodec#<init>
    pub fn new(values: Vec<E>, resolver: Box<dyn Fn(&str) -> Option<E> + Send + Sync>) -> Self {
        let values_for_id = values.clone();
        let to_id = Box::new(move |v: &E| -> i32 {
            let name = v.get_serialized_name();
            for (i, item) in values_for_id.iter().enumerate() {
                if item.get_serialized_name() == name {
                    return i as i32;
                }
            }
            -1_i32
        });
        let base_resolver = create_name_lookup(&values);
        let base = StringRepresentableCodec::new(values, base_resolver, to_id);
        Self { base, resolver }
    }

    // JAVA: net.minecraft.util.StringRepresentable$EnumCodec#byName
    pub fn by_name(&self, name: &str) -> Option<E> {
        (self.resolver)(name)
    }

    // JAVA: net.minecraft.util.StringRepresentable$EnumCodec#byName
    pub fn by_name_or_default(&self, name: &str, default: E) -> E {
        match self.by_name(name) {
            Some(v) => v,
            None => default,
        }
    }

    // JAVA: net.minecraft.util.StringRepresentable$EnumCodec#byName
    pub fn by_name_or_else<F>(&self, name: &str, supplier: F) -> E
    where
        F: FnOnce() -> E,
    {
        match self.by_name(name) {
            Some(v) => v,
            None => supplier(),
        }
    }

    pub fn base(&self) -> &StringRepresentableCodec<E> {
        &self.base
    }

    pub fn values(&self) -> &[E] {
        self.base.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestEnum {
        Alpha,
        Beta,
        Gamma,
    }

    impl StringRepresentable for TestEnum {
        // JAVA: net.minecraft.util.StringRepresentable#getSerializedName
        fn get_serialized_name(&self) -> &str {
            match self {
                TestEnum::Alpha => "alpha",
                TestEnum::Beta => "beta",
                TestEnum::Gamma => "gamma",
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Named(String);

    impl StringRepresentable for Named {
        // JAVA: net.minecraft.util.StringRepresentable#getSerializedName
        fn get_serialized_name(&self) -> &str {
            &self.0
        }
    }

    #[test]
    fn create_name_lookup_finds_and_misses() {
        let values = vec![TestEnum::Alpha, TestEnum::Beta, TestEnum::Gamma];
        let lookup = create_name_lookup(&values);
        assert_eq!(lookup("alpha"), Some(TestEnum::Alpha));
        assert_eq!(lookup("gamma"), Some(TestEnum::Gamma));
        assert_eq!(lookup("missing"), None);
        assert_eq!(lookup(""), None);
    }

    #[test]
    fn name_lookup_threshold_boundary_paths() {
        let small: Vec<Named> = (0..PRE_BUILT_MAP_THRESHOLD)
            .map(|i| Named(format!("v{i}")))
            .collect();
        let lookup_small = create_name_lookup(&small);
        assert_eq!(lookup_small("v0"), Some(Named("v0".to_owned())));
        assert_eq!(
            lookup_small(&format!("v{}", PRE_BUILT_MAP_THRESHOLD - 1)),
            Some(Named(format!("v{}", PRE_BUILT_MAP_THRESHOLD - 1)))
        );
        assert_eq!(lookup_small("missing"), None);

        let large: Vec<Named> = (0..(PRE_BUILT_MAP_THRESHOLD + 1))
            .map(|i| Named(format!("v{i}")))
            .collect();
        let lookup_large = create_name_lookup(&large);
        assert_eq!(lookup_large("v0"), Some(Named("v0".to_owned())));
        assert_eq!(
            lookup_large(&format!("v{}", PRE_BUILT_MAP_THRESHOLD)),
            Some(Named(format!("v{}", PRE_BUILT_MAP_THRESHOLD)))
        );
        assert_eq!(lookup_large("missing"), None);
    }

    #[test]
    fn enum_codec_by_name_variants() {
        let codec = from_enum(|| vec![TestEnum::Alpha, TestEnum::Beta, TestEnum::Gamma]);
        assert_eq!(codec.by_name("beta"), Some(TestEnum::Beta));
        assert_eq!(codec.by_name("none"), None);
        assert_eq!(
            codec.by_name_or_default("none", TestEnum::Gamma),
            TestEnum::Gamma
        );
        assert_eq!(
            codec.by_name_or_else("none", || TestEnum::Alpha),
            TestEnum::Alpha
        );
        assert_eq!(
            codec.by_name_or_else("alpha", || panic!("supplier should not be called")),
            TestEnum::Alpha
        );
    }

    #[test]
    fn from_enum_with_mapping_and_keys() {
        let codec = from_enum_with_mapping(
            || vec![TestEnum::Alpha, TestEnum::Beta],
            |s: &str| s.to_uppercase(),
        );
        assert_eq!(codec.by_name("ALPHA"), Some(TestEnum::Alpha));
        assert_eq!(codec.by_name("alpha"), None);

        let k = keys(&[TestEnum::Alpha, TestEnum::Beta, TestEnum::Gamma]);
        assert_eq!(k, vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()]);
    }

    #[test]
    fn from_values_id_lookup_negative_and_bounds() {
        let codec = from_values(|| vec![TestEnum::Alpha, TestEnum::Beta, TestEnum::Gamma]);
        assert_eq!(codec.to_id(&TestEnum::Alpha), 0_i32);
        assert_eq!(codec.to_id(&TestEnum::Gamma), 2_i32);
        assert_eq!(codec.by_id(1_i32), Some(TestEnum::Beta));
        assert_eq!(codec.by_id(-1_i32), None);
        assert_eq!(codec.by_id(i32::MIN), None);
        assert_eq!(codec.by_id(i32::MAX), None);
        assert_eq!(codec.by_id(3_i32), None);

        let manual = StringRepresentableCodec::new(
            vec![TestEnum::Alpha],
            create_name_lookup(&[TestEnum::Alpha]),
            Box::new(|_: &TestEnum| -1_i32),
        );
        assert_eq!(manual.to_id(&TestEnum::Alpha), -1_i32);
    }

    #[test]
    fn empty_and_overflow_safe_id_indexing() {
        let empty: Vec<TestEnum> = Vec::new();
        let lookup = create_name_lookup(&empty);
        assert_eq!(lookup("anything"), None);

        let codec = from_values(|| vec![TestEnum::Alpha, TestEnum::Beta]);
        // Large positive id should not panic or wrap into a valid index.
        let large_id = i32::MAX;
        assert_eq!(codec.by_id(large_id), None);
        let neg_id = -123_i32;
        assert_eq!(codec.by_id(neg_id), None);
        // Verify ordinal-like ids are stable.
        assert_eq!(codec.to_id(&TestEnum::Alpha).wrapping_add(1), codec.to_id(&TestEnum::Beta));
    }
}
