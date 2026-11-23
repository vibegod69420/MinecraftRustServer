//! Port of `net.minecraft.util.datafix.fixes.VillagerFollowRangeFix`

// JAVA: net.minecraft.util.datafix.fixes.VillagerFollowRangeFix#fixValue
/// Checks and updates a villager follow range attribute: if name is "generic.follow_range" and base is 16.0, changes base to 48.0.
pub fn fix_villager_follow_range(attribute_name: &str, base: f64) -> (bool, f64) {
    if attribute_name == "generic.follow_range" && base == 16.0 {
        (true, 48.0)
    } else {
        (false, base)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.VillagerFollowRangeFix#fix
/// Simplified fix method applying the attribute update.
pub struct VillagerFollowRangeFix;

impl VillagerFollowRangeFix {
    // JAVA: net.minecraft.util.datafix.fixes.VillagerFollowRangeFix#apply
    /// Applies the follow range fix to a base value.
    pub fn apply(attribute_name: &str, base: f64) -> f64 {
        let (_, new_base) = fix_villager_follow_range(attribute_name, base);
        new_base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follow_range_updates_from_16_to_48() {
        let (updated, new_base) = fix_villager_follow_range("generic.follow_range", 16.0);
        assert!(updated);
        assert_eq!(new_base, 48.0);
    }

    #[test]
    fn test_other_attribute_unchanged_at_16() {
        let (updated, new_base) = fix_villager_follow_range("generic.movement_speed", 16.0);
        assert!(!updated);
        assert_eq!(new_base, 16.0);
    }

    #[test]
    fn test_follow_range_different_base_not_changed() {
        let (updated, new_base) = fix_villager_follow_range("generic.follow_range", 32.0);
        assert!(!updated);
        assert_eq!(new_base, 32.0);
    }

    #[test]
    fn test_follow_range_negative_base_not_changed() {
        let (updated, new_base) = fix_villager_follow_range("generic.follow_range", -1.0);
        assert!(!updated);
        assert_eq!(new_base, -1.0);
    }

    #[test]
    fn test_follow_range_zero_base_not_changed() {
        let (updated, new_base) = fix_villager_follow_range("generic.follow_range", 0.0);
        assert!(!updated);
        assert_eq!(new_base, 0.0);
    }
}
