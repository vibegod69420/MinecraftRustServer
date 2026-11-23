// JAVA: net.minecraft.util.datafix.fixes.OminousBannerRarityFix

pub fn extract_translation_string(component_json: &str) -> Option<String> {
    let key = "\"translate\"";
    let idx = component_json.find(key)?;
    let rest = &component_json[idx + key.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    if !after.starts_with('"') {
        return None;
    }
    let inner = &after[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

pub fn create_translatable_component_json(translation_key: &str) -> String {
    format!(r#"{{"translate":"{}"}}"#, translation_key)
}

pub fn is_ominous_banner_translation(translation: Option<&str>) -> bool {
    translation == Some("block.minecraft.ominous_banner")
}

pub fn fix_rarity_if_ominous(translation: Option<&str>) -> Option<&'static str> {
    if is_ominous_banner_translation(translation) {
        Some("uncommon")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_translation() {
        let json = r#"{"translate":"block.minecraft.ominous_banner"}"#;
        assert_eq!(
            extract_translation_string(json),
            Some("block.minecraft.ominous_banner".to_string())
        );
    }

    #[test]
    fn create_component_json() {
        assert_eq!(
            create_translatable_component_json("block.minecraft.ominous_banner"),
            r#"{"translate":"block.minecraft.ominous_banner"}"#
        );
    }

    #[test]
    fn fix_rarity_for_ominous() {
        assert_eq!(
            fix_rarity_if_ominous(Some("block.minecraft.ominous_banner")),
            Some("uncommon")
        );
    }

    #[test]
    fn no_fix_for_other() {
        assert_eq!(fix_rarity_if_ominous(Some("block.minecraft.banner")), None);
        assert_eq!(fix_rarity_if_ominous(None), None);
    }
}
