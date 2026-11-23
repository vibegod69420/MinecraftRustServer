use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    value: DynValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataResult<T>(Option<T>);

impl<T> DataResult<T> {
    pub fn result(self) -> Option<T> {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn result(&self) -> Option<Dynamic> {
        self.0.clone()
    }

    pub fn as_string(&self) -> DataResult<String> {
        match &self.0 {
            Some(d) => d.as_string(),
            None => DataResult(None),
        }
    }

    pub fn as_string_or(&self, default: &str) -> String {
        self.as_string()
            .result()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn as_stream_opt(&self) -> DataResult<Vec<Dynamic>> {
        match &self.0 {
            Some(d) => d.as_stream_opt(),
            None => DataResult(None),
        }
    }
}

impl Dynamic {
    pub fn null() -> Self {
        Dynamic {
            value: DynValue::Null,
        }
    }

    pub fn empty_map() -> Self {
        Dynamic {
            value: DynValue::Map(BTreeMap::new()),
        }
    }

    pub fn string(s: &str) -> Self {
        Dynamic {
            value: DynValue::String(s.to_string()),
        }
    }

    pub fn map(entries: Vec<(&str, Dynamic)>) -> Self {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic {
            value: DynValue::Map(m),
        }
    }

    pub fn list(items: Vec<Dynamic>) -> Self {
        Dynamic {
            value: DynValue::List(items),
        }
    }

    pub fn create_string(&self, s: &str) -> Self {
        Dynamic::string(s)
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Self {
        Dynamic::list(items)
    }

    pub fn empty_map_value(&self) -> Self {
        Dynamic::empty_map()
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            DynValue::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    pub fn as_string(&self) -> DataResult<String> {
        match &self.value {
            DynValue::String(s) => DataResult(Some(s.clone())),
            _ => DataResult(None),
        }
    }

    pub fn as_stream_opt(&self) -> DataResult<Vec<Dynamic>> {
        match &self.value {
            DynValue::List(v) => DataResult(Some(v.clone())),
            _ => DataResult(None),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_string(), value);
                Dynamic {
                    value: DynValue::Map(m2),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic {
                    value: DynValue::Map(m2),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Dynamic {
        match self.get(key).result() {
            Some(current) => self.set(key, f(current)),
            None => {
                let current = Dynamic::null();
                let updated = f(current.clone());
                if updated == current {
                    self.clone()
                } else {
                    self.set(key, updated)
                }
            }
        }
    }
}

fn or_else<T>(opt: Option<T>, fallback: T) -> T {
    opt.unwrap_or(fallback)
}

#[derive(Clone, Debug)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct Type;

impl Type {
    pub fn find_field(&self, _name: &str) -> OpticFinder {
        OpticFinder
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OpticFinder;

#[derive(Clone, Debug)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug)]
pub struct TypeRewriteRule;

impl TypeRewriteRule {
    pub fn seq(_rules: Vec<TypeRewriteRule>) -> TypeRewriteRule {
        TypeRewriteRule
    }
}

pub struct References;

impl References {
    pub const ITEM_STACK: &'static str = "minecraft:item_stack";
    pub const ENTITY: &'static str = "minecraft:entity";
    pub const PLAYER: &'static str = "minecraft:player";
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub data: Dynamic,
}

impl Typed {
    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, _finder: RemainderFinder, f: F) -> Typed {
        Typed { data: f(self.data) }
    }

    pub fn update_typed<F: FnOnce(Typed) -> Typed>(self, _finder: OpticFinder, f: F) -> Typed {
        f(self)
    }
}

pub struct AttributesRenameLegacy {
    schema: Schema,
    name: String,
    renames: Box<dyn Fn(String) -> String>,
}

impl AttributesRenameLegacy {
    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameLegacy#<init>
    pub fn new(schema: Schema, name: String, renames: Box<dyn Fn(String) -> String>) -> Self {
        Self {
            schema,
            name,
            renames,
        }
    }

    fn get_input_schema(&self) -> &Schema {
        &self.schema
    }

    fn fix_type_everywhere_typed(
        &self,
        _name: String,
        _type: Type,
    ) -> TypeRewriteRule {
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameLegacy#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let item_stack_type = self.get_input_schema().get_type(References::ITEM_STACK);
        let _tag_finder = item_stack_type.find_field("tag");
        TypeRewriteRule::seq(vec![
            self.fix_type_everywhere_typed(
                format!("{} (ItemStack)", self.name),
                self.get_input_schema().get_type(References::ITEM_STACK),
            ),
            self.fix_type_everywhere_typed(
                format!("{} (Entity)", self.name),
                self.get_input_schema().get_type(References::ENTITY),
            ),
            self.fix_type_everywhere_typed(
                format!("{} (Player)", self.name),
                self.get_input_schema().get_type(References::PLAYER),
            ),
        ])
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameLegacy#fixName
    fn fix_name(&self, value: Dynamic) -> Dynamic {
        let renamed = value
            .as_string()
            .result()
            .map(|s| (self.renames)(s))
            .map(|s| value.create_string(&s));
        or_else(renamed, value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameLegacy#fixItemStackTag
    fn fix_item_stack_tag(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |data| {
            data.update("AttributeModifiers", |mods| {
                let fixed = mods.as_stream_opt().result().map(|stream| {
                    let mapped: Vec<Dynamic> = stream
                        .into_iter()
                        .map(|elem| elem.update("AttributeName", |n| self.fix_name(n)))
                        .collect();
                    mods.create_list(mapped)
                });
                or_else(fixed, mods)
            })
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameLegacy#fixEntity
    fn fix_entity(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |data| {
            data.update("Attributes", |attrs| {
                let fixed = attrs.as_stream_opt().result().map(|stream| {
                    let mapped: Vec<Dynamic> = stream
                        .into_iter()
                        .map(|elem| elem.update("Name", |n| self.fix_name(n)))
                        .collect();
                    attrs.create_list(mapped)
                });
                or_else(fixed, attrs)
            })
        })
    }
}

impl Schema {
    pub fn get_type(&self, _reference: &str) -> Type {
        Type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix() -> AttributesRenameLegacy {
        AttributesRenameLegacy::new(
            Schema,
            "test_rename".to_string(),
            Box::new(|s| if s == "old.attr" { "new.attr".to_string() } else { s }),
        )
    }

    #[test]
    fn fix_name_renames_matching_string() {
        let fix = make_fix();
        let d = Dynamic::string("old.attr");
        let out = fix.fix_name(d.clone());
        assert_eq!(out, Dynamic::string("new.attr"));
    }

    #[test]
    fn fix_name_passthrough_non_string() {
        let fix = make_fix();
        let d = Dynamic::map(vec![("k", Dynamic::string("v"))]);
        let out = fix.fix_name(d.clone());
        assert_eq!(out, d);
    }

    #[test]
    fn fix_item_stack_tag_renames_attribute_name() {
        let fix = make_fix();
        let tag = Dynamic::map(vec![(
            "AttributeModifiers",
            Dynamic::list(vec![
                Dynamic::map(vec![
                    ("AttributeName", Dynamic::string("old.attr")),
                    ("Amount", Dynamic::string("1")),
                ]),
                Dynamic::map(vec![
                    ("AttributeName", Dynamic::string("other")),
                    ("Amount", Dynamic::string("2")),
                ]),
            ]),
        )]);
        let out = fix.fix_item_stack_tag(Typed { data: tag });
        let mods = out
            .data
            .get("AttributeModifiers")
            .as_stream_opt()
            .result()
            .unwrap();
        assert_eq!(
            mods[0].get("AttributeName").as_string().result().unwrap(),
            "new.attr"
        );
        assert_eq!(
            mods[1].get("AttributeName").as_string().result().unwrap(),
            "other"
        );
        assert_eq!(mods[0].get("Amount").as_string().result().unwrap(), "1");
    }

    #[test]
    fn fix_entity_renames_name_in_attributes() {
        let fix = make_fix();
        let data = Dynamic::map(vec![(
            "Attributes",
            Dynamic::list(vec![Dynamic::map(vec![(
                "Name",
                Dynamic::string("old.attr"),
            )])]),
        )]);
        let out = fix.fix_entity(Typed { data });
        let attrs = out.data.get("Attributes").as_stream_opt().result().unwrap();
        assert_eq!(attrs[0].get("Name").as_string().result().unwrap(), "new.attr");
    }

    #[test]
    fn missing_or_invalid_fields_preserved() {
        let fix = make_fix();
        let tag = Dynamic::map(vec![("Other", Dynamic::string("x"))]);
        let out_tag = fix.fix_item_stack_tag(Typed { data: tag.clone() });
        assert_eq!(out_tag.data.get("Other").as_string().result().unwrap(), "x");
        assert!(out_tag.data.get("AttributeModifiers").result().is_none());

        let data = Dynamic::map(vec![("Attributes", Dynamic::string("bad"))]);
        let out_entity = fix.fix_entity(Typed { data });
        assert_eq!(
            out_entity.data.get("Attributes").as_string().result().unwrap(),
            "bad"
        );
    }
}
