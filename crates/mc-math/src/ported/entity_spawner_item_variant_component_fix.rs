// mc-math/src/ported/entity_spawner_item_variant_component_fix.rs

use std::fmt;

/// Minimal dynamic node representation for 1:1 porting of DataFixer Dynamic operations.
/// This is a self-contained, serde_json-like value type sufficient for the fix logic.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    String(String),
    Number(f64),
    Object(Vec<(String, DynamicValue)>),
    Array(Vec<DynamicValue>),
}

impl DynamicValue {
    pub fn empty_map() -> Self {
        DynamicValue::Object(Vec::new())
    }

    pub fn is_empty_map(&self) -> bool {
        matches!(self, DynamicValue::Object(entries) if entries.is_empty())
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            DynamicValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> DynamicRef<'_> {
        match self {
            DynamicValue::Object(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return DynamicRef::Some(v);
                    }
                }
                DynamicRef::None
            }
            _ => DynamicRef::None,
        }
    }

    pub fn get_or_empty(&self, key: &str) -> DynamicValue {
        match self.get(key) {
            DynamicRef::Some(v) => v.clone(),
            DynamicRef::None => DynamicValue::empty_map(),
        }
    }

    pub fn set(&self, key: &str, value: DynamicValue) -> DynamicValue {
        match self {
            DynamicValue::Object(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), value));
                }
                DynamicValue::Object(new_entries)
            }
            _ => {
                let mut entries = Vec::new();
                entries.push((key.to_string(), value));
                DynamicValue::Object(entries)
            }
        }
    }

    pub fn remove(&self, key: &str) -> DynamicValue {
        match self {
            DynamicValue::Object(entries) => {
                let new_entries: Vec<(String, DynamicValue)> = entries
                    .iter()
                    .filter(|(k, _)| k != key)
                    .cloned()
                    .collect();
                DynamicValue::Object(new_entries)
            }
            _ => self.clone(),
        }
    }

    pub fn update<F>(&self, key: &str, f: F) -> DynamicValue
    where
        F: FnOnce(DynamicValue) -> DynamicValue,
    {
        let current = self.get_or_empty(key);
        let updated = f(current);
        self.set(key, updated)
    }

    pub fn create_string(&self, s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    pub fn equals(&self, other: &DynamicValue) -> bool {
        self == other
    }
}

impl fmt::Display for DynamicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DynamicValue::Null => write!(f, "null"),
            DynamicValue::String(s) => write!(f, "\"{}\"", s),
            DynamicValue::Number(n) => write!(f, "{}", n),
            DynamicValue::Object(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            DynamicValue::Array(items) => {
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// A lightweight optional reference returned by `DynamicValue::get`.
#[derive(Clone, Debug)]
pub enum DynamicRef<'a> {
    Some(&'a DynamicValue),
    None,
}

impl<'a> DynamicRef<'a> {
    pub fn result(self) -> Option<&'a DynamicValue> {
        match self {
            DynamicRef::Some(v) => Some(v),
            DynamicRef::None => None,
        }
    }

    pub fn as_string(self) -> DynamicRefString<'a> {
        DynamicRefString(self.result())
    }

    pub fn as_number(self) -> DynamicRefNumber<'a> {
        DynamicRefNumber(self.result())
    }
}

pub struct DynamicRefString<'a>(Option<&'a DynamicValue>);

impl<'a> DynamicRefString<'a> {
    pub fn result(self) -> Option<&'a str> {
        self.0.and_then(|v| v.as_string())
    }
}

pub struct DynamicRefNumber<'a>(Option<&'a DynamicValue>);

impl<'a> DynamicRefNumber<'a> {
    pub fn result(self) -> Option<f64> {
        self.0.and_then(|v| v.as_number())
    }
}

