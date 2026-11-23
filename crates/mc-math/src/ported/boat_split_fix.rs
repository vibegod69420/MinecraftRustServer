// JAVA: net.minecraft.util.datafix.fixes.BoatSplitFix

pub fn is_normal_boat(id: &str) -> bool {
    id == "minecraft:boat"
}

pub fn is_chest_boat(id: &str) -> bool {
    id == "minecraft:chest_boat"
}

pub fn is_any_boat(id: &str) -> bool {
    is_normal_boat(id) || is_chest_boat(id)
}

pub fn map_variant_to_normal_boat(variant: &str) -> &'static str {
    match variant {
        "spruce" => "minecraft:spruce_boat",
        "birch" => "minecraft:birch_boat",
        "jungle" => "minecraft:jungle_boat",
        "acacia" => "minecraft:acacia_boat",
        "cherry" => "minecraft:cherry_boat",
        "dark_oak" => "minecraft:dark_oak_boat",
        "mangrove" => "minecraft:mangrove_boat",
        "bamboo" => "minecraft:bamboo_raft",
        _ => "minecraft:oak_boat",
    }
}

pub fn map_variant_to_chest_boat(variant: &str) -> &'static str {
    match variant {
        "spruce" => "minecraft:spruce_chest_boat",
        "birch" => "minecraft:birch_chest_boat",
        "jungle" => "minecraft:jungle_chest_boat",
        "acacia" => "minecraft:acacia_chest_boat",
        "cherry" => "minecraft:cherry_chest_boat",
        "dark_oak" => "minecraft:dark_oak_chest_boat",
        "mangrove" => "minecraft:mangrove_chest_boat",
        "bamboo" => "minecraft:bamboo_chest_raft",
        _ => "minecraft:oak_chest_boat",
    }
}

pub fn split_boat_id(id: &str, boat_type: Option<&str>) -> Option<String> {
    if !is_any_boat(id) {
        return None;
    }
    let variant = boat_type.unwrap_or("");
    let new_id = if is_chest_boat(id) {
        map_variant_to_chest_boat(variant)
    } else {
        map_variant_to_normal_boat(variant)
    };
    Some(new_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_boat_detection() {
        assert!(is_normal_boat("minecraft:boat"));
        assert!(!is_normal_boat("minecraft:chest_boat"));
    }

    #[test]
    fn chest_boat_detection() {
        assert!(is_chest_boat("minecraft:chest_boat"));
        assert!(!is_chest_boat("minecraft:boat"));
    }

    #[test]
    fn any_boat_detection() {
        assert!(is_any_boat("minecraft:boat"));
        assert!(is_any_boat("minecraft:chest_boat"));
        assert!(!is_any_boat("minecraft:oak_boat"));
    }

    #[test]
    fn normal_mapping() {
        assert_eq!(map_variant_to_normal_boat("spruce"), "minecraft:spruce_boat");
        assert_eq!(map_variant_to_normal_boat("unknown"), "minecraft:oak_boat");
        assert_eq!(map_variant_to_normal_boat("bamboo"), "minecraft:bamboo_raft");
    }

    #[test]
    fn chest_mapping() {
        assert_eq!(map_variant_to_chest_boat("bamboo"), "minecraft:bamboo_chest_raft");
        assert_eq!(map_variant_to_chest_boat("unknown"), "minecraft:oak_chest_boat");
    }

    #[test]
    fn split_boat() {
        assert_eq!(
            split_boat_id("minecraft:boat", Some("spruce")),
            Some("minecraft:spruce_boat".to_string())
        );
        assert_eq!(
            split_boat_id("minecraft:chest_boat", None),
            Some("minecraft:oak_chest_boat".to_string())
        );
        assert_eq!(split_boat_id("minecraft:oak_boat", Some("spruce")), None);
    }
}
