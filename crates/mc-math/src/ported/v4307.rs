use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub type TypeTemplateSupplier = Arc<dyn Fn() -> TypeTemplate>;

#[derive(Clone)]
pub enum TypeTemplate {
    Empty,
    Reference(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Fields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFieldsLazy(Vec<(String, TypeTemplateSupplier)>),
}

impl std::fmt::Debug for TypeTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Reference(r) => f.debug_tuple("Reference").field(r).finish(),
            Self::OptionalFields(v) => f.debug_tuple("OptionalFields").field(v).finish(),
            Self::Fields(v) => f.debug_tuple("Fields").field(v).finish(),
            Self::List(t) => f.debug_tuple("List").field(t).finish(),
            Self::Or(a, b) => f.debug_tuple("Or").field(a).field(b).finish(),
            Self::OptionalFieldsLazy(v) => {
                let resolved: Vec<_> = v.iter().map(|(k, s)| (k.clone(), s())).collect();
                f.debug_tuple("OptionalFieldsLazy").field(&resolved).finish()
            }
        }
    }
}

impl PartialEq for TypeTemplate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Reference(a), Self::Reference(b)) => a == b,
            (Self::OptionalFields(a), Self::OptionalFields(b)) => a == b,
            (Self::Fields(a), Self::Fields(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Or(a, b), Self::Or(c, d)) => a == c && b == d,
            (Self::OptionalFieldsLazy(a), Self::OptionalFieldsLazy(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|((ka, va), (kb, vb))| ka == kb && va() == vb())
            }
            _ => false,
        }
    }
}

impl Eq for TypeTemplate {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference(pub &'static str);

impl Reference {
    // JAVA: com.mojang.datafixers.DSL.TypeReference#in
    pub fn in_schema(self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.0.to_string())
    }
}

pub struct References;
impl References {
    pub const BLOCK_NAME: Reference = Reference("minecraft:block_name");
    pub const DATA_COMPONENTS: Reference = Reference("minecraft:data_components");
}

pub struct Dsl;
impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(fields)
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }

    // JAVA: com.mojang.datafixers.DSL#or
    pub fn or(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(a), Box::new(b))
    }

    // JAVA: com.mojang.datafixers.DSL#optionalFieldsLazy
    pub fn optional_fields_lazy(fields: Vec<(String, TypeTemplateSupplier)>) -> TypeTemplate {
        TypeTemplate::OptionalFieldsLazy(fields)
    }
}

#[derive(Clone, Default)]
pub struct SequencedMap {
    entries: Vec<(String, TypeTemplateSupplier)>,
}

