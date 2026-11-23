use std::collections::BTreeMap;

/// Minimal self-contained representation of a DFU type reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reference(pub &'static str);

/// Common Minecraft DataFixerUpper references used by V2832.
pub mod references {
    use super::Reference;

    pub const CHUNK: Reference = Reference("minecraft:chunk");
    pub const ENTITY_TREE: Reference = Reference("minecraft:entity_tree");
    pub const BLOCK_ENTITY: Reference = Reference("minecraft:block_entity");
    pub const BLOCK_NAME: Reference = Reference("minecraft:block_name");
    pub const BIOME: Reference = Reference("minecraft:biome");
    pub const BLOCK_STATE: Reference = Reference("minecraft:block_state");
    pub const STRUCTURE_FEATURE: Reference = Reference("minecraft:structure_feature");
    pub const MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: Reference =
        Reference("minecraft:multi_noise_biome_source_parameter_list");
    pub const WORLD_GEN_SETTINGS: Reference = Reference("minecraft:world_gen_settings");
}

/// Minimal self-contained DFU type template representation.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Remainder,
    String,
    NamespacedString,
    Const(Box<TypeTemplate>),
    Reference(String),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    Fields(Vec<(String, TypeTemplate)>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    CompoundList {
        key: Box<TypeTemplate>,
        value: Box<TypeTemplate>,
    },
    TaggedChoiceLazy {
        name: String,
        key_type: Box<TypeTemplate>,
        choices: Vec<(String, TypeTemplate)>,
    },
}

/// Minimal schema data container for registered types.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SchemaData {
    pub version: i32,
    pub types: BTreeMap<String, TypeTemplate>,
    pub recursive_types: BTreeMap<String, bool>,
}

impl SchemaData {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            types: BTreeMap::new(),
            recursive_types: BTreeMap::new(),
        }
    }

    pub fn register_type(&mut self, recursive: bool, reference: Reference, template: TypeTemplate) {
        self.types.insert(reference.0.to_string(), template);
        self.recursive_types
            .insert(reference.0.to_string(), recursive);
    }
}

/// Minimal DSL builder mirroring the DFU DSL operations used by V2832.
pub struct Dsl;

impl Dsl {
    pub fn remainder() -> TypeTemplate {
        TypeTemplate::Remainder
    }

    pub fn string() -> TypeTemplate {
        TypeTemplate::String
    }

