use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix porting.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    fn as_number(&self) -> Option<f64> {
        match self {
            Dynamic::Int(v) => Some(*v as f64),
            Dynamic::Long(v) => Some(*v as f64),
            Dynamic::Float(v) => Some(*v as f64),
            Dynamic::Double(v) => Some(*v),
            _ => None,
        }
    }

    fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            _ => default,
        }
    }

    fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.VillagerRebuildLevelAndXpFix#getMinXpPerLevel
fn get_min_xp_per_level(level: i32) -> i32 {
    const LEVEL_XP_THRESHOLDS: [i32; 5] = [0, 10, 70, 150, 250];
    let idx = level.wrapping_sub(1).clamp(0, 4) as usize;
    LEVEL_XP_THRESHOLDS[idx]
}

pub struct ZombieVillagerRebuildXpFix;

impl ZombieVillagerRebuildXpFix {
    // JAVA: net.minecraft.util.datafix.fixes.ZombieVillagerRebuildXpFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ZombieVillagerRebuildXpFix#fix
    pub fn fix(&self, data: Dynamic) -> Dynamic {
        match data {
            Dynamic::Map(mut map) => {
                let has_xp = map.get("Xp").and_then(|v| v.as_number()).is_some();
                if !has_xp {
                    let level = map
                        .get("VillagerData")
                        .and_then(|vd| vd.get("level"))
                        .map(|l| l.as_int(1))
                        .unwrap_or(1);
                    let xp = get_min_xp_per_level(level);
                    map.insert("Xp".to_string(), Dynamic::Int(xp));
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

impl Default for ZombieVillagerRebuildXpFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: &[(&str, Dynamic)]) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in pairs {
            map.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(map)
    }

    fn get_xp_int(d: &Dynamic) -> Option<i32> {
        match d.get("Xp") {
            Some(Dynamic::Int(v)) => Some(*v),
            _ => None,
        }
    }

    #[test]
    fn xp_present_unchanged() {
        let input = obj(&[("Xp", Dynamic::Int(42))]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn xp_missing_uses_villager_level() {
        let input = obj(&[(
            "VillagerData",
            obj(&[("level", Dynamic::Int(3))]),
        )]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input);
        assert_eq!(get_xp_int(&out), Some(70));
    }

    #[test]
    fn xp_missing_no_villager_data_defaults_to_level_1() {
        let input = obj(&[]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input);
        assert_eq!(get_xp_int(&out), Some(0));
    }

    #[test]
    fn xp_missing_level_clamped_high() {
        let input = obj(&[(
            "VillagerData",
            obj(&[("level", Dynamic::Int(99))]),
        )]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input);
        assert_eq!(get_xp_int(&out), Some(250));
    }

    #[test]
    fn xp_missing_negative_level_clamped_low() {
        let input = obj(&[(
            "VillagerData",
            obj(&[("level", Dynamic::Int(-5))]),
        )]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input);
        // (level - 1) clamps to 0 => LEVEL_XP_THRESHOLDS[0] == 0
        assert_eq!(get_xp_int(&out), Some(0));
    }

    #[test]
    fn xp_present_as_long_counts_as_number() {
        let input = obj(&[("Xp", Dynamic::Long(7))]);
        let out = ZombieVillagerRebuildXpFix::new().fix(input.clone());
        assert_eq!(out, input);
    }
}
