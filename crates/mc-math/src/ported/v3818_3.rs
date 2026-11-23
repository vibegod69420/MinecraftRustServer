use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reference {
    EntityTree,
    BlockEntity,
    ItemStack,
    BlockName,
    ItemName,
    TextComponent,
    DataComponents,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Ref(Reference),
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    OptionalFieldsLazy(Vec<(String, TypeTemplate)>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
    pub parent_version: i32,
    pub registered_types: BTreeMap<String, (bool, TypeTemplate)>,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
            registered_types: BTreeMap::new(),
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type(
        &mut self,
        recursive: bool,
        reference: Reference,
        template: TemplateSupplier,
    ) {
        self.registered_types
            .insert(format!("{reference:?}"), (recursive, template()));
    }
}

pub struct V3818_3 {
    pub version: i32,
    pub parent_version: i32,
}

impl V3818_3 {
    // JAVA: net.minecraft.util.datafix.schemas.V3818_3#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3818_3#components
    pub fn components(_schema: &Schema) -> Vec<(String, TemplateSupplier)> {
        let mut map: Vec<(String, TemplateSupplier)> = Vec::new();

        map.push((
            "minecraft:bees".to_string(),
            Box::new(|| {
                TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                    "entity_data".to_string(),
                    TypeTemplate::Ref(Reference::EntityTree),
                )])))
            }),
        ));
        map.push((
            "minecraft:block_entity_data".to_string(),
            Box::new(|| TypeTemplate::Ref(Reference::BlockEntity)),
        ));
        map.push((
            "minecraft:bundle_contents".to_string(),
            Box::new(|| TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack)))),
        ));
        map.push((
            "minecraft:can_break".to_string(),
            Box::new(|| {
                TypeTemplate::OptionalFields(vec![(
                    "predicates".to_string(),
                    TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                        "blocks".to_string(),
                        TypeTemplate::Or(
                            Box::new(TypeTemplate::Ref(Reference::BlockName)),
                            Box::new(TypeTemplate::List(Box::new(TypeTemplate::Ref(
                                Reference::BlockName,
                            )))),
                        ),
                    )]))),
                )])
            }),
        ));
        map.push((
            "minecraft:can_place_on".to_string(),
            Box::new(|| {
                TypeTemplate::OptionalFields(vec![(
                    "predicates".to_string(),
                    TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                        "blocks".to_string(),
                        TypeTemplate::Or(
                            Box::new(TypeTemplate::Ref(Reference::BlockName)),
                            Box::new(TypeTemplate::List(Box::new(TypeTemplate::Ref(
                                Reference::BlockName,
                            )))),
                        ),
                    )]))),
                )])
            }),
        ));
        map.push((
            "minecraft:charged_projectiles".to_string(),
            Box::new(|| TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack)))),
        ));
        map.push((
            "minecraft:container".to_string(),
            Box::new(|| {
                TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                    "item".to_string(),
                    TypeTemplate::Ref(Reference::ItemStack),
                )])))
            }),
        ));
        map.push((
            "minecraft:entity_data".to_string(),
            Box::new(|| TypeTemplate::Ref(Reference::EntityTree)),
        ));
        map.push((
            "minecraft:pot_decorations".to_string(),
            Box::new(|| TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemName)))),
        ));
        map.push((
            "minecraft:food".to_string(),
            Box::new(|| {
                TypeTemplate::OptionalFields(vec![(
                    "using_converts_to".to_string(),
                    TypeTemplate::Ref(Reference::ItemStack),
                )])
            }),
        ));
        map.push((
            "minecraft:custom_name".to_string(),
            Box::new(|| TypeTemplate::Ref(Reference::TextComponent)),
        ));
        map.push((
            "minecraft:item_name".to_string(),
            Box::new(|| TypeTemplate::Ref(Reference::TextComponent)),
        ));
        map.push((
            "minecraft:lore".to_string(),
            Box::new(|| TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::TextComponent)))),
        ));
        map.push((
            "minecraft:written_book_content".to_string(),
            Box::new(|| {
                TypeTemplate::OptionalFields(vec![(
                    "pages".to_string(),
                    TypeTemplate::List(Box::new(TypeTemplate::Or(
                        Box::new(TypeTemplate::OptionalFields(vec![
                            ("raw".to_string(), TypeTemplate::Ref(Reference::TextComponent)),
                            (
                                "filtered".to_string(),
                                TypeTemplate::Ref(Reference::TextComponent),
                            ),
                        ])),
                        Box::new(TypeTemplate::Ref(Reference::TextComponent)),
                    ))),
                )])
            }),
        ));

        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3818_3#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        _entity_types: &mut BTreeMap<String, TemplateSupplier>,
        _block_entity_types: &mut BTreeMap<String, TemplateSupplier>,
    ) {
        // super.registerTypes(schema, entityTypes, blockEntityTypes) is a no-op in this self-contained port.
        let components = Self::components(schema);
        let fields: Vec<(String, TypeTemplate)> =
            components.into_iter().map(|(k, v)| (k, v())).collect();
        schema.register_type(
            true,
            Reference::DataComponents,
            Box::new(move || TypeTemplate::OptionalFieldsLazy(fields.clone())),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Schema {
        Schema::new(3818, 3817)
    }

    #[test]
    fn components_count_and_order() {
        let comps = V3818_3::components(&schema());
        let keys: Vec<&str> = comps.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys.len(), 14);
        assert_eq!(keys[0], "minecraft:bees");
        assert_eq!(keys[1], "minecraft:block_entity_data");
        assert_eq!(keys[13], "minecraft:written_book_content");
    }

    #[test]
    fn bees_component_structure() {
        let comps = V3818_3::components(&schema());
        let tpl = (comps.iter().find(|(k, _)| k == "minecraft:bees").unwrap().1)();
        assert_eq!(
            tpl,
            TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                "entity_data".to_string(),
                TypeTemplate::Ref(Reference::EntityTree),
            )])))
        );
    }

    #[test]
    fn can_break_has_or_block_predicates() {
        let comps = V3818_3::components(&schema());
        let tpl = (comps
            .iter()
            .find(|(k, _)| k == "minecraft:can_break")
            .unwrap()
            .1)();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "predicates".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                    "blocks".to_string(),
                    TypeTemplate::Or(
                        Box::new(TypeTemplate::Ref(Reference::BlockName)),
                        Box::new(TypeTemplate::List(Box::new(TypeTemplate::Ref(
                            Reference::BlockName
                        )))),
                    ),
                )]))),
            )])
        );
    }

    #[test]
    fn written_book_content_nested_or() {
        let comps = V3818_3::components(&schema());
        let tpl = (comps
            .iter()
            .find(|(k, _)| k == "minecraft:written_book_content")
            .unwrap()
            .1)();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "pages".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Or(
                    Box::new(TypeTemplate::OptionalFields(vec![
                        ("raw".to_string(), TypeTemplate::Ref(Reference::TextComponent)),
                        (
                            "filtered".to_string(),
                            TypeTemplate::Ref(Reference::TextComponent)
                        ),
                    ])),
                    Box::new(TypeTemplate::Ref(Reference::TextComponent)),
                ))),
            )])
        );
    }

    #[test]
    fn register_types_registers_data_components() {
        let v = V3818_3::new(3818, 3817);
        let mut s = schema();
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut s, &mut entities, &mut block_entities);
        let (recursive, tpl) = s.registered_types.get("DataComponents").unwrap();
        assert!(*recursive);
        match tpl {
            TypeTemplate::OptionalFieldsLazy(fields) => {
                assert_eq!(fields.len(), 14);
                assert_eq!(fields[0].0, "minecraft:bees");
            }
            other => panic!("unexpected template: {other:?}"),
        }
    }

    #[test]
    fn constructor_stores_versions() {
        let v = V3818_3::new(3818, 3817);
        assert_eq!(v.version, 3818);
        assert_eq!(v.parent_version, 3817);
        let v2 = V3818_3::new(i32::MIN, -1);
        assert_eq!(v2.version, i32::MIN);
        assert_eq!(v2.parent_version, -1);
    }
}
