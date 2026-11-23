//! Faithful port of `net.minecraft.util.datafix.schemas.V1451_3` (Java 1.21.11).

use std::collections::BTreeMap;

pub type TypeTemplate = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reference {
    ItemStack,
    BlockState,
    BlockEntity,
    TextComponent,
    UntaggedSpawner,
}

impl Reference {
    // JAVA: net.minecraft.util.datafix.fixes.References (conceptual type reference names)
    pub fn type_name(self) -> &'static str {
        match self {
            Reference::ItemStack => "References.ITEM_STACK",
            Reference::BlockState => "References.BLOCK_STATE",
            Reference::BlockEntity => "References.BLOCK_ENTITY",
            Reference::TextComponent => "References.TEXT_COMPONENT",
            Reference::UntaggedSpawner => "References.UNTAGGED_SPAWNER",
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.References#in
    pub fn in_schema(self, _schema: &Schema) -> TypeTemplate {
        self.type_name().to_string()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schema {
    version: i32,
    parent_version: Option<i32>,
}

impl Schema {
    pub fn new(version: i32, parent_version: Option<i32>) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn parent_version(&self) -> Option<i32> {
        self.parent_version
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
    ) {
        map.insert(name.to_string(), "DSL.nil()".to_string());
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
        let template = template_factory(self);
        map.insert(name.to_string(), template);
    }
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: &[(&str, TypeTemplate)]) -> TypeTemplate {
        let mut out = String::from("DSL.optionalFields(");
        for (i, (name, template)) in fields.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push('"');
            out.push_str(name);
            out.push_str("\", ");
            out.push_str(template);
        }
        out.push(')');
        out
    }

    // JAVA: com.mojang.datafixers.DSL#optionalFields (varargs single reference form)
    pub fn optional_fields_refs(refs: &[Reference], schema: &Schema) -> TypeTemplate {
        let mut out = String::from("DSL.optionalFields(");
        for (i, r) in refs.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&r.in_schema(schema));
        }
        out.push(')');
        out
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(element: TypeTemplate) -> TypeTemplate {
        format!("DSL.list({})", element)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespacedSchema {
    schema: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            schema: Schema::new(version, Some(parent.version())),
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct V1451_3 {
    base: NamespacedSchema,
}

impl V1451_3 {
    // JAVA: net.minecraft.util.datafix.schemas.V1451_3#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    pub fn schema(&self) -> &Schema {
        self.base.schema()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1451_3#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_entities(schema);

        schema.register_simple(&mut map, "minecraft:egg");
        schema.register_simple(&mut map, "minecraft:ender_pearl");
        schema.register_simple(&mut map, "minecraft:fireball");
        schema.register(&mut map, "minecraft:potion", |s| {
            Dsl::optional_fields(&[("Potion", Reference::ItemStack.in_schema(s))])
        });
        schema.register_simple(&mut map, "minecraft:small_fireball");
        schema.register_simple(&mut map, "minecraft:snowball");
        schema.register_simple(&mut map, "minecraft:wither_skull");
        schema.register_simple(&mut map, "minecraft:xp_bottle");
        schema.register(&mut map, "minecraft:arrow", |s| {
            Dsl::optional_fields(&[("inBlockState", Reference::BlockState.in_schema(s))])
        });
        schema.register(&mut map, "minecraft:enderman", |s| {
            Dsl::optional_fields(&[("carriedBlockState", Reference::BlockState.in_schema(s))])
        });
        schema.register(&mut map, "minecraft:falling_block", |s| {
            Dsl::optional_fields(&[
                ("BlockState", Reference::BlockState.in_schema(s)),
                ("TileEntityData", Reference::BlockEntity.in_schema(s)),
            ])
        });
        schema.register(&mut map, "minecraft:spectral_arrow", |s| {
            Dsl::optional_fields(&[("inBlockState", Reference::BlockState.in_schema(s))])
        });
        schema.register(&mut map, "minecraft:chest_minecart", |s| {
            Dsl::optional_fields(&[
                ("DisplayState", Reference::BlockState.in_schema(s)),
                ("Items", Dsl::list(Reference::ItemStack.in_schema(s))),
            ])
        });
        schema.register(&mut map, "minecraft:commandblock_minecart", |s| {
            Dsl::optional_fields(&[
                ("DisplayState", Reference::BlockState.in_schema(s)),
                ("LastOutput", Reference::TextComponent.in_schema(s)),
            ])
        });
        schema.register(&mut map, "minecraft:furnace_minecart", |s| {
            Dsl::optional_fields(&[("DisplayState", Reference::BlockState.in_schema(s))])
        });
        schema.register(&mut map, "minecraft:hopper_minecart", |s| {
            Dsl::optional_fields(&[
                ("DisplayState", Reference::BlockState.in_schema(s)),
                ("Items", Dsl::list(Reference::ItemStack.in_schema(s))),
            ])
        });
        schema.register(&mut map, "minecraft:minecart", |s| {
            Dsl::optional_fields(&[("DisplayState", Reference::BlockState.in_schema(s))])
        });
        schema.register(&mut map, "minecraft:spawner_minecart", |s| {
            Dsl::optional_fields(&[
                ("DisplayState", Reference::BlockState.in_schema(s)),
            ]) + " + " + &Dsl::optional_fields_refs(&[Reference::UntaggedSpawner], s)
        });
        schema.register(&mut map, "minecraft:tnt_minecart", |s| {
            Dsl::optional_fields(&[("DisplayState", Reference::BlockState.in_schema(s))])
        });

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema() -> (V1451_3, Schema) {
        let parent = Schema::new(1451, None);
        let v = V1451_3::new(1451_3, parent);
        let schema = Schema::new(1451_3, Some(1451));
        (v, schema)
    }

    #[test]
    fn constructor_sets_version_and_parent() {
        let parent = Schema::new(1451, Some(1450));
        let v = V1451_3::new(1451_3, parent);
        assert_eq!(v.schema().version(), 1451_3);
        assert_eq!(v.schema().parent_version(), Some(1451));
    }

    #[test]
    fn register_entities_contains_expected_simple_entities() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        for name in [
            "minecraft:egg",
            "minecraft:ender_pearl",
            "minecraft:fireball",
            "minecraft:small_fireball",
            "minecraft:snowball",
            "minecraft:wither_skull",
            "minecraft:xp_bottle",
        ] {
            assert_eq!(map.get(name).map(String::as_str), Some("DSL.nil()"), "{name}");
        }
    }

    #[test]
    fn register_entities_potion_has_item_stack_field() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        let t = map.get("minecraft:potion").expect("potion");
        assert_eq!(t, "DSL.optionalFields(\"Potion\", References.ITEM_STACK)");
    }

    #[test]
    fn register_entities_arrow_and_enderman_have_block_state_fields() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        assert_eq!(
            map.get("minecraft:arrow").unwrap(),
            "DSL.optionalFields(\"inBlockState\", References.BLOCK_STATE)"
        );
        assert_eq!(
            map.get("minecraft:enderman").unwrap(),
            "DSL.optionalFields(\"carriedBlockState\", References.BLOCK_STATE)"
        );
        assert_eq!(
            map.get("minecraft:spectral_arrow").unwrap(),
            "DSL.optionalFields(\"inBlockState\", References.BLOCK_STATE)"
        );
    }