/// Minimal equivalent of `net.minecraft.util.datafix.ExtraDataFixUtils.dyeColorIdToName`.
/// Java: returns the dye color name for a color id (0-15), with a default for out-of-range.
/// The actual Java method maps ids to names; we replicate the standard 16 dye colors.
fn dye_color_id_to_name(id: i32) -> &'static str {
    // JAVA: ExtraDataFixUtils.dyeColorIdToName(int) - standard dye color mapping
    match id & 15 {
        0 => "white",
        1 => "orange",
        2 => "magenta",
        3 => "light_blue",
        4 => "yellow",
        5 => "lime",
        6 => "pink",
        7 => "gray",
        8 => "light_gray",
        9 => "cyan",
        10 => "purple",
        11 => "blue",
        12 => "brown",
        13 => "green",
        14 => "red",
        15 => "black",
        _ => "white",
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#getBaseColor
fn get_base_color(variant: i32) -> &'static str {
    // Java: ExtraDataFixUtils.dyeColorIdToName($$0 >> 16 & 0xFF)
    dye_color_id_to_name((variant >> 16) & 0xFF)
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#getPatternColor
fn get_pattern_color(variant: i32) -> &'static str {
    // Java: ExtraDataFixUtils.dyeColorIdToName($$0 >> 24 & 0xFF)
    dye_color_id_to_name((variant >> 24) & 0xFF)
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#getPattern
fn get_pattern(variant: i32) -> &'static str {
    // Java: switch ($$0 & 65535)
    match variant & 65535 {
        1 => "flopper",
        256 => "sunstreak",
        257 => "stripey",
        512 => "snooper",
        513 => "glitter",
        768 => "dasher",
        769 => "blockfish",
        1024 => "brinely",
        1025 => "betty",
        1280 => "spotty",
        1281 => "clayfish",
        _ => "kob",
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#fixTropicalFishBucket
fn fix_tropical_fish_bucket(components: &DynamicValue, bucket_entity_data: &DynamicValue) -> DynamicValue {
    // Optional<Number> $$2 = $$1.get("BucketVariantTag").asNumber().result();
    let bucket_variant_tag = bucket_entity_data.get("BucketVariantTag").as_number().result();
    if bucket_variant_tag.is_none() {
        return components.clone();
    }

    // int $$3 = $$2.get().intValue();
    let variant = bucket_variant_tag.unwrap() as i32;
    let pattern = get_pattern(variant);
    let base_color = get_base_color(variant);
    let pattern_color = get_pattern_color(variant);

    // return $$0.update("minecraft:bucket_entity_data", $$0x -> $$0x.remove("BucketVariantTag"))
    //     .set("minecraft:tropical_fish/pattern", $$0.createString($$4))
    //     .set("minecraft:tropical_fish/base_color", $$0.createString($$5))
    //     .set("minecraft:tropical_fish/pattern_color", $$0.createString($$6));
    components
        .update("minecraft:bucket_entity_data", |bed| {
            bed.remove("BucketVariantTag")
        })
        .set(
            "minecraft:tropical_fish/pattern",
            components.create_string(pattern),
        )
        .set(
            "minecraft:tropical_fish/base_color",
            components.create_string(base_color),
        )
        .set(
            "minecraft:tropical_fish/pattern_color",
            components.create_string(pattern_color),
        )
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#fixAxolotlBucket
fn fix_axolotl_bucket(components: &DynamicValue, bucket_entity_data: &DynamicValue) -> DynamicValue {
    // Optional<Number> $$2 = $$1.get("Variant").asNumber().result();
    let variant_opt = bucket_entity_data.get("Variant").as_number().result();
    if variant_opt.is_none() {
        return components.clone();
    }

    let variant_id = variant_opt.unwrap() as i32;
    let variant_name = match variant_id {
        1 => "wild",
        2 => "gold",
        3 => "cyan",
        4 => "blue",
        _ => "lucy",
    };

    // return $$0.update("minecraft:bucket_entity_data", $$0x -> $$0x.remove("Variant")).set("minecraft:axolotl/variant", $$0.createString($$3));
    components
        .update("minecraft:bucket_entity_data", |bed| bed.remove("Variant"))
        .set(
            "minecraft:axolotl/variant",
            components.create_string(variant_name),
        )
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#fixSalmonBucket
fn fix_salmon_bucket(components: &DynamicValue, bucket_entity_data: &DynamicValue) -> DynamicValue {
    // Optional<Dynamic<T>> $$2 = $$1.get("type").result();
    let type_result = bucket_entity_data.get("type").result();
    if type_result.is_none() {
        return components.clone();
    }
    let type_val = type_result.unwrap().clone();

    // return $$2.isEmpty() ? $$0 : $$0.update("minecraft:bucket_entity_data", $$0x -> $$0x.remove("type")).set("minecraft:salmon/size", $$2.get());
    components
        .update("minecraft:bucket_entity_data", |bed| bed.remove("type"))
        .set("minecraft:salmon/size", type_val)
}

// JAVA: net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix#fixPainting
fn fix_painting(components: &DynamicValue) -> DynamicValue {
    // Optional<Dynamic<T>> $$1 = $$0.get("minecraft:entity_data").result();
    let entity_data_opt = components.get("minecraft:entity_data").result();
    if entity_data_opt.is_none() {
        return components.clone();
    }
    let entity_data = entity_data_opt.unwrap();

    // if ($$1.get().get("id").asString().result().filter($$0x -> $$0x.equals("minecraft:painting")).isEmpty()) { return $$0; }
    let id_matches = entity_data
        .get("id")
        .as_string()
        .result()
        .map(|s| s == "minecraft:painting")
        .unwrap_or(false);
    if !id_matches {
        return components.clone();
    }

    // Optional<Dynamic<T>> $$2 = $$1.get().get("variant").result();
    let variant_opt = entity_data.get("variant").result().cloned();
    // Dynamic<T> $$3 = $$1.get().remove("variant");
    let entity_data_without_variant = entity_data.remove("variant");

    let mut result = components.clone();
    // if ($$3.remove("id").equals($$3.emptyMap())) { $$0 = $$0.remove("minecraft:entity_data"); }
    // else { $$0 = $$0.set("minecraft:entity_data", $$3); }
    let without_id = entity_data_without_variant.remove("id");
    if without_id.equals(&DynamicValue::empty_map()) {
        result = result.remove("minecraft:entity_data");
    } else {
        result = result.set("minecraft:entity_data", entity_data_without_variant);
    }

    // if ($$2.isPresent()) { $$0 = $$0.set("minecraft:painting/variant", $$2.get()); }
    if let Some(variant) = variant_opt {
        result = result.set("minecraft:painting/variant", variant);
    }

    result
}

/// Applies the fixer logic for a component dynamic, dispatching based on bucket_entity_data presence.
/// This corresponds to the `Fixer` functional interface behavior in Java.
fn fix_remainder(components: &DynamicValue, item_id: &str) -> DynamicValue {
    // default <T> Dynamic<T> fixRemainder(Dynamic<T> $$0) {
    //     return $$0.get("minecraft:bucket_entity_data").result().map($$1 -> this.fixRemainder($$0, (Dynamic<T>)$$1)).orElse($$0);
    // }
    let bucket_entity_data_opt = components.get("minecraft:bucket_entity_data").result().cloned();
    match bucket_entity_data_opt {
        Some(bed) => fix_remainder_with_bucket(components, &bed, item_id),
        None => components.clone(),
    }
}

fn fix_remainder_with_bucket(components: &DynamicValue, bucket_entity_data: &DynamicValue, item_id: &str) -> DynamicValue {
    match item_id {
        "minecraft:salmon_bucket" => fix_salmon_bucket(components, bucket_entity_data),
        "minecraft:axolotl_bucket" => fix_axolotl_bucket(components, bucket_entity_data),
        "minecraft:tropical_fish_bucket" => fix_tropical_fish_bucket(components, bucket_entity_data),
        _ => components.clone(),
    }
}

/// Top-level fix entry point for an item stack represented as (item_id, components).
/// Mirrors the `makeRule` dispatch logic for the relevant item IDs.
///
/// Returns the (possibly modified) components dynamic.
pub fn fix_item_stack_components(item_id: &str, components: &DynamicValue) -> DynamicValue {
    match item_id {
        "minecraft:salmon_bucket" => {
            let bed = components.get("minecraft:bucket_entity_data").result().cloned();
            match bed {
                Some(bed) => fix_salmon_bucket(components, &bed),
                None => components.clone(),
            }
        }
        "minecraft:axolotl_bucket" => {
            let bed = components.get("minecraft:bucket_entity_data").result().cloned();
            match bed {
                Some(bed) => fix_axolotl_bucket(components, &bed),
                None => components.clone(),
            }
        }
        "minecraft:tropical_fish_bucket" => {
            let bed = components.get("minecraft:bucket_entity_data").result().cloned();
            match bed {
                Some(bed) => fix_tropical_fish_bucket(components, &bed),
                None => components.clone(),
            }
        }
        "minecraft:painting" => fix_painting(components),
        _ => components.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Object(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn str_val(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    fn num_val(n: f64) -> DynamicValue {
        DynamicValue::Number(n)
    }

    #[test]
    fn test_get_pattern_known_and_default() {
        assert_eq!(get_pattern(1), "flopper");
        assert_eq!(get_pattern(256), "sunstreak");
        assert_eq!(get_pattern(257), "stripey");
        assert_eq!(get_pattern(512), "snooper");
        assert_eq!(get_pattern(513), "glitter");
        assert_eq!(get_pattern(768), "dasher");
        assert_eq!(get_pattern(769), "blockfish");
        assert_eq!(get_pattern(1024), "brinely");
        assert_eq!(get_pattern(1025), "betty");
        assert_eq!(get_pattern(1280), "spotty");
        assert_eq!(get_pattern(1281), "clayfish");
        assert_eq!(get_pattern(0), "kob");
        assert_eq!(get_pattern(999), "kob");
        // Only low 16 bits matter
        assert_eq!(get_pattern(65536 + 1), "flopper");
    }

    #[test]
    fn test_tropical_fish_bucket_fix() {
        // Variant encoding: (pattern_color << 24) | (base_color << 16) | pattern
        // pattern=1 (flopper), base_color=14 (red), pattern_color=0 (white)
        let variant: i32 = (0 << 24) | (14 << 16) | 1;
        let components = obj(vec![(
            "minecraft:bucket_entity_data",
            obj(vec![("BucketVariantTag", num_val(variant as f64))]),
        )]);

        let fixed = fix_item_stack_components("minecraft:tropical_fish_bucket", &components);

        assert_eq!(
            fixed.get("minecraft:tropical_fish/pattern").as_string().result(),
            Some("flopper")
        );
        assert_eq!(
            fixed.get("minecraft:tropical_fish/base_color").as_string().result(),
            Some("red")
        );
        assert_eq!(
            fixed.get("minecraft:tropical_fish/pattern_color").as_string().result(),
            Some("white")
        );
        // BucketVariantTag should be removed from bucket_entity_data
        let bed = fixed.get("minecraft:bucket_entity_data").result().unwrap();
        assert!(bed.get("BucketVariantTag").result().is_none());
    }

    #[test]
    fn test_axolotl_bucket_fix_variants() {
        for (id, name) in [(0, "lucy"), (1, "wild"), (2, "gold"), (3, "cyan"), (4, "blue"), (99, "lucy")] {
            let components = obj(vec![(
                "minecraft:bucket_entity_data",
                obj(vec![("Variant", num_val(id as f64))]),
            )]);
            let fixed = fix_item_stack_components("minecraft:axolotl_bucket", &components);
            assert_eq!(
                fixed.get("minecraft:axolotl/variant").as_string().result(),
                Some(name),
                "variant id {} should map to {}",
                id,
                name
            );
            let bed = fixed.get("minecraft:bucket_entity_data").result().unwrap();
            assert!(bed.get("Variant").result().is_none());
        }
    }

    #[test]
    fn test_salmon_bucket_fix() {
        let components = obj(vec![(
            "minecraft:bucket_entity_data",
            obj(vec![("type", str_val("large"))]),
        )]);
        let fixed = fix_item_stack_components("minecraft:salmon_bucket", &components);
        assert_eq!(
            fixed.get("minecraft:salmon/size").as_string().result(),
            Some("large")
        );
        let bed = fixed.get("minecraft:bucket_entity_data").result().unwrap();
        assert!(bed.get("type").result().is_none());

        // Missing type -> no change
        let components_no_type = obj(vec![(
            "minecraft:bucket_entity_data",
            obj(vec![("other", str_val("x"))]),
        )]);
        let fixed2 = fix_item_stack_components("minecraft:salmon_bucket", &components_no_type);
        assert_eq!(fixed2, components_no_type);
    }

    #[test]
    fn test_painting_fix_moves_variant_and_cleans_entity_data() {
        let components = obj(vec![(
            "minecraft:entity_data",
            obj(vec![
                ("id", str_val("minecraft:painting")),
                ("variant", str_val("minecraft:kebab")),
            ]),
        )]);
        let fixed = fix_item_stack_components("minecraft:painting", &components);
        assert_eq!(
            fixed.get("minecraft:painting/variant").as_string().result(),
            Some("minecraft:kebab")
        );
        // entity_data only had id+variant, removing variant+id leaves empty -> entity_data removed
        assert!(fixed.get("minecraft:entity_data").result().is_none());

        // If entity_data has extra fields, keep it (without variant)
        let components2 = obj(vec![(
            "minecraft:entity_data",
            obj(vec![
                ("id", str_val("minecraft:painting")),
                ("variant", str_val("minecraft:aztec")),
                ("CustomName", str_val("Art")),
            ]),
        )]);
        let fixed2 = fix_item_stack_components("minecraft:painting", &components2);
        assert_eq!(
            fixed2.get("minecraft:painting/variant").as_string().result(),
            Some("minecraft:aztec")
        );
        let ed = fixed2.get("minecraft:entity_data").result().unwrap();
        assert!(ed.get("variant").result().is_none());
        assert_eq!(ed.get("id").as_string().result(), Some("minecraft:painting"));
        assert_eq!(ed.get("CustomName").as_string().result(), Some("Art"));
    }

    #[test]
    fn test_painting_fix_no_op_cases() {
        // No entity_data
        let components = obj(vec![("other", str_val("x"))]);
        assert_eq!(fix_item_stack_components("minecraft:painting", &components), components);

        // entity_data with wrong id
        let components2 = obj(vec![(
            "minecraft:entity_data",
            obj(vec![
                ("id", str_val("minecraft:creeper")),
                ("variant", str_val("minecraft:kebab")),
            ]),
        )]);
        assert_eq!(fix_item_stack_components("minecraft:painting", &components2), components2);

        // Unknown item id
        let components3 = obj(vec![("minecraft:bucket_entity_data", obj(vec![("Variant", num_val(1.0))]))]);
        assert_eq!(fix_item_stack_components("minecraft:stone", &components3), components3);
    }
}
