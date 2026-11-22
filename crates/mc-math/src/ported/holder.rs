use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

/// Minimal stand-in for `net.minecraft.resources.Identifier`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal stand-in for `net.minecraft.resources.ResourceKey<T>`.
#[derive(Clone, Debug)]
pub struct ResourceKey<T> {
    pub identifier: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ResourceKey<T> {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            _marker: PhantomData,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl<T> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<T> Eq for ResourceKey<T> {}

impl<T> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

/// Minimal stand-in for `net.minecraft.tags.TagKey<T>`.
#[derive(Clone, Debug)]
pub struct TagKey<T> {
    pub identifier: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<T> Eq for TagKey<T> {}

impl<T> Hash for TagKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

/// Minimal stand-in for `net.minecraft.core.HolderOwner<T>`.
pub trait HolderOwner<T> {
    fn can_serialize_in(&self, owner: &dyn HolderOwner<T>) -> bool;
}

/// Minimal stand-in for `com.mojang.datafixers.util.Either`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Port of `net.minecraft.core.Holder.Kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Reference,
    Direct,
}

/// Port of `net.minecraft.core.Holder<T>`.
///
/// Java uses an interface with default methods; in Rust this is represented as
/// a trait plus concrete `Direct` and `Reference` implementations.
pub trait Holder<T: Clone + fmt::Debug> {
    // JAVA: net.minecraft.core.Holder#value
    fn value(&self) -> &T;

    // JAVA: net.minecraft.core.Holder#isBound
    fn is_bound(&self) -> bool;

    // JAVA: net.minecraft.core.Holder#is(net.minecraft.resources.Identifier)
    fn is_identifier(&self, id: &Identifier) -> bool;

    // JAVA: net.minecraft.core.Holder#is(net.minecraft.resources.ResourceKey)
    fn is_resource_key(&self, key: &ResourceKey<T>) -> bool;

    // JAVA: net.minecraft.core.Holder#is(java.util.function.Predicate)
    fn is_predicate(&self, predicate: &dyn Fn(&ResourceKey<T>) -> bool) -> bool;

    // JAVA: net.minecraft.core.Holder#is(net.minecraft.tags.TagKey)
    fn is_tag_key(&self, tag: &TagKey<T>) -> bool;

    // JAVA: net.minecraft.core.Holder#is(net.minecraft.core.Holder)
    fn is_holder(&self, other: &dyn Holder<T>) -> bool;

    // JAVA: net.minecraft.core.Holder#tags
    fn tags(&self) -> Vec<TagKey<T>>;

    // JAVA: net.minecraft.core.Holder#unwrap
    fn unwrap(&self) -> Either<ResourceKey<T>, T>;

    // JAVA: net.minecraft.core.Holder#unwrapKey
    fn unwrap_key(&self) -> Option<ResourceKey<T>>;

    // JAVA: net.minecraft.core.Holder#kind
    fn kind(&self) -> Kind;

    // JAVA: net.minecraft.core.Holder#canSerializeIn
    fn can_serialize_in(&self, owner: &dyn HolderOwner<T>) -> bool;

    // JAVA: net.minecraft.core.Holder#getRegisteredName
    fn get_registered_name(&self) -> String {
        match self.unwrap_key() {
            Some(key) => key.identifier().to_string(),
            None => "[unregistered]".to_string(),
        }
    }
}

// JAVA: net.minecraft.core.Holder#direct
pub fn direct<T: Clone + PartialEq + fmt::Debug>(value: T) -> Direct<T> {
    Direct::new(value)
}

/// Port of `net.minecraft.core.Holder.Direct<T>`.
#[derive(Clone, Debug)]
pub struct Direct<T> {
    value: T,
}

impl<T: Clone + fmt::Debug> Direct<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone + PartialEq + fmt::Debug> Holder<T> for Direct<T> {
    // JAVA: net.minecraft.core.Holder$Direct#value
    fn value(&self) -> &T {
        &self.value
    }

    // JAVA: net.minecraft.core.Holder$Direct#isBound
    fn is_bound(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.core.Holder$Direct#is(net.minecraft.resources.Identifier)
    fn is_identifier(&self, _id: &Identifier) -> bool {
        false
    }

    // JAVA: net.minecraft.core.Holder$Direct#is(net.minecraft.resources.ResourceKey)
    fn is_resource_key(&self, _key: &ResourceKey<T>) -> bool {
        false
    }

    // JAVA: net.minecraft.core.Holder$Direct#is(net.minecraft.tags.TagKey)
    fn is_tag_key(&self, _tag: &TagKey<T>) -> bool {
        false
    }

    // JAVA: net.minecraft.core.Holder$Direct#is(net.minecraft.core.Holder)
    fn is_holder(&self, other: &dyn Holder<T>) -> bool {
        self.value == *other.value()
    }

    // JAVA: net.minecraft.core.Holder$Direct#is(java.util.function.Predicate)
    fn is_predicate(&self, _predicate: &dyn Fn(&ResourceKey<T>) -> bool) -> bool {
        false
    }

    // JAVA: net.minecraft.core.Holder$Direct#unwrap
    fn unwrap(&self) -> Either<ResourceKey<T>, T> {
        Either::Right(self.value.clone())
    }

    // JAVA: net.minecraft.core.Holder$Direct#unwrapKey
    fn unwrap_key(&self) -> Option<ResourceKey<T>> {
        None
    }

    // JAVA: net.minecraft.core.Holder$Direct#kind
    fn kind(&self) -> Kind {
        Kind::Direct
    }

    // JAVA: net.minecraft.core.Holder$Direct#toString
    // Rust Display/Debug equivalent is provided via fmt::Display below.

    // JAVA: net.minecraft.core.Holder$Direct#canSerializeIn
    fn can_serialize_in(&self, _owner: &dyn HolderOwner<T>) -> bool {
        true
    }

    // JAVA: net.minecraft.core.Holder$Direct#tags
    fn tags(&self) -> Vec<TagKey<T>> {
        Vec::new()
    }
}

impl<T: fmt::Debug> fmt::Display for Direct<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Direct{{{:?}}}", self.value)
    }
}

/// Port of `net.minecraft.core.Holder.Reference.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferenceType {
    StandAlone,
    Intrusive,
}

