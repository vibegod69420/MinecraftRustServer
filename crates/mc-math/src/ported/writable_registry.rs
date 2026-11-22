//! Faithful port of `net.minecraft.core.WritableRegistry`.

use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.resources.ResourceKey`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKey<T> {
    _marker: PhantomData<T>,
    pub location: String,
}

impl<T> ResourceKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            _marker: PhantomData,
            location: location.into(),
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.tags.TagKey`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagKey<T> {
    _marker: PhantomData<T>,
    pub location: String,
}

impl<T> TagKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            _marker: PhantomData,
            location: location.into(),
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.server.packs.repository.KnownPack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack;

/// Minimal self-contained stub for `com.mojang.serialization.Lifecycle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
}

impl Lifecycle {
    pub fn stable() -> Self {
        Self::Stable
    }
}

/// Minimal self-contained stub for `net.minecraft.core.RegistrationInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationInfo {
    pub known_pack_info: Option<KnownPack>,
    pub lifecycle: Lifecycle,
}

/// Minimal self-contained stub for `net.minecraft.core.Holder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    _marker: PhantomData<T>,
}

/// Minimal self-contained stub for `net.minecraft.core.Holder.Reference`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReference<T> {
    _marker: PhantomData<T>,
    pub key: ResourceKey<T>,
}

/// Minimal self-contained stub for `net.minecraft.core.HolderGetter`.
pub trait HolderGetter<T> {}

/// Minimal self-contained stub for `net.minecraft.core.Registry`.
pub trait Registry<T> {}

// JAVA: net.minecraft.core.WritableRegistry
pub trait WritableRegistry<T>: Registry<T> {
    // JAVA: net.minecraft.core.WritableRegistry#register
    fn register(
        &mut self,
        key: ResourceKey<T>,
        value: T,
        info: RegistrationInfo,
    ) -> HolderReference<T>;

    // JAVA: net.minecraft.core.WritableRegistry#bindTag
    fn bind_tag(&mut self, key: TagKey<T>, values: Vec<Holder<T>>);

    // JAVA: net.minecraft.core.WritableRegistry#isEmpty
    fn is_empty(&self) -> bool;

    // JAVA: net.minecraft.core.WritableRegistry#createRegistrationLookup
    fn create_registration_lookup(&self) -> Box<dyn HolderGetter<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLookup;
    impl HolderGetter<i32> for DummyLookup {}

    struct DummyRegistry {
        empty: bool,
        registered: usize,
        bound_tags: usize,
    }

    impl Registry<i32> for DummyRegistry {}

    impl WritableRegistry<i32> for DummyRegistry {
        fn register(
            &mut self,
            key: ResourceKey<i32>,
            _value: i32,
            _info: RegistrationInfo,
        ) -> HolderReference<i32> {
            self.registered += 1;
            self.empty = false;
            HolderReference {
                _marker: PhantomData,
                key,
            }
        }

        fn bind_tag(&mut self, _key: TagKey<i32>, values: Vec<Holder<i32>>) {
            self.bound_tags += values.len();
        }

        fn is_empty(&self) -> bool {
            self.empty
        }

        fn create_registration_lookup(&self) -> Box<dyn HolderGetter<i32>> {
            Box::new(DummyLookup)
        }
    }

    #[test]
    fn empty_registry_reports_empty() {
        let reg = DummyRegistry {
            empty: true,
            registered: 0,
            bound_tags: 0,
        };
        assert!(reg.is_empty());
    }

    #[test]
    fn register_makes_registry_non_empty() {
        let mut reg = DummyRegistry {
            empty: true,
            registered: 0,
            bound_tags: 0,
        };
        let key = ResourceKey::new("test:item");
        let info = RegistrationInfo {
            known_pack_info: None,
            lifecycle: Lifecycle::stable(),
        };
        let holder = reg.register(key.clone(), 42, info);
        assert!(!reg.is_empty());
        assert_eq!(reg.registered, 1);
        assert_eq!(holder.key, key);
    }

    #[test]
    fn bind_tag_records_values() {
        let mut reg = DummyRegistry {
            empty: true,
            registered: 0,
            bound_tags: 0,
        };
        let tag = TagKey::new("test:tag");
        reg.bind_tag(
            tag,
            vec![Holder { _marker: PhantomData }, Holder { _marker: PhantomData }],
        );
        assert_eq!(reg.bound_tags, 2);
    }

    #[test]
    fn create_registration_lookup_returns_lookup() {
        let reg = DummyRegistry {
            empty: true,
            registered: 0,
            bound_tags: 0,
        };
        let _lookup: Box<dyn HolderGetter<i32>> = reg.create_registration_lookup();
    }
}
