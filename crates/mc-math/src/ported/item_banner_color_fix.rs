use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<DynamicValue>),
    Map(BTreeMap<String, DynamicValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn new(value: DynamicValue) -> Self {
        Self { value }
    }

    pub fn null() -> Self {
        Self::new(DynamicValue::Null)
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(map) => map.get(key).cloned().map(Dynamic::new).unwrap_or_else(Dynamic::null),
            _ => Dynamic::null(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        let mut map = match &self.value {
            DynamicValue::Map(m) => m.clone(),
            _ => BTreeMap::new(),
        };
        map.insert(key.to_string(), value.value);
        Dynamic::new(DynamicValue::Map(map))
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        let mut map = match &self.value {
            DynamicValue::Map(m) => m.clone(),
            _ => BTreeMap::new(),
        };
        map.remove(key);
        Dynamic::new(DynamicValue::Map(map))
    }

    pub fn result(&self) -> Option<Dynamic> {
        if matches!(self.value, DynamicValue::Null) {
            None
        } else {
            Some(self.clone())
        }
    }

    pub fn as_number_result(&self) -> Option<Dynamic> {
        match &self.value {
            DynamicValue::Short(_)
            | DynamicValue::Int(_)
            | DynamicValue::Long(_)
            | DynamicValue::Float(_)
            | DynamicValue::Double(_) => Some(self.clone()),
            _ => None,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            DynamicValue::Short(v) => *v as i32,
            DynamicValue::Int(v) => *v,
            DynamicValue::Long(v) => *v as i32,
            DynamicValue::Float(v) => *v as i32,
            DynamicValue::Double(v) => *v as i32,
            _ => default,
        }
    }

    pub fn as_string_result(&self) -> Option<String> {
        match &self.value {
            DynamicValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn create_short(&self, v: i16) -> Dynamic {
        Dynamic::new(DynamicValue::Short(v))
    }

    pub fn create_string(&self, v: &str) -> Dynamic {
        Dynamic::new(DynamicValue::String(v.to_string()))
    }

    pub fn create_list(&self, values: Vec<Dynamic>) -> Dynamic {
        Dynamic::new(DynamicValue::List(values.into_iter().map(|d| d.value).collect()))
    }

    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            if let DynamicValue::String(s) = k.value {
                map.insert(s, v.value);
            }
        }
        Dynamic::new(DynamicValue::Map(map))
    }

    pub fn is_map(&self) -> bool {
        matches!(self.value, DynamicValue::Map(_))
    }
}

impl fmt::Display for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn get_second(&self) -> &B {
        &self.1
    }
}

#[derive(Clone, Debug)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct TypeRewriteRule;

pub struct ItemBannerColorFix {
    _schema: Schema,
    _changes_type: bool,
}

impl ItemBannerColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemBannerColorFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemBannerColorFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // DFU rule construction is stubbed; the faithful transformation logic
        // is exposed via `fix_item_stack_dynamic`.
        TypeRewriteRule
    }

    /// Faithful port of the lambda body inside `makeRule`.
    /// Operates on a simplified item-stack dynamic representation:
    /// - top-level map may contain `id` (string) and `tag` (map)
    /// - `tag.BlockEntityTag.Base` is the banner base color
    /// - `Damage` is set on the top-level/remainder dynamic
    // JAVA: net.minecraft.util.datafix.fixes.ItemBannerColorFix#makeRule
    pub fn fix_item_stack_dynamic(&self, item_stack: Dynamic) -> Dynamic {
        let id = get_item_id(&item_stack);
        let is_banner = id
            .as_ref()
            .map(|p| p.get_second() == "minecraft:banner")
            .unwrap_or(false);

        if !is_banner {
            return item_stack;
        }

        let mut remainder = item_stack.clone();

        let tag = item_stack.get("tag");
        if !tag.is_map() {
            return remainder;
        }

        let block_entity_tag = tag.get("BlockEntityTag");
        if !block_entity_tag.is_map() {
            return remainder;
        }

        let base = block_entity_tag.get("Base");
        if base.as_number_result().is_none() {
            return remainder;
        }

        let base_int = base.as_int(0);
        let damage = (base_int & 15) as i16;
        remainder = remainder.set("Damage", remainder.create_short(damage));

        let display_opt = tag.get("display").result();
        if let Some(display) = display_opt {
            let lore_key = display.create_string("Lore");
            let lore_value = display.create_list(vec![display.create_string("(+NBT")]);
            let lore_only_display = display.create_map(vec![(lore_key, lore_value)]);
            if display == lore_only_display {
                return remainder;
            }
        }

        // Faithful to source: remove() result is discarded in the provided Java.
        let _ = block_entity_tag.remove("Base");

        let new_block_entity_tag = block_entity_tag;
        let new_tag = tag.set("BlockEntityTag", new_block_entity_tag);
        let mut out = remainder;
        out = out.set("tag", new_tag);
        out
    }
}

