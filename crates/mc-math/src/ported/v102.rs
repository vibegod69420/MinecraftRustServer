use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reference(pub &'static str);

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref(self.0.to_string())
    }
}

pub struct References;

impl References {
    pub const ITEM_STACK: Reference = Reference("minecraft:item_stack");
    pub const ITEM_NAME: Reference = Reference("minecraft:item_name");
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Named(String),
    Ref(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Hook {
        template: Box<TypeTemplate>,
        add: String,
        function: String,
    },
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        )
    }

    pub fn hook(template: TypeTemplate, add: &str, function: &str) -> TypeTemplate {
        TypeTemplate::Hook {
            template: Box::new(template),
            add: add.to_string(),
            function: function.to_string(),
        }
    }
}

pub struct HookFunction;

impl HookFunction {
    pub const IDENTITY: &'static str = "identity";
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
    pub registered_types: BTreeMap<String, (bool, TypeTemplate)>,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            registered_types: BTreeMap::new(),
        }
    }

    pub fn register_type(
        &mut self,
        recursive: bool,
        reference: &Reference,
        template: TypeTemplate,
    ) {
        self.registered_types
            .insert(reference.0.to_string(), (recursive, template));
    }
}

pub struct V99;

impl V99 {
    pub const ADD_NAMES: &'static str = "V99.addNames";

    // JAVA: net.minecraft.util.datafix.schemas.V99#itemStackTag(Schema)
    pub fn item_stack_tag(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Named("V99.itemStackTag".to_string())
    }
}

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct V102 {
    version: i32,
}

impl V102 {
    // JAVA: net.minecraft.util.datafix.schemas.V102#<init>(int, Schema)
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V102#registerTypes(Schema, Map, Map)
    pub fn register_types(
        &self,
        schema: &mut Schema,
        types: &mut BTreeMap<String, TypeTemplateSupplier>,
        entity_types: &mut BTreeMap<String, TypeTemplateSupplier>,
    ) {
        // super.registerTypes(schema, types, entity_types) - base Schema is no-op
        let _ = (types, entity_types);
        let id_ref = References::ITEM_NAME.in_schema(schema);
        let tag = V99::item_stack_tag(schema);
        let template = Dsl::hook(
            Dsl::optional_fields(vec![("id", id_ref), ("tag", tag)]),
            V99::ADD_NAMES,
            HookFunction::IDENTITY,
        );
        schema.register_type(true, &References::ITEM_STACK, template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v102_new_stores_version() {
        let v = V102::new(102, Schema::new(99));
        assert_eq!(v.version, 102);
    }

    #[test]
    fn v102_registers_item_stack_type() {
        let mut schema = Schema::new(102);
        let mut types = BTreeMap::new();
        let mut entity_types = BTreeMap::new();
        let v = V102::new(102, Schema::new(99));
        v.register_types(&mut schema, &mut types, &mut entity_types);

        assert!(types.is_empty());
        assert!(entity_types.is_empty());
        let (recursive, _) = schema
            .registered_types
            .get("minecraft:item_stack")
            .expect("item_stack registered");
        assert!(*recursive);
    }

    #[test]
    fn v102_item_stack_uses_hook_and_optional_fields() {
        let mut schema = Schema::new(102);
        let mut types = BTreeMap::new();
        let mut entity_types = BTreeMap::new();
        V102::new(102, Schema::new(99)).register_types(&mut schema, &mut types, &mut entity_types);

        let (_, tpl) = &schema.registered_types["minecraft:item_stack"];
        match tpl {
            TypeTemplate::Hook {
                template,
                add,
                function,
            } => {
                assert_eq!(add, "V99.addNames");
                assert_eq!(function, "identity");
                match template.as_ref() {
                    TypeTemplate::OptionalFields(fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "id");
                        assert_eq!(
                            fields[0].1,
                            TypeTemplate::Ref("minecraft:item_name".to_string())
                        );
                        assert_eq!(fields[1].0, "tag");
                        assert_eq!(
                            fields[1].1,
                            TypeTemplate::Named("V99.itemStackTag".to_string())
                        );
                    }
                    other => panic!("expected OptionalFields, got {:?}", other),
                }
            }
            other => panic!("expected Hook, got {:?}", other),
        }
    }

    #[test]
    fn v102_item_stack_tag_reference() {
        let schema = Schema::new(102);
        let tag = V99::item_stack_tag(&schema);
        assert_eq!(tag, TypeTemplate::Named("V99.itemStackTag".to_string()));
    }
}
