//! Faithful port of `net.minecraft.util.datafix.schemas.V2551` (Minecraft Java 1.21.11).
//!
//! This schema wires the `WORLD_GEN_SETTINGS` datafix type template, including
//! dimension generators for `minecraft:debug`, `minecraft:flat`, and
//! `minecraft:noise` with their nested biome/block references.
//!
//! The original class is a thin registration hook into Mojang DataFixerUpper's
//! schema DSL. This Rust module preserves the structural contract as a
//! self-contained, inspectable schema description without depending on an
//! unported DFU runtime.

use std::collections::BTreeMap;

pub const SCHEMA_VERSION: i32 = 2551;
pub const REFERENCES_WORLD_GEN_SETTINGS: &str = "world_gen_settings";
pub const REFERENCES_BIOME: &str = "biome";
pub const REFERENCES_BLOCK_NAME: &str = "block_name";

/// Minimal local representation of a DataFixerUpper type-template node.
///
/// The Java source builds templates through `DSL.*` combinators; this enum keeps
/// the same shape so schema structure and tests can verify 1:1 field coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    /// `DSL.remainder()`
    Remainder,
    /// `DSL.constType(DSL.string())` / `DSL.string()`-like constant string node.
    ConstString,
    /// `References.<NAME>.in(schema)`
    Reference {
        reference: &'static str,
    },
    /// `DSL.fields("k1", t1, "k2", t2, ...)`
    Fields {
        fields: Vec<(String, TypeTemplate)>,
    },
    /// `DSL.optionalFields("k1", t1, "k2", t2, ...)`
    OptionalFields {
        fields: Vec<(String, TypeTemplate)>,
    },
    /// `DSL.compoundList(key, value)`
    CompoundList {
        key: Box<TypeTemplate>,
        value: Box<TypeTemplate>,
    },
    /// `DSL.list(value)`
    List {
        value: Box<TypeTemplate>,
    },
    /// `DSL.taggedChoiceLazy("type", DSL.string(), map)`
    TaggedChoiceLazy {
        tag: String,
        tag_type: Box<TypeTemplate>,
        choices: BTreeMap<String, TypeTemplate>,
    },
    /// `DSL.or(left, right)`
    Or {
        left: Box<TypeTemplate>,
        right: Box<TypeTemplate>,
    },
}

/// Minimal local schema trait, mirroring the DFU `Schema#registerType` hook used
/// by `V2551#registerTypes`.
pub trait Schema {
    fn register_type(&mut self, recursive: bool, reference: &'static str, template: TypeTemplate);
}

/// Minimal namespaced-schema base matching `NamespacedSchema`'s role for this port.
#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#namespacedString
    pub fn namespaced_string() -> TypeTemplate {
        TypeTemplate::ConstString
    }

    pub fn version(&self) -> i32 {
        self.version
    }
}

/// Rust port of `net.minecraft.util.datafix.schemas.V2551`.
#[derive(Debug, Clone)]
pub struct V2551 {
    base: NamespacedSchema,
}

impl V2551 {
    // JAVA: net.minecraft.util.datafix.schemas.V2551#<init>
    pub fn new(version: i32) -> Self {
        Self {
            base: NamespacedSchema::new(version),
        }
    }

    pub fn version(&self) -> i32 {
        self.base.version()
    }

    fn reference(reference: &'static str) -> TypeTemplate {
        TypeTemplate::Reference { reference }
    }

