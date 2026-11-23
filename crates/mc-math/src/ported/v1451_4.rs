use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Unit,
    Const(Box<TypeTemplate>),
    NamespacedString,
}

pub type TypeTemplateSupplier = Arc<dyn Fn() -> TypeTemplate + Send + Sync>;

pub struct References;

impl References {
    pub const BLOCK_NAME: &'static str = "block_name";
}

pub struct Dsl;

impl Dsl {
    pub fn const_type(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Const(Box::new(template))
    }
}

#[derive(Debug, Default)]
pub struct Schema {
    pub version_key: i32,
    pub type_registrations: Vec<(bool, String, TypeTemplate)>,
}

impl Schema {
    pub fn new(version_key: i32) -> Self {
        Self {
            version_key,
            type_registrations: Vec::new(),
        }
    }

    pub fn register_type(
        &mut self,
        recursive: bool,
        name: &str,
        supplier: TypeTemplateSupplier,
    ) {
        self.type_registrations
            .push((recursive, name.to_string(), supplier()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1451_4 {
    pub version_key: i32,
}

impl V1451_4 {
    // JAVA: net.minecraft.util.datafix.schemas.V1451_4#<init>
    pub fn new(version_key: i32, _parent: Option<&Schema>) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#namespacedString
    pub fn namespaced_string(&self) -> TypeTemplate {
        TypeTemplate::NamespacedString
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1451_4#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        _entity_types: &mut HashMap<String, TypeTemplateSupplier>,
        _block_entity_types: &mut HashMap<String, TypeTemplateSupplier>,
    ) {
        // super.registerTypes(schema, entityTypes, blockEntityTypes);
        let ns = self.namespaced_string();
        schema.register_type(
            false,
            References::BLOCK_NAME,
            Arc::new(move || Dsl::const_type(ns.clone())),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_key_including_edges() {
        let v = V1451_4::new(1451, None);
        assert_eq!(v.version_key, 1451);
        assert_eq!(V1451_4::new(i32::MIN, None).version_key, i32::MIN);
        assert_eq!(V1451_4::new(0, None).version_key, 0);
    }

    #[test]
    fn register_types_registers_block_name_non_recursive() {
        let v = V1451_4::new(1451, None);
        let mut schema = Schema::new(1451);
        let mut entities = HashMap::new();
        let mut block_entities = HashMap::new();

        v.register_types(&mut schema, &mut entities, &mut block_entities);

        assert_eq!(schema.type_registrations.len(), 1);
        let (recursive, name, template) = &schema.type_registrations[0];
        assert!(!(*recursive));
        assert_eq!(name, "block_name");
        assert_eq!(
            template,
            &TypeTemplate::Const(Box::new(TypeTemplate::NamespacedString))
        );
    }

    #[test]
    fn register_types_does_not_modify_input_maps() {
        let v = V1451_4::new(1451, None);
        let mut schema = Schema::new(1451);
        let mut entities = HashMap::new();
        entities.insert("minecraft:foo".to_string(), Arc::new(|| TypeTemplate::Unit) as TypeTemplateSupplier);
        let mut block_entities = HashMap::new();

        v.register_types(&mut schema, &mut entities, &mut block_entities);

        assert_eq!(entities.len(), 1);
        assert!(entities.contains_key("minecraft:foo"));
        assert!(block_entities.is_empty());
    }

    #[test]
    fn namespaced_string_returns_expected_template() {
        let v = V1451_4::new(-1451, None);
        assert_eq!(v.namespaced_string(), TypeTemplate::NamespacedString);
    }
}
