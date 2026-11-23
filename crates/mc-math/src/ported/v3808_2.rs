use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    OptionalFields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
    Ref {
        reference: &'static str,
        schema_version: i32,
    },
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeReference(pub &'static str);

impl TypeReference {
    // JAVA: com.mojang.datafixers.DSL$TypeReference#in
    pub fn in_schema(self, schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref {
            reference: self.0,
            schema_version: schema.version,
        }
    }
}

pub struct References;

impl References {
    pub const ITEM_STACK: TypeReference = TypeReference("ITEM_STACK");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
    pub parent_version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
        template: TypeTemplate,
    ) {
        map.insert(name.to_string(), template);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V3808_2 {
    pub base: NamespacedSchema,
}

impl V3808_2 {
    // JAVA: net.minecraft.util.datafix.schemas.V3808_2#V3808_2
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3808_2#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_entities(schema);
        schema.register(
            &mut map,
            "minecraft:trader_llama",
            Dsl::optional_fields(vec![
                (
                    "Items",
                    Dsl::list(References::ITEM_STACK.in_schema(schema)),
                ),
                ("SaddleItem", References::ITEM_STACK.in_schema(schema)),
            ]),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parent_schema() -> Schema {
        Schema::new(3807, 0)
    }

    #[test]
    fn registers_trader_llama_structure() {
        let parent = parent_schema();
        let schema = V3808_2::new(3808, &parent);
        let map = schema.register_entities(&Schema::new(3808, 3807));

        let template = map.get("minecraft:trader_llama").expect("missing trader_llama");
        match template {
            TypeTemplate::OptionalFields(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "Items");
                assert_eq!(fields[1].0, "SaddleItem");
                assert_eq!(
                    fields[0].1,
                    TypeTemplate::List(Box::new(TypeTemplate::Ref {
                        reference: "ITEM_STACK",
                        schema_version: 3808,
                    }))
                );
                assert_eq!(
                    fields[1].1,
                    TypeTemplate::Ref {
                        reference: "ITEM_STACK",
                        schema_version: 3808,
                    }
                );
            }
            _ => panic!("expected OptionalFields"),
        }
    }

    #[test]
    fn negative_version_edge_case() {
        let parent = Schema::new(-1, i32::MIN);
        let schema = V3808_2::new(-100, &parent);
        assert_eq!(schema.base.version, -100);
        assert_eq!(schema.base.parent_version, -1);

        let map = schema.register_entities(&Schema::new(-100, -1));
        let template = map.get("minecraft:trader_llama").expect("missing trader_llama");
        if let TypeTemplate::OptionalFields(fields) = template {
            if let TypeTemplate::Ref { schema_version, .. } = &fields[1].1 {
                assert_eq!(*schema_version, -100);
            } else {
                panic!("expected ref");
            }
        } else {
            panic!("expected OptionalFields");
        }
    }

    #[test]
    fn overflow_version_edge_case() {
        let parent = Schema::new(i32::MAX, i32::MIN);
        let schema = V3808_2::new(i32::MAX, &parent);
        let target = Schema::new(i32::MAX, i32::MIN);
        let map = schema.register_entities(&target);

        let template = map.get("minecraft:trader_llama").expect("missing trader_llama");
        if let TypeTemplate::OptionalFields(fields) = template {
            if let TypeTemplate::List(inner) = &fields[0].1 {
                if let TypeTemplate::Ref {
                    schema_version,
                    reference,
                } = inner.as_ref()
                {
                    assert_eq!(*schema_version, i32::MAX);
                    assert_eq!(*reference, "ITEM_STACK");
                } else {
                    panic!("expected ref in list");
                }
            } else {
                panic!("expected list");
            }
        } else {
            panic!("expected OptionalFields");
        }
    }

    #[test]
    fn version_float_bits_roundtrip() {
        let v = f32::from_bits(0xBF80_0000u32).to_bits() as i32; // -1.0f32 bits
        let parent = Schema::new(v, 0);
        let schema = V3808_2::new(v, &parent);
        assert_eq!(schema.base.version, v);
        assert_eq!(f32::from_bits(schema.base.version as u32).to_bits() as i32, v);
    }

    #[test]
    fn register_overwrites_existing_key() {
        let mut map = BTreeMap::new();
        let schema = Schema::new(1, 0);
        schema.register(&mut map, "k", TypeTemplate::List(Box::new(TypeTemplate::Ref {
            reference: "ITEM_STACK",
            schema_version: 1,
        })));
        schema.register(&mut map, "k", TypeTemplate::OptionalFields(vec![]));
        assert_eq!(map.get("k"), Some(&TypeTemplate::OptionalFields(vec![])));
    }
}
