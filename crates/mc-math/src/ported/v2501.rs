use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    ItemStack,
    TextComponent,
    Recipe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    Ref(Reference),
    ConstInt,
    List(Box<TypeTemplate>),
    CompoundList(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
        template_factory: F,
    ) where
        F: FnOnce(&Schema) -> TypeTemplate,
    {
        map.insert(name.to_string(), template_factory(self));
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

pub struct V2501 {
    base: NamespacedSchema,
}

impl V2501 {
    // JAVA: net.minecraft.util.datafix.schemas.V2501#<init>(int,Schema)
    pub fn new(version: i32, parent: Option<&Schema>) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2501#registerFurnace(Schema,Map,String)
    pub fn register_furnace(
        schema: &Schema,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
    ) {
        schema.register(map, name, |_s| {
            TypeTemplate::OptionalFields(vec![
                (
                    "Items".to_string(),
                    TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
                ),
                (
                    "CustomName".to_string(),
                    TypeTemplate::Ref(Reference::TextComponent),
                ),
                (
                    "RecipesUsed".to_string(),
                    TypeTemplate::CompoundList(
                        Box::new(TypeTemplate::Ref(Reference::Recipe)),
                        Box::new(TypeTemplate::ConstInt),
                    ),
                ),
            ])
        });
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2501#registerBlockEntities(Schema)
    pub fn register_block_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_block_entities(schema);
        Self::register_furnace(schema, &mut map, "minecraft:furnace");
        Self::register_furnace(schema, &mut map, "minecraft:smoker");
        Self::register_furnace(schema, &mut map, "minecraft:blast_furnace");
        map
    }

    pub fn schema(&self) -> &Schema {
        self.base.schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn furnace_template() -> TypeTemplate {
        TypeTemplate::OptionalFields(vec![
            (
                "Items".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
            ),
            (
                "CustomName".to_string(),
                TypeTemplate::Ref(Reference::TextComponent),
            ),
            (
                "RecipesUsed".to_string(),
                TypeTemplate::CompoundList(
                    Box::new(TypeTemplate::Ref(Reference::Recipe)),
                    Box::new(TypeTemplate::ConstInt),
                ),
            ),
        ])
    }

    #[test]
    fn test_register_furnace_adds_template() {
        let schema = Schema::new(2501);
        let mut map = BTreeMap::new();
        V2501::register_furnace(&schema, &mut map, "minecraft:furnace");
        assert_eq!(map.get("minecraft:furnace"), Some(&furnace_template()));
    }

    #[test]
    fn test_register_block_entities_has_all_furnaces() {
        let v2501 = V2501::new(2501, None);
        let schema = Schema::new(2501);
        let map = v2501.register_block_entities(&schema);
        for name in ["minecraft:furnace", "minecraft:smoker", "minecraft:blast_furnace"] {
            assert_eq!(map.get(name), Some(&furnace_template()), "{name}");
        }
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_furnace_template_structure() {
        let template = furnace_template();
        if let TypeTemplate::OptionalFields(fields) = template {
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "Items");
            assert_eq!(fields[1].0, "CustomName");
            assert_eq!(fields[2].0, "RecipesUsed");
            assert_eq!(fields[1].1, TypeTemplate::Ref(Reference::TextComponent));
        } else {
            panic!("expected OptionalFields");
        }
    }

    #[test]
    fn test_schema_version() {
        let v2501 = V2501::new(2501, None);
        assert_eq!(v2501.schema().version(), 2501);
    }
}
