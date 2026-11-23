use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Ref(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Fields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;
pub type TemplateMap = BTreeMap<String, TemplateSupplier>;

#[derive(Debug, Clone)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn register(
        &mut self,
        map: &mut TemplateMap,
        name: &str,
        factory: Box<dyn Fn(&str) -> TypeTemplate>,
    ) {
        let n = name.to_string();
        let f = factory;
        map.insert(name.to_string(), Box::new(move || f(&n)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeReference(pub &'static str);

impl TypeReference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref(self.0.to_string())
    }
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

pub struct References;

impl References {
    pub const BLOCK_STATE: TypeReference = TypeReference("block_state");
    pub const ITEM_STACK: TypeReference = TypeReference("item_stack");
}

pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &mut Schema) -> TemplateMap {
        BTreeMap::new()
    }
}

pub struct V3938 {
    pub base: NamespacedSchema,
}

impl V3938 {
    // JAVA: net.minecraft.util.datafix.schemas.V3938#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3938#abstractArrow
    pub fn abstract_arrow(schema: &Schema) -> TypeTemplate {
        Dsl::optional_fields(vec![
            ("inBlockState", References::BLOCK_STATE.in_schema(schema)),
            ("item", References::ITEM_STACK.in_schema(schema)),
            ("weapon", References::ITEM_STACK.in_schema(schema)),
        ])
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3938#registerEntities
    pub fn register_entities(&self, schema: &mut Schema) -> TemplateMap {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();

        let s1 = s.clone();
        schema.register(
            &mut map,
            "minecraft:spectral_arrow",
            Box::new(move |_name| V3938::abstract_arrow(&s1)),
        );

        let s2 = s.clone();
        schema.register(
            &mut map,
            "minecraft:arrow",
            Box::new(move |_name| V3938::abstract_arrow(&s2)),
        );

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_and_parent() {
        let v = V3938::new(3938, Schema::new(3900));
        assert_eq!(v.base.version, 3938);
        assert_eq!(v.base.parent.version, 3900);
    }

    #[test]
    fn abstract_arrow_template() {
        let schema = Schema::new(3938);
        let tpl = V3938::abstract_arrow(&schema);
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![
                ("inBlockState".to_string(), TypeTemplate::Ref("block_state".to_string())),
                ("item".to_string(), TypeTemplate::Ref("item_stack".to_string())),
                ("weapon".to_string(), TypeTemplate::Ref("item_stack".to_string())),
            ])
        );
    }

    #[test]
    fn register_entities_contains_arrows() {
        let mut schema = Schema::new(3938);
        let v = V3938::new(3938, Schema::new(3900));
        let map = v.register_entities(&mut schema);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("minecraft:spectral_arrow"));
        assert!(map.contains_key("minecraft:arrow"));
    }

    #[test]
    fn registered_arrows_use_abstract_arrow_template() {
        let mut schema = Schema::new(3938);
        let v = V3938::new(3938, Schema::new(3900));
        let map = v.register_entities(&mut schema);
        let expected = V3938::abstract_arrow(&Schema::new(3938));
        assert_eq!(map.get("minecraft:spectral_arrow").unwrap()(), expected);
        assert_eq!(map.get("minecraft:arrow").unwrap()(), expected);
    }
}