    #[test]
    fn register_entities_falling_block_and_minecarts_have_expected_templates() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        assert_eq!(
            map.get("minecraft:falling_block").unwrap(),
            "DSL.optionalFields(\"BlockState\", References.BLOCK_STATE, \"TileEntityData\", References.BLOCK_ENTITY)"
        );
        assert_eq!(
            map.get("minecraft:chest_minecart").unwrap(),
            "DSL.optionalFields(\"DisplayState\", References.BLOCK_STATE, \"Items\", DSL.list(References.ITEM_STACK))"
        );
        assert_eq!(
            map.get("minecraft:commandblock_minecart").unwrap(),
            "DSL.optionalFields(\"DisplayState\", References.BLOCK_STATE, \"LastOutput\", References.TEXT_COMPONENT)"
        );
        assert_eq!(
            map.get("minecraft:hopper_minecart").unwrap(),
            "DSL.optionalFields(\"DisplayState\", References.BLOCK_STATE, \"Items\", DSL.list(References.ITEM_STACK))"
        );
    }

    #[test]
    fn register_entities_spawner_minecart_includes_untagged_spawner() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        let t = map.get("minecraft:spawner_minecart").expect("spawner_minecart");
        assert!(t.contains("DisplayState"));
        assert!(t.contains("References.BLOCK_STATE"));
        assert!(t.contains("References.UNTAGGED_SPAWNER"));
    }
}