/// Port of `net.minecraft.core.Holder.Reference<T>`.
///
/// Java stores a `HolderOwner<T>` reference; Rust uses `Arc<dyn HolderOwner<T>>`
/// to keep the module self-contained without borrowing lifetimes tied to registries.
pub struct Reference<T: Clone + fmt::Debug> {
    owner: Arc<dyn HolderOwner<T> + Send + Sync>,
    tags: Option<HashSet<TagKey<T>>>,
    ref_type: ReferenceType,
    key: Option<ResourceKey<T>>,
    value: Option<T>,
}

impl<T: Clone + fmt::Debug> Reference<T> {
    fn new(
        ref_type: ReferenceType,
        owner: Arc<dyn HolderOwner<T> + Send + Sync>,
        key: Option<ResourceKey<T>>,
        value: Option<T>,
    ) -> Self {
        Self {
            owner,
            tags: None,
            ref_type,
            key,
            value,
        }
    }

    // JAVA: net.minecraft.core.Holder$Reference#createStandAlone
    pub fn create_stand_alone(
        owner: Arc<dyn HolderOwner<T> + Send + Sync>,
        key: ResourceKey<T>,
    ) -> Self {
        Self::new(ReferenceType::StandAlone, owner, Some(key), None)
    }

    // JAVA: net.minecraft.core.Holder$Reference#createIntrusive
    pub fn create_intrusive(owner: Arc<dyn HolderOwner<T> + Send + Sync>, value: Option<T>) -> Self {
        Self::new(ReferenceType::Intrusive, owner, None, value)
    }

    // JAVA: net.minecraft.core.Holder$Reference#key
    pub fn key(&self) -> &ResourceKey<T> {
        match &self.key {
            Some(key) => key,
            None => panic!(
                "Trying to access unbound value '{:?}' from registry {}",
                self.value,
                self.owner_debug()
            ),
        }
    }

    fn owner_debug(&self) -> String {
        // HolderOwner has no Display/Debug in Java signature; provide a stable placeholder.
        "<owner>".to_string()
    }

    fn bound_tags(&self) -> &HashSet<TagKey<T>> {
        match &self.tags {
            Some(tags) => tags,
            None => panic!("Tags not bound"),
        }
    }

    // JAVA: net.minecraft.core.Holder$Reference#bindKey
    pub fn bind_key(&mut self, key: ResourceKey<T>) {
        if let Some(existing) = &self.key {
            if existing != &key {
                panic!(
                    "Can't change holder key: existing={:?}, new={:?}",
                    existing.identifier(),
                    key.identifier()
                );
            }
        }
        self.key = Some(key);
    }

