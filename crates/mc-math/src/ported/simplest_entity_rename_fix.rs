/// Self-contained port of `net.minecraft.util.datafix.fixes.SimplestEntityRenameFix`.
///
/// The original class extends DFU `DataFix` and builds a `TypeRewriteRule`.
/// DFU schema/type machinery is not available in `mc-math`; this module preserves
/// the fix identity, constructor contract, and rename behavior.
#[derive(Clone)]
pub struct SimplestEntityRenameFix {
    name: String,
    changes_type: bool,
    rename_fn: fn(&str) -> String,
}

impl std::fmt::Debug for SimplestEntityRenameFix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplestEntityRenameFix")
            .field("name", &self.name)
            .field("changes_type", &self.changes_type)
            .finish()
    }
}

impl SimplestEntityRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.SimplestEntityRenameFix#<init>
    pub fn new(name: String, changes_type: bool, rename_fn: fn(&str) -> String) -> Self {
        Self {
            name,
            changes_type,
            rename_fn,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn changes_type(&self) -> bool {
        self.changes_type
    }

    // JAVA: net.minecraft.util.datafix.fixes.SimplestEntityRenameFix#rename
    pub fn rename(&self, input: &str) -> String {
        (self.rename_fn)(input)
    }

    // JAVA: net.minecraft.util.datafix.fixes.SimplestEntityRenameFix#makeRule
    pub fn make_rule(&self, entity_type: &str, entity_name: Option<&str>) -> (String, Option<String>) {
        let renamed_type = self.rename(entity_type);
        let renamed_name = entity_name.map(|n| self.rename(n));
        (renamed_type, renamed_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename_zombie(input: &str) -> String {
        if input == "minecraft:zombie" {
            "minecraft:husk".to_string()
        } else {
            input.to_string()
        }
    }

    #[test]
    fn new_and_accessors() {
        let fix = SimplestEntityRenameFix::new("TestFix".to_string(), true, rename_zombie);
        assert_eq!(fix.name(), "TestFix");
        assert!(fix.changes_type());
    }

    #[test]
    fn rename_matching_and_non_matching() {
        let fix = SimplestEntityRenameFix::new("TestFix".to_string(), false, rename_zombie);
        assert_eq!(fix.rename("minecraft:zombie"), "minecraft:husk");
        assert_eq!(fix.rename("minecraft:skeleton"), "minecraft:skeleton");
    }

    #[test]
    fn make_rule_renames_type_and_name() {
        let fix = SimplestEntityRenameFix::new("TestFix".to_string(), true, rename_zombie);
        let (ty, name) = fix.make_rule("minecraft:zombie", Some("minecraft:zombie"));
        assert_eq!(ty, "minecraft:husk");
        assert_eq!(name.as_deref(), Some("minecraft:husk"));
    }

    #[test]
    fn make_rule_none_name_and_empty_string() {
        let fix = SimplestEntityRenameFix::new("Empty".to_string(), false, |s| s.to_string());
        let (ty, name) = fix.make_rule("minecraft:pig", None);
        assert_eq!(ty, "minecraft:pig");
        assert_eq!(name, None);
        assert_eq!(fix.rename(""), "");
    }

    #[test]
    fn make_rule_preserves_unrelated_values() {
        let fix = SimplestEntityRenameFix::new("TestFix".to_string(), true, rename_zombie);
        let (ty, name) = fix.make_rule("minecraft:creeper", Some("custom:name"));
        assert_eq!(ty, "minecraft:creeper");
        assert_eq!(name.as_deref(), Some("custom:name"));
    }
}
