use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for Minecraft registry/holder infrastructure.
// These are pragmatic type-erased representations sufficient for the
// RegistrySetBuilder logic.
// ---------------------------------------------------------------------------

pub type Erased = Rc<dyn Any>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub registry: Identifier,
    pub location: Identifier,
}

impl ResourceKey {
    pub fn new(registry: Identifier, location: Identifier) -> Self {
        Self { registry, location }
    }

    pub fn is_for(&self, registry_key: &ResourceKey) -> bool {
        self.registry == registry_key.location
    }

    pub fn identifier(&self) -> &Identifier {
        &self.location
    }
}

impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.registry, self.location)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
}

impl Lifecycle {
    pub fn stable() -> Self {
        Lifecycle::Stable
    }

    pub fn experimental() -> Self {
        Lifecycle::Experimental
    }

    pub fn add(self, other: Lifecycle) -> Lifecycle {
        if self == Lifecycle::Experimental || other == Lifecycle::Experimental {
            Lifecycle::Experimental
        } else {
            Lifecycle::Stable
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagKey {
    pub registry: Identifier,
    pub location: Identifier,
}

#[derive(Clone, Debug)]
pub struct NamedHolderSet {
    pub key: TagKey,
}

#[derive(Clone, Debug)]
pub struct OwnerToken(Rc<()>);

impl OwnerToken {
    pub fn new() -> Self {
        Self(Rc::new(()))
    }
}

impl Default for OwnerToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct HolderReference {
    owner: OwnerToken,
    key: Option<ResourceKey>,
    inner: Rc<RefCell<HolderRefInner>>,
}

struct HolderRefInner {
    value: Option<Erased>,
    supplier: Option<Rc<dyn Fn() -> Erased>>,
}

impl HolderReference {
    // JAVA: net.minecraft.core.Holder.Reference#createStandAlone
    pub fn create_stand_alone(owner: OwnerToken, key: ResourceKey) -> Self {
        Self {
            owner,
            key: Some(key),
            inner: Rc::new(RefCell::new(HolderRefInner {
                value: None,
                supplier: None,
            })),
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$LazyHolder#<init>
    pub fn create_lazy(owner: OwnerToken, key: ResourceKey) -> Self {
        Self::create_stand_alone(owner, key)
    }

    // JAVA: net.minecraft.core.Holder.Reference#bindValue
    // JAVA: net.minecraft.core.RegistrySetBuilder$LazyHolder#bindValue
    pub fn bind_value(&self, value: Erased) {
        let mut inner = self.inner.borrow_mut();
        inner.value = Some(value);
        inner.supplier = None;
    }

    // JAVA: net.minecraft.core.Holder.Reference#value
    // JAVA: net.minecraft.core.RegistrySetBuilder$LazyHolder#value
    pub fn value(&self) -> Erased {
        let supplier = {
            let mut inner = self.inner.borrow_mut();
            inner.supplier.take()
        };
        if let Some(sup) = supplier {
            let v = sup();
            self.inner.borrow_mut().value = Some(v);
        }
        self.inner
            .borrow()
            .value
            .clone()
            .expect("Holder not bound")
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$LazyHolder#supplier (field assignment helper)
    pub fn set_supplier(&self, supplier: Rc<dyn Fn() -> Erased>) {
        self.inner.borrow_mut().supplier = Some(supplier);
    }

    pub fn key(&self) -> Option<&ResourceKey> {
        self.key.as_ref()
    }

    pub fn owner(&self) -> &OwnerToken {
        &self.owner
    }

    pub fn is_bound(&self) -> bool {
        let inner = self.inner.borrow();
        inner.value.is_some() || inner.supplier.is_some()
    }
}

impl fmt::Debug for HolderReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HolderReference")
            .field("key", &self.key)
            .field("bound", &self.is_bound())
            .finish()
    }
}

pub trait HolderGetter {
    fn get(&self, key: &ResourceKey) -> Option<HolderReference>;

    fn get_tag(&self, _tag: &TagKey) -> Option<NamedHolderSet> {
        None
    }
}

pub trait RegistryLookup: HolderGetter {
    fn registry_key(&self) -> &ResourceKey;
    fn registry_lifecycle(&self) -> Lifecycle;
    fn list_elements(&self) -> Vec<HolderReference>;

    fn list_tags(&self) -> Vec<NamedHolderSet> {
        Vec::new()
    }
}

pub trait HolderLookupProvider {
    fn list_registry_keys(&self) -> Vec<ResourceKey>;
    fn lookup(&self, registry_key: &ResourceKey) -> Option<Rc<dyn RegistryLookup>>;

    fn lookup_or_throw(&self, registry_key: &ResourceKey) -> Rc<dyn RegistryLookup> {
        self.lookup(registry_key)
            .unwrap_or_else(|| panic!("Missing registry: {}", registry_key))
    }
}

pub trait RegistryAccess {
    fn registries(&self) -> Vec<Rc<dyn RegistryLookup>>;
    fn list_registry_keys(&self) -> Vec<ResourceKey> {
        self.registries()
            .into_iter()
            .map(|r| r.registry_key().clone())
            .collect()
    }
}

pub struct SimpleRegistryAccess {
    pub lookups: Vec<Rc<dyn RegistryLookup>>,
}

impl RegistryAccess for SimpleRegistryAccess {
    fn registries(&self) -> Vec<Rc<dyn RegistryLookup>> {
        self.lookups.clone()
    }
}

pub trait Cloner {
    fn clone_value(
        &self,
        value: Erased,
        source: &dyn HolderLookupProvider,
        target: &dyn HolderLookupProvider,
    ) -> Erased;
}

pub trait ClonerFactory {
    fn cloner(&self, registry_key: &ResourceKey) -> Option<Rc<dyn Cloner>>;
}

pub trait BootstrapContext {
    fn register(&mut self, key: ResourceKey, value: Erased, lifecycle: Lifecycle)
        -> HolderReference;
    fn lookup_getter(&self, registry_key: &ResourceKey) -> Rc<dyn HolderGetter>;
}

// ---------------------------------------------------------------------------
// RegistrySetBuilder port
// ---------------------------------------------------------------------------

pub struct RegistrySetBuilder {
    entries: Vec<RegistryStub>,
}

pub type RegistryBootstrapFn = Box<dyn Fn(&mut dyn BootstrapContext)>;

pub struct RegistryStub {
    key: ResourceKey,
    lifecycle: Lifecycle,
    bootstrap: RegistryBootstrapFn,
}

#[derive(Clone)]
pub struct RegisteredValue {
    value: Erased,
    lifecycle: Lifecycle,
}

#[derive(Clone)]
pub struct ValueAndHolder {
    value: RegisteredValue,
    holder: Option<HolderReference>,
}

pub struct RegistryContents {
    key: ResourceKey,
    lifecycle: Lifecycle,
    values: HashMap<ResourceKey, ValueAndHolder>,
}

pub struct PatchedRegistries {
    pub full: Rc<dyn HolderLookupProvider>,
    pub patches: Rc<dyn HolderLookupProvider>,
}

pub struct UniversalOwner {
    token: OwnerToken,
}

pub struct UniversalLookup {
    owner: UniversalOwner,
    holders: RefCell<HashMap<ResourceKey, HolderReference>>,
}

pub struct BuildState {
    owner: UniversalOwner,
    lookup: Rc<UniversalLookup>,
    registries: HashMap<Identifier, Rc<dyn HolderGetter>>,
    registered_values: RefCell<HashMap<ResourceKey, RegisteredValue>>,
    errors: RefCell<Vec<String>>,
}

struct EmptyTagLookup {
    owner: OwnerToken,
    delegate: Rc<dyn HolderGetter>,
}

struct EmptyTagLookupWrapper {
    owner: OwnerToken,
    parent: Rc<dyn RegistryLookup>,
}

struct EmptyTagRegistryLookupFromMap {
    owner: OwnerToken,
    key: ResourceKey,
    lifecycle: Lifecycle,
    elements: HashMap<ResourceKey, HolderReference>,
}

struct ProviderWithContext {
    entries: HashMap<Identifier, ProviderEntry>,
}

struct ProviderEntry {
    lookup: Rc<dyn RegistryLookup>,
}

impl RegistrySetBuilder {
    // JAVA: net.minecraft.core.RegistrySetBuilder#<init>
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#wrapContextLookup
    pub fn wrap_context_lookup(lookup: Rc<dyn RegistryLookup>) -> Rc<dyn HolderGetter> {
        Rc::new(EmptyTagLookup {
            owner: OwnerToken::new(),
            delegate: lookup,
        })
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#lookupFromMap
    pub fn lookup_from_map(
        key: ResourceKey,
        lifecycle: Lifecycle,
        owner: OwnerToken,
        elements: HashMap<ResourceKey, HolderReference>,
    ) -> Rc<dyn RegistryLookup> {
        Rc::new(EmptyTagRegistryLookupFromMap {
            owner,
            key,
            lifecycle,
            elements,
        })
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#add
    pub fn add(
        mut self,
        key: ResourceKey,
        lifecycle: Lifecycle,
        bootstrap: RegistryBootstrapFn,
    ) -> Self {
        self.entries.push(RegistryStub {
            key,
            lifecycle,
            bootstrap,
        });
        self
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#add
    pub fn add_stable(self, key: ResourceKey, bootstrap: RegistryBootstrapFn) -> Self {
        self.add(key, Lifecycle::stable(), bootstrap)
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#createState
    fn create_state(&self, registry_access: &dyn RegistryAccess) -> BuildState {
        let keys = self.entries.iter().map(|e| e.key.clone());
        let mut state = BuildState::create(registry_access, keys);
        for entry in &self.entries {
            entry.apply(&mut state);
        }
        state
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#buildProviderWithContext
    fn build_provider_with_context(
        _owner: &UniversalOwner,
        registry_access: &dyn RegistryAccess,
        new_lookups: Vec<Rc<dyn RegistryLookup>>,
    ) -> Rc<dyn HolderLookupProvider> {
        let mut entries: HashMap<Identifier, ProviderEntry> = HashMap::new();

        for reg in registry_access.registries() {
            let reg_key = reg.registry_key().clone();
            let wrapped: Rc<dyn RegistryLookup> = Rc::new(EmptyTagLookupWrapper {
                owner: OwnerToken::new(),
                parent: reg.clone(),
            });
            entries.insert(
                reg_key.identifier().clone(),
                ProviderEntry { lookup: wrapped },
            );
        }

        for reg in new_lookups {
            let reg_key = reg.registry_key().clone();
            let wrapped: Rc<dyn RegistryLookup> = Rc::new(EmptyTagLookupWrapper {
                owner: OwnerToken::new(),
                parent: reg,
            });
            entries.insert(
                reg_key.identifier().clone(),
                ProviderEntry { lookup: wrapped },
            );
        }

        Rc::new(ProviderWithContext { entries })
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#build
    pub fn build(&self, registry_access: &dyn RegistryAccess) -> Rc<dyn HolderLookupProvider> {
        let state = self.create_state(registry_access);
        let new_lookups: Vec<Rc<dyn RegistryLookup>> = self
            .entries
            .iter()
            .map(|e| e.collect_registered_values(&state).build_as_lookup(&state.owner))
            .collect();
        let provider =
            Self::build_provider_with_context(&state.owner, registry_access, new_lookups);
        state.report_not_collected_holders();
        state.report_unclaimed_registered_values();
        state.throw_on_error();
        provider
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#createLazyFullPatchedRegistries
    fn create_lazy_full_patched_registries(
        &self,
        registry_access: &dyn RegistryAccess,
        original: &Rc<dyn HolderLookupProvider>,
        cloner_factory: &dyn ClonerFactory,
        registry_contents: &HashMap<ResourceKey, RegistryContents>,
        patches: &Rc<dyn HolderLookupProvider>,
    ) -> Rc<dyn HolderLookupProvider> {
        let owner = UniversalOwner::new();
        let holder_provider: Rc<RefCell<Option<Rc<dyn HolderLookupProvider>>>> =
            Rc::new(RefCell::new(None));

        let lookups: Vec<Rc<dyn RegistryLookup>> = registry_contents
            .keys()
            .map(|rk| {
                self.create_lazy_full_patched_registry(
                    &owner,
                    cloner_factory,
                    rk,
                    original,
                    patches,
                    &holder_provider,
                )
            })
            .collect();

        let provider = Self::build_provider_with_context(&owner, registry_access, lookups);
        *holder_provider.borrow_mut() = Some(provider.clone());
        provider
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#createLazyFullPatchedRegistries
    fn create_lazy_full_patched_registry(
        &self,
        owner: &UniversalOwner,
        cloner_factory: &dyn ClonerFactory,
        registry_key: &ResourceKey,
        original: &Rc<dyn HolderLookupProvider>,
        patches: &Rc<dyn HolderLookupProvider>,
        holder_provider: &Rc<RefCell<Option<Rc<dyn HolderLookupProvider>>>>,
    ) -> Rc<dyn RegistryLookup> {
        let cloner = cloner_factory
            .cloner(registry_key)
            .unwrap_or_else(|| panic!("No cloner for {}", registry_key.identifier()));

        let mut elements: HashMap<ResourceKey, HolderReference> = HashMap::new();
        let patch_lookup = patches.lookup_or_throw(registry_key);

        for holder in patch_lookup.list_elements() {
            let Some(elem_key) = holder.key().cloned() else {
                continue;
            };
            let lazy_holder = HolderReference::create_lazy(owner.token.clone(), elem_key.clone());
            let cloner_c = cloner.clone();
            let original_c = original.clone();
            let hp_c = holder_provider.clone();
            let holder_c = holder.clone();
            lazy_holder.set_supplier(Rc::new(move || {
                let target = hp_c.borrow().clone().expect("holder provider not set");
                cloner_c.clone_value(holder_c.value(), original_c.as_ref(), target.as_ref())
            }));
            elements.insert(elem_key, lazy_holder);
        }

        let original_lookup = original.lookup_or_throw(registry_key);
        for holder in original_lookup.list_elements() {
            let Some(elem_key) = holder.key().cloned() else {
                continue;
            };
            elements.entry(elem_key.clone()).or_insert_with(|| {
                let lazy_holder =
                    HolderReference::create_lazy(owner.token.clone(), elem_key.clone());
                let cloner_c = cloner.clone();
                let patches_c = patches.clone();
                let hp_c = holder_provider.clone();
                let holder_c = holder.clone();
                lazy_holder.set_supplier(Rc::new(move || {
                    let target = hp_c.borrow().clone().expect("holder provider not set");
                    cloner_c.clone_value(holder_c.value(), patches_c.as_ref(), target.as_ref())
                }));
                lazy_holder
            });
        }

        let lifecycle = patch_lookup
            .registry_lifecycle()
            .add(original_lookup.registry_lifecycle());
        Self::lookup_from_map(registry_key.clone(), lifecycle, owner.token.clone(), elements)
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder#buildPatch
    pub fn build_patch(
        &self,
        registry_access: &dyn RegistryAccess,
        original: Rc<dyn HolderLookupProvider>,
        cloner_factory: &dyn ClonerFactory,
    ) -> PatchedRegistries {
        let state = self.create_state(registry_access);
        let mut registry_contents: HashMap<ResourceKey, RegistryContents> = HashMap::new();

        for entry in &self.entries {
            let contents = entry.collect_registered_values(&state);
            registry_contents.insert(contents.key.clone(), contents);
        }

        let existing_keys: HashSet<ResourceKey> = registry_access
            .list_registry_keys()
            .into_iter()
            .collect();
        for rk in original.list_registry_keys() {
            if !existing_keys.contains(&rk) {
                registry_contents.entry(rk.clone()).or_insert_with(|| RegistryContents {
                    key: rk,
                    lifecycle: Lifecycle::stable(),
                    values: HashMap::new(),
                });
            }
        }

        let new_lookups: Vec<Rc<dyn RegistryLookup>> = registry_contents
            .values()
            .map(|c| c.build_as_lookup(&state.owner))
            .collect();
        let patches =
            Self::build_provider_with_context(&state.owner, registry_access, new_lookups);
        state.report_unclaimed_registered_values();
        state.throw_on_error();

        let full = self.create_lazy_full_patched_registries(
            registry_access,
            &original,
            cloner_factory,
            &registry_contents,
            &patches,
        );
        PatchedRegistries { full, patches }
    }
}

impl Default for RegistrySetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalOwner {
    // JAVA: net.minecraft.core.RegistrySetBuilder$UniversalOwner#<init>
    pub fn new() -> Self {
        Self {
            token: OwnerToken::new(),
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$UniversalOwner#cast
    pub fn cast_token(&self) -> OwnerToken {
        self.token.clone()
    }
}

impl Default for UniversalOwner {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalLookup {
    // JAVA: net.minecraft.core.RegistrySetBuilder$UniversalLookup#<init>
    pub fn new(owner: UniversalOwner) -> Self {
        Self {
            owner,
            holders: RefCell::new(HashMap::new()),
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$UniversalLookup#getOrCreate
    pub fn get_or_create(&self, key: &ResourceKey) -> HolderReference {
        let mut holders = self.holders.borrow_mut();
        holders
            .entry(key.clone())
            .or_insert_with(|| HolderReference::create_stand_alone(self.owner.token.clone(), key.clone()))
            .clone()
    }

    fn keys(&self) -> Vec<ResourceKey> {
        self.holders.borrow().keys().cloned().collect()
    }
}

impl HolderGetter for UniversalLookup {
    // JAVA: net.minecraft.core.RegistrySetBuilder$UniversalLookup#get
    fn get(&self, key: &ResourceKey) -> Option<HolderReference> {
        Some(self.get_or_create(key))
    }

    fn get_tag(&self, tag: &TagKey) -> Option<NamedHolderSet> {
        Some(NamedHolderSet { key: tag.clone() })
    }
}

impl BuildState {
    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState#create
    pub fn create(
        registry_access: &dyn RegistryAccess,
        new_registry_keys: impl Iterator<Item = ResourceKey>,
    ) -> Self {
        let owner = UniversalOwner::new();
        let lookup = Rc::new(UniversalLookup::new(UniversalOwner {
            token: owner.token.clone(),
        }));
        let mut registries: HashMap<Identifier, Rc<dyn HolderGetter>> = HashMap::new();

        for reg in registry_access.registries() {
            let reg_key = reg.registry_key().clone();
            registries.insert(
                reg_key.identifier().clone(),
                RegistrySetBuilder::wrap_context_lookup(reg),
            );
        }
        for rk in new_registry_keys {
            registries.insert(rk.identifier().clone(), lookup.clone() as Rc<dyn HolderGetter>);
        }

        Self {
            owner,
            lookup,
            registries,
            registered_values: RefCell::new(HashMap::new()),
            errors: RefCell::new(Vec::new()),
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState#bootstrapContext
    pub fn bootstrap_context<'a>(&'a self) -> BuildStateBootstrapContext<'a> {
        BuildStateBootstrapContext { state: self }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState#reportUnclaimedRegisteredValues
    pub fn report_unclaimed_registered_values(&self) {
        let vals = self.registered_values.borrow();
        for (k, _v) in vals.iter() {
            self.errors
                .borrow_mut()
                .push(format!("Orphaned value for key {}", k));
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState#reportNotCollectedHolders
    pub fn report_not_collected_holders(&self) {
        for k in self.lookup.keys() {
            self.errors
                .borrow_mut()
                .push(format!("Unreferenced key: {}", k));
        }
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState#throwOnError
    pub fn throw_on_error(&self) {
        let errors = self.errors.borrow();
        if !errors.is_empty() {
            let mut msg = String::from("Errors during registry creation");
            for e in errors.iter() {
                msg.push_str("; ");
                msg.push_str(e);
            }
            panic!("{}", msg);
        }
    }
}

pub struct BuildStateBootstrapContext<'a> {
    state: &'a BuildState,
}

impl<'a> BootstrapContext for BuildStateBootstrapContext<'a> {
    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState$1#register
    fn register(
        &mut self,
        key: ResourceKey,
        value: Erased,
        lifecycle: Lifecycle,
    ) -> HolderReference {
        let prev = self.state.registered_values.borrow_mut().insert(
            key.clone(),
            RegisteredValue {
                value,
                lifecycle,
            },
        );
        if prev.is_some() {
            self.state
                .errors
                .borrow_mut()
                .push(format!("Duplicate registration for {}", key));
        }
        self.state.lookup.get_or_create(&key)
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$BuildState$1#lookup
    fn lookup_getter(&self, registry_key: &ResourceKey) -> Rc<dyn HolderGetter> {
        self.state
            .registries
            .get(registry_key.identifier())
            .cloned()
            .unwrap_or_else(|| self.state.lookup.clone() as Rc<dyn HolderGetter>)
    }
}

impl RegistryStub {
    // JAVA: net.minecraft.core.RegistrySetBuilder$RegistryStub#apply
    fn apply(&self, state: &mut BuildState) {
        let mut ctx = state.bootstrap_context();
        (self.bootstrap)(&mut ctx);
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$RegistryStub#collectRegisteredValues
    fn collect_registered_values(&self, state: &BuildState) -> RegistryContents {
        let mut values: HashMap<ResourceKey, ValueAndHolder> = HashMap::new();
        let mut registered = state.registered_values.borrow_mut();
        let keys: Vec<ResourceKey> = registered.keys().cloned().collect();
        for k in keys {
            if k.is_for(&self.key) {
                if let Some(rv) = registered.remove(&k) {
                    let holder = state.lookup.holders.borrow_mut().remove(&k);
                    values.insert(
                        k,
                        ValueAndHolder {
                            value: rv,
                            holder,
                        },
                    );
                }
            }
        }
        RegistryContents {
            key: self.key.clone(),
            lifecycle: self.lifecycle,
            values,
        }
    }
}

impl RegistryContents {
    // JAVA: net.minecraft.core.RegistrySetBuilder$RegistryContents#buildAsLookup
    pub fn build_as_lookup(&self, owner: &UniversalOwner) -> Rc<dyn RegistryLookup> {
        let mut map: HashMap<ResourceKey, HolderReference> = HashMap::new();
        for (k, vh) in &self.values {
            let holder = vh
                .holder
                .clone()
                .unwrap_or_else(|| HolderReference::create_stand_alone(owner.token.clone(), k.clone()));
            holder.bind_value(vh.value.value.clone());
            map.insert(k.clone(), holder);
        }
        RegistrySetBuilder::lookup_from_map(
            self.key.clone(),
            self.lifecycle,
            owner.token.clone(),
            map,
        )
    }
}

impl HolderGetter for EmptyTagLookup {
    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookup#get
    fn get(&self, key: &ResourceKey) -> Option<HolderReference> {
        self.delegate.get(key)
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookup#get
    fn get_tag(&self, tag: &TagKey) -> Option<NamedHolderSet> {
        Some(NamedHolderSet { key: tag.clone() })
    }
}

impl HolderGetter for EmptyTagLookupWrapper {
    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookupWrapper#get (delegated via HolderGetter)
    fn get(&self, key: &ResourceKey) -> Option<HolderReference> {
        self.parent.get(key)
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookupWrapper#get (tag, via EmptyTagLookup behavior)
    fn get_tag(&self, tag: &TagKey) -> Option<NamedHolderSet> {
        Some(NamedHolderSet { key: tag.clone() })
    }
}

impl RegistryLookup for EmptyTagLookupWrapper {
    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookupWrapper#parent
    fn registry_key(&self) -> &ResourceKey {
        self.parent.registry_key()
    }

    fn registry_lifecycle(&self) -> Lifecycle {
        self.parent.registry_lifecycle()
    }

    fn list_elements(&self) -> Vec<HolderReference> {
        self.parent.list_elements()
    }

    fn list_tags(&self) -> Vec<NamedHolderSet> {
        // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagRegistryLookup#listTags
        panic!("Tags are not available in datagen")
    }
}

impl HolderGetter for EmptyTagRegistryLookupFromMap {
    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookup#get
    fn get(&self, key: &ResourceKey) -> Option<HolderReference> {
        self.elements.get(key).cloned()
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagLookup#get
    fn get_tag(&self, tag: &TagKey) -> Option<NamedHolderSet> {
        Some(NamedHolderSet { key: tag.clone() })
    }
}

impl RegistryLookup for EmptyTagRegistryLookupFromMap {
    fn registry_key(&self) -> &ResourceKey {
        &self.key
    }

    fn registry_lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }

    fn list_elements(&self) -> Vec<HolderReference> {
        self.elements.values().cloned().collect()
    }

    // JAVA: net.minecraft.core.RegistrySetBuilder$EmptyTagRegistryLookup#listTags
    fn list_tags(&self) -> Vec<NamedHolderSet> {
        panic!("Tags are not available in datagen")
    }
}

impl HolderLookupProvider for ProviderWithContext {
    fn list_registry_keys(&self) -> Vec<ResourceKey> {
        self.entries.values().map(|e| e.lookup.registry_key().clone()).collect()
    }

    fn lookup(&self, registry_key: &ResourceKey) -> Option<Rc<dyn RegistryLookup>> {
        self.entries
            .get(registry_key.identifier())
            .map(|e| e.lookup.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn id(s: &str) -> Identifier {
        Identifier::new(s)
    }

    fn reg_key(reg: &str, loc: &str) -> ResourceKey {
        ResourceKey::new(id(reg), id(loc))
    }

    fn registry_key(name: &str) -> ResourceKey {
        // ResourceKey for a registry itself: registry="minecraft:root", location=name
        ResourceKey::new(id("minecraft:root"), id(name))
    }

    fn element_key(registry_name: &str, path: &str) -> ResourceKey {
        ResourceKey::new(id(registry_name), id(path))
    }

    struct EmptyAccess;
    impl RegistryAccess for EmptyAccess {
        fn registries(&self) -> Vec<Rc<dyn RegistryLookup>> {
            Vec::new()
        }
    }

    struct SimpleLookup {
        key: ResourceKey,
        lifecycle: Lifecycle,
        elements: Vec<HolderReference>,
    }
    impl HolderGetter for SimpleLookup {
        fn get(&self, key: &ResourceKey) -> Option<HolderReference> {
            self.elements.iter().find(|h| h.key() == Some(key)).cloned()
        }
    }
    impl RegistryLookup for SimpleLookup {
        fn registry_key(&self) -> &ResourceKey { &self.key }
        fn registry_lifecycle(&self) -> Lifecycle { self.lifecycle }
        fn list_elements(&self) -> Vec<HolderReference> { self.elements.clone() }
    }

    struct PassthroughCloner;
    impl Cloner for PassthroughCloner {
        fn clone_value(&self, value: Erased, _source: &dyn HolderLookupProvider, _target: &dyn HolderLookupProvider) -> Erased {
            value
        }
    }
    struct PassthroughClonerFactory;
    impl ClonerFactory for PassthroughClonerFactory {
        fn cloner(&self, _registry_key: &ResourceKey) -> Option<Rc<dyn Cloner>> {
            Some(Rc::new(PassthroughCloner))
        }
    }

    #[test]
    fn build_registers_and_resolves_elements() {
        let dim_reg = registry_key("minecraft:dimension_type");
        let k1 = element_key("minecraft:dimension_type", "minecraft:overworld");
        let k1c = k1.clone();
        let builder = RegistrySetBuilder::new().add_stable(
            dim_reg.clone(),
            Box::new(move |ctx: &mut dyn BootstrapContext| {
                ctx.register(k1c.clone(), Rc::new(42_i32) as Erased, Lifecycle::stable());
            }),
        );
        let provider = builder.build(&EmptyAccess);
        let lookup = provider.lookup(&dim_reg).expect("lookup");
        let holder = lookup.get(&k1).expect("holder");
        let val = holder.value();
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
        assert_eq!(lookup.list_elements().len(), 1);
    }

    #[test]
    fn duplicate_registration_reports_error() {
        let reg = registry_key("minecraft:test");
        let k = element_key("minecraft:test", "minecraft:a");
        let k1 = k.clone();
        let k2 = k.clone();
        let builder = RegistrySetBuilder::new().add_stable(
            reg,
            Box::new(move |ctx: &mut dyn BootstrapContext| {
                ctx.register(k1.clone(), Rc::new(1_i32) as Erased, Lifecycle::stable());
                ctx.register(k2.clone(), Rc::new(2_i32) as Erased, Lifecycle::stable());
            }),
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = builder.build(&EmptyAccess);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn unreferenced_holder_reports_error() {
        let reg = registry_key("minecraft:test");
        let missing = element_key("minecraft:test", "minecraft:missing");
        let builder = RegistrySetBuilder::new().add_stable(
            reg.clone(),
            Box::new(move |ctx: &mut dyn BootstrapContext| {
                let getter = ctx.lookup_getter(&reg);
                // Request a key but never register it -> unreferenced
                let _ = getter.get(&missing);
            }),
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = builder.build(&EmptyAccess);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn lifecycle_add_matches_java_semantics() {
        assert_eq!(Lifecycle::stable().add(Lifecycle::stable()), Lifecycle::Stable);
        assert_eq!(Lifecycle::stable().add(Lifecycle::experimental()), Lifecycle::Experimental);
        assert_eq!(Lifecycle::experimental().add(Lifecycle::stable()), Lifecycle::Experimental);
        assert_eq!(Lifecycle::experimental().add(Lifecycle::experimental()), Lifecycle::Experimental);
    }

    #[test]
    fn build_patch_creates_lazy_holders_and_preserves_values() {
        let reg = registry_key("minecraft:test");
        let k_patch = element_key("minecraft:test", "minecraft:patched");
        let k_orig = element_key("minecraft:test", "minecraft:original");
        let k_orig_for_boot = k_orig.clone();

        let owner = OwnerToken::new();
        let orig_holder = HolderReference::create_stand_alone(owner.clone(), k_orig.clone());
        orig_holder.bind_value(Rc::new(100_i32) as Erased);
        let orig_lookup: Rc<dyn RegistryLookup> = Rc::new(SimpleLookup {
            key: reg.clone(),
            lifecycle: Lifecycle::stable(),
            elements: vec![orig_holder],
        });
        let orig_access = SimpleRegistryAccess { lookups: vec![orig_lookup.clone()] };
        let orig_provider: Rc<dyn HolderLookupProvider> = Rc::new(ProviderWithContext {
            entries: {
                let mut m = HashMap::new();
                m.insert(reg.identifier().clone(), ProviderEntry { lookup: orig_lookup });
                m
            },
        });

        let k_patch_c = k_patch.clone();
        let builder = RegistrySetBuilder::new().add_stable(
            reg.clone(),
            Box::new(move |ctx: &mut dyn BootstrapContext| {
                ctx.register(k_patch_c.clone(), Rc::new(7_i32) as Erased, Lifecycle::stable());
                let _ = k_orig_for_boot.clone(); // ensure key is in scope; original remains
            }),
        );

        let patched = builder.build_patch(&orig_access, orig_provider, &PassthroughClonerFactory);
        let full_lookup = patched.full.lookup(&reg).expect("full lookup");
        let patched_val = full_lookup.get(&k_patch).expect("patched").value();
        assert_eq!(*patched_val.downcast_ref::<i32>().unwrap(), 7);
        let orig_val = full_lookup.get(&k_orig).expect("original").value();
        assert_eq!(*orig_val.downcast_ref::<i32>().unwrap(), 100);
    }

    #[test]
    fn resource_key_is_for_uses_registry_location() {
        let reg = registry_key("minecraft:worldgen/biome");
        let elem = element_key("minecraft:worldgen/biome", "minecraft:plains");
        let other = element_key("minecraft:dimension_type", "minecraft:overworld");
        assert!(elem.is_for(&reg));
        assert!(!other.is_for(&reg));
    }
}