    // JAVA: net.minecraft.core.Holder$Reference#bindValue
    pub fn bind_value(&mut self, value: T)
    where
        T: PartialEq,
    {
        if self.ref_type == ReferenceType::Intrusive {
            if let Some(existing) = &self.value {
                if existing != &value {
                    panic!(
                        "Can't change holder {:?} value: existing={:?}, new={:?}",
                        self.key.as_ref().map(|k| k.identifier()),
                        existing,
                        value
                    );
                }
            }
        }
        self.value = Some(value);
    }

    // JAVA: net.minecraft.core.Holder$Reference#bindTags
    pub fn bind_tags<I>(&mut self, tags: I)
    where
        I: IntoIterator<Item = TagKey<T>>,
    {
        self.tags = Some(tags.into_iter().collect());
    }
}

impl<T: Clone + PartialEq + fmt::Debug> Holder<T> for Reference<T> {
    // JAVA: net.minecraft.core.Holder$Reference#value
    fn value(&self) -> &T {
        match &self.value {
            Some(value) => value,
            None => panic!(
                "Trying to access unbound value '{:?}' from registry {}",
                self.key.as_ref().map(|k| k.identifier()),
                self.owner_debug()
            ),
        }
    }

    // JAVA: net.minecraft.core.Holder$Reference#isBound
    fn is_bound(&self) -> bool {
        self.key.is_some() && self.value.is_some()
    }

    // JAVA: net.minecraft.core.Holder$Reference#is(net.minecraft.resources.Identifier)
    fn is_identifier(&self, id: &Identifier) -> bool {
        self.key().identifier() == id
    }

    // JAVA: net.minecraft.core.Holder$Reference#is(net.minecraft.resources.ResourceKey)
    fn is_resource_key(&self, key: &ResourceKey<T>) -> bool {
        self.key() == key
    }

    // JAVA: net.minecraft.core.Holder$Reference#is(net.minecraft.tags.TagKey)
    fn is_tag_key(&self, tag: &TagKey<T>) -> bool {
        self.bound_tags().contains(tag)
    }

    // JAVA: net.minecraft.core.Holder$Reference#is(net.minecraft.core.Holder)
    fn is_holder(&self, other: &dyn Holder<T>) -> bool {
        other.is_resource_key(self.key())
    }

    // JAVA: net.minecraft.core.Holder$Reference#is(java.util.function.Predicate)
    fn is_predicate(&self, predicate: &dyn Fn(&ResourceKey<T>) -> bool) -> bool {
        predicate(self.key())
    }

    // JAVA: net.minecraft.core.Holder$Reference#canSerializeIn
    fn can_serialize_in(&self, owner: &dyn HolderOwner<T>) -> bool {
        self.owner.can_serialize_in(owner)
    }

    // JAVA: net.minecraft.core.Holder$Reference#unwrap
    fn unwrap(&self) -> Either<ResourceKey<T>, T> {
        Either::Left(self.key().clone())
    }

    // JAVA: net.minecraft.core.Holder$Reference#unwrapKey
    fn unwrap_key(&self) -> Option<ResourceKey<T>> {
        Some(self.key().clone())
    }

    // JAVA: net.minecraft.core.Holder$Reference#kind
    fn kind(&self) -> Kind {
        Kind::Reference
    }

    // JAVA: net.minecraft.core.Holder$Reference#tags
    fn tags(&self) -> Vec<TagKey<T>> {
        self.bound_tags().iter().cloned().collect()
    }
}

