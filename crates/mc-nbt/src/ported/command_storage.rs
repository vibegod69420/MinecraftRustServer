use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::NbtTag;

/// Minimal local representation of `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#fromNamespaceAndPath
    pub fn from_namespace_and_path(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#getNamespace
    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    // JAVA: net.minecraft.resources.Identifier#getPath
    pub fn get_path(&self) -> &str {
        &self.path
    }
}

/// Minimal local representation of `net.minecraft.world.level.saveddata.SavedDataType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SavedDataType {
    pub id: String,
}

/// Minimal local abstraction for `net.minecraft.world.level.storage.DimensionDataStorage`.
pub trait DimensionDataStorage {
    // JAVA: net.minecraft.world.level.storage.DimensionDataStorage#get
    fn get(&mut self, type_: &SavedDataType) -> Option<Rc<RefCell<Container>>>;

    // JAVA: net.minecraft.world.level.storage.DimensionDataStorage#computeIfAbsent
    fn compute_if_absent(&mut self, type_: &SavedDataType) -> Rc<RefCell<Container>>;
}

/// Port of `net.minecraft.world.level.storage.CommandStorage$Container`.
pub struct Container {
    storage: HashMap<String, NbtTag>,
    dirty: bool,
}

impl Container {
    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#<init>
    fn new(storage: HashMap<String, NbtTag>) -> Self {
        Self {
            storage,
            dirty: false,
        }
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#<init>
    pub fn new_empty() -> Self {
        Self::new(HashMap::new())
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#type
    pub fn saved_data_type(namespace: &str) -> SavedDataType {
        SavedDataType {
            id: create_id(namespace),
        }
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#get
    pub fn get(&self, key: &str) -> NbtTag {
        self.storage
            .get(key)
            .cloned()
            .unwrap_or_else(|| NbtTag::Compound(vec![]))
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#put
    pub fn put(&mut self, key: String, tag: NbtTag) {
        let is_empty = matches!(&tag, NbtTag::Compound(entries) if entries.is_empty());
        if is_empty {
            self.storage.remove(&key);
        } else {
            self.storage.insert(key, tag);
        }
        self.set_dirty();
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage$Container#getKeys
    pub fn get_keys(&self, namespace: &str) -> impl Iterator<Item = Identifier> + '_ {
        let ns = namespace.to_string();
        self.storage
            .keys()
            .map(move |k| Identifier::from_namespace_and_path(ns.clone(), k.clone()))
    }

    // JAVA: net.minecraft.world.level.saveddata.SavedData#setDirty
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    // JAVA: net.minecraft.world.level.saveddata.SavedData#isDirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Port of `net.minecraft.world.level.storage.CommandStorage`.
pub struct CommandStorage<S: DimensionDataStorage> {
    namespaces: HashMap<String, Rc<RefCell<Container>>>,
    storage: S,
}

impl<S: DimensionDataStorage> CommandStorage<S> {
    // JAVA: net.minecraft.world.level.storage.CommandStorage#<init>
    pub fn new(storage: S) -> Self {
        Self {
            namespaces: HashMap::new(),
            storage,
        }
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage#get
    pub fn get(&mut self, id: &Identifier) -> NbtTag {
        match self.get_container(id.get_namespace()) {
            Some(container) => container.borrow().get(id.get_path()),
            None => NbtTag::Compound(vec![]),
        }
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage#getContainer
    fn get_container(&mut self, namespace: &str) -> Option<Rc<RefCell<Container>>> {
        if let Some(existing) = self.namespaces.get(namespace) {
            return Some(Rc::clone(existing));
        }
        let loaded = self
            .storage
            .get(&Container::saved_data_type(namespace));
        if let Some(container) = &loaded {
            self.namespaces
                .insert(namespace.to_string(), Rc::clone(container));
        }
        loaded
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage#getOrCreateContainer
    fn get_or_create_container(&mut self, namespace: &str) -> Rc<RefCell<Container>> {
        if let Some(existing) = self.namespaces.get(namespace) {
            return Rc::clone(existing);
        }
        let created = self
            .storage
            .compute_if_absent(&Container::saved_data_type(namespace));
        self.namespaces
            .insert(namespace.to_string(), Rc::clone(&created));
        created
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage#set
    pub fn set(&mut self, id: &Identifier, tag: NbtTag) {
        let container = self.get_or_create_container(id.get_namespace());
        container
            .borrow_mut()
            .put(id.get_path().to_string(), tag);
    }

    // JAVA: net.minecraft.world.level.storage.CommandStorage#keys
    pub fn keys(&self) -> Vec<Identifier> {
        self.namespaces
            .iter()
            .flat_map(|(ns, container)| {
                let borrowed = container.borrow();
                borrowed.get_keys(ns).collect::<Vec<_>>()
            })
            .collect()
    }
}

// JAVA: net.minecraft.world.level.storage.CommandStorage#createId
pub fn create_id(namespace: &str) -> String {
    format!("command_storage_{}", namespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStorage {
        containers: HashMap<String, Rc<RefCell<Container>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                containers: HashMap::new(),
            }
        }
    }

    impl DimensionDataStorage for MockStorage {
        fn get(&mut self, type_: &SavedDataType) -> Option<Rc<RefCell<Container>>> {
            self.containers.get(&type_.id).cloned()
        }

        fn compute_if_absent(&mut self, type_: &SavedDataType) -> Rc<RefCell<Container>> {
            self.containers
                .entry(type_.id.clone())
                .or_insert_with(|| Rc::new(RefCell::new(Container::new_empty())))
                .clone()
        }
    }

    #[test]
    fn test_get_set_and_create_id() {
        let mut cs = CommandStorage::new(MockStorage::new());
        let id = Identifier::from_namespace_and_path("ns", "path");
        assert_eq!(cs.get(&id), NbtTag::Compound(vec![]));

        let tag = NbtTag::Compound(vec![("k".to_string(), NbtTag::Int(42))]);
        cs.set(&id, tag.clone());
        assert_eq!(cs.get(&id), tag);
        assert_eq!(create_id("foo"), "command_storage_foo");
    }

    #[test]
    fn test_put_empty_removes_and_sets_dirty() {
        let mut cs = CommandStorage::new(MockStorage::new());
        let id = Identifier::from_namespace_and_path("ns", "a");
        cs.set(
            &id,
            NbtTag::Compound(vec![("x".to_string(), NbtTag::Byte(1))]),
        );

        let container = cs.get_or_create_container("ns");
        assert!(container.borrow().is_dirty());
        container.borrow_mut().dirty = false;

        cs.set(&id, NbtTag::Compound(vec![]));
        let container = cs.get_or_create_container("ns");
        assert!(container.borrow().is_dirty());
        assert!(container.borrow().storage.get("a").is_none());
        assert_eq!(cs.get(&id), NbtTag::Compound(vec![]));
    }

    #[test]
    fn test_negative_and_integer_extremes() {
        let mut cs = CommandStorage::new(MockStorage::new());
        let id = Identifier::from_namespace_and_path("ns", "neg");
        let tag = NbtTag::Compound(vec![
            ("b".to_string(), NbtTag::Byte(i8::MIN)),
            ("s".to_string(), NbtTag::Short(i16::MIN)),
            ("i".to_string(), NbtTag::Int(i32::MIN)),
            ("imax".to_string(), NbtTag::Int(i32::MAX)),
            ("l".to_string(), NbtTag::Long(i64::MIN)),
            ("lmax".to_string(), NbtTag::Long(i64::MAX)),
        ]);
        cs.set(&id, tag.clone());
        let got = cs.get(&id);
        assert_eq!(got, tag);
        match got.compound_get("i") {
            Some(NbtTag::Int(v)) => assert_eq!(*v, i32::MIN),
            _ => panic!("expected int"),
        }
        match got.compound_get("lmax") {
            Some(NbtTag::Long(v)) => assert_eq!(*v, i64::MAX),
            _ => panic!("expected long"),
        }
    }

    #[test]
    fn test_float_bits_preserved() {
        let mut cs = CommandStorage::new(MockStorage::new());
        let id = Identifier::from_namespace_and_path("ns", "floats");
        let tag = NbtTag::Compound(vec![
            ("nan".to_string(), NbtTag::Float(f32::NAN)),
            ("neg0".to_string(), NbtTag::Float(-0.0f32)),
            ("inf".to_string(), NbtTag::Double(f64::INFINITY)),
            ("ninf".to_string(), NbtTag::Double(f64::NEG_INFINITY)),
        ]);
        cs.set(&id, tag);
        let got = cs.get(&id);
        match got.compound_get("nan") {
            Some(NbtTag::Float(v)) => assert!(v.is_nan() && v.to_bits() == f32::NAN.to_bits()),
            _ => panic!("expected float nan"),
        }
        match got.compound_get("neg0") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), (-0.0f32).to_bits()),
            _ => panic!("expected neg0"),
        }
        match got.compound_get("inf") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), f64::INFINITY.to_bits()),
            _ => panic!("expected inf"),
        }
        match got.compound_get("ninf") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), f64::NEG_INFINITY.to_bits()),
            _ => panic!("expected ninf"),
        }
    }

    #[test]
    fn test_keys_namespace_isolation_and_storage_load() {
        let mut mock = MockStorage::new();
        let preloaded_id = Container::saved_data_type("pre");
        let preloaded = Rc::new(RefCell::new(Container::new_empty()));
        preloaded.borrow_mut().put(
            "k".to_string(),
            NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(-1))]),
        );
        preloaded.borrow_mut().dirty = false;
        mock.containers.insert(preloaded_id.id.clone(), preloaded);

        let mut cs = CommandStorage::new(mock);
        // Loading from storage should cache and return value.
        let got = cs.get(&Identifier::from_namespace_and_path("pre", "k"));
        assert_eq!(
            got,
            NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(-1))])
        );

        cs.set(
            &Identifier::from_namespace_and_path("a", "x"),
            NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(1))]),
        );
        cs.set(
            &Identifier::from_namespace_and_path("b", "y"),
            NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(2))]),
        );

        let mut keys = cs.keys();
        keys.sort_by(|a, b| {
            (a.get_namespace(), a.get_path()).cmp(&(b.get_namespace(), b.get_path()))
        });
        assert!(keys.contains(&Identifier::from_namespace_and_path("a", "x")));
        assert!(keys.contains(&Identifier::from_namespace_and_path("b", "y")));
        assert!(keys.contains(&Identifier::from_namespace_and_path("pre", "k")));
        assert_eq!(
            cs.get(&Identifier::from_namespace_and_path("a", "y")),
            NbtTag::Compound(vec![])
        );
    }
}
