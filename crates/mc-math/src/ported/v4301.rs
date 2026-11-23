use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    ItemStack,
    EntityEquipment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    Ref(Reference),
    OptionalField(Box<(String, TypeTemplate)>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    version: i32,
    types: BTreeMap<String, (bool, TypeTemplate)>,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32) -> Self {
        Self {
            version,
            types: BTreeMap::new(),
        }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type<F>(&mut self, recursive: bool, reference: Reference, template_factory: F)
    where
        F: FnOnce(&Schema) -> TypeTemplate,
    {
        let template = template_factory(&*self);
        let key = format!("{:?}", reference);
        self.types.insert(key, (recursive, template));
    }

    pub fn get_type(&self, reference: &Reference) -> Option<&(bool, TypeTemplate)> {
        let key = format!("{:?}", reference);
        self.types.get(&key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespacedSchema {
    schema: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, _parent: Option<&Schema>) -> Self {
        Self {
            schema: Schema::new(version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &mut self,
        _schema: &Schema,
        _entity_types: &mut BTreeMap<String, TypeTemplate>,
        _block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn schema_mut(&mut self) -> &mut Schema {
        &mut self.schema
    }
}

pub struct V4301 {
    base: NamespacedSchema,
}

impl V4301 {
    // JAVA: net.minecraft.util.datafix.schemas.V4301#<init>(int,Schema)
    pub fn new(version: i32, parent: Option<&Schema>) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4301#registerTypes(Schema,Map,Map)
    pub fn register_types(
        &mut self,
        schema_snapshot: &Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.base
            .register_types(schema_snapshot, entity_types, block_entity_types);
        self.base.schema_mut().register_type(true, Reference::EntityEquipment, |_s| {
            TypeTemplate::OptionalField(Box::new((
                "equipment".to_string(),
                TypeTemplate::OptionalFields(vec![
                    ("mainhand".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("offhand".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("feet".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("legs".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("chest".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("head".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("body".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                    ("saddle".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ]),
            )))
        });
    }

    pub fn schema(&self) -> &Schema {
        self.base.schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equipment_template() -> TypeTemplate {
        TypeTemplate::OptionalField(Box::new((
            "equipment".to_string(),
            TypeTemplate::OptionalFields(vec![
                ("mainhand".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("offhand".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("feet".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("legs".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("chest".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("head".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("body".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
                ("saddle".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
            ]),
        )))
    }

    #[test]
    fn test_register_types_registers_entity_equipment() {
        let mut v4301 = V4301::new(4301, None);
        let schema_snapshot = Schema::new(4301);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v4301.register_types(&schema_snapshot, &mut entities, &mut block_entities);
        let ty = v4301.schema().get_type(&Reference::EntityEquipment).expect("type missing");
        assert!(ty.0, "entity equipment type should be recursive");
        assert_eq!(ty.1, equipment_template());
    }

    #[test]
    fn test_equipment_has_all_slots() {
        let mut v4301 = V4301::new(4301, None);
        let schema_snapshot = Schema::new(4301);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v4301.register_types(&schema_snapshot, &mut entities, &mut block_entities);
        let (_, template) = v4301.schema().get_type(&Reference::EntityEquipment).unwrap();
        if let TypeTemplate::OptionalField(boxed) = template {
            let (name, inner) = &**boxed;
            assert_eq!(name, "equipment");
            if let TypeTemplate::OptionalFields(fields) = inner {
                assert_eq!(fields.len(), 8);
                let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                assert_eq!(
                    names,
                    vec!["mainhand", "offhand", "feet", "legs", "chest", "head", "body", "saddle"]
                );
            } else {
                panic!("expected OptionalFields");
            }
        } else {
            panic!("expected OptionalField");
        }
    }

    #[test]
    fn test_register_types_does_not_modify_entity_maps() {
        let mut v4301 = V4301::new(4301, None);
        let schema_snapshot = Schema::new(4301);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v4301.register_types(&schema_snapshot, &mut entities, &mut block_entities);
        assert!(entities.is_empty());
        assert!(block_entities.is_empty());
    }

    #[test]
    fn test_schema_version() {
        let v4301 = V4301::new(4301, None);
        assert_eq!(v4301.schema().version(), 4301);
    }
}