impl<T: Clone + fmt::Debug> fmt::Display for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reference{{{:?}={:?}}}",
            self.key.as_ref().map(|k| k.identifier()),
            self.value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestOwner {
        id: u32,
    }

    impl<T> HolderOwner<T> for TestOwner {
        fn can_serialize_in(&self, owner: &dyn HolderOwner<T>) -> bool {
            let ptr = owner as *const dyn HolderOwner<T> as *const () as usize;
            let self_ptr = self as *const Self as *const () as usize;
            // For tests, only same concrete instance pointer is considered same.
            ptr == self_ptr
        }
    }

    fn id(ns: &str, path: &str) -> Identifier {
        Identifier::new(ns, path)
    }

    fn key<T>(ns: &str, path: &str) -> ResourceKey<T> {
        ResourceKey::new(id(ns, path))
    }

    fn tag<T>(ns: &str, path: &str) -> TagKey<T> {
        TagKey::new(id(ns, path))
    }

    #[test]
    fn direct_holder_behaves_like_java() {
        let h = direct(42_i32);
        assert!(h.is_bound());
        assert_eq!(h.value(), &42);
        assert!(!h.is_identifier(&id("minecraft", "stone")));
        assert!(!h.is_resource_key(&key("minecraft", "stone")));
        assert!(!h.is_tag_key(&tag("minecraft", "mineable")));
        assert!(!h.is_predicate(&|_| true));
        assert_eq!(h.kind(), Kind::Direct);
        assert_eq!(h.unwrap_key(), None);
        assert_eq!(h.unwrap(), Either::Right(42));
        assert_eq!(h.get_registered_name(), "[unregistered]");
        assert!(h.tags().is_empty());
        assert_eq!(format!("{}", h), "Direct{42}");
    }

    #[test]
    fn direct_holder_is_compares_value() {
        let h1 = direct(String::from("abc"));
        let h2 = direct(String::from("abc"));
        let h3 = direct(String::from("def"));
        assert!(h1.is_holder(&h2));
        assert!(!h1.is_holder(&h3));
    }

    #[test]
    fn reference_stand_alone_bind_and_query() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let k = key::<i32>("minecraft", "stone");
        let mut r = Reference::create_stand_alone(owner, k.clone());

        assert!(!r.is_bound());
        r.bind_value(7);
        assert!(r.is_bound());
        assert_eq!(r.value(), &7);
        assert_eq!(r.key(), &k);
        assert!(r.is_identifier(&id("minecraft", "stone")));
        assert!(!r.is_identifier(&id("minecraft", "dirt")));
        assert!(r.is_resource_key(&k));
        assert!(r.is_predicate(&|rk| rk.identifier().path == "stone"));
        assert!(!r.is_predicate(&|rk| rk.identifier().path == "dirt"));
        assert_eq!(r.kind(), Kind::Reference);
        assert_eq!(r.unwrap(), Either::Left(k.clone()));
        assert_eq!(r.unwrap_key(), Some(k.clone()));
        assert_eq!(r.get_registered_name(), "minecraft:stone");
    }

    #[test]
    fn reference_tags_must_be_bound_and_are_queryable() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let mut r = Reference::create_stand_alone(owner, key("minecraft", "stone"));
        r.bind_value(1);

        let t1 = tag::<i32>("minecraft", "mineable");
        let t2 = tag::<i32>("minecraft", "natural");
        r.bind_tags(vec![t1.clone(), t2.clone()]);

        assert!(r.is_tag_key(&t1));
        assert!(r.is_tag_key(&t2));
        assert!(!r.is_tag_key(&tag("minecraft", "other")));
        let mut tags = r.tags();
        tags.sort_by(|a, b| a.identifier.to_string().cmp(&b.identifier.to_string()));
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].identifier.to_string(), "minecraft:mineable");
        assert_eq!(tags[1].identifier.to_string(), "minecraft:natural");
    }

    #[test]
    #[should_panic(expected = "Tags not bound")]
    fn reference_unbound_tags_panics() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let r = Reference::create_stand_alone(owner, key("minecraft", "stone"));
        let _ = r.tags();
    }

    #[test]
    fn reference_bind_key_rejects_change_and_intrusive_value_rejects_change() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let mut stand_alone = Reference::create_stand_alone(owner.clone(), key("minecraft", "a"));
        stand_alone.bind_key(key("minecraft", "a")); // same key ok

        let mut intrusive = Reference::create_intrusive(owner, Some(10_i32));
        intrusive.bind_key(key("minecraft", "intrusive"));
        intrusive.bind_value(10); // same value ok for intrusive
        assert_eq!(intrusive.value(), &10);
    }

    #[test]
    #[should_panic(expected = "Can't change holder key")]
    fn reference_bind_key_change_panics() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let mut r = Reference::create_stand_alone(owner, key("minecraft", "a"));
        r.bind_key(key("minecraft", "b"));
    }

    #[test]
    #[should_panic(expected = "Can't change holder")]
    fn reference_intrusive_bind_value_change_panics() {
        let owner: Arc<dyn HolderOwner<i32> + Send + Sync> = Arc::new(TestOwner { id: 1 });
        let mut r = Reference::create_intrusive(owner, Some(1_i32));
        r.bind_key(key("minecraft", "x"));
        r.bind_value(2);
    }
}
