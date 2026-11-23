use std::collections::HashMap;
use std::sync::LazyLock;

pub static RENAMED_IDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "minecraft:zombie_pigman_spawn_egg".to_string(),
        "minecraft:zombified_piglin_spawn_egg".to_string(),
    );
    map
});

pub struct EntityZombifiedPiglinRenameFix;

impl EntityZombifiedPiglinRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityZombifiedPiglinRenameFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityZombifiedPiglinRenameFix#rename
    pub fn rename(&self, name: &str) -> String {
        if name == "minecraft:zombie_pigman" {
            "minecraft:zombified_piglin".to_string()
        } else {
            name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_zombie_pigman() {
        let fix = EntityZombifiedPiglinRenameFix::new(());
        assert_eq!(
            fix.rename("minecraft:zombie_pigman"),
            "minecraft:zombified_piglin"
        );
    }

    #[test]
    fn preserves_other_names_and_empty() {
        let fix = EntityZombifiedPiglinRenameFix::new(());
        assert_eq!(fix.rename("minecraft:zombie"), "minecraft:zombie");
        assert_eq!(fix.rename(""), "");
    }

    #[test]
    fn renamed_ids_contains_spawn_egg_mapping() {
        assert_eq!(
            RENAMED_IDS
                .get("minecraft:zombie_pigman_spawn_egg")
                .map(String::as_str),
            Some("minecraft:zombified_piglin_spawn_egg")
        );
        assert_eq!(RENAMED_IDS.len(), 1);
    }

    #[test]
    fn numeric_edge_case_strings_unchanged() {
        let fix = EntityZombifiedPiglinRenameFix::new(());
        let neg = (-1_i32).to_string();
        let min_i64 = i64::MIN.to_string();
        let max_u64 = u64::MAX.to_string();
        assert_eq!(fix.rename(&neg), neg);
        assert_eq!(fix.rename(&min_i64), min_i64);
        assert_eq!(fix.rename(&max_u64), max_u64);
    }

    #[test]
    fn float_bit_pattern_strings_unchanged() {
        let fix = EntityZombifiedPiglinRenameFix::new(());
        let nan_bits = f64::NAN.to_bits().to_string();
        let neg_zero_bits = (-0.0f32).to_bits().to_string();
        assert_eq!(fix.rename(&nan_bits), nan_bits);
        assert_eq!(fix.rename(&neg_zero_bits), neg_zero_bits);
    }
}
