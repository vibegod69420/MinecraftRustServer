use std::collections::BTreeMap;

pub const SPLASH: i32 = 16384;
pub const DEFAULT: &str = "minecraft:water";

pub const POTIONS: [Option<&str>; 128] = [
    Some("minecraft:water"),              // 0
    Some("minecraft:regeneration"),       // 1
    Some("minecraft:swiftness"),          // 2
    Some("minecraft:fire_resistance"),    // 3
    Some("minecraft:poison"),             // 4
    Some("minecraft:healing"),            // 5
    Some("minecraft:night_vision"),       // 6
    None,                                 // 7
    Some("minecraft:weakness"),           // 8
    Some("minecraft:strength"),           // 9
    Some("minecraft:slowness"),           // 10
    Some("minecraft:leaping"),            // 11
    Some("minecraft:harming"),            // 12
    Some("minecraft:water_breathing"),    // 13
    Some("minecraft:invisibility"),       // 14
    None,                                 // 15
    Some("minecraft:awkward"),            // 16
    Some("minecraft:regeneration"),       // 17
    Some("minecraft:swiftness"),          // 18
    Some("minecraft:fire_resistance"),    // 19
    Some("minecraft:poison"),             // 20
    Some("minecraft:healing"),            // 21
    Some("minecraft:night_vision"),       // 22
    None,                                 // 23
    Some("minecraft:weakness"),           // 24
    Some("minecraft:strength"),           // 25
    Some("minecraft:slowness"),           // 26
    Some("minecraft:leaping"),            // 27
    Some("minecraft:harming"),            // 28
    Some("minecraft:water_breathing"),    // 29
    Some("minecraft:invisibility"),       // 30
    None,                                 // 31
    Some("minecraft:thick"),              // 32
    Some("minecraft:strong_regeneration"),// 33
    Some("minecraft:strong_swiftness"),   // 34
    Some("minecraft:fire_resistance"),    // 35
    Some("minecraft:strong_poison"),      // 36
    Some("minecraft:strong_healing"),     // 37
    Some("minecraft:night_vision"),       // 38
    None,                                 // 39
    Some("minecraft:weakness"),           // 40
    Some("minecraft:strong_strength"),    // 41
    Some("minecraft:slowness"),           // 42
    Some("minecraft:strong_leaping"),     // 43
    Some("minecraft:strong_harming"),     // 44
    Some("minecraft:water_breathing"),    // 45
    Some("minecraft:invisibility"),       // 46
    None,                                 // 47
    None,                                 // 48
    Some("minecraft:strong_regeneration"),// 49
    Some("minecraft:strong_swiftness"),   // 50
    Some("minecraft:fire_resistance"),    // 51
    Some("minecraft:strong_poison"),      // 52
    Some("minecraft:strong_healing"),     // 53
    Some("minecraft:night_vision"),       // 54
    None,                                 // 55
    Some("minecraft:weakness"),           // 56
    Some("minecraft:strong_strength"),    // 57
    Some("minecraft:slowness"),           // 58
    Some("minecraft:strong_leaping"),     // 59
    Some("minecraft:strong_harming"),     // 60
    Some("minecraft:water_breathing"),    // 61
    Some("minecraft:invisibility"),       // 62
    None,                                 // 63
    Some("minecraft:mundane"),            // 64
    Some("minecraft:long_regeneration"),  // 65
    Some("minecraft:long_swiftness"),     // 66
    Some("minecraft:long_fire_resistance"),// 67
    Some("minecraft:long_poison"),        // 68
    Some("minecraft:healing"),            // 69
    Some("minecraft:long_night_vision"),  // 70
    None,                                 // 71
    Some("minecraft:long_weakness"),      // 72
    Some("minecraft:long_strength"),      // 73
    Some("minecraft:long_slowness"),      // 74
    Some("minecraft:long_leaping"),       // 75
    Some("minecraft:harming"),            // 76
    Some("minecraft:long_water_breathing"),// 77
    Some("minecraft:long_invisibility"),  // 78
    None,                                 // 79
    Some("minecraft:awkward"),            // 80
    Some("minecraft:long_regeneration"),  // 81
    Some("minecraft:long_swiftness"),     // 82
    Some("minecraft:long_fire_resistance"),// 83
    Some("minecraft:long_poison"),        // 84
    Some("minecraft:healing"),            // 85
    Some("minecraft:long_night_vision"),  // 86
    None,                                 // 87
    Some("minecraft:long_weakness"),      // 88
    Some("minecraft:long_strength"),      // 89
    Some("minecraft:long_slowness"),      // 90
    Some("minecraft:long_leaping"),       // 91
    Some("minecraft:harming"),            // 92
    Some("minecraft:long_water_breathing"),// 93
    Some("minecraft:long_invisibility"),  // 94
    None,                                 // 95
    Some("minecraft:thick"),              // 96
    Some("minecraft:regeneration"),       // 97
    Some("minecraft:swiftness"),          // 98
    Some("minecraft:long_fire_resistance"),// 99
    Some("minecraft:poison"),             // 100
    Some("minecraft:strong_healing"),     // 101
    Some("minecraft:long_night_vision"),  // 102
    None,                                 // 103
    Some("minecraft:long_weakness"),      // 104
    Some("minecraft:strength"),           // 105
    Some("minecraft:long_slowness"),      // 106
    Some("minecraft:leaping"),            // 107
    Some("minecraft:strong_harming"),     // 108
    Some("minecraft:long_water_breathing"),// 109
    Some("minecraft:long_invisibility"),  // 110
    None,                                 // 111
    None,                                 // 112
    Some("minecraft:regeneration"),       // 113
    Some("minecraft:swiftness"),          // 114
    Some("minecraft:long_fire_resistance"),// 115
    Some("minecraft:poison"),             // 116
    Some("minecraft:strong_healing"),     // 117
    Some("minecraft:long_night_vision"),  // 118
    None,                                 // 119
    Some("minecraft:long_weakness"),      // 120
    Some("minecraft:strength"),           // 121
    Some("minecraft:long_slowness"),      // 122
    Some("minecraft:leaping"),            // 123
    Some("minecraft:strong_harming"),     // 124
    Some("minecraft:long_water_breathing"),// 125
    Some("minecraft:long_invisibility"),  // 126
    None,                                 // 127
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagValue {
    String(String),
    Short(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStackData {
    pub id: String,
    pub damage: i16,
    pub tag: Option<BTreeMap<String, TagValue>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ItemPotionFix;

impl ItemPotionFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemPotionFix#ItemPotionFix
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemPotionFix#makeRule
    pub fn make_rule(&self, stack: &ItemStackData) -> ItemStackData {
        let mut out = stack.clone();
        if out.id != "minecraft:potion" {
            return out;
        }

        let damage: i16 = out.damage;
        let damage_i32: i32 = damage as i32;

        if let Some(tag) = out.tag.as_mut() {
            let has_potion_string = matches!(tag.get("Potion"), Some(TagValue::String(_)));
            if !has_potion_string {
                let idx = ((damage_i32) & 127) as usize;
                let potion = POTIONS[idx].unwrap_or(DEFAULT);
                tag.insert("Potion".to_string(), TagValue::String(potion.to_string()));

                if (damage_i32 & SPLASH) == SPLASH {
                    out.id = "minecraft:splash_potion".to_string();
                }
            }

            if damage != 0i16 {
                out.damage = 0i16;
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, damage: i16, tag: Option<BTreeMap<String, TagValue>>) -> ItemStackData {
        ItemStackData {
            id: id.to_string(),
            damage,
            tag,
        }
    }

    #[test]
    fn fixes_potion_without_potion_tag() {
        let fix = ItemPotionFix::new();
        let input = stack("minecraft:potion", 1, Some(BTreeMap::new()));
        let out = fix.make_rule(&input);
        assert_eq!(out.id, "minecraft:potion");
        assert_eq!(out.damage, 0);
        assert_eq!(
            out.tag.as_ref().unwrap().get("Potion"),
            Some(&TagValue::String("minecraft:regeneration".to_string()))
        );
    }

    #[test]
    fn splash_potion_from_damage_bit() {
        let fix = ItemPotionFix::new();
        let damage = ((SPLASH) | 2) as i16; // splash + swiftness
        let input = stack("minecraft:potion", damage, Some(BTreeMap::new()));
        let out = fix.make_rule(&input);
        assert_eq!(out.id, "minecraft:splash_potion");
        assert_eq!(out.damage, 0);
        assert_eq!(
            out.tag.as_ref().unwrap().get("Potion"),
            Some(&TagValue::String("minecraft:swiftness".to_string()))
        );
    }

    #[test]
    fn negative_damage_masks_index_and_splash() {
        let fix = ItemPotionFix::new();
        let input = stack("minecraft:potion", -1i16, Some(BTreeMap::new()));
        let out = fix.make_rule(&input);
        // -1i16 & 127 == 127 -> POTIONS[127] is None => DEFAULT
        // -1i16 & 16384 == 16384 => splash
        assert_eq!(out.id, "minecraft:splash_potion");
        assert_eq!(out.damage, 0);
        assert_eq!(
            out.tag.as_ref().unwrap().get("Potion"),
            Some(&TagValue::String("minecraft:water".to_string()))
        );
    }

    #[test]
    fn existing_potion_preserved_damage_reset_and_extra_fields_kept() {
        let fix = ItemPotionFix::new();
        let mut tag = BTreeMap::new();
        tag.insert(
            "Potion".to_string(),
            TagValue::String("minecraft:custom".to_string()),
        );
        tag.insert("Custom".to_string(), TagValue::Short(42));
        let input = stack("minecraft:potion", 5, Some(tag));
        let out = fix.make_rule(&input);
        assert_eq!(out.id, "minecraft:potion");
        assert_eq!(out.damage, 0);
        let out_tag = out.tag.as_ref().unwrap();
        assert_eq!(
            out_tag.get("Potion"),
            Some(&TagValue::String("minecraft:custom".to_string()))
        );
        assert_eq!(out_tag.get("Custom"), Some(&TagValue::Short(42)));
    }

    #[test]
    fn no_tag_or_non_potion_item_unchanged() {
        let fix = ItemPotionFix::new();
        let no_tag = stack("minecraft:potion", 5, None);
        assert_eq!(fix.make_rule(&no_tag), no_tag);

        let other_item = stack("minecraft:apple", 3, Some(BTreeMap::new()));
        assert_eq!(fix.make_rule(&other_item), other_item);
    }

    #[test]
    fn zero_damage_sets_default_potion_and_keeps_damage_zero() {
        let fix = ItemPotionFix::new();
        let input = stack("minecraft:potion", 0, Some(BTreeMap::new()));
        let out = fix.make_rule(&input);
        assert_eq!(out.id, "minecraft:potion");
        assert_eq!(out.damage, 0);
        assert_eq!(
            out.tag.as_ref().unwrap().get("Potion"),
            Some(&TagValue::String("minecraft:water".to_string()))
        );
    }
}
