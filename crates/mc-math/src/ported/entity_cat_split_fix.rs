// JAVA: net.minecraft.util.datafix.fixes.EntityCatSplitFix#getNewNameAndTag
/// Ported from EntityCatSplitFix.getNewNameAndTag(String, Dynamic<?>)
pub fn entity_cat_split_fix(
    name: &str,
    cat_type: i32,
    owner: &str,
    owner_uuid: &str,
) -> (String, bool, String) {
    if name == "minecraft:ocelot" {
        if cat_type == 0 {
            let trusting = !owner.is_empty() || !owner_uuid.is_empty();
            return (name.to_string(), trusting, owner_uuid.to_string());
        } else if cat_type > 0 && cat_type < 4 {
            let new_owner_uuid = "";
            ("minecraft:cat".to_string(), false, new_owner_uuid.to_string())
        } else {
            (name.to_string(), false, owner_uuid.to_string())
        }
    } else {
        (name.to_string(), false, owner_uuid.to_string())
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntityCatSplitFix#<init>
#[derive(Debug)]
pub struct EntityCatSplitFix {
    _name: String,
    _schema: String,
    _boolean: bool,
}

impl EntityCatSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityCatSplitFix#<init>
    pub fn new(_name: String, _schema: String, _boolean: bool) -> Self {
        Self { _name, _schema, _boolean }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ocelot_cat_type_0_with_owner() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:ocelot", 0, "Steve", "");
        assert_eq!(name, "minecraft:ocelot");
        assert!(trusting);
        assert_eq!(uuid, "");
    }
    
    #[test]
    fn test_ocelot_cat_type_0_no_owner() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:ocelot", 0, "", "");
        assert_eq!(name, "minecraft:ocelot");
        assert!(!trusting);
        assert_eq!(uuid, "");
    }
    
    #[test]
    fn test_ocelot_cat_type_1() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:ocelot", 1, "Steve", "uuid123");
        assert_eq!(name, "minecraft:cat");
        assert!(!trusting);
        assert_eq!(uuid, "");
    }
    
    #[test]
    fn test_ocelot_cat_type_3() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:ocelot", 3, "", "uuid");
        assert_eq!(name, "minecraft:cat");
        assert!(!trusting);
        assert_eq!(uuid, "");
    }
    
    #[test]
    fn test_ocelot_cat_type_4_out_of_range() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:ocelot", 4, "", "");
        assert_eq!(name, "minecraft:ocelot");
        assert!(!trusting);
        assert_eq!(uuid, "");
    }
    
    #[test]
    fn test_non_ocelot_name() {
        let (name, trusting, uuid) = entity_cat_split_fix("minecraft:wolf", 0, "", "");
        assert_eq!(name, "minecraft:wolf");
        assert!(!trusting);
        assert_eq!(uuid, "");
    }
}
