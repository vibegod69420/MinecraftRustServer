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
    pub const TEXT_COMPONENT: TypeReference = TypeReference("TEXT_COMPONENT");
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1488 {
    pub base: NamespacedSchema,
}

impl V1488 {
    // JAVA: net.minecraft.util.datafix.schemas.V1488#V1488
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1488#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_block_entities(schema);
        schema.register(
            &mut map,
            "minecraft:command_block",
            Dsl::optional_fields(vec![
                ("CustomName", References::TEXT_COMPONENT.in_schema(schema)),
                ("LastOutput", References::TEXT_COMPONENT.in_schema(schema)),
            ]),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_command_block_structure() {
        let parent = Schema::new(1487, 0);
        let schema = V1488::new(1488, &parent);
        let map = schema.register_block_entities(&Schema::new(1488, 1487));

        let template = map.get("minecraft:command_block").expect("missing command_block");
        let expected = TypeTemplate::OptionalFields(vec![
            (
                "CustomName".to_string(),
                TypeTemplate::Ref {
                    reference: "TEXT_COMPONENT",
                    schema_version: 1488,
                },
            ),
            (
                "LastOutput".to_string(),
                TypeTemplate::Ref {
                    reference: "TEXT_COMPONENT",
                    schema_version: 1488,
                },
            ),
        ]);
        assert_eq!(template, &expected);
    }

    #[test]
    fn negative_version_edge_case() {
        let parent = Schema::new(-1, -10);
        let schema = V1488::new(-1488, &parent);
        assert_eq!(schema.base.version, -1488);
        let map = schema.register_block_entities(&Schema::new(-1488, -1));
        let template = map.get("minecraft:command_block").expect("missing command_block");
        if let TypeTemplate::OptionalFields(fields) = template {
            if let TypeTemplate::Ref { schema_version, .. } = &fields[0].1 {
                assert_eq!(*schema_version, -1488);
            } else {
                panic!("expected ref");
            }
        } else {
            panic!("expected OptionalFields");
        }
    }

    #[test]
    fn overflow_version_edge_case() {
        let parent = Schema::new(i32::MAX, 0);
        let schema = V1488::new(i32::MAX, &parent);
        let map = schema.register_block_entities(&Schema::new(i32::MAX, 0));
        let template = map.get("minecraft:command_block").expect("missing command_block");
        if let TypeTemplate::OptionalFields(fields) = template {
            for (_, t) in fields {
                if let TypeTemplate::Ref { schema_version, .. } = t {
                    assert_eq!(*schema_version, i32::MAX);
                }
            }
        } else {
            panic!("expected OptionalFields");
        }
    }

    #[test]
    fn version_float_bits_roundtrip() {
        let v = f32::from_bits(0x7F80_0000u32).to_bits() as i32; // +inf bits
        let parent = Schema::new(0, 0);
        let schema = V1488::new(v, &parent);
        assert_eq!(f32::from_bits(schema.base.version as u32).to_bits() as i32, v);
    }

    #[test]
    fn only_registers_command_block() {
        let parent = Schema::new(1487, 0);
        let schema = V1488::new(1488, &parent);
        let map = schema.register_block_entities(&Schema::new(1488, 1487));
        assert_eq!(map.len(), 1);
        assert_eq!(map.keys().next().map(String::as_str), Some("minecraft:command_block"));
    }
}
