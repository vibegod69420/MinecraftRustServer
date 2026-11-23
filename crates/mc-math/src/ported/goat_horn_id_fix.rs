/// Minimal self-contained dynamic value for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Int(i32),
    Map(Vec<(String, Dynamic)>),
    Other,
}

struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    fn as_int(&self, default: i32) -> i32 {
        match &self.0 {
            Some(Dynamic::Int(n)) => *n,
            _ => default,
        }
    }
}

impl Dynamic {
    fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Self::Map(entries) => OptionalDynamic(
                entries
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone()),
            ),
            _ => OptionalDynamic(None),
        }
    }

    fn remove(self, key: &str) -> Self {
        match self {
            Self::Map(mut entries) => {
                entries.retain(|(k, _)| k != key);
                Self::Map(entries)
            }
            other => other,
        }
    }

    fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Self::Map(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    *v = value;
                } else {
                    entries.push((key.to_string(), value));
                }
                Self::Map(entries)
            }
            other => other,
        }
    }

    fn create_string(&self, s: String) -> Self {
        Self::String(s)
    }
}

/// Minimal stub for DFU Schema.
pub struct Schema;

// JAVA: net.minecraft.util.datafix.fixes.GoatHornIdFix#INSTRUMENTS
const INSTRUMENTS: [&str; 8] = [
    "minecraft:ponder_goat_horn",
    "minecraft:sing_goat_horn",
    "minecraft:seek_goat_horn",
    "minecraft:feel_goat_horn",
    "minecraft:admire_goat_horn",
    "minecraft:call_goat_horn",
    "minecraft:yearn_goat_horn",
    "minecraft:dream_goat_horn",
];

pub struct GoatHornIdFix;

impl GoatHornIdFix {
    // JAVA: net.minecraft.util.datafix.fixes.GoatHornIdFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.GoatHornIdFix#fixItemStackTag
    pub fn fix_item_stack_tag(&self, value: Dynamic) -> Dynamic {
        let sound_variant: i32 = value.get("SoundVariant").as_int(0);
        let idx: usize = if sound_variant >= 0 && (sound_variant as usize) < INSTRUMENTS.len() {
            sound_variant as usize
        } else {
            0usize
        };
        let instrument = value.create_string(INSTRUMENTS[idx].to_string());
        value.remove("SoundVariant").set("instrument", instrument)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn maps_sound_variant_to_instrument() {
        let fix = GoatHornIdFix::new(Schema);
        let input = map(vec![("SoundVariant", Dynamic::Int(2))]);
        let out = fix.fix_item_stack_tag(input);
        assert_eq!(
            out,
            map(vec![(
                "instrument",
                Dynamic::String("minecraft:seek_goat_horn".to_string())
            )])
        );
    }

    #[test]
    fn missing_sound_variant_defaults_to_zero() {
        let fix = GoatHornIdFix::new(Schema);
        let input = map(vec![("Other", Dynamic::Other)]);
        let out = fix.fix_item_stack_tag(input);
        assert_eq!(
            out,
            map(vec![
                ("Other", Dynamic::Other),
                (
                    "instrument",
                    Dynamic::String("minecraft:ponder_goat_horn".to_string())
                ),
            ])
        );
    }

    #[test]
    fn out_of_range_positive_defaults_to_zero() {
        let fix = GoatHornIdFix::new(Schema);
        let input = map(vec![("SoundVariant", Dynamic::Int(8))]);
        let out = fix.fix_item_stack_tag(input);
        assert_eq!(
            out,
            map(vec![(
                "instrument",
                Dynamic::String("minecraft:ponder_goat_horn".to_string())
            )])
        );
    }

    #[test]
    fn negative_and_large_i32_default_to_zero() {
        let fix = GoatHornIdFix::new(Schema);
        let neg = map(vec![("SoundVariant", Dynamic::Int(-1))]);
        let max = map(vec![("SoundVariant", Dynamic::Int(i32::MAX))]);
        let expected = map(vec![(
            "instrument",
            Dynamic::String("minecraft:ponder_goat_horn".to_string())
        )]);
        assert_eq!(fix.fix_item_stack_tag(neg), expected);
        assert_eq!(fix.fix_item_stack_tag(max), expected);
    }

    #[test]
    fn removes_sound_variant_and_preserves_other_fields() {
        let fix = GoatHornIdFix::new(Schema);
        let input = map(vec![
            ("SoundVariant", Dynamic::Int(7)),
            ("Charged", Dynamic::Other),
        ]);
        let out = fix.fix_item_stack_tag(input);
        assert_eq!(
            out,
            map(vec![
                ("Charged", Dynamic::Other),
                (
                    "instrument",
                    Dynamic::String("minecraft:dream_goat_horn".to_string())
                ),
            ])
        );
    }
}
