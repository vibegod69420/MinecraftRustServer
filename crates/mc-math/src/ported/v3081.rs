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
    pub const GAME_EVENT_NAME: TypeReference = TypeReference("GAME_EVENT_NAME");
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
pub struct V3081 {
    pub base: NamespacedSchema,
}

impl V3081 {
    // JAVA: net.minecraft.util.datafix.schemas.V3081#V3081
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3081#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_entities(schema);
        schema.register(
            &mut map,
            "minecraft:warden",
            Dsl::optional_fields(vec![(
                "listener",
                Dsl::optional_fields(vec![(
                    "event",
                    Dsl::optional_fields(vec![(
                        "game_event",
                        References::GAME_EVENT_NAME.in_schema(schema),
                    )]),
                )]),
            )]),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_warden_nested_structure() {
        let parent = Schema::new(3080, 0);
        let schema = V3081::new(3081, &parent);
        let map = schema.register_entities(&Schema::new(3081, 3080));

        let template = map.get("minecraft:warden").expect("missing warden");
        let expected = TypeTemplate::OptionalFields(vec![(
            "listener".to_string(),
            TypeTemplate::OptionalFields(vec![(
                "event".to_string(),
                TypeTemplate::OptionalFields(vec![(
                    "game_event".to_string(),
                    TypeTemplate::Ref {
                        reference: "GAME_EVENT_NAME",
                        schema_version: 3081,
                    },
                )]),
            )]),
        )]);
        assert_eq!(template, &expected);
    }

    #[test]
    fn negative_version_edge_case() {
        let parent = Schema::new(-2, -3);
        let schema = V3081::new(i32::MIN, &parent);
        assert_eq!(schema.base.version, i32::MIN);
        assert_eq!(schema.base.parent_version, -2);

        let map = schema.register_entities(&Schema::new(i32::MIN, -2));
        let template = map.get("minecraft:warden").expect("missing warden");
        if let TypeTemplate::OptionalFields(fields) = template {
            if let TypeTemplate::OptionalFields(l) = &fields[0].1 {
                if let TypeTemplate::OptionalFields(e) = &l[0].1 {
                    if let TypeTemplate::Ref { schema_version, .. } = &e[0].1 {
                        assert_eq!(*schema_version, i32::MIN);
                        return;
                    }
                }
            }
        }
        panic!("unexpected structure");
    }

    #[test]
    fn overflow_version_edge_case() {
        let parent = Schema::new(i32::MAX, i32::MAX - 1);
        let schema = V3081::new(i32::MAX, &parent);
        let map = schema.register_entities(&Schema::new(i32::MAX, i32::MAX - 1));
        assert!(map.contains_key("minecraft:warden"));
        assert_eq!(schema.base.version.wrapping_add(0), i32::MAX);
    }

    #[test]
    fn version_float_bits_roundtrip() {
        let v = f32::from_bits(0x7FC0_0001u32).to_bits() as i32; // NaN payload bits
        let parent = Schema::new(0, 0);
        let schema = V3081::new(v, &parent);
        assert_eq!(schema.base.version, v);
        assert_eq!(f32::from_bits(schema.base.version as u32).to_bits() as i32, v);
    }

    #[test]
    fn only_registers_warden() {
        let parent = Schema::new(3080, 0);
        let schema = V3081::new(3081, &parent);
        let map = schema.register_entities(&Schema::new(3081, 3080));
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:warden"));
    }
}
