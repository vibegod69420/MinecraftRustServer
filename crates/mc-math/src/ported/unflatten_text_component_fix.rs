use serde_json::Value;

// JAVA: net.minecraft.util.datafix.fixes.UnflattenTextComponentFix#unflattenJson
pub fn unflatten_json(json_str: &str) -> Value {
    if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
        if !parsed.is_null() {
            return parsed;
        }
    }
    Value::String(json_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_json_object() {
        let input = r#"{"text":"Hello"}"#;
        let result = unflatten_json(input);
        assert!(!result.is_null());
        assert_eq!(result.get("text"), Some(&Value::String("Hello".into())));
    }
    
    #[test]
    fn test_invalid_json_returns_string() {
        let input = "not valid";
        let result = unflatten_json(input);
        assert!(result.is_string());
        assert_eq!(result.as_str(), Some("not valid"));
    }
    
    #[test]
    fn test_empty_string_returns_string() {
        let result = unflatten_json("");
        assert!(result.is_string());
    }
    
    #[test]
    fn test_literal_null_returns_string() {
        let result = unflatten_json("null");
        assert!(result.is_string());
    }
    
    #[test]
    fn test_malformed_json_returns_string() {
        let result = unflatten_json("{broken");
        assert!(result.is_string());
    }
}