    pub fn const_type(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Const(Box::new(template))
    }

    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }

    pub fn or(left: TypeTemplate, right: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(left), Box::new(right))
    }

    pub fn fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields(
            pairs
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    pub fn compound_list(key: TypeTemplate, value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundList {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    pub fn compound_list_values(value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundList {
            key: Box::new(TypeTemplate::String),
            value: Box::new(value),
        }
    }

    pub fn tagged_choice_lazy(
        name: &str,
        key_type: TypeTemplate,
        choices: Vec<(&str, TypeTemplate)>,
    ) -> TypeTemplate {
        TypeTemplate::TaggedChoiceLazy {
            name: name.to_string(),
            key_type: Box::new(key_type),
            choices: choices
                .into_iter()
                .map(|(choice_name, template)| (choice_name.to_string(), template))
                .collect(),
        }
    }
}

impl Reference {
    pub fn in_schema(&self, _schema: &SchemaData) -> TypeTemplate {
        TypeTemplate::Reference(self.0.to_string())
    }
}

/// Minimal NamespacedSchema base.
#[derive(Clone, Debug, PartialEq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: Option<i32>,
}

impl NamespacedSchema {
    pub fn new(version: i32, parent_version: Option<i32>) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    pub fn namespaced_string(&self) -> TypeTemplate {
        TypeTemplate::NamespacedString
    }

    pub fn register_types_base(
        &self,
        _target: &mut SchemaData,
        _types: &mut BTreeMap<String, TypeTemplate>,
        _recursive_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        // Base NamespacedSchema type registration hook; no additional types required for this port.
    }
}

/// Port of `net.minecraft.util.datafix.schemas.V2832`.
#[derive(Clone, Debug, PartialEq)]
pub struct V2832 {
    pub base: NamespacedSchema,
}

impl V2832 {
    // JAVA: net.minecraft.util.datafix.schemas.V2832#<init>
    pub fn new(version: i32, parent: Option<&SchemaData>) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent.map(|p| p.version)),
        }
    }

    fn namespaced_string(&self) -> TypeTemplate {
        self.base.namespaced_string()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2832#registerTypes
    pub fn register_types(
        &self,
        target: &mut SchemaData,
        types: &mut BTreeMap<String, TypeTemplate>,
        recursive_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.base.register_types_base(target, types, recursive_types);

        target.register_type(
            false,
            references::CHUNK,
            Dsl::fields(vec![(
                "Level",
                Dsl::optional_fields(vec![
                    (
                        "Entities",
                        Dsl::list(references::ENTITY_TREE.in_schema(target)),
                    ),
                    (
                        "TileEntities",
                        Dsl::list(Dsl::or(
                            references::BLOCK_ENTITY.in_schema(target),
                            Dsl::remainder(),
                        )),
                    ),
                    (
                        "TileTicks",
                        Dsl::list(Dsl::fields(vec![(
                            "i",
                            references::BLOCK_NAME.in_schema(target),
                        )])),
                    ),
                    (
                        "Sections",
                        Dsl::list(Dsl::optional_fields(vec![
                            (
                                "biomes",
                                Dsl::optional_fields(vec![(
                                    "palette",
                                    Dsl::list(references::BIOME.in_schema(target)),
                                )]),
                            ),
                            (
                                "block_states",
                                Dsl::optional_fields(vec![(
                                    "palette",
                                    Dsl::list(references::BLOCK_STATE.in_schema(target)),
                                )]),
                            ),
                        ])),
                    ),
                    (
                        "Structures",
                        Dsl::optional_fields(vec![(
                            "Starts",
                            Dsl::compound_list_values(
                                references::STRUCTURE_FEATURE.in_schema(target),
                            ),
                        )]),
                    ),
                ]),
            )]),
        );

        target.register_type(
            false,
            references::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST,
            Dsl::const_type(self.namespaced_string()),
        );

        target.register_type(
            false,
            references::WORLD_GEN_SETTINGS,
            Dsl::fields(vec![(
                "dimensions",
                Dsl::compound_list(
                    Dsl::const_type(self.namespaced_string()),
                    Dsl::fields(vec![(
                        "generator",
                        Dsl::tagged_choice_lazy(
                            "type",
                            Dsl::string(),
                            vec![
                                ("minecraft:debug", Dsl::remainder()),
                                (
                                    "minecraft:flat",
                                    Dsl::optional_fields(vec![(
                                        "settings",
                                        Dsl::optional_fields(vec![
                                            (
                                                "biome",
                                                references::BIOME.in_schema(target),
                                            ),
                                            (
                                                "layers",
                                                Dsl::list(Dsl::optional_fields(vec![(
                                                    "block",
                                                    references::BLOCK_NAME.in_schema(target),
                                                )])),
                                            ),
                                        ]),
                                    )]),
                                ),
                                (
                                    "minecraft:noise",
                                    Dsl::optional_fields(vec![
                                        (
                                            "biome_source",
                                            Dsl::tagged_choice_lazy(
                                                "type",
                                                Dsl::string(),
                                                vec![
                                                    (
                                                        "minecraft:fixed",
                                                        Dsl::fields(vec![(
                                                            "biome",
                                                            references::BIOME.in_schema(target),
                                                        )]),
                                                    ),
                                                    (
                                                        "minecraft:multi_noise",
                                                        Dsl::or(
                                                            Dsl::fields(vec![(
                                                                "preset",
                                                                references::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST
                                                                    .in_schema(target),
                                                            )]),
                                                            Dsl::list(Dsl::fields(vec![(
                                                                "biome",
                                                                references::BIOME.in_schema(target),
                                                            )])),
                                                        ),
                                                    ),
                                                    (
                                                        "minecraft:checkerboard",
                                                        Dsl::fields(vec![(
                                                            "biomes",
                                                            Dsl::list(
                                                                references::BIOME.in_schema(target),
                                                            ),
                                                        )]),
                                                    ),
                                                    ("minecraft:the_end", Dsl::remainder()),
                                                ],
                                            ),
                                        ),
                                        (
                                            "settings",
                                            Dsl::or(
                                                Dsl::const_type(Dsl::string()),
                                                Dsl::optional_fields(vec![
                                                    (
                                                        "default_block",
                                                        references::BLOCK_NAME.in_schema(target),
                                                    ),
                                                    (
                                                        "default_fluid",
                                                        references::BLOCK_NAME.in_schema(target),
                                                    ),
                                                ]),
                                            ),
                                        ),
                                    ]),
                                ),
                            ],
                        ),
                    )]),
                ),
            )]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_schema() -> SchemaData {
        let parent = SchemaData::new(2831);
        let v2832 = V2832::new(2832, Some(&parent));
        let mut target = SchemaData::new(2832);
        let mut types = BTreeMap::new();
        let mut recursive_types = BTreeMap::new();
        v2832.register_types(&mut target, &mut types, &mut recursive_types);
        target
    }

    #[test]
    fn constructor_and_base_metadata() {
        let parent = SchemaData::new(2831);
        let v2832 = V2832::new(2832, Some(&parent));
        assert_eq!(v2832.base.version, 2832);
        assert_eq!(v2832.base.parent_version, Some(2831));

        let no_parent = V2832::new(2832, None);
        assert_eq!(no_parent.base.parent_version, None);
        assert_eq!(no_parent.namespaced_string(), TypeTemplate::NamespacedString);
    }

    #[test]
    fn registers_expected_types_non_recursive() {
        let schema = build_schema();
        assert_eq!(schema.types.len(), 3);
        for key in [
            references::CHUNK.0,
            references::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST.0,
            references::WORLD_GEN_SETTINGS.0,
        ] {
            assert_eq!(schema.recursive_types.get(key), Some(&false));
            assert!(schema.types.contains_key(key));
        }
    }

    #[test]
    fn chunk_structure_contains_expected_nested_fields() {
        let schema = build_schema();
        let chunk = schema.types.get(references::CHUNK.0).expect("chunk type");
        let TypeTemplate::Fields(top) = chunk else {
            panic!("chunk should be fields");
        };
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, "Level");

        let TypeTemplate::OptionalFields(level_fields) = &top[0].1 else {
            panic!("Level should be optional fields");
        };
        let names: Vec<&str> = level_fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            vec!["Entities", "TileEntities", "TileTicks", "Sections", "Structures"]
        );

        let TypeTemplate::List(tile_entities) = &level_fields[1].1 else {
            panic!("TileEntities should be a list");
        };
        let TypeTemplate::Or(left, right) = tile_entities.as_ref() else {
            panic!("TileEntities element should be or");
        };
        assert_eq!(
            left.as_ref(),
            &TypeTemplate::Reference(references::BLOCK_ENTITY.0.to_string())
        );
        assert_eq!(right.as_ref(), &TypeTemplate::Remainder);
    }

    #[test]
    fn world_gen_settings_has_expected_tagged_choices() {
        let schema = build_schema();
        let wgs = schema
            .types
            .get(references::WORLD_GEN_SETTINGS.0)
            .expect("world gen settings");
        let TypeTemplate::Fields(top) = wgs else {
            panic!("world gen settings should be fields");
        };
        let TypeTemplate::CompoundList { key, value } = &top[0].1 else {
            panic!("dimensions should be compound list");
        };
        assert_eq!(
            key.as_ref(),
            &TypeTemplate::Const(Box::new(TypeTemplate::NamespacedString))
        );

        let TypeTemplate::Fields(dim_fields) = value.as_ref() else {
            panic!("dimension value should be fields");
        };
        let TypeTemplate::TaggedChoiceLazy {
            name,
            key_type,
            choices,
        } = &dim_fields[0].1
        else {
            panic!("generator should be tagged choice lazy");
        };
        assert_eq!(name, "type");
        assert_eq!(key_type.as_ref(), &TypeTemplate::String);
        let choice_names: Vec<&str> = choices.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            choice_names,
            vec!["minecraft:debug", "minecraft:flat", "minecraft:noise"]
        );
    }

    #[test]
    fn noise_biome_source_choices_are_faithful() {
        let schema = build_schema();
        let wgs = schema.types.get(references::WORLD_GEN_SETTINGS.0).unwrap();
        let TypeTemplate::Fields(top) = wgs else { panic!("bad shape") };
        let TypeTemplate::CompoundList { value, .. } = &top[0].1 else { panic!("bad shape") };
        let TypeTemplate::Fields(dim_fields) = value.as_ref() else { panic!("bad shape") };
        let TypeTemplate::TaggedChoiceLazy { choices, .. } = &dim_fields[0].1 else { panic!("bad shape") };

        let noise = choices
            .iter()
            .find(|(n, _)| n == "minecraft:noise")
            .map(|(_, t)| t)
            .expect("noise choice");
        let TypeTemplate::OptionalFields(noise_fields) = noise else { panic!("bad shape") };

        let biome_source = noise_fields
            .iter()
            .find(|(n, _)| n == "biome_source")
            .map(|(_, t)| t)
            .expect("biome_source");
        let TypeTemplate::TaggedChoiceLazy { choices, .. } = biome_source else { panic!("bad shape") };
        let names: Vec<&str> = choices.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "minecraft:fixed",
                "minecraft:multi_noise",
                "minecraft:checkerboard",
                "minecraft:the_end"
            ]
        );

        let multi_noise = choices
            .iter()
            .find(|(n, _)| n == "minecraft:multi_noise")
            .map(|(_, t)| t)
            .unwrap();
        let TypeTemplate::Or(left, right) = multi_noise else { panic!("multi_noise should be or") };
        let TypeTemplate::Fields(preset_fields) = left.as_ref() else { panic!("preset side") };
        assert_eq!(preset_fields[0].0, "preset");
        assert_eq!(
            preset_fields[0].1,
            TypeTemplate::Reference(references::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST.0.to_string())
        );
        let TypeTemplate::List(list_elem) = right.as_ref() else { panic!("list side") };
        let TypeTemplate::Fields(biome_fields) = list_elem.as_ref() else { panic!("biome field") };
        assert_eq!(
            biome_fields[0].1,
            TypeTemplate::Reference(references::BIOME.0.to_string())
        );
    }

    #[test]
    fn multi_noise_parameter_list_uses_namespaced_string_const() {
        let schema = build_schema();
        let t = schema
            .types
            .get(references::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST.0)
            .expect("multi noise parameter list");
        assert_eq!(
            t,
            &TypeTemplate::Const(Box::new(TypeTemplate::NamespacedString))
        );

        // Edge-case sanity: negative/overflow-like version values are preserved as-is in metadata.
        let v = V2832::new(i32::MIN, None);
        assert_eq!(v.base.version, i32::MIN);
        let v2 = V2832::new(i32::MAX, Some(&SchemaData::new(-1)));
        assert_eq!(v2.base.version, i32::MAX);
        assert_eq!(v2.base.parent_version, Some(-1));
    }
}