fn get_item_id(item_stack: &Dynamic) -> Option<Pair<String, String>> {
    let id = item_stack.get("id");
    let s = id.as_string_result()?;
    Some(Pair("id".to_string(), s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banner_stack(base: i32) -> Dynamic {
        let d = Dynamic::null();
        let block_entity_tag = d
            .create_map(vec![])
            .set("Base", Dynamic::new(DynamicValue::Int(base)));
        let tag = d.create_map(vec![]).set("BlockEntityTag", block_entity_tag);
        d.create_map(vec![])
            .set("id", Dynamic::new(DynamicValue::String("minecraft:banner".to_string())))
            .set("tag", tag)
    }

    #[test]
    fn non_banner_unchanged() {
        let fix = ItemBannerColorFix::new(Schema, false);
        let stack = Dynamic::null()
            .create_map(vec![])
            .set("id", Dynamic::new(DynamicValue::String("minecraft:stone".to_string())));
        let out = fix.fix_item_stack_dynamic(stack.clone());
        assert_eq!(out, stack);
    }

    #[test]
    fn banner_sets_damage_masked() {
        let fix = ItemBannerColorFix::new(Schema, false);
        let out = fix.fix_item_stack_dynamic(banner_stack(16 + 3));
        assert_eq!(out.get("Damage"), Dynamic::new(DynamicValue::Short(3)));
    }

    #[test]
    fn banner_negative_base_masks_to_15_bits() {
        let fix = ItemBannerColorFix::new(Schema, false);
        let out = fix.fix_item_stack_dynamic(banner_stack(-1));
        assert_eq!(out.get("Damage"), Dynamic::new(DynamicValue::Short(15)));
    }

    #[test]
    fn banner_with_only_lore_nbt_returns_remainder_only() {
        let fix = ItemBannerColorFix::new(Schema, false);
        let d = Dynamic::null();
        let display = d.create_map(vec![(
            d.create_string("Lore"),
            d.create_list(vec![d.create_string("(+NBT")]),
        )]);
        let block_entity_tag = d.create_map(vec![]).set("Base", Dynamic::new(DynamicValue::Int(5)));
        let tag = d
            .create_map(vec![])
            .set("BlockEntityTag", block_entity_tag)
            .set("display", display);
        let stack = d
            .create_map(vec![])
            .set("id", Dynamic::new(DynamicValue::String("minecraft:banner".to_string())))
            .set("tag", tag.clone());
        let out = fix.fix_item_stack_dynamic(stack);
        assert_eq!(out.get("Damage"), Dynamic::new(DynamicValue::Short(5)));
        // Early return path: returns item_stack.set(DSL.remainderFinder(), $$5), i.e. no tag rewrite.
        assert_eq!(out.get("tag"), tag);
    }

    #[test]
    fn missing_base_returns_remainder() {
        let fix = ItemBannerColorFix::new(Schema, false);
        let d = Dynamic::null();
        let block_entity_tag = d.create_map(vec![]);
        let tag = d.create_map(vec![]).set("BlockEntityTag", block_entity_tag);
        let stack = d
            .create_map(vec![])
            .set("id", Dynamic::new(DynamicValue::String("minecraft:banner".to_string())))
            .set("tag", tag.clone());
        let out = fix.fix_item_stack_dynamic(stack);
        assert_eq!(out.get("Damage"), Dynamic::null());
        assert_eq!(out.get("tag"), tag);
    }
}