impl SequencedMap {
    // JAVA: java.util.SequencedMap#new (construction helper)
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: java.util.Map#put
    pub fn put(&mut self, key: impl Into<String>, value: impl Fn() -> TypeTemplate + 'static) {
        let key = key.into();
        let value: TypeTemplateSupplier = Arc::new(value);
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == &key) {
            self.entries[pos] = (key, value);
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn into_entries(self) -> Vec<(String, TypeTemplateSupplier)> {
        self.entries
    }

    pub fn entries(&self) -> &[(String, TypeTemplateSupplier)] {
        &self.entries
    }
}

#[derive(Clone, Default)]
pub struct Schema {
    pub version: i32,
    registered_types: Rc<RefCell<Vec<(bool, Reference, TypeTemplateSupplier)>>>,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#<init>
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self {
            version,
            registered_types: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerTypes
    pub fn register_types(
        &self,
        _schema: &Schema,
        _entity_types: &mut HashMap<String, TypeTemplateSupplier>,
        _block_entity_types: &mut HashMap<String, TypeTemplateSupplier>,
    ) {
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type(
        &self,
        recursive: bool,
        reference: Reference,
        template: impl Fn() -> TypeTemplate + 'static,
    ) {
        self.registered_types
            .borrow_mut()
            .push((recursive, reference, Arc::new(template)));
    }

    pub fn registered_types(&self) -> Vec<(bool, Reference, TypeTemplateSupplier)> {
        self.registered_types.borrow().clone()
    }
}

/// Minimal stub for `net.minecraft.util.datafix.schemas.V4059`.
pub struct V4059;
impl V4059 {
    // JAVA: net.minecraft.util.datafix.schemas.V4059#components
    pub fn components(_schema: &Schema) -> SequencedMap {
        SequencedMap::new()
    }
}

/// Port of `net.minecraft.util.datafix.schemas.V4307`.
/// Extends `NamespacedSchema` in Java.
pub struct V4307 {
    pub version: i32,
    pub parent: Schema,
}

impl V4307 {
    // JAVA: net.minecraft.util.datafix.schemas.V4307#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4307#components
    pub fn components(schema: &Schema) -> SequencedMap {
        let mut map = V4059::components(schema);
        let s = schema.clone();
        map.put("minecraft:can_place_on", move || {
            Self::adventure_mode_predicate(&s)
        });
        let s = schema.clone();
        map.put("minecraft:can_break", move || {
            Self::adventure_mode_predicate(&s)
        });
        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4307#adventureModePredicate
    fn adventure_mode_predicate(schema: &Schema) -> TypeTemplate {
        let t1 = Dsl::optional_fields(vec![(
            "blocks".to_string(),
            Dsl::or(
                References::BLOCK_NAME.in_schema(schema),
                Dsl::list(References::BLOCK_NAME.in_schema(schema)),
            ),
        )]);
        Dsl::or(t1.clone(), Dsl::list(t1))
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4307#registerTypes
    pub fn register_types(
        &self,
        schema: &Schema,
        entity_types: &mut HashMap<String, TypeTemplateSupplier>,
        block_entity_types: &mut HashMap<String, TypeTemplateSupplier>,
    ) {
        self.parent
            .register_types(schema, entity_types, block_entity_types);
        let s = schema.clone();
        schema.register_type(true, References::DATA_COMPONENTS, move || {
            Dsl::optional_fields_lazy(Self::components(&s).into_entries())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_schema() -> Schema {
        Schema::new(0, Schema::default())
    }

    #[test]
    fn components_adds_can_place_on_and_can_break() {
        let schema = base_schema();
        let map = V4307::components(&schema);
        let entries = map.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "minecraft:can_place_on");
        assert_eq!(entries[1].0, "minecraft:can_break");
        assert_eq!(entries[0].1(), entries[1].1());
    }

    #[test]
    fn adventure_mode_predicate_shape() {
        let schema = base_schema();
        let map = V4307::components(&schema);
        let template = map.entries()[0].1();
        let block_ref = TypeTemplate::Reference("minecraft:block_name".to_string());
        let inner = TypeTemplate::OptionalFields(vec![(
            "blocks".to_string(),
            TypeTemplate::Or(
                Box::new(block_ref.clone()),
                Box::new(TypeTemplate::List(Box::new(block_ref))),
            ),
        )]);
        let expected = TypeTemplate::Or(Box::new(inner.clone()), Box::new(TypeTemplate::List(Box::new(inner))));
        assert_eq!(template, expected);
    }

    #[test]
    fn register_types_registers_data_components() {
        let schema = base_schema();
        let v = V4307::new(4307, Schema::default());
        let mut entities = HashMap::new();
        let mut block_entities = HashMap::new();
        v.register_types(&schema, &mut entities, &mut block_entities);
        let registered = schema.registered_types();
        assert_eq!(registered.len(), 1);
        assert!(registered[0].0);
        assert_eq!(registered[0].1, References::DATA_COMPONENTS);
        let template = (registered[0].2)();
        if let TypeTemplate::OptionalFieldsLazy(fields) = template {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "minecraft:can_place_on");
            assert_eq!(fields[1].0, "minecraft:can_break");
        } else {
            panic!("expected OptionalFieldsLazy");
        }
    }

    #[test]
    fn constructor_sets_version_and_parent() {
        let parent = Schema::new(4059, Schema::default());
        let v = V4307::new(4307, parent);
        assert_eq!(v.version, 4307);
        assert_eq!(v.parent.version, 4059);
    }
}
