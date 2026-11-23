// JAVA: net.minecraft.util.datafix.fixes.ObjectiveRenderTypeFix#getRenderType
/// Ported from ObjectiveRenderTypeFix.getRenderType(String)
pub fn get_render_type(criteria: &str) -> String {
    if criteria == "health" {
        "hearts".to_string()
    } else {
        "integer".to_string()
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ObjectiveRenderTypeFix#resolve
/// Ported from ObjectiveRenderTypeFix makeRule render type resolution
pub fn resolve_render_type(criteria_name: &str, render_type: Option<&str>) -> String {
    if let Some(rt) = render_type {
        rt.to_string()
    } else {
        get_render_type(criteria_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_render_type_health() {
        assert_eq!(get_render_type("health"), "hearts");
    }
    
    #[test]
    fn test_get_render_type_score() {
        assert_eq!(get_render_type("score"), "integer");
    }
    
    #[test]
    fn test_get_render_type_custom() {
        assert_eq!(get_render_type("custom_objective"), "integer");
    }
    
    #[test]
    fn test_resolve_type_default() {
        assert_eq!(resolve_render_type("health", None), "hearts");
    }
    
    #[test]
    fn test_resolve_type_existing() {
        assert_eq!(resolve_render_type("score", Some("hearts")), "hearts");
    }
    
    #[test]
    fn test_resolve_type_existing_integer() {
        assert_eq!(resolve_render_type("score", Some("integer")), "integer");
    }
}