    fn fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields {
            fields: pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields {
            fields: pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    fn list(value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List {
            value: Box::new(value),
        }
    }

    fn compound_list(key: TypeTemplate, value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundList {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    fn tagged_choice_lazy(tag: &str, choices: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        let mut map = BTreeMap::new();
        for (k, v) in choices {
            map.insert(k.to_string(), v);
        }
        TypeTemplate::TaggedChoiceLazy {
            tag: tag.to_string(),
            tag_type: Box::new(TypeTemplate::ConstString),
            choices: map,
        }
    }

    fn or(left: TypeTemplate, right: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn world_gen_settings_template() -> TypeTemplate {
        let debug = TypeTemplate::Remainder;

        let flat = Self::optional_fields(vec![(
            "settings",
            Self::optional_fields(vec![
                ("biome", Self::reference(REFERENCES_BIOME)),
                (
                    "layers",
                    Self::list(Self::optional_fields(vec![(
                        "block",
                        Self::reference(REFERENCES_BLOCK_NAME),
                    )])),
                ),
            ]),
        )]);

        let noise_biome_source = Self::tagged_choice_lazy(
            "type",
            vec![
                (
                    "minecraft:fixed",
                    Self::fields(vec![("biome", Self::reference(REFERENCES_BIOME))]),
                ),
                (
                    "minecraft:multi_noise",
                    Self::list(Self::fields(vec![("biome", Self::reference(REFERENCES_BIOME))])),
                ),
                (
                    "minecraft:checkerboard",
                    Self::fields(vec![(
                        "biomes",
                        Self::list(Self::reference(REFERENCES_BIOME)),
                    )]),
                ),
                ("minecraft:vanilla_layered", TypeTemplate::Remainder),
                ("minecraft:the_end", TypeTemplate::Remainder),
            ],
        );

        let noise = Self::optional_fields(vec![
            ("biome_source", noise_biome_source),
            (
                "settings",
                Self::or(
                    TypeTemplate::ConstString,
                    Self::optional_fields(vec![
                        ("default_block", Self::reference(REFERENCES_BLOCK_NAME)),
                        ("default_fluid", Self::reference(REFERENCES_BLOCK_NAME)),
                    ]),
                ),
            ),
        ]);

        let generator = Self::tagged_choice_lazy(
            "type",
            vec![
                ("minecraft:debug", debug),
                ("minecraft:flat", flat),
                ("minecraft:noise", noise),
            ],
        );

        Self::fields(vec![(
            "dimensions",
            Self::compound_list(NamespacedSchema::namespaced_string(), Self::fields(vec![("generator", generator)])),
        )])
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2551#registerTypes
    pub fn register_types<S: Schema>(&self, schema: &mut S) {
        // super.registerTypes($$0, $$1, $$2);
        // The Java base class registers many types; this faithful port only models
        // the V2551-specific WORLD_GEN_SETTINGS registration visible in the source.
        schema.register_type(false, REFERENCES_WORLD_GEN_SETTINGS, Self::world_gen_settings_template());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingSchema {
        types: Vec<(bool, &'static str, TypeTemplate)>,
    }

    impl Schema for RecordingSchema {
        fn register_type(&mut self, recursive: bool, reference: &'static str, template: TypeTemplate) {
            self.types.push((recursive, reference, template));
        }
    }

    fn find_field<'a>(fields: &'a [(String, TypeTemplate)], name: &str) -> &'a TypeTemplate {
        fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .expect("field present")
    }

    fn find_optional_field<'a>(fields: &'a [(String, TypeTemplate)], name: &str) -> &'a TypeTemplate {
        fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .expect("optional field present")
    }

    #[test]
    fn constructor_and_version() {
        let schema = V2551::new(SCHEMA_VERSION);
        assert_eq!(schema.version(), 2551);
        assert_eq!(NamespacedSchema::new(99).version(), 99);
    }

    #[test]
    fn register_types_registers_world_gen_settings() {
        let v2551 = V2551::new(SCHEMA_VERSION);
        let mut schema = RecordingSchema::default();
        v2551.register_types(&mut schema);

        assert_eq!(schema.types.len(), 1);
        let (recursive, reference, template) = &schema.types[0];
        assert!(!recursive);
        assert_eq!(*reference, REFERENCES_WORLD_GEN_SETTINGS);

        let TypeTemplate::Fields { fields } = template else {
            panic!("top-level should be fields");
        };
        let dimensions = find_field(fields, "dimensions");
        let TypeTemplate::CompoundList { key, value } = dimensions else {
            panic!("dimensions should be compound list");
        };
        assert_eq!(**key, TypeTemplate::ConstString);
        let TypeTemplate::Fields { fields: dim_fields } = &**value else {
            panic!("dimension value should be fields");
        };
        let generator = find_field(dim_fields, "generator");
        let TypeTemplate::TaggedChoiceLazy { tag, tag_type, choices } = generator else {
            panic!("generator should be tagged choice");
        };
        assert_eq!(tag, "type");
        assert_eq!(**tag_type, TypeTemplate::ConstString);
        assert!(choices.contains_key("minecraft:debug"));
        assert!(choices.contains_key("minecraft:flat"));
        assert!(choices.contains_key("minecraft:noise"));
    }

    #[test]
    fn flat_generator_structure() {
        let mut schema = RecordingSchema::default();
        V2551::new(2551).register_types(&mut schema);
        let template = &schema.types[0].2;
        let TypeTemplate::Fields { fields } = template else { panic!() };
        let TypeTemplate::CompoundList { value, .. } = find_field(fields, "dimensions") else { panic!() };
        let TypeTemplate::Fields { fields: dim_fields } = &**value else { panic!() };
        let TypeTemplate::TaggedChoiceLazy { choices, .. } = find_field(dim_fields, "generator") else { panic!() };

        let flat = choices.get("minecraft:flat").expect("flat");
        let TypeTemplate::OptionalFields { fields: flat_fields } = flat else { panic!("flat optional") };
        let settings = find_optional_field(flat_fields, "settings");
        let TypeTemplate::OptionalFields { fields: settings_fields } = settings else { panic!() };
        assert_eq!(
            find_optional_field(settings_fields, "biome"),
            &TypeTemplate::Reference { reference: REFERENCES_BIOME }
        );
        let layers = find_optional_field(settings_fields, "layers");
        let TypeTemplate::List { value } = layers else { panic!("layers list") };
        let TypeTemplate::OptionalFields { fields: layer_fields } = &**value else { panic!() };
        assert_eq!(
            find_optional_field(layer_fields, "block"),
            &TypeTemplate::Reference { reference: REFERENCES_BLOCK_NAME }
        );
    }

    #[test]
    fn noise_generator_structure() {
        let mut schema = RecordingSchema::default();
        V2551::new(2551).register_types(&mut schema);
        let template = &schema.types[0].2;
        let TypeTemplate::Fields { fields } = template else { panic!() };
        let TypeTemplate::CompoundList { value, .. } = find_field(fields, "dimensions") else { panic!() };
        let TypeTemplate::Fields { fields: dim_fields } = &**value else { panic!() };
        let TypeTemplate::TaggedChoiceLazy { choices, .. } = find_field(dim_fields, "generator") else { panic!() };

        let noise = choices.get("minecraft:noise").expect("noise");
        let TypeTemplate::OptionalFields { fields: noise_fields } = noise else { panic!() };

        let biome_source = find_optional_field(noise_fields, "biome_source");
        let TypeTemplate::TaggedChoiceLazy { tag, choices: bs_choices, .. } = biome_source else { panic!() };
        assert_eq!(tag, "type");
        assert!(matches!(bs_choices.get("minecraft:vanilla_layered"), Some(TypeTemplate::Remainder)));
        assert!(matches!(bs_choices.get("minecraft:the_end"), Some(TypeTemplate::Remainder)));

        let fixed = bs_choices.get("minecraft:fixed").expect("fixed");
        let TypeTemplate::Fields { fields: fixed_fields } = fixed else { panic!() };
        assert_eq!(find_field(fixed_fields, "biome"), &TypeTemplate::Reference { reference: REFERENCES_BIOME });

        let multi = bs_choices.get("minecraft:multi_noise").expect("multi_noise");
        let TypeTemplate::List { value } = multi else { panic!() };
        let TypeTemplate::Fields { fields: multi_fields } = &**value else { panic!() };
        assert_eq!(find_field(multi_fields, "biome"), &TypeTemplate::Reference { reference: REFERENCES_BIOME });

        let checker = bs_choices.get("minecraft:checkerboard").expect("checkerboard");
        let TypeTemplate::Fields { fields: checker_fields } = checker else { panic!() };
        let TypeTemplate::List { value } = find_field(checker_fields, "biomes") else { panic!() };
        assert_eq!(**value, TypeTemplate::Reference { reference: REFERENCES_BIOME });

        let settings = find_optional_field(noise_fields, "settings");
        let TypeTemplate::Or { left, right } = settings else { panic!("settings or") };
        assert_eq!(**left, TypeTemplate::ConstString);
        let TypeTemplate::OptionalFields { fields: settings_fields } = &**right else { panic!() };
        assert_eq!(
            find_optional_field(settings_fields, "default_block"),
            &TypeTemplate::Reference { reference: REFERENCES_BLOCK_NAME }
        );
        assert_eq!(
            find_optional_field(settings_fields, "default_fluid"),
            &TypeTemplate::Reference { reference: REFERENCES_BLOCK_NAME }
        );
    }

    #[test]
    fn debug_generator_is_remainder() {
        let mut schema = RecordingSchema::default();
        V2551::new(2551).register_types(&mut schema);
        let template = &schema.types[0].2;
        let TypeTemplate::Fields { fields } = template else { panic!() };
        let TypeTemplate::CompoundList { value, .. } = find_field(fields, "dimensions") else { panic!() };
        let TypeTemplate::Fields { fields: dim_fields } = &**value else { panic!() };
        let TypeTemplate::TaggedChoiceLazy { choices, .. } = find_field(dim_fields, "generator") else { panic!() };
        assert_eq!(choices.get("minecraft:debug"), Some(&TypeTemplate::Remainder));
    }

    #[test]
    fn namespaced_string_is_const_string() {
        assert_eq!(NamespacedSchema::namespaced_string(), TypeTemplate::ConstString);
    }
}
