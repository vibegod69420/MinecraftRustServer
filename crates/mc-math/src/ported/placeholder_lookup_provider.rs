// === FILE: placeholder_lookup_provider.rs ===
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;

/// Marker type used as the erased generic parameter, corresponding to Java's `Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Erased;

/// Minimal stub for `net.minecraft.core.Registry<T>`.
pub struct Registry<T: ?Sized = Erased>(PhantomData<fn() -> T>);

/// Minimal stub for `net.minecraft.resources.ResourceKey<T>`.
#[derive(Clone, Debug)]
pub struct ResourceKey<T: ?Sized = Erased> {
    registry: String,
    location: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ResourceKey<T> {
    pub fn new(registry: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            location: location.into(),
            _marker: PhantomData,
        }
    }

    pub fn registry(&self) -> &str {
        &self.registry
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn erase(&self) -> ResourceKey<Erased> {
        ResourceKey {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }

    pub fn cast<U: ?Sized>(&self) -> ResourceKey<U> {
        ResourceKey {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.location == other.location
    }
}

impl<T: ?Sized> Eq for ResourceKey<T> {}

impl<T: ?Sized> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.location.hash(state);
    }
}

/// Minimal stub for `net.minecraft.tags.TagKey<T>`.
#[derive(Clone, Debug)]
pub struct TagKey<T: ?Sized = Erased> {
    registry: String,
    location: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> TagKey<T> {
    pub fn new(registry: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            location: location.into(),
            _marker: PhantomData,
        }
    }

    pub fn registry(&self) -> &str {
        &self.registry
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn erase(&self) -> TagKey<Erased> {
        TagKey {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }

    pub fn cast<U: ?Sized>(&self) -> TagKey<U> {
        TagKey {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.location == other.location
    }
}

impl<T: ?Sized> Eq for TagKey<T> {}

impl<T: ?Sized> Hash for TagKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.location.hash(state);
    }
}

/// Minimal stub for `net.minecraft.core.HolderOwner<T>`.
pub trait HolderOwner<T: ?Sized> {}

/// Minimal stub for `net.minecraft.core.Holder.Reference<T>`.
#[derive(Clone, Debug)]
pub struct HolderReference<T: ?Sized = Erased> {
    key: ResourceKey<T>,
}

impl<T: ?Sized> HolderReference<T> {
    pub fn create_stand_alone(_owner: &dyn HolderOwner<T>, key: ResourceKey<T>) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &ResourceKey<T> {
        &self.key
    }

    pub fn erase(&self) -> HolderReference<Erased> {
        HolderReference {
            key: self.key.erase(),
        }
    }

    pub fn cast<U: ?Sized>(&self) -> HolderReference<U> {
        HolderReference {
            key: self.key.cast(),
        }
    }
}

impl<T: ?Sized> PartialEq for HolderReference<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: ?Sized> Eq for HolderReference<T> {}

/// Minimal stub for `net.minecraft.core.HolderSet.Named<T>`.
#[derive(Clone, Debug)]
pub struct HolderSetNamed<T: ?Sized = Erased> {
    key: TagKey<T>,
}

impl<T: ?Sized> HolderSetNamed<T> {
    pub fn empty_named(_owner: &dyn HolderOwner<T>, key: TagKey<T>) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &TagKey<T> {
        &self.key
    }

    pub fn erase(&self) -> HolderSetNamed<Erased> {
        HolderSetNamed {
            key: self.key.erase(),
        }
    }

