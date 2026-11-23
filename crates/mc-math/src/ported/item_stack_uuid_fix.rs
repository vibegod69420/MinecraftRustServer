use std::collections::BTreeMap;

/// Minimal self-contained dynamic representation for datafix porting.
/// Mirrors the subset of `com.mojang.serialization.Dynamic` operations used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    IntArray(Vec<i32>),
    Byte(i8),
    Short(i16),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(list) => list.clone(),
            _ => Vec::new(),
        }
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(items)
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                let child = map.remove(key).unwrap_or(Dynamic::Null);
                let updated = f(child);
                map.insert(key.to_string(), updated);
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                map.insert(key.to_string(), value);
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                map.remove(key);
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

/// Minimal typed wrapper representing DFU `Typed<?>`.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn new(value: Dynamic) -> Self {
        Self { value }
    }

    pub fn get_optional_id(&self) -> Option<String> {
        self.value.get("id").and_then(|v| v.as_string()).map(|s| s.to_string())
    }

    pub fn update_tag<F>(self, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed::new(self.value.update("tag", f))
    }
}

/// Helper corresponding to `AbstractUUIDFix#replaceUUIDLeastMost`.
/// Replaces an old UUID representation (most/least) with an int-array "UUID" field.
/// This is a faithful simplified port of the common DFU UUID replacement pattern.
fn replace_uuid_least_most(dynamic: Dynamic, old_key: &str, new_key: &str) -> Option<Dynamic> {
    let most_key = format!("{}Most", old_key);
    let least_key = format!("{}Least", old_key);
    let most = dynamic.get(&most_key).and_then(|v| match v {
        Dynamic::Long(v) => Some(*v),
        _ => None,
    });
    let least = dynamic.get(&least_key).and_then(|v| match v {
        Dynamic::Long(v) => Some(*v),
        _ => None,
    });

    if let (Some(m), Some(l)) = (most, least) {
        let uuid_arr = vec![
            ((m >> 32) & 0xFFFF_FFFF) as i32,
            (m & 0xFFFF_FFFF) as i32,
            ((l >> 32) & 0xFFFF_FFFF) as i32,
            (l & 0xFFFF_FFFF) as i32,
        ];
        let updated = dynamic.remove(&most_key).remove(&least_key).set(new_key, Dynamic::IntArray(uuid_arr));
        Some(updated)
    } else {
        None
    }
}

/// Helper corresponding to `AbstractUUIDFix#replaceUUIDString`.
/// Converts a string UUID "Id" to an int-array UUID representation.
fn replace_uuid_string(dynamic: Dynamic, old_key: &str, new_key: &str) -> Option<Dynamic> {
    let s = dynamic.get(old_key).and_then(|v| v.as_string())?;
    let parsed = parse_uuid_string_to_int_array(s)?;
    Some(dynamic.remove(old_key).set(new_key, Dynamic::IntArray(parsed)))
}

fn parse_uuid_string_to_int_array(s: &str) -> Option<Vec<i32>> {
    // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let hex: String = s.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        let byte_str = &hex[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16).ok()?;
    }
    let mut out = Vec::with_capacity(4);
    for i in 0..4 {
        let val = i32::from_be_bytes([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]]);
        out.push(val);
    }
    Some(out)
}

/// Faithful structural port of `net.minecraft.util.datafix.fixes.ItemStackUUIDFix`.
pub struct ItemStackUUIDFix {
    pub name: String,
}

