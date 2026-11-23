#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTippedArrowFix {
    pub name: String,
    pub schema: Schema,
    pub changes_type: bool,
}

impl EntityTippedArrowFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityTippedArrowFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            name: "EntityTippedArrowFix".to_string(),
            schema,
            changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityTippedArrowFix#rename
    pub fn rename(&self, name: &str) -> String {
        if name == "TippedArrow" {
            "Arrow".to_string()
        } else {
            name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_tipped_arrow() {
        let fix = EntityTippedArrowFix::new(Schema, true);
        assert_eq!(fix.rename("TippedArrow"), "Arrow");
    }

    #[test]
    fn leaves_other_names_unchanged() {
        let fix = EntityTippedArrowFix::new(Schema, false);
        assert_eq!(fix.rename("Arrow"), "Arrow");
        assert_eq!(fix.rename("minecraft:arrow"), "minecraft:arrow");
        assert_eq!(fix.rename("SpectralArrow"), "SpectralArrow");
    }

    #[test]
    fn case_sensitive_and_empty_edge_cases() {
        let fix = EntityTippedArrowFix::new(Schema, true);
        assert_eq!(fix.rename("tippedarrow"), "tippedarrow");
        assert_eq!(fix.rename("TIPPEDARROW"), "TIPPEDARROW");
        assert_eq!(fix.rename(""), "");
    }

    #[test]
    fn constructor_sets_fields() {
        let fix = EntityTippedArrowFix::new(Schema, true);
        assert_eq!(fix.name, "EntityTippedArrowFix");
        assert!(fix.changes_type);
        assert_eq!(fix.schema, Schema);

        let fix2 = EntityTippedArrowFix::new(Schema, false);
        assert!(!fix2.changes_type);
    }
}