    pub fn cast<U: ?Sized>(&self) -> HolderSetNamed<U> {
        HolderSetNamed {
            key: self.key.cast(),
        }
    }
}

impl<T: ?Sized> PartialEq for HolderSetNamed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: ?Sized> Eq for HolderSetNamed<T> {}

/// Minimal stub for `com.mojang.serialization.Lifecycle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
}

impl Lifecycle {
    pub fn experimental() -> Self {
        Self::Experimental
    }
}

struct PlaceholderState {
    holders: HashMap<ResourceKey<Erased>, HolderReference<Erased>>,
    holder_sets: HashMap<TagKey<Erased>, HolderSetNamed<Erased>>,
}

impl PlaceholderState {
    fn new() -> Self {
        Self {
            holders: HashMap::new(),
            holder_sets: HashMap::new(),
        }
    }
}

/// Port of `PlaceholderLookupProvider.UniversalLookup`.
/// Implements `HolderGetter<Object>` and `HolderOwner<Object>` behavior.
#[derive(Clone)]
pub struct UniversalLookup {
    state: Rc<RefCell<PlaceholderState>>,
}

impl UniversalLookup {
    fn new(state: Rc<RefCell<PlaceholderState>>) -> Self {
        Self { state }
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#get(ResourceKey)
    pub fn get_resource<T: ?Sized>(&self, key: &ResourceKey<T>) -> Option<HolderReference<T>> {
        Some(self.get_or_create_resource(key))
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#getOrThrow(ResourceKey)
    pub fn get_or_throw_resource<T: ?Sized>(&self, key: &ResourceKey<T>) -> HolderReference<T> {
        self.get_or_create_resource(key)
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#getOrCreate(ResourceKey)
    fn get_or_create_resource<T: ?Sized>(&self, key: &ResourceKey<T>) -> HolderReference<T> {
        let erased_key = key.erase();
        let mut state = self.state.borrow_mut();
        let entry = state
            .holders
            .entry(erased_key.clone())
            .or_insert_with(|| {
                HolderReference::create_stand_alone(self as &dyn HolderOwner<Erased>, erased_key)
            });
        entry.cast()
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#get(TagKey)
    pub fn get_tag<T: ?Sized>(&self, key: &TagKey<T>) -> Option<HolderSetNamed<T>> {
        Some(self.get_or_create_tag(key))
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#getOrThrow(TagKey)
    pub fn get_or_throw_tag<T: ?Sized>(&self, key: &TagKey<T>) -> HolderSetNamed<T> {
        self.get_or_create_tag(key)
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#getOrCreate(TagKey)
    fn get_or_create_tag<T: ?Sized>(&self, key: &TagKey<T>) -> HolderSetNamed<T> {
        let erased_key = key.erase();
        let mut state = self.state.borrow_mut();
        let entry = state
            .holder_sets
            .entry(erased_key.clone())
            .or_insert_with(|| {
                HolderSetNamed::empty_named(self as &dyn HolderOwner<Erased>, erased_key)
            });
        entry.cast()
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#castAsLookup
    pub fn cast_as_lookup<T: ?Sized>(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider.UniversalLookup#castAsOwner
    pub fn cast_as_owner<T: ?Sized>(&self) -> Self {
        self.clone()
    }
}

impl<T: ?Sized> HolderOwner<T> for UniversalLookup {}

/// Minimal stub for `net.minecraft.resources.RegistryOps.RegistryInfo<T>`.
pub struct RegistryInfo<T: ?Sized = Erased> {
    pub owner: UniversalLookup,
    pub getter: UniversalLookup,
    pub lifecycle: Lifecycle,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> RegistryInfo<T> {
    pub fn new(owner: UniversalLookup, getter: UniversalLookup, lifecycle: Lifecycle) -> Self {
        Self {
            owner,
            getter,
            lifecycle,
            _marker: PhantomData,
        }
    }

    pub fn erase(&self) -> RegistryInfo<Erased> {
        RegistryInfo {
            owner: self.owner.clone(),
            getter: self.getter.clone(),
            lifecycle: self.lifecycle,
            _marker: PhantomData,
        }
    }
}

/// Minimal context trait for looking up registry info, used by placeholder fallback.
pub trait ContextRegistryLookup {
    fn lookup_registry_info(
        &self,
        registry_key: &ResourceKey<Erased>,
    ) -> Option<RegistryInfo<Erased>>;
}

/// Minimal stub for `net.minecraft.resources.RegistryOps.RegistryInfoLookup`.
pub trait RegistryInfoLookup {
    fn lookup_erased(&self, registry_key: &ResourceKey<Erased>) -> Option<RegistryInfo<Erased>>;
}

/// Minimal stub for `com.mojang.serialization.DynamicOps<T>`.
pub trait DynamicOps<V: ?Sized> {}

/// Minimal stub for `net.minecraft.resources.RegistryOps<V>`.
pub struct RegistryOps<V: 'static> {
    _delegate: Box<dyn DynamicOps<V>>,
    lookup: Box<dyn RegistryInfoLookup>,
}

impl<V: 'static> RegistryOps<V> {
    pub fn create(delegate: Box<dyn DynamicOps<V>>, lookup: Box<dyn RegistryInfoLookup>) -> Self {
        Self {
            _delegate: delegate,
            lookup,
        }
    }

    pub fn lookup_erased(&self, registry_key: &ResourceKey<Erased>) -> Option<RegistryInfo<Erased>> {
        self.lookup.lookup_erased(registry_key)
    }
}

impl<V: 'static> DynamicOps<V> for RegistryOps<V> {}

/// Minimal stub for `com.mojang.serialization.DataResult<T>`.
#[derive(Debug)]
pub struct DataResult<T>(Result<T, String>);

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        Self(Ok(value))
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self(Err(msg.into()))
    }

    pub fn flat_map<U, F: FnOnce(T) -> DataResult<U>>(self, f: F) -> DataResult<U> {
        match self.0 {
            Ok(v) => f(v),
            Err(e) => DataResult(Err(e)),
        }
    }

    pub fn result(self) -> Result<T, String> {
        self.0
    }

    pub fn is_success(&self) -> bool {
        self.0.is_ok()
    }

    pub fn is_error(&self) -> bool {
        self.0.is_err()
    }
}

/// Minimal stub for JavaOps value type.
#[derive(Clone, Debug)]
pub struct JavaValue;

/// Minimal stub for `com.mojang.serialization.JavaOps`.
#[derive(Debug, Clone, Copy)]
pub struct JavaOps;

impl JavaOps {
    pub const INSTANCE: JavaOps = JavaOps;
}

impl DynamicOps<JavaValue> for JavaOps {}

/// Minimal stub for `com.mojang.serialization.Codec<T>`.
pub trait Codec<T> {
    fn encode_start<V: 'static>(
        &self,
        ops: &dyn DynamicOps<V>,
        input: &T,
    ) -> DataResult<V>;

    fn parse<V: 'static>(
        &self,
        ops: &dyn DynamicOps<V>,
        input: &V,
    ) -> DataResult<T>;
}

/// Minimal stub for `net.minecraft.core.HolderLookup.Provider` capability used by swapper.
pub trait HolderLookupProviderLike {
    fn create_serialization_context<V: 'static>(&self, ops: Box<dyn DynamicOps<V>>) -> RegistryOps<V>;
}

/// Minimal stub for `net.minecraft.resources.RegistryContextSwapper`.
pub trait RegistryContextSwapper {
    fn swap_to<T, Co: Codec<T>, Ctx: HolderLookupProviderLike>(
        &self,
        codec: &Co,
        value: &T,
        target: &Ctx,
    ) -> DataResult<T>;
}

struct PlaceholderRegistryInfoLookup<C: ContextRegistryLookup> {
    context: Rc<C>,
    lookup: UniversalLookup,
}

impl<C: ContextRegistryLookup> RegistryInfoLookup for PlaceholderRegistryInfoLookup<C> {
    // JAVA: net.minecraft.util.PlaceholderLookupProvider#createSerializationContext$RegistryInfoLookup#lookup
    fn lookup_erased(&self, registry_key: &ResourceKey<Erased>) -> Option<RegistryInfo<Erased>> {
        self.context.lookup_registry_info(registry_key).or_else(|| {
            Some(RegistryInfo::new(
                self.lookup.cast_as_owner::<Erased>(),
                self.lookup.cast_as_lookup::<Erased>(),
                Lifecycle::experimental(),
            ))
        })
    }
}

struct PlaceholderRegistryContextSwapper<C: ContextRegistryLookup + 'static> {
    context: Rc<C>,
    lookup: UniversalLookup,
}

impl<C: ContextRegistryLookup + 'static> RegistryContextSwapper
    for PlaceholderRegistryContextSwapper<C>
{
    // JAVA: net.minecraft.util.PlaceholderLookupProvider#createSwapper$RegistryContextSwapper#swapTo
    fn swap_to<T, Co: Codec<T>, Ctx: HolderLookupProviderLike>(
        &self,
        codec: &Co,
        value: &T,
        target: &Ctx,
    ) -> DataResult<T> {
        let source_ops: RegistryOps<JavaValue> = RegistryOps::create(
            Box::new(JavaOps),
            Box::new(PlaceholderRegistryInfoLookup {
                context: Rc::clone(&self.context),
                lookup: self.lookup.clone(),
            }),
        );
        let encoded: DataResult<JavaValue> =
            codec.encode_start(&source_ops as &dyn DynamicOps<JavaValue>, value);
        encoded.flat_map(|v: JavaValue| {
            let target_ops: RegistryOps<JavaValue> =
                target.create_serialization_context(Box::new(JavaOps));
            codec.parse(&target_ops as &dyn DynamicOps<JavaValue>, &v)
        })
    }
}

/// Faithful port of `net.minecraft.util.PlaceholderLookupProvider`.
pub struct PlaceholderLookupProvider<C> {
    context: Rc<C>,
    lookup: UniversalLookup,
    state: Rc<RefCell<PlaceholderState>>,
}

impl<C> PlaceholderLookupProvider<C> {
    // JAVA: net.minecraft.util.PlaceholderLookupProvider#<init>
    pub fn new(context: C) -> Self {
        let state = Rc::new(RefCell::new(PlaceholderState::new()));
        let lookup = UniversalLookup::new(Rc::clone(&state));
        Self {
            context: Rc::new(context),
            lookup,
            state,
        }
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider#lookup
    pub fn lookup<T: ?Sized>(&self, _registry_key: &ResourceKey<Registry<T>>) -> Option<UniversalLookup> {
        Some(self.lookup.cast_as_lookup::<T>())
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider#hasRegisteredPlaceholders
    pub fn has_registered_placeholders(&self) -> bool {
        let state = self.state.borrow();
        !state.holders.is_empty() || !state.holder_sets.is_empty()
    }

    pub fn universal_lookup(&self) -> &UniversalLookup {
        &self.lookup
    }
}

impl<C: ContextRegistryLookup + 'static> PlaceholderLookupProvider<C> {
    // JAVA: net.minecraft.util.PlaceholderLookupProvider#createSerializationContext
    pub fn create_serialization_context<V: 'static>(
        &self,
        ops: Box<dyn DynamicOps<V>>,
    ) -> RegistryOps<V> {
        RegistryOps::create(
            ops,
            Box::new(PlaceholderRegistryInfoLookup {
                context: Rc::clone(&self.context),
                lookup: self.lookup.clone(),
            }),
        )
    }

    // JAVA: net.minecraft.util.PlaceholderLookupProvider#createSwapper
    pub fn create_swapper(&self) -> impl RegistryContextSwapper {
        PlaceholderRegistryContextSwapper {
            context: Rc::clone(&self.context),
            lookup: self.lookup.clone(),
        }
    }
}

impl<C: ContextRegistryLookup + 'static> HolderLookupProviderLike for PlaceholderLookupProvider<C> {
    fn create_serialization_context<V: 'static>(&self, ops: Box<dyn DynamicOps<V>>) -> RegistryOps<V> {
        PlaceholderLookupProvider::create_serialization_context(self, ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyContext;
    impl ContextRegistryLookup for EmptyContext {
        fn lookup_registry_info(
            &self,
            _registry_key: &ResourceKey<Erased>,
        ) -> Option<RegistryInfo<Erased>> {
            None
        }
    }

    struct FixedContext;
    impl ContextRegistryLookup for FixedContext {
        fn lookup_registry_info(
            &self,
            _registry_key: &ResourceKey<Erased>,
        ) -> Option<RegistryInfo<Erased>> {
            let state = Rc::new(RefCell::new(PlaceholderState::new()));
            let ul = UniversalLookup::new(state);
            Some(RegistryInfo::new(ul.clone(), ul, Lifecycle::Stable))
        }
    }

    #[test]
    fn has_registered_placeholders_tracks_holders_and_tags() {
        let provider = PlaceholderLookupProvider::new(EmptyContext);
        assert!(!provider.has_registered_placeholders());

        let lookup = provider
            .lookup::<Erased>(&ResourceKey::new("root", "reg"))
            .unwrap();
        let key = ResourceKey::<Erased>::new("reg", "item");
        let _ = lookup.get_or_throw_resource(&key);
        assert!(provider.has_registered_placeholders());

        let provider2 = PlaceholderLookupProvider::new(EmptyContext);
        let lookup2 = provider2
            .lookup::<Erased>(&ResourceKey::new("root", "reg"))
            .unwrap();
        let tag = TagKey::<Erased>::new("reg", "tag");
        let _ = lookup2.get_or_throw_tag(&tag);
        assert!(provider2.has_registered_placeholders());
    }

    #[test]
    fn get_or_create_resource_caches_by_key() {
        let provider = PlaceholderLookupProvider::new(EmptyContext);
        let lookup = provider
            .lookup::<Erased>(&ResourceKey::new("root", "reg"))
            .unwrap();
        let key = ResourceKey::<Erased>::new("reg", "value_a");
        let a = lookup.get_or_throw_resource(&key);
        let b = lookup.get_resource(&key).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.key(), &key);
    }

    #[test]
    fn different_keys_produce_different_placeholders() {
        let provider = PlaceholderLookupProvider::new(EmptyContext);
        let lookup = provider
            .lookup::<Erased>(&ResourceKey::new("root", "reg"))
            .unwrap();
        let k1 = ResourceKey::<Erased>::new("reg", "a");
        let k2 = ResourceKey::<Erased>::new("reg", "b");
        assert_ne!(
            lookup.get_or_throw_resource(&k1),
            lookup.get_or_throw_resource(&k2)
        );

        let t1 = TagKey::<Erased>::new("reg", "ta");
        let t2 = TagKey::<Erased>::new("reg", "tb");
        assert_ne!(lookup.get_or_throw_tag(&t1), lookup.get_or_throw_tag(&t2));
    }

    #[test]
    fn lookup_always_returns_universal_lookup() {
        let provider = PlaceholderLookupProvider::new(EmptyContext);
        let k1: ResourceKey<Registry<Erased>> = ResourceKey::new("root", "a");
        let k2: ResourceKey<Registry<Erased>> = ResourceKey::new("root", "b");
        let l1 = provider.lookup::<Erased>(&k1).unwrap();
        let l2 = provider.lookup::<Erased>(&k2).unwrap();

        let key = ResourceKey::<Erased>::new("reg", "shared");
        let h1 = l1.get_or_throw_resource(&key);
        let h2 = l2.get_or_throw_resource(&key);
        assert_eq!(h1, h2);
    }

    #[test]
    fn create_serialization_context_falls_back_to_placeholder() {
        let provider = PlaceholderLookupProvider::new(EmptyContext);
        let ops = provider.create_serialization_context(Box::new(JavaOps));
        let reg_key = ResourceKey::<Erased>::new("minecraft:dimension", "minecraft:overworld");
        let info = ops.lookup_erased(&reg_key).unwrap();
        assert_eq!(info.lifecycle, Lifecycle::Experimental);
    }

    #[test]
    fn create_serialization_context_uses_context_when_present() {
        let provider = PlaceholderLookupProvider::new(FixedContext);
        let ops = provider.create_serialization_context(Box::new(JavaOps));
        let reg_key = ResourceKey::<Erased>::new("minecraft:dimension", "minecraft:overworld");
        let info = ops.lookup_erased(&reg_key).unwrap();
        assert_eq!(info.lifecycle, Lifecycle::Stable);
    }
}