impl ItemStackUUIDFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackUUIDFix#<init>
    pub fn new() -> Self {
        Self {
            name: "ItemStackUUIDFix".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackUUIDFix#updateAttributeModifiers
    fn update_attribute_modifiers(&self, dynamic: Dynamic) -> Dynamic {
        dynamic.update("AttributeModifiers", |modifiers| {
            let items = modifiers.as_stream();
            let mapped: Vec<Dynamic> = items
                .into_iter()
                .map(|entry| replace_uuid_least_most(entry.clone(), "UUID", "UUID").unwrap_or(entry))
                .collect();
            modifiers.create_list(mapped)
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackUUIDFix#updateSkullOwner
    fn update_skull_owner(&self, dynamic: Dynamic) -> Dynamic {
        dynamic.update("SkullOwner", |owner| replace_uuid_string(owner.clone(), "Id", "Id").unwrap_or(owner))
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackUUIDFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Typed) -> Typed + '_ {
        move |typed: Typed| {
            let is_player_head = typed
                .get_optional_id()
                .map(|id| id == "minecraft:player_head")
                .unwrap_or(false);
            typed.update_tag(|tag| {
                let mut tag = self.update_attribute_modifiers(tag);
                if is_player_head {
                    tag = self.update_skull_owner(tag);
                }
                tag
            })
        }
    }

    pub fn apply(&self, typed: Typed) -> Typed {
        let rule = self.make_rule();
        rule(typed)
    }
}

impl Default for ItemStackUUIDFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_stack(id: &str, tag: Dynamic) -> Typed {
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), Dynamic::String(id.to_string()));
        map.insert("tag".to_string(), tag);
        Typed::new(Dynamic::Map(map))
    }

    #[test]
    fn test_attribute_modifiers_uuid_least_most_replaced() {
        let fix = ItemStackUUIDFix::new();
        let mut modifier = BTreeMap::new();
        modifier.insert("UUIDMost".to_string(), Dynamic::Long(0x0123_4567_89ab_cdef_u64 as i64));
        modifier.insert("UUIDLeast".to_string(), Dynamic::Long(0x0fed_cba9_8765_4321_u64 as i64));
        modifier.insert("Name".to_string(), Dynamic::String("test".to_string()));
        let mut tag = BTreeMap::new();
        tag.insert("AttributeModifiers".to_string(), Dynamic::List(vec![Dynamic::Map(modifier)]));
        let input = item_stack("minecraft:stone", Dynamic::Map(tag));
        let output = fix.apply(input);
        let out_tag = output.value.get("tag").unwrap();
        let mods = out_tag.get("AttributeModifiers").unwrap().as_stream();
        assert_eq!(mods.len(), 1);
        let m = &mods[0];
        assert!(m.get("UUIDMost").is_none());
        assert!(m.get("UUIDLeast").is_none());
        match m.get("UUID").unwrap() {
            Dynamic::IntArray(arr) => {
                assert_eq!(arr, &vec![0x0123_4567, 0x89ab_cdef_u32 as i32, 0x0fed_cba9_u32 as i32, 0x8765_4321_u32 as i32]);
            }
            other => panic!("expected IntArray, got {:?}", other),
        }
        assert_eq!(m.get("Name"), Some(&Dynamic::String("test".to_string())));
    }

    #[test]
    fn test_player_head_skull_owner_uuid_string_replaced() {
        let fix = ItemStackUUIDFix::new();
        let mut skull_owner = BTreeMap::new();
        skull_owner.insert("Id".to_string(), Dynamic::String("12345678-1234-5678-9abc-def012345678".to_string()));
        skull_owner.insert("Name".to_string(), Dynamic::String("Player".to_string()));
        let mut tag = BTreeMap::new();
        tag.insert("SkullOwner".to_string(), Dynamic::Map(skull_owner));
        let input = item_stack("minecraft:player_head", Dynamic::Map(tag));
        let output = fix.apply(input);
        let owner = output.value.get("tag").unwrap().get("SkullOwner").unwrap();
        assert!(owner.get("Id").is_some());
        match owner.get("Id").unwrap() {
            Dynamic::IntArray(arr) => assert_eq!(arr.len(), 4),
            other => panic!("expected IntArray, got {:?}", other),
        }
        assert_eq!(owner.get("Name"), Some(&Dynamic::String("Player".to_string())));
    }

    #[test]
    fn test_non_player_head_skull_owner_not_changed() {
        let fix = ItemStackUUIDFix::new();
        let mut skull_owner = BTreeMap::new();
        skull_owner.insert("Id".to_string(), Dynamic::String("12345678-1234-5678-9abc-def012345678".to_string()));
        let mut tag = BTreeMap::new();
        tag.insert("SkullOwner".to_string(), Dynamic::Map(skull_owner));
        let input = item_stack("minecraft:stone", Dynamic::Map(tag));
        let output = fix.apply(input);
        let owner = output.value.get("tag").unwrap().get("SkullOwner").unwrap();
        assert_eq!(
            owner.get("Id"),
            Some(&Dynamic::String("12345678-1234-5678-9abc-def012345678".to_string()))
        );
    }

    #[test]
    fn test_invalid_uuid_string_left_alone() {
        let fix = ItemStackUUIDFix::new();
        let mut skull_owner = BTreeMap::new();
        skull_owner.insert("Id".to_string(), Dynamic::String("not-a-uuid".to_string()));
        let mut tag = BTreeMap::new();
        tag.insert("SkullOwner".to_string(), Dynamic::Map(skull_owner));
        let input = item_stack("minecraft:player_head", Dynamic::Map(tag));
        let output = fix.apply(input);
        let owner = output.value.get("tag").unwrap().get("SkullOwner").unwrap();
        assert_eq!(owner.get("Id"), Some(&Dynamic::String("not-a-uuid".to_string())));
    }

    #[test]
    fn test_modifier_without_uuid_fields_unchanged() {
        let fix = ItemStackUUIDFix::new();
        let mut modifier = BTreeMap::new();
        modifier.insert("Name".to_string(), Dynamic::String("speed".to_string()));
        let mut tag = BTreeMap::new();
        tag.insert("AttributeModifiers".to_string(), Dynamic::List(vec![Dynamic::Map(modifier.clone())]));
        let input = item_stack("minecraft:diamond", Dynamic::Map(tag));
        let output = fix.apply(input);
        let mods = output.value.get("tag").unwrap().get("AttributeModifiers").unwrap().as_stream();
        assert_eq!(mods[0], Dynamic::Map(modifier));
    }
}
