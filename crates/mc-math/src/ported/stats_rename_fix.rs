//! Port of `net.minecraft.util.datafix.fixes.StatsRenameFix`.
//!
//! The original Java class is a DataFixerUpper `DataFix` that rewrites
//! objective criteria and stats map keys using a rename map. In this
//! self-contained Rust port we preserve the 1:1 rename semantics:
//! - missing keys are left unchanged;
//! - present keys are replaced by the mapped value.

use std::collections::HashMap;

/// JAVA: net.minecraft.util.datafix.fixes.StatsRenameFix
#[derive(Debug, Clone)]
pub struct StatsRenameFix {
    name: String,
    renames: HashMap<String, String>,
}

impl StatsRenameFix {
    // JAVA: StatsRenameFix.<init>(Schema, String, Map<String, String>)
    pub fn new(name: String, renames: HashMap<String, String>) -> Self {
        Self { name, renames }
    }

    /// Returns the configured fix name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the rename map.
    pub fn renames(&self) -> &HashMap<String, String> {
        &self.renames
    }

    /// Applies the rename map to a single namespaced id string.
    ///
    /// Mirrors `renames.getOrDefault(id, id)` used in both
    /// `createCriteriaRule` and `createStatRule`.
    // JAVA: StatsRenameFix#createCriteriaRule / StatsRenameFix#createStatRule (rename lambda)
    pub fn rename(&self, id: &str) -> String {
        self.renames.get(id).cloned().unwrap_or_else(|| id.to_string())
    }

    /// Applies renames to objective criteria ids.
    ///
    /// This is the self-contained equivalent of the criteria rewrite rule:
    /// only custom criteria ids are rewritten through the rename map.
    // JAVA: StatsRenameFix#createCriteriaRule()
    pub fn apply_criteria_renames(&self, criteria_ids: &[String]) -> Vec<String> {
        criteria_ids.iter().map(|id| self.rename(id)).collect()
    }

    /// Applies renames to custom stat keys in a stats map.
    ///
    /// This is the self-contained equivalent of the stat rewrite rule:
    /// keys are rewritten through the rename map while values are preserved.
    // JAVA: StatsRenameFix#createStatRule()
    pub fn apply_stat_renames<V: Clone>(&self, stats: &HashMap<String, V>) -> HashMap<String, V> {
        let mut out = HashMap::with_capacity(stats.len());
        for (key, value) in stats {
            out.insert(self.rename(key), value.clone());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::StatsRenameFix;
    use std::collections::HashMap;

    fn sample_fix() -> StatsRenameFix {
        let mut renames = HashMap::new();
        renames.insert("minecraft:jump".to_string(), "minecraft:custom/minecraft:jump".to_string());
        renames.insert("minecraft:old".to_string(), "minecraft:new".to_string());
        StatsRenameFix::new("StatsRenameFix".to_string(), renames)
    }

    #[test]
    fn rename_present_and_missing_keys() {
        let fix = sample_fix();
        assert_eq!(fix.rename("minecraft:jump"), "minecraft:custom/minecraft:jump");
        assert_eq!(fix.rename("minecraft:old"), "minecraft:new");
        assert_eq!(fix.rename("minecraft:unchanged"), "minecraft:unchanged");
    }

    #[test]
    fn criteria_renames_preserve_order_and_defaults() {
        let fix = sample_fix();
        let input = vec![
            "minecraft:jump".to_string(),
            "minecraft:unchanged".to_string(),
            "minecraft:old".to_string(),
        ];
        let output = fix.apply_criteria_renames(&input);
        assert_eq!(
            output,
            vec![
                "minecraft:custom/minecraft:jump".to_string(),
                "minecraft:unchanged".to_string(),
                "minecraft:new".to_string(),
            ]
        );
    }

    #[test]
    fn stat_renames_preserve_values() {
        let fix = sample_fix();
        let mut stats = HashMap::new();
        stats.insert("minecraft:jump".to_string(), 7_i32);
        stats.insert("minecraft:keep".to_string(), -3_i32);

        let renamed = fix.apply_stat_renames(&stats);
        assert_eq!(renamed.get("minecraft:custom/minecraft:jump"), Some(&7));
        assert_eq!(renamed.get("minecraft:keep"), Some(&-3));
        assert!(!renamed.contains_key("minecraft:jump"));
        assert_eq!(renamed.len(), 2);
    }

    #[test]
    fn empty_rename_map_is_identity() {
        let fix = StatsRenameFix::new("noop".to_string(), HashMap::new());
        assert_eq!(fix.rename("minecraft:anything"), "minecraft:anything");
        let mut stats = HashMap::new();
        stats.insert("a".to_string(), 1_i64);
        let renamed = fix.apply_stat_renames(&stats);
        assert_eq!(renamed.get("a"), Some(&1_i64));
    }
}
