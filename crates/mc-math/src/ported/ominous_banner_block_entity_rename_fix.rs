// JAVA: net.minecraft.util.datafix.fixes.OminousBannerBlockEntityRenameFix#fix
/// Ported from OminousBannerBlockEntityRenameFix.fix(Typed<?>)
/// Replaces the translate key in CustomName from "block.minecraft.illager_banner" to "block.minecraft.ominous_banner".
pub fn fix_custom_name(custom_name: &str) -> String {
    custom_name.replace(
        "\"translate\":\"block.minecraft.illager_banner\"",
        "\"translate\":\"block.minecraft.ominous_banner\""
    )
}

// JAVA: net.minecraft.util.datafix.fixes.OminousBannerBlockEntityRenameFix#<init>
#[derive(Debug)]
pub struct OminousBannerBlockEntityRenameFix {
    _schema: String,
    _boolean: bool,
}

impl OminousBannerBlockEntityRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.OminousBannerBlockEntityRenameFix#<init>
    pub fn new(_schema: String, _boolean: bool) -> Self {
        Self { _schema, _boolean }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replace_illager_to_ominous() {
        let input = r#"{"translate":"block.minecraft.illager_banner"}"#;
        let expected = r#"{"translate":"block.minecraft.ominous_banner"}"#;
        assert_eq!(fix_custom_name(input), expected);
    }
    
    #[test]
    fn test_no_match_unchanged() {
        let input = r#"{"translate":"block.minecraft.banner"}"#;
        assert_eq!(fix_custom_name(input), input);
    }
    
    #[test]
    fn test_empty_string() {
        assert_eq!(fix_custom_name(""), "");
    }
    
    #[test]
    fn test_multiple_occurrences() {
        let input = r#"{"translate":"block.minecraft.illager_banner","other":"illager_banner"}"#;
        let expected = r#"{"translate":"block.minecraft.ominous_banner","other":"illager_banner"}"#;
        assert_eq!(fix_custom_name(input), expected);
    }
    
    #[test]
    fn test_null_like_input() {
        let input = r#"{"text":"hello"}"#;
        assert!(!fix_custom_name(input).contains("illager_banner"));
    }
}
