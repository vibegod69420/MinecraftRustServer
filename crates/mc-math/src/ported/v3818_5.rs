use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reference {
    ItemStack,
    ItemName,
    DataComponents,
}

pub struct References;

impl References {
    pub const ITEM_STACK: Reference = Reference::ItemStack;
    pub const ITEM_NAME: Reference = Reference::ItemName;
    pub const DATA_COMPONENTS: Reference = Reference::DataComponents;
}

impl Reference {
    // JAVA: com.mojang.datafixers.types.templates.TypeTemplate (reference via References#in)
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    Reference(Reference),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            fields
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }
}

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Debug, Default)]
pub struct Schema {
    pub type_registrations: RefCell<Vec<(bool, Reference, TypeTemplate)>>,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type<F>(&self, recursive: bool, reference: Reference, template: F)
    where
        F: FnOnce() -> TypeTemplate,
    {
        let tpl = template();
        self.type_registrations
            .borrow_mut()
            .push((recursive, reference, tpl));
    }
}

pub struct NamespacedSchema {
    pub version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, _parent: &Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &self,
        _schema: &Schema,
        _entities: &mut BTreeMap<String, TypeTemplateSupplier>,
        _block_entities: &mut BTreeMap<String, TypeTemplateSupplier>,
    ) {
    }
}

pub struct V3818_5 {
    pub base: NamespacedSchema,
}

impl V3818_5 {
    // JAVA: net.minecraft.util.datafix.schemas.V3818_5#<init>
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3818_5#registerTypes
    pub fn register_types(
        &self,
        schema: &Schema,
        entities: &mut BTreeMap<String, TypeTemplateSupplier>,
        block_entities: &mut BTreeMap<String, TypeTemplateSupplier>,
    ) {
        self.base.register_types(schema, entities, block_entities);
        schema.register_type(true, References::ITEM_STACK, || {
            Dsl::optional_fields(vec![
                ("id", References::ITEM_NAME.in_schema(schema)),
                ("components", References::DATA_COMPONENTS.in_schema(schema)),
            ])
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_edge_cases() {
        let schema = Schema::default();
        assert_eq!(V3818_5::new(i32::MIN, &schema).base.version, i32::MIN);
        assert_eq!(V3818_5::new(i32::MAX, &schema).base.version, i32::MAX);
        assert_eq!(V3818_5::new(-5, &schema).base.version, -5);
    }

    #[test]
    fn register_types_registers_item_stack_recursively() {
        let schema = Schema::default();
        let v = V3818_5::new(3818, &schema);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&schema, &mut entities, &mut block_entities);

        let regs = schema.type_registrations.borrow();
        assert_eq!(regs.len(), 1);
        assert!(regs[0].0);
        assert_eq!(regs[0].1, Reference::ItemStack);
    }

    #[test]
    fn item_stack_template_has_id_and_components_fields() {
        let schema = Schema::default();
        let v = V3818_5::new(3818, &schema);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&schema, &mut entities, &mut block_entities);

        let regs = schema.type_registrations.borrow();
        assert_eq!(
            regs[0].2,
            TypeTemplate::OptionalFields(vec![
                (
                    "id".to_string(),
                    TypeTemplate::Reference(Reference::ItemName),
                ),
                (
                    "components".to_string(),
                    TypeTemplate::Reference(Reference::DataComponents),
                ),
            ])
        );
    }

    #[test]
    fn register_types_preserves_input_maps() {
        let schema = Schema::default();
        let v = V3818_5::new(3818, &schema);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        block_entities.insert(
            "pre_be".to_string(),
            Box::new(|| TypeTemplate::Empty) as TypeTemplateSupplier,
        );
        v.register_types(&schema, &mut entities, &mut block_entities);
        assert!(entities.is_empty());
        assert_eq!(block_entities.len(), 1);
        assert!(block_entities.contains_key("pre_be"));
    }

    #[test]
    fn version_float_bits_and_wrapping_edge_cases() {
        let schema = Schema::default();
        let nan_bits = f64::NAN.to_bits() as i32;
        assert_eq!(V3818_5::new(nan_bits, &schema).base.version, nan_bits);

        let inf_bits = f32::INFINITY.to_bits() as i32;
        assert_eq!(V3818_5::new(inf_bits, &schema).base.version, inf_bits);

        let wrapped = i32::MAX.wrapping_add(2);
        assert_eq!(wrapped, i32::MIN.wrapping_add(1));
        assert_eq!(V3818_5::new(wrapped, &schema).base.version, wrapped);
    }
}
