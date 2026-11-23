use std::collections::HashMap;
use std::sync::OnceLock;

/// Faithful port of `net.minecraft.util.datafix.fixes.ScoreboardDisplaySlotFix`.
///
/// Renames legacy scoreboard display slot IDs (e.g. `slot_0`) to modern names.
pub struct ScoreboardDisplaySlotFix;

impl ScoreboardDisplaySlotFix {
    // JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplaySlotFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplaySlotFix#makeRule
    pub fn fix_display_slot<'a>(&self, slot: &'a str) -> &'a str {
        rename(slot).unwrap_or(slot)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplaySlotFix#SLOT_RENAMES
pub fn slot_renames() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("slot_0", "list"),
            ("slot_1", "sidebar"),
            ("slot_2", "below_name"),
            ("slot_3", "sidebar.team.black"),
            ("slot_4", "sidebar.team.dark_blue"),
            ("slot_5", "sidebar.team.dark_green"),
            ("slot_6", "sidebar.team.dark_aqua"),
            ("slot_7", "sidebar.team.dark_red"),
            ("slot_8", "sidebar.team.dark_purple"),
            ("slot_9", "sidebar.team.gold"),
            ("slot_10", "sidebar.team.gray"),
            ("slot_11", "sidebar.team.dark_gray"),
            ("slot_12", "sidebar.team.blue"),
            ("slot_13", "sidebar.team.green"),
            ("slot_14", "sidebar.team.aqua"),
            ("slot_15", "sidebar.team.red"),
            ("slot_16", "sidebar.team.light_purple"),
            ("slot_17", "sidebar.team.yellow"),
            ("slot_18", "sidebar.team.white"),
        ])
    })
}

// JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplaySlotFix#rename
pub fn rename(slot: &str) -> Option<&'static str> {
    slot_renames().get(slot).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_known_slots() {
        assert_eq!(rename("slot_0"), Some("list"));
        assert_eq!(rename("slot_1"), Some("sidebar"));
        assert_eq!(rename("slot_2"), Some("below_name"));
        assert_eq!(rename("slot_18"), Some("sidebar.team.white"));
    }

    #[test]
    fn test_rename_unknown_returns_none() {
        assert_eq!(rename("slot_19"), None);
        assert_eq!(rename("slot_-1"), None);
        assert_eq!(rename("list"), None);
        assert_eq!(rename(""), None);
    }

    #[test]
    fn test_fix_display_slot_passthrough_and_rename() {
        let fix = ScoreboardDisplaySlotFix::new();
        assert_eq!(fix.fix_display_slot("slot_5"), "sidebar.team.dark_green");
        assert_eq!(fix.fix_display_slot("unknown"), "unknown");
        assert_eq!(fix.fix_display_slot("sidebar"), "sidebar");
    }

    #[test]
    fn test_slot_renames_completeness() {
        let map = slot_renames();
        assert_eq!(map.len(), 19);
        for i in 0..=18 {
            let key = format!("slot_{}", i);
            assert!(map.contains_key(key.as_str()), "missing {}", key);
        }
    }
}
