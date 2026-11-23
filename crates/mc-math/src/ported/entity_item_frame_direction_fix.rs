use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    Bool(bool),
    Byte(i8),
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Compound(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_bool(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(b) => *b,
            Dynamic::Byte(b) => *b != 0,
            _ => default,
        }
    }

    pub fn as_byte(&self, default: i8) -> i8 {
        match self {
            Dynamic::Byte(b) => *b,
            Dynamic::Int(i) => *i as i8,
            Dynamic::Long(l) => *l as i8,
            _ => default,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_byte(&self, v: i8) -> Dynamic {
        Dynamic::Byte(v)
    }

    pub fn create_string(&self, v: &str) -> Dynamic {
        Dynamic::String(v.to_string())
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_string(), value);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }
}

pub struct EntityItemFrameDirectionFix;

impl EntityItemFrameDirectionFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityItemFrameDirectionFix#fixTag
    pub fn fix_tag(d: &Dynamic) -> Dynamic {
        d.set(
            "Facing",
            d.create_byte(Self::direction_2d_to_3d(d.get("Facing").as_byte(0))),
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityItemFrameDirectionFix#fix
    pub fn fix(d: &Dynamic) -> Dynamic {
        Self::fix_tag(d)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityItemFrameDirectionFix#direction2dTo3d
    pub fn direction_2d_to_3d(dir: i8) -> i8 {
        match dir {
            0 => 3,
            1 => 4,
            2 => 2,
            3 => 5,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn compound(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Compound(m)
    }

    #[test]
    fn direction_mapping_all_known_cases() {
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(0), 3);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(1), 4);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(2), 2);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(3), 5);
    }

    #[test]
    fn direction_mapping_negative_and_overflow_edges() {
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(-1), 2);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(i8::MIN), 2);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(i8::MAX), 2);
        assert_eq!(EntityItemFrameDirectionFix::direction_2d_to_3d(4), 2);
    }

    #[test]
    fn fix_tag_updates_facing_and_preserves_other_fields() {
        let d = compound(vec![
            ("Facing", Dynamic::Byte(0)),
            ("Item", Dynamic::String("minecraft:stone".to_string())),
            ("TileX", Dynamic::Int(-10)),
        ]);
        let fixed = EntityItemFrameDirectionFix::fix_tag(&d);
        assert_eq!(fixed.get("Facing"), Dynamic::Byte(3));
        assert_eq!(fixed.get("Item"), Dynamic::String("minecraft:stone".to_string()));
        assert_eq!(fixed.get("TileX"), Dynamic::Int(-10));
    }

    #[test]
    fn fix_tag_missing_facing_defaults_to_zero_mapping() {
        let d = compound(vec![("id", Dynamic::String("minecraft:item_frame".to_string()))]);
        let fixed = EntityItemFrameDirectionFix::fix_tag(&d);
        assert_eq!(fixed.get("Facing"), Dynamic::Byte(3));
    }

    #[test]
    fn fix_preserves_float_bits_in_unrelated_fields() {
        let bits = f32::to_bits(f32::NAN) as i32;
        let d = compound(vec![
            ("Facing", Dynamic::Byte(1)),
            ("floatBits", Dynamic::Int(bits)),
        ]);
        let fixed = EntityItemFrameDirectionFix::fix(&d);
        assert_eq!(fixed.get("Facing"), Dynamic::Byte(4));
        assert_eq!(fixed.get("floatBits"), Dynamic::Int(bits));
    }
}
